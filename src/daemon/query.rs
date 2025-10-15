// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{
    ErrorKind, NetworkState, NmError, NmIpcConnection, NmNoDaemon,
    NmstateQueryOption, NmstateStateKind,
};

use super::{
    config::NmDaemonConfig, plugin::NmDaemonPlugins,
    share_data::NmDaemonShareData,
};

pub(crate) async fn query_network_state(
    conn: &mut NmIpcConnection,
    plugins: &NmDaemonPlugins,
    opt: NmstateQueryOption,
    mut share_data: NmDaemonShareData,
) -> Result<NetworkState, NmError> {
    conn.log_debug(format!("querying network state with option {opt}"))
        .await;
    match opt.kind {
        NmstateStateKind::RunningNetworkState => {
            let mut net_state =
                NmNoDaemon::query_network_state(opt.clone()).await?;

            let plugins_net_states =
                plugins.query_network_state(opt, conn).await?;

            for plugins_net_state in plugins_net_states {
                net_state.merge(&plugins_net_state)?;
            }
            share_data.dhcpv4_manager()?.query(&mut net_state)?;

            // TODO: Mark interface/routes not int saved state as ignored.
            Ok(net_state)
        }
        NmstateStateKind::SavedNetworkState => {
            Ok(NmDaemonConfig::read_applied_state().await?)
        }
        _ => Err(NmError::new(
            ErrorKind::NoSupport,
            format!("Unsupported query option: {}", opt.kind),
        )),
    }
}
