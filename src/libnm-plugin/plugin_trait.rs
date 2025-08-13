// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use nm::{NmError, NmIpcConnection};

use crate::{NmIpcListener, NmPluginClient, NmPluginCmd, NmPluginInfo};

pub trait NmPlugin: Send + Sync + Sized + 'static {
    const PLUGIN_NAME: &'static str;

    fn init() -> impl Future<Output = Result<Self, NmError>> + Send;

    /// Default implementation is `std::process::exit(0)`
    fn quit(_plugin: &Arc<Self>) -> impl Future<Output = ()> + Send {
        async {
            std::process::exit(0);
        }
    }

    fn plugin_info(
        plugin: &Arc<Self>,
    ) -> impl Future<Output = Result<NmPluginInfo, NmError>> + Send;

    /// Process the command from Daemon.
    fn process(
        plugin: &Arc<Self>,
        cmd: NmPluginCmd,
        conn: &mut NmIpcConnection,
    ) -> impl Future<Output = Result<(), NmError>> + Send;

    /// The `&self` will cloned and move to forked thread for each connection.
    fn run() -> impl Future<Output = Result<(), NmError>> + Send {
        let mut log_builder = env_logger::Builder::new();
        log_builder.filter(Some("nm"), log::LevelFilter::Debug);
        log_builder.filter(Some("nm_plugin"), log::LevelFilter::Debug);
        log_builder.filter(
            Some(&format!("NetworkManager-plugin-{}", Self::PLUGIN_NAME)),
            log::LevelFilter::Debug,
        );
        log_builder.init();

        async {
            let plugin = Arc::new(Self::init().await?);

            let socket_path = format!(
                "{}/{}",
                NmPluginClient::DEFAULT_SOCKET_DIR,
                Self::PLUGIN_NAME
            );
            let ipc = NmIpcListener::new(&socket_path)?;
            log::debug!("Listening on {socket_path}");

            loop {
                if let Ok(conn) = ipc.accept().await {
                    log::debug!("Got daemon connection {conn:?}");
                    let plugin_clone = plugin.clone();
                    tokio::spawn(async move {
                        Self::process_connection(plugin_clone, conn).await
                    });
                }
            }
        }
    }

    fn process_connection(
        plugin: Arc<Self>,
        mut conn: NmIpcConnection,
    ) -> impl Future<Output = Result<(), NmError>> + Send {
        async move {
            loop {
                let cmd = conn.recv::<NmPluginCmd>().await?;
                log::debug!("Got {cmd:?} from daemon");
                match cmd {
                    NmPluginCmd::QueryPluginInfo => {
                        conn.send(Self::plugin_info(&plugin).await).await?
                    }
                    NmPluginCmd::Quit => {
                        Self::quit(&plugin).await;
                    }
                    cmd => Self::process(&plugin, cmd, &mut conn).await?,
                }
            }
        }
    }
}
