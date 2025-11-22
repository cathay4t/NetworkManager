// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use nm::{ErrorKind, NmClient, NmError, NmIpcConnection};
use nm_plugin::NmIpcListener;

use super::{api::process_api_connection, commander::NmCommander};

#[derive(Debug)]
pub(crate) struct NmDaemon {
    api_ipc: NmIpcListener,
    // Daemon will fork(tokio is controlling maximum threads) new thread for
    // each client connection, this commander will be cloned and move to all
    // forked threads.
    commander: NmCommander,
}

impl NmDaemon {
    pub(crate) async fn new() -> Result<Self, NmError> {
        let api_ipc = NmIpcListener::new(NmClient::DEFAULT_SOCKET_PATH)?;
        // Make the API IPC globally read and writable for non-root user to
        // query and ping
        std::fs::set_permissions(
            NmClient::DEFAULT_SOCKET_PATH,
            Permissions::from_mode(0o0666),
        )
        .map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!(
                    "Failed to set permission of {} to 0666: {e}",
                    NmClient::DEFAULT_SOCKET_PATH
                ),
            )
        })?;

        let mut commander = NmCommander::new().await?;
        if let Err(e) = commander.load_saved_state().await {
            log::error!(
                "Failed to load saved state: {e}, starting with empty state"
            );
        }

        Ok(Self { api_ipc, commander })
    }

    /// Please run this function in a thread
    pub(crate) async fn run(&mut self) {
        loop {
            tokio::select! {
                result = self.api_ipc.accept() => {
                    self.handle_api_connection(result).await;
                },
                // TODO(Gris Ge): Handle TERM signal here:
                //  * Request plugin to quit
                else => break,
            }
        }
    }

    async fn handle_api_connection(
        &mut self,
        result: Result<NmIpcConnection, NmError>,
    ) {
        match result {
            Ok(conn) => {
                let commander = self.commander.clone();
                tokio::spawn(async move {
                    process_api_connection(conn, commander).await
                });
            }
            Err(e) => {
                log::info!("Ignoring failure of accepting API connection: {e}");
            }
        }
    }
}
