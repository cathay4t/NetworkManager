// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NetworkState, NmError, NmstateInterface};

use super::{NmConfCmd, NmConfReply, NmConfWorker};
use crate::TaskManager;

#[derive(Debug, Clone)]
pub(crate) struct NmConfManager {
    mgr: TaskManager<NmConfCmd, NmConfReply>,
}

impl NmConfManager {
    pub(crate) async fn new() -> Result<Self, NmError> {
        Ok(Self {
            mgr: TaskManager::new::<NmConfWorker>("conf").await?,
        })
    }

    /// Override saved state
    pub(crate) async fn save_state(
        &mut self,
        mut state: NetworkState,
    ) -> Result<(), NmError> {
        // Should remove interface index
        for iface in state.ifaces.kernel_ifaces.values_mut() {
            iface.base_iface_mut().iface_index = None;
        }

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
