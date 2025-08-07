// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{CanIpc, NmError, NmIpcConnection};

#[derive(Debug)]
pub struct NmClient {
    pub(crate) ipc: NmIpcConnection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NmClientCmd {
    Ping,
}

impl CanIpc for NmClientCmd {
    fn kind(&self) -> String {
        match self {
            Self::Ping => "ping".to_string(),
        }
    }
}

impl NmClient {
    pub const DEFAULT_SOCKET_PATH: &'static str =
        "/var/run/NetworkManager/daemon_socket";

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
        self.ipc.send(NmClientCmd::Ping).await?;
        self.ipc.recv::<String>().await
    }
}
