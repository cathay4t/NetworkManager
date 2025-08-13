// SPDX-License-Identifier: Apache-2.0

use nmstate::{NetworkState, NmstateQueryOption};
use serde::{Deserialize, Serialize};

use crate::{NmCanIpc, NmError, NmIpcConnection};

impl NmCanIpc for NetworkState {
    fn ipc_kind(&self) -> String {
        "network_state".to_string()
    }
}

#[derive(Debug)]
pub struct NmClient {
    pub(crate) ipc: NmIpcConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NmClientCmd {
    Ping,
    QueryNetworkState(Box<NmstateQueryOption>),
}

impl NmCanIpc for NmClientCmd {
    fn ipc_kind(&self) -> String {
        match self {
            Self::Ping => "ping".to_string(),
            Self::QueryNetworkState(_) => "query-network-state".to_string(),
        }
    }
}

impl NmClient {
    pub const DEFAULT_SOCKET_PATH: &'static str =
        "/var/run/NetworkManager/sockets/daemon";

    /// Create IPC connect to NetworkManager daemon
    pub async fn new() -> Result<Self, NmError> {
        Ok(Self {
            ipc: NmIpcConnection::new_with_path(
                Self::DEFAULT_SOCKET_PATH,
                "client",
                "daemon",
            )
            .await?,
        })
    }

    pub async fn ping(&mut self) -> Result<String, NmError> {
        self.ipc.send(Ok(NmClientCmd::Ping)).await?;
        self.ipc.recv::<String>().await
    }

    pub async fn query_network_state(
        &mut self,
        option: NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        self.ipc
            .send(Ok(NmClientCmd::QueryNetworkState(Box::new(option))))
            .await?;
        self.ipc.recv::<NetworkState>().await
    }
}
