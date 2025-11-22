// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{
    NetworkState, NmError, NmstateApplyOption, NmstateInterface,
    NmstateQueryOption,
};
use nm_plugin::{NmPluginClient, NmPluginInfo};

#[derive(Debug, Clone)]
pub(crate) struct NmDaemonPlugin {
    pub(crate) name: String,
    pub(crate) plugin_info: NmPluginInfo,
    pub(crate) socket_path: String,
}

impl NmDaemonPlugin {
    // TODO(Gris Ge):
    // * Timeout
    // * Ignore failure of plugins
    pub(crate) async fn query_network_state(
        &self,
        opt: &NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        let mut cli = NmPluginClient::new(&self.socket_path).await?;
        cli.query_network_state(opt.clone()).await
    }

    // TODO(Gris Ge):
    // * Timeout
    // * Ignore failure of plugins
    pub(crate) async fn apply_network_state(
        &self,
        apply_state: &NetworkState,
        opt: &NmstateApplyOption,
    ) -> Result<(), NmError> {
        let mut new_state = NetworkState::new();
        // Include only interfaces supported by plugin
        for iface in apply_state.ifaces.iter() {
            if self.plugin_info.iface_types.contains(iface.iface_type()) {
                new_state.ifaces.push(iface.clone());
            }
        }
        if new_state.is_empty() {
            log::trace!("No state require {} to apply", self.name);
            Ok(())
        } else {
            log::trace!(
                "Plugin {} apply_network_state {}",
                self.name,
                new_state
            );

            let mut cli = NmPluginClient::new(&self.socket_path).await?;
            cli.apply_network_state(new_state, opt.clone()).await
        }
    }
}
