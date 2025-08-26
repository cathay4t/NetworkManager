// SPDX-License-Identifier: Apache-2.0

use nm::{NmError, NmIpcConnection, NmNoDaemon};
use nmstate::{MergedNetworkState, NetworkState, NmstateApplyOption};

use super::{
    config::NmDaemonConfig, plugin::NmDaemonPlugins, query::query_network_state,
};

const RETRY_COUNT: usize = 10;
const RETRY_INTERVAL_MS: u64 = 500;

pub(crate) async fn apply_network_state(
    conn: &mut NmIpcConnection,
    plugins: &NmDaemonPlugins,
    desired_state: NetworkState,
    opt: NmstateApplyOption,
) -> Result<NetworkState, NmError> {
    conn.log_debug(format!("apply {desired_state} with option {opt}"))
        .await;

    let mut previous_applied_state =
        NmDaemonConfig::read_applied_state().await?;
    let mut new_desired_state = previous_applied_state.clone();

    new_desired_state.merge(&desired_state)?;

    conn.log_debug(format!(
        "Merged desired state with saved state, got {new_desired_state}"
    ))
    .await;

    let pre_apply_current_state =
        query_network_state(conn, plugins, Default::default()).await?;

    conn.log_debug(format!(
        "Pre-apply current state {pre_apply_current_state}"
    ))
    .await;

    let revert_state =
        new_desired_state.generate_revert(&pre_apply_current_state)?;

    let merged_state = MergedNetworkState::new(
        new_desired_state,
        pre_apply_current_state.clone(),
        opt.clone(),
    )?;

    if let Err(e) = apply(conn, &merged_state, plugins, &opt).await {
        conn.log_warn(format!("Failed to apply desired state: {e}"))
            .await;
        conn.log_warn(format!("Failed to apply merged state: {merged_state}"))
            .await;
        conn.log_warn("Rollback to state before apply".to_string())
            .await;
        conn.log_debug(format!(
            "Rollback to state before apply {revert_state}"
        ))
        .await;
        if let Err(e) = rollback(conn, revert_state, plugins).await {
            log::error!("Failed to rollback: {e}");
        }
        return Err(e);
    }

    let apply_state = merged_state.gen_state_for_apply();
    previous_applied_state.merge(&apply_state)?;
    let new_applied_state = previous_applied_state;

    if let Err(e) = NmDaemonConfig::save_state(&new_applied_state).await {
        conn.log_warn(format!(
            "BUG: Failed to persistent desired state {new_applied_state}: {e}"
        ))
        .await;
    }

    let diff_state = merged_state
        .gen_state_for_apply()
        .gen_diff(&pre_apply_current_state)?;

    Ok(diff_state)
}

async fn apply(
    conn: &mut NmIpcConnection,
    merged_state: &MergedNetworkState,
    plugins: &NmDaemonPlugins,
    opt: &NmstateApplyOption,
) -> Result<(), NmError> {
    let apply_state = merged_state.gen_state_for_apply();

    conn.log_debug(format!("apply_state {apply_state}")).await;

    NmNoDaemon::apply_merged_state(merged_state).await?;
    plugins
        .apply_network_state(&apply_state, opt, conn)
        .await?;

    let mut result: Result<(), NmError> = Ok(());
    if !opt.no_verify {
        for cur_retry_count in 1..(RETRY_COUNT + 1) {
            result = verify(conn, merged_state, plugins).await;
            if let Err(e) = &result {
                conn.log_info(format!(
                    "Retrying({cur_retry_count}/{RETRY_COUNT}) on \
                     verification error: {e}"
                ))
                .await;
                tokio::time::sleep(std::time::Duration::from_millis(
                    RETRY_INTERVAL_MS,
                ))
                .await;
            } else {
                break;
            }
        }
    }
    result
}

async fn rollback(
    conn: &mut NmIpcConnection,
    revert_state: NetworkState,
    plugins: &NmDaemonPlugins,
) -> Result<(), NmError> {
    let mut opt = NmstateApplyOption::default();
    opt.no_verify = true;

    let current_state =
        query_network_state(conn, plugins, Default::default()).await?;
    let merged_state =
        MergedNetworkState::new(revert_state, current_state, opt.clone())?;

    let apply_state = merged_state.gen_state_for_apply();

    NmNoDaemon::apply_merged_state(&merged_state).await?;
    plugins
        .apply_network_state(&apply_state, &opt, conn)
        .await?;

    Ok(())
}

async fn verify(
    conn: &mut NmIpcConnection,
    merged_state: &MergedNetworkState,
    plugins: &NmDaemonPlugins,
) -> Result<(), NmError> {
    let post_apply_current_state =
        query_network_state(conn, plugins, Default::default()).await?;
    conn.log_debug(format!(
        "Post apply network state: {post_apply_current_state}"
    ))
    .await;
    merged_state.verify(&post_apply_current_state)?;
    Ok(())
}
