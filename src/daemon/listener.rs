// SPDX-License-Identifier: Apache-2.0

use std::fs::remove_file;

use nm::{ErrorKind, NmError, NmIpcConnection};
use tokio::net::UnixListener;

#[derive(Debug)]
pub(crate) struct NmIpcListener {
    path: String,
    socket: UnixListener,
}

impl NmIpcListener {
    pub(crate) fn new(path: &str) -> Result<Self, NmError> {
        remove_file(path).ok();

        let dir_path = match std::path::Path::new(path).parent() {
            Some(d) => d,
            None => {
                return Err(NmError::new(
                    ErrorKind::IpcFailure,
                    format!("Failed to find folder path of {path}"),
                ));
            }
        };

        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path).map_err(|e| {
                NmError::new(
                    ErrorKind::IpcFailure,
                    format!("Failed to create dir {}: {e}", dir_path.display()),
                )
            })?;
        }

        Ok(Self {
            path: path.to_string(),
            socket: UnixListener::bind(path).map_err(|e| {
                NmError::new(
                    ErrorKind::IpcFailure,
                    format!("Failed to bind UnixListener to {path}: {e}"),
                )
            })?,
        })
    }

    pub(crate) async fn accept(&self) -> Result<NmIpcConnection, NmError> {
        let (stream, _) = self.socket.accept().await.map_err(|e| {
            NmError::new(
                ErrorKind::IpcFailure,
                format!("Failed to accept socket connection {e}"),
            )
        })?;
        log::trace!("Accepted Unix socket({}) connection", self.path);
        Ok(NmIpcConnection::new_with_stream(stream, "daemon", "client"))
    }
}
