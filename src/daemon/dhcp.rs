// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mozim::{DhcpV4Client, DhcpV4Config, DhcpV4Lease, DhcpV4State};
use nm::{
    BaseInterface, DhcpState, ErrorKind, Interface, InterfaceIpAddr,
    InterfaceIpv4, MergedNetworkState, NetworkState, NmError, NmIpcConnection,
    NmNoDaemon, NmstateApplyOption, NmstateInterface,
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
    pub(crate) fn new() -> Self {
        Self::default()
    }

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

    pub(crate) fn add_dhcp_worker(&mut self, worker: NmDhcpV4Worker) {
        self.workers
            .insert(worker.base_iface.name.to_string(), worker);
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
    _quit_notifer: Sender<()>,
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
            _quit_notifer: sender,
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
        let iface_index = match base_iface.iface_index {
            Some(m) => m,
            None => {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Got no interface index for DHCPv4 on interface {}({})",
                        base_iface.name, base_iface.iface_type
                    ),
                ));
            }
        };
        let mut dhcp_config = DhcpV4Config::new(base_iface.name.as_str());
        dhcp_config
            .set_iface_index(iface_index)
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
    base_iface: &BaseInterface,
    lease: &DhcpV4Lease,
) -> Result<(), NmError> {
    log::debug!(
        "Applying DHCPv4 lease {}/{} to interface {}({})",
        lease.yiaddr,
        lease.prefix_length(),
        base_iface.name,
        base_iface.iface_type
    );

    let mut ip_addr = InterfaceIpAddr::new(
        lease.yiaddr.clone().into(),
        lease.prefix_length(),
    );
    ip_addr.preferred_life_time = Some(format!("{}sec", lease.lease_time_sec));
    ip_addr.valid_life_time = Some(format!("{}sec", lease.lease_time_sec));

    let mut ipv4_conf = InterfaceIpv4::new();
    ipv4_conf.enabled = Some(true);
    ipv4_conf.dhcp = Some(true);
    ipv4_conf.addresses = Some(vec![ip_addr]);

    let mut apply_base_iface = base_iface.clone_name_type_only();

    apply_base_iface.ipv4 = Some(ipv4_conf);
    let iface_state: Interface = apply_base_iface.into();
    let mut net_state = NetworkState::new();
    net_state.ifaces.push(iface_state);

    let apply_opt = NmstateApplyOption::new();
    NmNoDaemon::apply_network_state(net_state, apply_opt).await?;
    Ok(())
}

pub(crate) async fn apply_dhcp_config(
    conn: &mut NmIpcConnection,
    merged_state: &MergedNetworkState,
    mut share_data: NmDaemonShareData,
) -> Result<(), NmError> {
    for merged_iface in merged_state
        .ifaces
        .kernel_ifaces
        .values()
        .filter(|i| i.is_changed())
    {
        let mut apply_iface = match merged_iface.for_apply.as_ref() {
            Some(i) => i.clone(),
            None => {
                continue;
            }
        };
        if apply_iface.base_iface().mac_address.is_none() {
            apply_iface.base_iface_mut().mac_address =
                merged_iface.merged.base_iface().mac_address.clone();
        }
        apply_iface.base_iface_mut().iface_index =
            merged_iface.merged.base_iface().iface_index.clone();
        if apply_iface.is_up() {
            if let Some(dhcp_enabled) =
                apply_iface.base_iface().ipv4.as_ref().map(|i| i.is_auto())
            {
                if dhcp_enabled {
                    conn.log_debug(format!(
                        "Starting DHCPv4 on interface {}({})",
                        apply_iface.name(),
                        apply_iface.iface_type()
                    ))
                    .await;
                    let dhcp_worker =
                        NmDhcpV4Worker::new(apply_iface.base_iface()).await?;
                    share_data.dhcpv4_manager()?.add_dhcp_worker(dhcp_worker);
                } else {
                    conn.log_debug(format!(
                        "Stopping DHCPv4 on interface {}({})",
                        apply_iface.name(),
                        apply_iface.iface_type()
                    ))
                    .await;
                    share_data
                        .dhcpv4_manager()?
                        .remove_dhcp_worker(apply_iface.name());
                }
            }
        } else {
            share_data
                .dhcpv4_manager()?
                .remove_dhcp_worker(apply_iface.name());
        }
    }
    Ok(())
}
