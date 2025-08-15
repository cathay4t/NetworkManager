// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use nm::{
    ErrorKind, NmError, NmIpcConnection,
    nmstate::{NetworkState, NmstateApplyOption, NmstateQueryOption},
};

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

        // TODO(Gris Ge): Do we need to ping daemon to make sure daemon is
        // still alive?
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
                    log::debug!("Got daemon connection");
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
                log::debug!("Got {cmd} from daemon");
                match cmd {
                    NmPluginCmd::QueryPluginInfo => {
                        conn.send(Self::plugin_info(&plugin).await).await?
                    }
                    NmPluginCmd::Quit => {
                        Self::quit(&plugin).await;
                    }
                    NmPluginCmd::QueryNetworkState(opt) => {
                        let result =
                            Self::query_network_state(&plugin, *opt, &mut conn)
                                .await;
                        conn.send(result).await?
                    }
                    NmPluginCmd::ApplyNetworkState(opt) => {
                        let (desired_state, opt) = *opt;
                        let result = Self::apply_network_state(
                            &plugin,
                            desired_state,
                            opt,
                            &mut conn,
                        )
                        .await;
                        conn.send(result).await?
                    }
                }
            }
        }
    }

    /// Return network state managed by this plugin only.
    /// Optionally, you may send log via `conn::log_debug()` and etc.
    /// Default implementation is return no support error.
    fn query_network_state(
        _plugin: &Arc<Self>,
        _opt: NmstateQueryOption,
        _conn: &mut NmIpcConnection,
    ) -> impl Future<Output = Result<NetworkState, NmError>> + Send {
        async {
            Err(NmError::new(
                ErrorKind::NoSupport,
                format!(
                    "Plugin {} has not implemented query_network_state()",
                    Self::PLUGIN_NAME
                ),
            ))
        }
    }

    /// Apply network state managed by this plugin only.
    /// Optionally, you may send log via `conn::log_debug()` and etc.
    fn apply_network_state(
        _plugin: &Arc<Self>,
        _desired_state: NetworkState,
        _opt: NmstateApplyOption,
        _conn: &mut NmIpcConnection,
    ) -> impl Future<Output = Result<(), NmError>> + Send {
        async {
            Err(NmError::new(
                ErrorKind::NoSupport,
                format!(
                    "Plugin {} has not implemented apply_network_state()",
                    Self::PLUGIN_NAME
                ),
            ))
        }
    }
}
