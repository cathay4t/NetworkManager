// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NmError, nmstate::NetworkState};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) struct NmDaemonConfig;

impl NmDaemonConfig {
    const INTERNAL_STATE_DIR: &'static str =
        "/etc/NetworkManager/states/internal";
    const APPLIED_STATE_PATH: &'static str =
        "/etc/NetworkManager/states/internal/applied.yml";

    pub(crate) async fn save_state(
        net_state: &NetworkState,
    ) -> Result<(), NmError> {
        create_instal_state_dir()?;

        let yaml_str = serde_yaml::to_string(&net_state).map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!("Failed to generate YAML for {net_state}: {e}"),
            )
        })?;
        let mut fd = File::create(Self::APPLIED_STATE_PATH).await?;
        fd.write_all(yaml_str.as_bytes()).await?;
        Ok(())
    }

    pub(crate) async fn read_applied_state() -> Result<NetworkState, NmError> {
        let file_path = std::path::Path::new(Self::APPLIED_STATE_PATH);
        if file_path.exists() {
            let mut fd = File::open(Self::APPLIED_STATE_PATH).await?;
            let mut content = vec![];
            fd.read_to_end(&mut content).await?;
            let yaml_str = String::from_utf8(content).map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Corrupted applied state {}, not valid UTF-8 string: \
                         {e}",
                        Self::APPLIED_STATE_PATH
                    ),
                )
            })?;
            serde_yaml::from_str::<NetworkState>(&yaml_str).map_err(|e| {
                NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "Corrupted applied state {}, not NetworkState YAML: \
                         {e}",
                        Self::APPLIED_STATE_PATH
                    ),
                )
            })
        } else {
            Ok(NetworkState::default())
        }
    }
}

fn create_instal_state_dir() -> Result<(), NmError> {
    let dir_path = std::path::Path::new(NmDaemonConfig::INTERNAL_STATE_DIR);
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
