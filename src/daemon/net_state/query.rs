// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NmError, NmIpcConnection, NmNoDaemon};
use nmstate::{NetworkState, NmstateQueryOption, NmstateStateKind};

use super::super::plugin::NmDaemonPlugins;

pub(crate) async fn query_network_state(
    conn: &mut NmIpcConnection,
    plugins: &NmDaemonPlugins,
    opt: NmstateQueryOption,
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
            Ok(net_state)
        }
        _ => Err(NmError::new(
            ErrorKind::NoSupport,
            format!("Unsupported query option: {}", opt.kind),
        )),
    }
}
