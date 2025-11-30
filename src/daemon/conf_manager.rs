// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NetworkState, NmError};

use super::{
    conf_worker::{NmConfCmd, NmConfReply, NmConfWorker},
    worker::NmManager,
};

#[derive(Debug, Clone)]
pub(crate) struct NmConfManager {
    mgr: NmManager<NmConfCmd, NmConfReply>,
}

impl NmConfManager {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            mgr: NmManager::new::<NmConfWorker>("conf").await?,
        })
    }

    pub(crate) async fn save_state(
        &mut self,
        state: NetworkState,
    ) -> Result<(), NmError> {
        self.mgr.exec(NmConfCmd::SaveState(Box::new(state))).await?;
        Ok(())
    }

    pub(crate) async fn query_state(
        &mut self,
    ) -> Result<NetworkState, NmError> {
        let reply = self.mgr.exec(NmConfCmd::QueryState).await?;
        if let NmConfReply::State(s) = reply {
            Ok(*s)
        } else {
            Err(NmError::new(
                ErrorKind::Bug,
                format!(
                    "NmConfCmd::Query is not replying with \
                     NmConfReply::State, but {reply:?}"
                ),
            ))
        }
    }
}
