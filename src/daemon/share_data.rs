// SPDX-License-Identifier: GPL-3.0-or-later

use nm::NmError;

use super::{
    conf_manager::NmConfManager, dhcp_manager::NmDhcpV4Manager,
    monitor_manager::NmMonitorManager,
};

/// Share data among all threads of NM daemon
///
/// This struct is safe to clone and move to threads
#[derive(Debug, Clone)]
pub(crate) struct NmDaemonShareData {
    pub(crate) dhcpv4_manager: NmDhcpV4Manager,
    pub(crate) monitor_manager: NmMonitorManager,
    pub(crate) conf_manager: NmConfManager,
}

impl NmDaemonShareData {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            dhcpv4_manager: NmDhcpV4Manager::new().await?,
            monitor_manager: NmMonitorManager::new().await?,
            conf_manager: NmConfManager::new().await?,
        })
    }
}
