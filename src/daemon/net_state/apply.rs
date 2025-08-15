// SPDX-License-Identifier: Apache-2.0

use nm::{NmError, NmIpcConnection, NmNoDaemon};
use nmstate::{MergedNetworkState, NetworkState, NmstateApplyOption};

use super::{super::plugin::NmDaemonPlugins, query::query_network_state};

pub(crate) async fn apply_network_state(
    conn: &mut NmIpcConnection,
    plugins: &NmDaemonPlugins,
    desired_state: NetworkState,
    opt: NmstateApplyOption,
) -> Result<NetworkState, NmError> {
    conn.log_debug(format!("apply {desired_state} with option {opt}"))
        .await;

    let pre_apply_current_state =
        query_network_state(conn, plugins, Default::default()).await?;

    let merged_state = MergedNetworkState::new(
        desired_state.clone(),
        pre_apply_current_state.clone(),
        opt.clone(),
    )?;

    let apply_state = merged_state.gen_state_for_apply();

    NmNoDaemon::apply_merged_state(&merged_state).await?;

    plugins
        .apply_network_state(&apply_state, &opt, conn)
        .await?;

    if !opt.no_verify {
        let post_apply_current_state =
            query_network_state(conn, plugins, Default::default()).await?;
        conn.log_debug(format!(
            "Post apply network state: {post_apply_current_state}"
        ))
        .await;
        merged_state.verify(&post_apply_current_state)?;
    }

    let diff_state = merged_state
        .gen_state_for_apply()
        .gen_diff(&pre_apply_current_state)?;

    Ok(diff_state)
}
