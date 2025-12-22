// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;

use futures_channel::mpsc::UnboundedSender;
use nm::{
    InterfaceType, NetworkState, NmError, NmNoDaemon, NmstateInterface,
    NmstateQueryOption,
};

use super::{
    conf::NmConfManager, daemon::NmManagerCmd, dhcp::NmDhcpV4Manager,
    monitor::NmMonitorManager, plugin::NmPluginManager,
    udev::udev_net_device_is_initialized,
};

const BOOTUP_NIC_CHECK_MAX_COUNT: u64 = 30;
const BOOTUP_NIC_CHECK_MAX_QUICK: u64 = 10;
// During quick retry, we retry every 0.5 second.
const BOOTUP_NIC_CHECK_INTERVAL_MS_QUICK: u64 = 500;
// After quick retry, we only retry every 10 seconds.
const BOOTUP_NIC_CHECK_INTERVAL_SEC_SLOW: u64 = 10;

/// Commander manages all the task managers.
/// This struct is safe to clone and move to threads
#[derive(Debug, Clone)]
pub(crate) struct NmCommander {
    pub(crate) dhcpv4_manager: NmDhcpV4Manager,
    pub(crate) monitor_manager: NmMonitorManager,
    pub(crate) conf_manager: NmConfManager,
    pub(crate) plugin_manager: NmPluginManager,
}

impl NmCommander {
    pub(crate) async fn new(
        sender: UnboundedSender<NmManagerCmd>,
    ) -> Result<Self, NmError> {
        Ok(Self {
            dhcpv4_manager: NmDhcpV4Manager::new().await?,
            monitor_manager: NmMonitorManager::new(sender.clone()).await?,
            conf_manager: NmConfManager::new().await?,
            plugin_manager: NmPluginManager::new().await?,
        })
    }

    // Workflow:
    //  1. Query current network state.
    //  2. For each non-virtual interface mentioned in saved state, if udev has
    //     it initialized, apply its config.
    //  3. Keep retry with timeout and interval for missing interfaces.
    pub(crate) async fn load_saved_state(&mut self) -> Result<(), NmError> {
        let mut saved_state = self.conf_manager.query_state().await?;
        if saved_state.is_empty() {
            log::info!("Saved state is empty");
        } else {
            log::trace!("Loading saved state: {saved_state}");
            for retry_count in 0..BOOTUP_NIC_CHECK_MAX_COUNT {
                let iface_names = get_initialized_nics(&saved_state).await?;

                let nic_ready_state =
                    remove_ready_state(&mut saved_state, &iface_names);

                if !nic_ready_state.is_empty() {
                    for iface in nic_ready_state.ifaces.iter() {
                        log::info!(
                            "Applying saved state for interface {}/{}",
                            iface.name(),
                            iface.iface_type()
                        );
                    }
                    log::debug!("Applying saved state: {nic_ready_state}");
                    self.apply_network_state(
                        None,
                        nic_ready_state,
                        Default::default(),
                    )
                    .await?;
                    log::info!("Remaining saved state: {saved_state}");
                }
                if saved_state.is_empty() {
                    log::info!("All saved state applied successfully");
                    break;
                }

                if retry_count < BOOTUP_NIC_CHECK_MAX_QUICK {
                    tokio::time::sleep(std::time::Duration::from_millis(
                        BOOTUP_NIC_CHECK_INTERVAL_MS_QUICK,
                    ))
                    .await;
                } else {
                    tokio::time::sleep(std::time::Duration::from_secs(
                        BOOTUP_NIC_CHECK_INTERVAL_SEC_SLOW,
                    ))
                    .await;
                }
            }
        }
        Ok(())
    }
}

async fn get_initialized_nics(
    saved_state: &NetworkState,
) -> Result<Vec<String>, NmError> {
    let cur_state =
        NmNoDaemon::query_network_state(NmstateQueryOption::running()).await?;

    let mut ret = Vec::new();

    // TODO: Handle [InterfaceIdentifier]
    for iface_name in saved_state
        .ifaces
        .kernel_ifaces
        .values()
        .filter(|i| !i.is_virtual())
        .map(|i| i.name())
    {
        if let Some(cur_iface) = cur_state.ifaces.kernel_ifaces.get(iface_name)
            && let Some(cur_iface_index) = cur_iface.base_iface().iface_index
            && udev_net_device_is_initialized(cur_iface_index)
        {
            log::debug!(
                "Got Initialized NIC: {}/{}",
                cur_iface.name(),
                cur_iface.iface_type()
            );
            ret.push(iface_name.to_string());
        }
    }
    Ok(ret)
}

fn remove_ready_state(
    state: &mut NetworkState,
    ready_iface_names: &[String],
) -> NetworkState {
    let mut ret = NetworkState::default();
    // HashSet of `(iface_name, iface_type)`.
    let mut pending_ifaces: HashSet<(String, Option<InterfaceType>)> =
        HashSet::new();
    for iface_name in ready_iface_names {
        if let Some(iface) = state.ifaces.get(iface_name.as_str(), None) {
            if iface.base_iface().controller.is_none() {
                pending_ifaces.insert((iface.name().to_string(), None));
            }
        }
    }

    // Include all virtual interface if not controller or controller has all
    // ports ready
    for iface in state.ifaces.iter().filter(|i| i.is_virtual()) {
        if iface.is_controller() {
            if let Some(ports) = iface.ports()
                && is_all_virtual_or_ready(&ports, ready_iface_names, state)
            {
                pending_ifaces.insert((
                    iface.name().to_string(),
                    Some(iface.iface_type().clone()),
                ));
                for port in ports {
                    pending_ifaces.insert((port.to_string(), None));
                }
            }
        } else {
            pending_ifaces.insert((
                iface.name().to_string(),
                Some(iface.iface_type().clone()),
            ));
        }
    }

    for (iface_name, iface_type) in pending_ifaces.drain() {
        if let Some(iface) = state
            .ifaces
            .remove(iface_name.as_str(), iface_type.as_ref())
        {
            ret.ifaces.push(iface);
        }

        if iface_type.map(|i| i.is_userspace()) != Some(true) {
            ret.routes = state.routes.clone();
            if let Some(routes) = ret.routes.config.as_mut() {
                routes.retain(|r| {
                    r.next_hop_iface.is_some()
                        || r.next_hop_iface.as_ref() == Some(&iface_name)
                });
            }
            if let Some(routes) = state.routes.config.as_mut() {
                routes.retain(|r| {
                    r.next_hop_iface.is_none()
                        || r.next_hop_iface.as_ref() != Some(&iface_name)
                });
            }
        }
    }
    ret
}

fn is_all_virtual_or_ready(
    ports: &[&str],
    ready_iface_names: &[String],
    saved_state: &NetworkState,
) -> bool {
    for port in ports {
        let port = port.to_string();
        if !ready_iface_names.contains(&port)
            && saved_state
                .ifaces
                .kernel_ifaces
                .get(&port)
                .map(|i| i.is_virtual())
                != Some(true)
        {
            return false;
        }
    }
    true
}
