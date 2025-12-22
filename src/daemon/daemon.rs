// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use futures_channel::mpsc::{UnboundedReceiver, unbounded};
use futures_util::stream::StreamExt;
use nm::{ErrorKind, NmClient, NmError, NmIpcConnection};
use nm_plugin::NmIpcListener;

use super::{
    api::process_api_connection, commander::NmCommander, event::NmLinkEvent,
};

#[derive(Debug, Clone)]
pub(crate) enum NmManagerCmd {
    LinkEvent(Box<NmLinkEvent>),
}

#[derive(Debug)]
pub(crate) struct NmDaemon {
    api_ipc: NmIpcListener,
    managers_ipc: UnboundedReceiver<NmManagerCmd>,
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

        let (sender, receiver) = unbounded::<NmManagerCmd>();

        let commander = NmCommander::new(sender).await?;
        // Start a thread to load saved state instead of hanging
        let mut new_commander = commander.clone();
        tokio::spawn(async move {
            if let Err(e) = new_commander.load_saved_state().await {
                log::error!(
                    "Failed to load saved state: {e}, starting with empty \
                     state"
                );
            }
        });

        Ok(Self {
            api_ipc,
            commander,
            managers_ipc: receiver,
        })
    }

    /// Please run this function in a thread
    pub(crate) async fn run(&mut self) {
        loop {
            tokio::select! {
                result = self.api_ipc.accept() => {
                    self.handle_api_connection(result).await;
                },
                cmd = self.managers_ipc.next() => {
                    if let Some(cmd) = cmd {
                        log::trace!("Got command from manager {cmd:?}");
                        if let Err(e) = self.handle_manager_cmd(cmd).await {
                            log::error!("{e}");
                        }
                    }
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

    async fn handle_manager_cmd(
        &mut self,
        cmd: NmManagerCmd,
    ) -> Result<(), NmError> {
        match cmd {
            NmManagerCmd::LinkEvent(event) => {
                self.commander.handle_link_event(*event).await?
            }
        }
        Ok(())
    }
}
