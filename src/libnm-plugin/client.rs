// SPDX-License-Identifier: Apache-2.0

use nm::{NmCanIpc, NmError, NmIpcConnection};
use nmstate::{
    JsonDisplay, NetworkState, NmstateApplyOption, NmstateQueryOption,
};
use serde::{Deserialize, Serialize};

use crate::NmPluginInfo;

#[derive(Debug)]
pub struct NmPluginClient {
    pub(crate) ipc: NmIpcConnection,
}

/// Command send from daemon to plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonDisplay)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NmPluginCmd {
    /// Query plugin info, should reply with [NmPluginInfo]
    QueryPluginInfo,
    /// Query network state, should reply with [NetworkState]
    QueryNetworkState(Box<NmstateQueryOption>),
    ApplyNetworkState(Box<(NetworkState, NmstateApplyOption)>),
    Quit,
}

impl NmCanIpc for NmPluginCmd {
    fn ipc_kind(&self) -> String {
        match self {
            Self::QueryPluginInfo => "query-plugin-info".to_string(),
            Self::QueryNetworkState(_) => "query-network-state".to_string(),
            Self::ApplyNetworkState(_) => "apply-network-state".to_string(),
            Self::Quit => "quit".to_string(),
        }
    }
}

impl NmPluginClient {
    pub const DEFAULT_SOCKET_DIR: &'static str =
        "/var/run/NetworkManager/sockets/plugin";

    /// Create IPC connect from daemon to plugin
    pub async fn new(socket_path: &str) -> Result<Self, NmError> {
        let dst_name = std::path::Path::new(socket_path)
            .file_name()
            .and_then(|p| p.to_str())
            .unwrap_or("plugin");
        Ok(Self {
            ipc: NmIpcConnection::new_with_path(
                socket_path,
                "daemon",
                dst_name,
            )
            .await?,
        })
    }

    pub async fn query_plugin_info(&mut self) -> Result<NmPluginInfo, NmError> {
        self.ipc.send(Ok(NmPluginCmd::QueryPluginInfo)).await?;
        self.ipc.recv::<NmPluginInfo>().await
    }

    pub async fn query_network_state(
        &mut self,
        opt: NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        self.ipc
            .send(Ok(NmPluginCmd::QueryNetworkState(Box::new(opt))))
            .await?;
        self.ipc.recv::<NetworkState>().await
    }

    pub async fn apply_network_state(
        &mut self,
        desired_state: NetworkState,
        opt: NmstateApplyOption,
    ) -> Result<(), NmError> {
        self.ipc
            .send(Ok(NmPluginCmd::ApplyNetworkState(Box::new((
                desired_state,
                opt,
            )))))
            .await?;
        self.ipc.recv::<()>().await
    }

    pub async fn send<T>(
        &mut self,
        data: Result<T, NmError>,
    ) -> Result<(), NmError>
    where
        T: NmCanIpc,
    {
        self.ipc.send::<T>(data).await
    }

    pub async fn recv<T>(&mut self) -> Result<T, NmError>
    where
        T: NmCanIpc,
    {
        self.ipc.recv::<T>().await
    }
}
