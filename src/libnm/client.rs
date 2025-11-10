// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    JsonDisplayHideSecrets, LinkEvent, NetworkState, NmCanIpc, NmError,
    NmIpcConnection, NmstateApplyOption, NmstateQueryOption,
};

impl NmCanIpc for NetworkState {
    fn ipc_kind(&self) -> String {
        "network_state".to_string()
    }
}

#[derive(Debug)]
pub struct NmClient {
    pub(crate) ipc: NmIpcConnection,
}

#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, JsonDisplayHideSecrets,
)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum NmClientCmd {
    Ping,
    QueryNetworkState(Box<NmstateQueryOption>),
    ApplyNetworkState(Box<(NetworkState, NmstateApplyOption)>),
    NotifyLinkEvent(Box<LinkEvent>),
}

impl NmCanIpc for NmClientCmd {
    fn ipc_kind(&self) -> String {
        match self {
            Self::Ping => "ping".to_string(),
            Self::QueryNetworkState(_) => "query-network-state".to_string(),
            Self::ApplyNetworkState(_) => "apply-network-state".to_string(),
            Self::NotifyLinkEvent(_) => "notify-link-event".to_string(),
        }
    }
}

impl NmClientCmd {
    pub fn hide_secrets(&mut self) {
        if let NmClientCmd::ApplyNetworkState(cmd) = self {
            cmd.0.hide_secrets();
        }
    }
}

impl NmClient {
    pub const DEFAULT_SOCKET_PATH: &'static str =
        "/var/run/NetworkManager/sockets/daemon";

    /// Create IPC connect to NetworkManager daemon
    pub async fn new() -> Result<Self, NmError> {
        Self::new_with_name("client").await
    }

    pub async fn new_with_name(name: &str) -> Result<Self, NmError> {
        Ok(Self {
            ipc: NmIpcConnection::new_with_path(
                Self::DEFAULT_SOCKET_PATH,
                name,
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

    pub async fn apply_network_state(
        &mut self,
        desired_state: NetworkState,
        option: NmstateApplyOption,
    ) -> Result<NetworkState, NmError> {
        self.ipc
            .send(Ok(NmClientCmd::ApplyNetworkState(Box::new((
                desired_state,
                option,
            )))))
            .await?;
        self.ipc.recv::<NetworkState>().await
    }

    /// Inform daemon with a link event
    pub async fn notify_link_event(
        &mut self,
        link_event: LinkEvent,
    ) -> Result<(), NmError> {
        self.ipc
            .send(Ok(NmClientCmd::NotifyLinkEvent(Box::new(link_event))))
            .await?;
        Ok(())
    }
}
