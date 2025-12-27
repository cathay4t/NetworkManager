// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{
    BaseInterface, MergedNetworkState, NetworkState, NmError, NmIpcConnection,
    NmstateInterface,
};

use super::{NmDhcpCmd, NmDhcpReply, NmDhcpV4Worker};
use crate::{TaskManager, log_debug};

#[derive(Debug, Clone)]
pub(crate) struct NmDhcpV4Manager {
    mgr: TaskManager<NmDhcpCmd, NmDhcpReply>,
}

// Do not add `async` function to NmDhcpV4Manager because it will be stored
// into Mutex protected `NmDaemonShareData`. The
// `MutexGuard` will cause function not `Send`.
impl NmDhcpV4Manager {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            mgr: TaskManager::new::<NmDhcpV4Worker>("dhcp").await?,
        })
    }

    /// Fill the NetworkState with DHCP states
    pub(crate) async fn fill_dhcp_states(
        &mut self,
        net_state: &mut NetworkState,
    ) -> Result<(), NmError> {
        if let NmDhcpReply::QueryReply(mut dhcp_states) =
            self.mgr.exec(NmDhcpCmd::Query).await?
        {
            for (iface_name, dhcp_state) in dhcp_states.drain() {
                if let Some(iface) =
                    net_state.ifaces.kernel_ifaces.get_mut(iface_name.as_str())
                {
                    let ipv4_conf = iface
                        .base_iface_mut()
                        .ipv4
                        .get_or_insert(Default::default());
                    ipv4_conf.enabled = Some(true);
                    ipv4_conf.dhcp = Some(true);
                    ipv4_conf.dhcp_state = Some(dhcp_state);
                }
            }
        }
        Ok(())
    }

    async fn start_iface_dhcp(
        &mut self,
        base_iface: &BaseInterface,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmDhcpCmd::StartIfaceDhcp(Box::new(base_iface.clone())))
            .await?;
        Ok(())
    }

    async fn stop_iface_dhcp(
        &mut self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmDhcpCmd::StopIfaceDhcp(iface_name.to_string()))
            .await?;
        Ok(())
    }

    // The reason we take full share_data instead of `&mut self` is because
    // Mutex cannot be Send, so it cannot work with async function.
    pub(crate) async fn apply_dhcp_config(
        &mut self,
        mut conn: Option<&mut NmIpcConnection>,
        merged_state: &MergedNetworkState,
    ) -> Result<(), NmError> {
        for merged_iface in merged_state
            .ifaces
            .iter()
            .filter(|i| i.is_changed() && !i.merged.is_userspace())
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
                merged_iface.merged.base_iface().iface_index;
            if apply_iface.is_up() {
                if let Some(dhcp_enabled) =
                    apply_iface.base_iface().ipv4.as_ref().map(|i| i.is_auto())
                {
                    if dhcp_enabled {
                        log_debug(
                            conn.as_deref_mut(),
                            format!(
                                "Starting DHCPv4 on interface {}({})",
                                apply_iface.name(),
                                apply_iface.iface_type()
                            ),
                        )
                        .await;
                        self.start_iface_dhcp(apply_iface.base_iface()).await?;
                    } else {
                        log_debug(
                            conn.as_deref_mut(),
                            format!(
                                "Stopping DHCPv4 on interface {}({})",
                                apply_iface.name(),
                                apply_iface.iface_type()
                            ),
                        )
                        .await;
                        self.stop_iface_dhcp(apply_iface.name()).await?;
                    }
                }
            } else {
                self.stop_iface_dhcp(apply_iface.name()).await?;
            }
        }

        Ok(())
    }
}
