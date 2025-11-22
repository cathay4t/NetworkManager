// SPDX-License-Identifier: Apache-2.0

use nm::{
    ErrorKind, NetworkState, NmError, NmstateApplyOption, NmstateQueryOption,
};

use super::{NmPluginCmd, NmPluginReply, NmPluginWorker};
use crate::TaskManager;

#[derive(Debug, Clone)]
pub(crate) struct NmPluginManager {
    mgr: TaskManager<NmPluginCmd, NmPluginReply>,
}

impl NmPluginManager {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            mgr: TaskManager::new::<NmPluginWorker>("plugin").await?,
        })
    }

    // TODO: Support redirect logs from plugin to user
    pub(crate) async fn query_network_state(
        &mut self,
        opt: NmstateQueryOption,
    ) -> Result<Vec<NetworkState>, NmError> {
        let reply = self
            .mgr
            .exec(NmPluginCmd::QueryNetworkState(Box::new(opt)))
            .await?;
        if let NmPluginReply::States(s) = reply {
            Ok(s)
        } else {
            Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "NmPluginCmd::QueryNetworkState is not replying with \
                     NmPluginReply::States, but {reply:?}"
                ),
            ))
        }
    }

    // TODO: Support redirect logs from plugin to user
    pub(crate) async fn apply_network_state(
        &mut self,
        state: &NetworkState,
        opt: &NmstateApplyOption,
    ) -> Result<(), NmError> {
        self.mgr
            .exec(NmPluginCmd::ApplyNetworkState(Box::new((
                state.clone(),
                opt.clone(),
            ))))
            .await?;
        Ok(())
    }
}
