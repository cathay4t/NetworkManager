// SPDX-License-Identifier: Apache-2.0

use super::inter_ifaces::apply_ifaces;
use crate::{
    ErrorKind, MergedNetworkState, NetworkState, NmError, NmNoDaemon,
    NmstateApplyOption,
};

const RETRY_COUNT: usize = 10;
const RETRY_INTERVAL_MS: u64 = 500;

impl NmNoDaemon {
    pub async fn apply_network_state(
        desired_state: NetworkState,
        option: NmstateApplyOption,
    ) -> Result<NetworkState, NmError> {
        if option.version != 1 {
            return Err(NmError::new(
                ErrorKind::InvalidSchemaVersion,
                format!(
                    "Only support version 1, but desired {}",
                    option.version
                ),
            ));
        }
        let current_state =
            Self::query_network_state(Default::default()).await?;

        log::debug!("Current state {current_state}");
        log::debug!("Applying {desired_state} with option {option}");
        let merged_state = MergedNetworkState::new(
            desired_state.clone(),
            current_state.clone(),
            option.clone(),
        )?;

        Self::apply_merged_state(&merged_state).await?;

        let mut result: Result<(), NmError> = Ok(());
        if !option.no_verify {
            for cur_retry_count in 1..(RETRY_COUNT + 1) {
                let post_apply_current_state =
                    Self::query_network_state(Default::default()).await?;
                log::debug!(
                    "Post apply network state: {post_apply_current_state}"
                );
                result = merged_state.verify(&post_apply_current_state);
                if let Err(e) = &result {
                    log::info!(
                        "Retrying({cur_retry_count}/{RETRY_COUNT}) on \
                         verification error: {e}"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(
                        RETRY_INTERVAL_MS,
                    ))
                    .await;
                } else {
                    break;
                }
            }
        }
        result?;

        let diff_state = merged_state
            .gen_state_for_apply()
            .gen_diff(&current_state)?;

        Ok(diff_state)
    }

    pub async fn apply_merged_state(
        merged_state: &MergedNetworkState,
    ) -> Result<(), NmError> {
        apply_ifaces(&merged_state.ifaces).await?;
        Ok(())
    }
}
