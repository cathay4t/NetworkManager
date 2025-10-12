// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mozim::{DhcpV4Client, DhcpV4Config, DhcpV4Lease, DhcpV4State};
use nm::{
    BaseInterface, DhcpState, ErrorKind, MergedNetworkState, NetworkState,
    NmError, NmstateInterface,
};
use tokio::sync::mpsc::{Receiver, Sender};

use super::share_data::NmDaemonShareData;

const MPSC_CHANNEL_BUFFER: usize = 64;

#[derive(Debug, Default)]
pub(crate) struct NmDhcpV4Manager {
    workers: HashMap<String, NmDhcpV4Worker>,
}

// Do not add `async` function to NmDhcpV4Manager because it will be stored
// into Mutex protected `NmDaemonShareData`. The
// `MutexGuard` will cause function not `Send`.
impl NmDhcpV4Manager {
    /// Fill the NetworkState with DHCP states
    pub(crate) fn query(
        &self,
        net_state: &mut NetworkState,
    ) -> Result<(), NmError> {
        for (iface_name, worker) in self.workers.iter() {
            if let Some(iface) =
                net_state.ifaces.kernel_ifaces.get_mut(iface_name.as_str())
            {
                let ipv4_conf = iface
                    .base_iface_mut()
                    .ipv4
                    .get_or_insert(Default::default());
                ipv4_conf.enabled = Some(true);
                ipv4_conf.dhcp = Some(true);
                ipv4_conf.dhcp_state = Some(worker.get_state()?);
            }
        }
        Ok(())
    }

    pub(crate) fn add_dhcp_worker(
        &mut self,
        worker: NmDhcpV4Worker,
    ) -> Result<(), NmError> {
        self.workers
            .insert(worker.base_iface.name.to_string(), worker);
        Ok(())
    }

    /// Remove DHCP worker will cause DHCP thread been terminated
    pub(crate) fn remove_dhcp_worker(&mut self, iface_name: &str) {
        self.workers.remove(iface_name);
    }
}

#[derive(Debug)]
pub(crate) struct NmDhcpV4Worker {
    base_iface: BaseInterface,
    // No need to send any data. Dropping this Sender will cause
    // Receiver.recv() got None which trigger DHCP thread quit.
    quit_notifer: Sender<()>,
    share_data: Arc<Mutex<NmDhcpShareData>>,
}

#[derive(Debug, Default)]
pub(crate) struct NmDhcpShareData {
    state: DhcpState,
}

impl NmDhcpV4Worker {
    pub(crate) async fn new(
        base_iface: &BaseInterface,
    ) -> Result<Self, NmError> {
        let (sender, receiver) =
            tokio::sync::mpsc::channel::<()>(MPSC_CHANNEL_BUFFER);
        let ret = Self {
            base_iface: base_iface.clone(),
            quit_notifer: sender,
            share_data: Arc::new(Mutex::new(NmDhcpShareData::default())),
        };
        let mac_addr = match base_iface.mac_address.as_deref() {
            Some(m) => m,
            None => {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Got no MAC address for DHCPv4 on interface {}({})",
                        base_iface.name, base_iface.iface_type
                    ),
                ));
            }
        };
        let mut dhcp_config = DhcpV4Config::new(base_iface.name.as_str());
        dhcp_config
            .set_iface_index(base_iface.iface_index)
            .set_iface_mac(mac_addr)
            .map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Failed to set iface {}/{} MAC {}: {e}",
                        base_iface.name, base_iface.iface_type, mac_addr,
                    ),
                )
            })?
            .use_mac_as_client_id();
        // TODO(Gris Ge): Support loading previous stored lease
        let dhcp_client =
            DhcpV4Client::init(dhcp_config, None).await.map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Failed to start DHCPv4 client on iface {}/{}: {e}",
                        base_iface.name, base_iface.iface_type,
                    ),
                )
            })?;

        let base_iface = base_iface.clone();
        let share_data = ret.share_data.clone();
        tokio::spawn(async move {
            if let Err(e) =
                dhcp_thread(dhcp_client, base_iface, receiver, share_data).await
            {
                log::error!("{e}");
            }
        });
        Ok(ret)
    }

    pub(crate) fn get_state(&self) -> Result<DhcpState, NmError> {
        match self.share_data.lock() {
            Ok(data) => Ok(data.state.clone()),
            Err(e) => Err(NmError::new(
                ErrorKind::Bug,
                format!("Failed to lock on NmDhcpV4Worker share data: {e}"),
            )),
        }
    }
}

async fn dhcp_thread(
    mut dhcp_client: DhcpV4Client,
    base_iface: BaseInterface,
    mut quit_indicator: Receiver<()>,
    share_data: Arc<Mutex<NmDhcpShareData>>,
) -> Result<(), NmError> {
    // TODO(Gris Ge): Wait link carrier
    match share_data.lock() {
        Ok(mut share_data) => {
            share_data.state = DhcpState::Running;
        }
        Err(e) => {
            return Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "Failed to lock DHCPv4 {}({}) share data: {e}",
                    base_iface.name, base_iface.iface_type,
                ),
            ));
        }
    }
    if let Err(e) = loop {
        tokio::select! {
            result = dhcp_client.run() => {
                match result {
                    Ok(DhcpV4State::Done(lease)) => {
                        log::info!(
                            "DHCPv4 on {}({}) got lease {}",
                            base_iface.name,
                            base_iface.iface_type,
                            lease.yiaddr,
                        );
                        match share_data.lock() {
                            Ok(mut share_data) => {
                                share_data.state = DhcpState::Done;
                            }
                            Err(e) => {
                                break Err::<(), NmError>(NmError::new(
                                    ErrorKind::Bug,
                                    format!("Unhandled DHCPv4 error: {e}"),
                                ));
                            }
                        }
                        if let Err(e) = apply_lease(&base_iface, &lease).await {
                            break Err(e);
                        }
                    }
                    Ok(dhcp_state) => {
                        log::info!(
                            "DHCPv4 on {}({}) reach {} state",
                            base_iface.name,
                            base_iface.iface_type,
                            dhcp_state
                        );
                    }
                    Err(e) => {
                        break Err(NmError::new(
                            ErrorKind::Bug,
                            format!("Unhandled DHCPv4 error: {e}"),
                        ));
                    }
                }
            }
            _ = quit_indicator.recv() => {
                log::info!(
                    "DHCPv4 on {}({}) stopped",
                    base_iface.name,
                    base_iface.iface_type,
                );
                return Ok(());
            }
        }
    } {
        match share_data.lock() {
            Ok(mut share_data) => {
                share_data.state = DhcpState::Error(e.to_string());
            }
            Err(e) => {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Failed to lock DHCPv4 {}({}) share data: {e}",
                        base_iface.name, base_iface.iface_type,
                    ),
                ));
            }
        }
    }
    Ok(())
}

async fn apply_lease(
    _base_iface: &BaseInterface,
    _lease: &DhcpV4Lease,
) -> Result<(), NmError> {
    todo!()
}

pub(crate) async fn apply_dhcp_config(
    _merged_state: &MergedNetworkState,
    _share_data: Arc<Mutex<NmDaemonShareData>>,
) -> Result<(), NmError> {
    todo!()
}
