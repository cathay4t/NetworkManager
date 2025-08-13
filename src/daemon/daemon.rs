// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};

use nm::{NmClient, NmError, NmIpcConnection};
use nm_plugin::NmIpcListener;

use super::{
    api::process_api_connection, plugin::NmDaemonPlugins,
    share_data::NmDaemonShareData,
};

#[derive(Debug)]
pub(crate) struct NmDaemon {
    api_ipc: NmIpcListener,
    plugins: NmDaemonPlugins,
    // Daemon will fork(tokio is controlling maximum threads) new thread for
    // each client connection, this share data will shared along all forked
    // threads.
    share_data: Arc<Mutex<NmDaemonShareData>>,
}

impl NmDaemon {
    pub(crate) async fn new() -> Result<Self, NmError> {
        let plugins = NmDaemonPlugins::new().await?;

        let api_ipc = NmIpcListener::new(NmClient::DEFAULT_SOCKET_PATH)?;

        Ok(Self {
            api_ipc,
            plugins,
            share_data: Arc::new(Mutex::new(NmDaemonShareData::new())),
        })
    }

    pub(crate) async fn run(&mut self) -> Result<(), NmError> {
        loop {
            tokio::select! {
                result = self.api_ipc.accept() => {
                    self.handle_api_connection(result).await;
                },
                // TODO(Gris Ge): Handle TERM signal here
                else => break,
            }
        }
        Ok(())
    }

    async fn handle_api_connection(
        &mut self,
        result: Result<NmIpcConnection, NmError>,
    ) {
        match result {
            Ok(conn) => {
                let share_data = self.share_data.clone();
                let plugins = self.plugins.clone();
                tokio::spawn(async move {
                    process_api_connection(conn, share_data, plugins).await
                });
            }
            Err(e) => {
                log::info!("Ignoring failure of accepting API connection: {e}");
            }
        }
    }
}
