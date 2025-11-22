// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{
    ErrorKind, NetworkState, NmError, NmIpcConnection, NmNoDaemon,
    NmstateQueryOption, NmstateStateKind,
};

use super::commander::NmCommander;

impl NmCommander {
    pub(crate) async fn query_network_state(
        &mut self,
        conn: Option<&mut NmIpcConnection>,
        opt: NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        if let Some(conn) = conn {
            conn.log_debug(format!("querying network state with option {opt}"))
                .await;
        } else {
            log::debug!("querying network state with option {opt}");
        }
        match opt.kind {
            NmstateStateKind::RunningNetworkState => {
                let mut net_state =
                    NmNoDaemon::query_network_state(opt.clone()).await?;

                let plugins_net_states = self
                    .plugin_manager
                    .query_network_state(opt.clone())
                    .await?;

                for plugins_net_state in plugins_net_states {
                    net_state.merge(&plugins_net_state)?;
                }
                self.dhcpv4_manager.fill_dhcp_states(&mut net_state).await?;

                if !opt.include_secrets {
                    net_state.hide_secrets();
                }

                // TODO: Mark interface/routes not int saved state as ignored.
                Ok(net_state)
            }
            NmstateStateKind::SavedNetworkState => {
                let mut state = self.conf_manager.query_state().await?;
                if !opt.include_secrets {
                    state.hide_secrets();
                }
                Ok(state)
            }
            _ => Err(NmError::new(
                ErrorKind::NoSupport,
                format!("Unsupported query option: {}", opt.kind),
            )),
        }
    }
}
