// SPDX-License-Identifier: GPL-3.0-or-later

use nm::NmError;

use super::{
    conf::NmConfManager, dhcp::NmDhcpV4Manager, monitor::NmMonitorManager,
    plugin::NmPluginManager,
};

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
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            dhcpv4_manager: NmDhcpV4Manager::new().await?,
            monitor_manager: NmMonitorManager::new().await?,
            conf_manager: NmConfManager::new().await?,
            plugin_manager: NmPluginManager::new().await?,
        })
    }

    pub(crate) async fn load_saved_state(&mut self) -> Result<(), NmError> {
        let saved_state = self.conf_manager.query_state().await?;
        if saved_state.is_empty() {
            log::info!("Saved state is empty");
        } else {
            log::trace!("Loading saved state: {saved_state}");
            self.apply_network_state(None, saved_state, Default::default())
                .await?;
        }
        Ok(())
    }
}
