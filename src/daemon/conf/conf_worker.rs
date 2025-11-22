// SPDX-License-Identifier: Apache-2.0

use std::os::unix::fs::PermissionsExt;

use futures::channel::{mpsc::UnboundedReceiver, oneshot::Sender};
use nm::{ErrorKind, InterfaceType, NetworkState, NmError, NmstateInterface};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::TaskWorker;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NmConfCmd {
    /// Override saved network state
    SaveState(Box<NetworkState>),
    QueryState,
}

impl std::fmt::Display for NmConfCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaveState(_) => {
                write!(f, "save-state")
            }
            Self::QueryState => {
                write!(f, "query-state")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NmConfReply {
    None,
    State(Box<NetworkState>),
}

type FromManager = (NmConfCmd, Sender<Result<NmConfReply, NmError>>);

const INTERNAL_STATE_DIR: &str = "/etc/NetworkManager/states/internal";
const APPLIED_STATE_PATH: &str =
    "/etc/NetworkManager/states/internal/applied.yml";

#[derive(Debug)]
pub(crate) struct NmConfWorker {
    receiver: UnboundedReceiver<FromManager>,
    saved_state: NetworkState,
}

impl TaskWorker for NmConfWorker {
    type Cmd = NmConfCmd;
    type Reply = NmConfReply;

    async fn new(
        receiver: UnboundedReceiver<FromManager>,
    ) -> Result<Self, NmError> {
        Ok(Self {
            receiver,
            saved_state: read_state_from_file()?,
        })
    }

    fn receiver(&mut self) -> &mut UnboundedReceiver<FromManager> {
        &mut self.receiver
    }

    async fn process_cmd(
        &mut self,
        cmd: NmConfCmd,
    ) -> Result<NmConfReply, NmError> {
        log::debug!("Processing config command: {cmd}");
        match cmd {
            NmConfCmd::SaveState(mut state) => {
                discard_absent_iface(&mut state);
                save_state_to_file(&state).await?;
                self.saved_state = *state;
                Ok(NmConfReply::None)
            }
            NmConfCmd::QueryState => {
                Ok(NmConfReply::State(Box::new(self.saved_state.clone())))
            }
        }
    }
}

fn read_state_from_file() -> Result<NetworkState, NmError> {
    let content = if std::path::Path::new(APPLIED_STATE_PATH).exists() {
        match std::fs::read_to_string(APPLIED_STATE_PATH) {
            Ok(s) => s,
            Err(e) => {
                log::debug!(
                    "Failed to load saved state from {APPLIED_STATE_PATH}: {e}"
                );
                return Ok(NetworkState::default());
            }
        }
    } else {
        log::debug!("Saved state file {APPLIED_STATE_PATH} does not exist");
        return Ok(NetworkState::default());
    };

    match serde_yaml::from_str::<NetworkState>(&content) {
        Ok(s) => Ok(s),
        Err(e) => {
            log::debug!(
                "Deleting corrupted saved state file {APPLIED_STATE_PATH}: {e}"
            );
            std::fs::remove_file(APPLIED_STATE_PATH).ok();
            Ok(NetworkState::default())
        }
    }
}

async fn save_state_to_file(net_state: &NetworkState) -> Result<(), NmError> {
    create_instal_state_dir()?;
    log::trace!("Saving state {net_state}");

    let yaml_str = serde_yaml::to_string(&net_state).map_err(|e| {
        NmError::new(
            ErrorKind::Bug,
            format!("Failed to generate YAML for {net_state}: {e}"),
        )
    })?;
    // We should remove the file first to make sure newly created
    // `APPLIED_STATE_PATH` is own by daemon uid.
    std::fs::remove_file(APPLIED_STATE_PATH).ok();
    let mut fd = File::create(APPLIED_STATE_PATH).await?;
    fd.set_permissions(PermissionsExt::from_mode(0o600)).await?;
    fd.write_all(yaml_str.as_bytes()).await?;

    Ok(())
}

fn create_instal_state_dir() -> Result<(), NmError> {
    let dir_path = std::path::Path::new(INTERNAL_STATE_DIR);
    if !dir_path.exists() {
        log::debug!("Creating dir {}", dir_path.display());
        std::fs::create_dir_all(dir_path).map_err(|e| {
            NmError::new(
                ErrorKind::DaemonFailure,
                format!("Failed to create dir {}: {e}", dir_path.display()),
            )
        })?;
    }
    Ok(())
}

fn discard_absent_iface(state_to_save: &mut NetworkState) {
    let pending_changes: Vec<(String, InterfaceType)> = state_to_save
        .ifaces
        .iter()
        .filter_map(|i| {
            if i.is_absent() {
                Some((i.name().to_string(), i.iface_type().clone()))
            } else {
                None
            }
        })
        .collect();
    for (iface_name, iface_type) in pending_changes {
        state_to_save
            .ifaces
            .remove(iface_name.as_str(), Some(&iface_type));
    }
}
