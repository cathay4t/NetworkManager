// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    env::current_exe,
    os::unix::fs::{FileTypeExt, PermissionsExt},
};

use futures::{StreamExt, stream::FuturesUnordered};
use nm::{
    NetworkState, NmError, NmIpcConnection, NmstateApplyOption,
    NmstateInterface, NmstateQueryOption,
};
use nm_plugin::{NmPluginClient, NmPluginInfo};

const NM_PLUGIN_PREFIX: &str = "NetworkManager-plugin-";
const NM_PLUGIN_CONN_RETRY: i8 = 50;
const NM_PLUGIN_CONN_RETRY_INTERVAL_MS: u64 = 200;

fn get_file_paths_in_dir(dir: &str) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        log::debug!("Failed to read dir {dir}: {e}");
                        continue;
                    }
                };
                if !entry.path().is_dir()
                    && let Some(p) = entry.path().to_str()
                {
                    ret.push(p.to_string());
                }
            }
        }
        Err(e) => {
            log::debug!("Failed to read dir {dir}: {e}");
        }
    }
    ret
}

fn get_plugin_files() -> Vec<String> {
    let mut plugins: Vec<String> = Vec::new();

    let search_dir = if let Some(p) = current_exe().ok().and_then(|p| {
        p.parent().and_then(|s| s.to_str()).map(|s| s.to_string())
    }) {
        p
    } else {
        return plugins;
    };

    for file_path in get_file_paths_in_dir(&search_dir) {
        let path = std::path::Path::new(&file_path);
        if is_executable(path)
            && path
                .strip_prefix(&search_dir)
                .ok()
                .and_then(|p| p.to_str())
                .map(|p| p.starts_with(NM_PLUGIN_PREFIX))
                .unwrap_or_default()
        {
            plugins.push(file_path);
        }
    }

    plugins
}

fn is_executable(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| (meta.permissions().mode() & 0o100) > 0)
        .unwrap_or_default()
}

fn is_socket(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.file_type().is_socket())
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub(crate) struct NmDaemonPlugins {
    plugins: HashMap<String, NmDaemonPlugin>,
}

impl NmDaemonPlugins {
    // TODO: start plugin in sandbox?
    pub(crate) async fn new() -> Result<Self, NmError> {
        let plugin_paths = get_plugin_files();

        let mut expected_plugin_count = 0;
        for plugin_path in plugin_paths {
            log::debug!("Starting NetworkManager plugin {}", plugin_path);
            if let Err(e) = std::process::Command::new(&plugin_path).spawn() {
                log::info!("Ignoring plugin {plugin_path} due to error: {e}");
            }
            expected_plugin_count += 1;
        }

        let mut plugins: HashMap<String, NmDaemonPlugin> = HashMap::new();
        let mut retry_left = NM_PLUGIN_CONN_RETRY;

        while plugins.len() < expected_plugin_count && retry_left >= 0 {
            retry_left -= 1;
            connect_plugins(&mut plugins).await;
            tokio::time::sleep(std::time::Duration::from_millis(
                NM_PLUGIN_CONN_RETRY_INTERVAL_MS,
            ))
            .await;
        }
        Ok(Self { plugins })
    }

    pub(crate) async fn query_network_state(
        &self,
        opt: NmstateQueryOption,
        conn: &mut NmIpcConnection,
    ) -> Result<Vec<NetworkState>, NmError> {
        let mut ret = Vec::new();
        // TODO(Gris Ge): Should querying all plugin at the same time instead
        // of one by one.
        for plugin in self.plugins.values() {
            match plugin.query_network_state(&opt).await {
                Ok(net_state) => ret.push(net_state),
                Err(e) => {
                    conn.log_info(e.msg.to_string()).await;
                }
            }
        }

        Ok(ret)
    }

    pub(crate) async fn apply_network_state(
        &self,
        apply_state: &NetworkState,
        opt: &NmstateApplyOption,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        // TODO(Gris Ge): Should request all plugin at the same time instead
        // of one by one.
        let mut result_futures = FuturesUnordered::new();
        for plugin in self.plugins.values() {
            let result_future = plugin.apply_network_state(apply_state, opt);
            result_futures.push(result_future);
        }

        while let Some(result) = result_futures.next().await {
            // It is OK for plugin to fail, verification process will
            // noticed the difference
            if let Err(e) = result {
                conn.log_warn(e.to_string()).await;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NmDaemonPlugin {
    name: String,
    plugin_info: NmPluginInfo,
    socket_path: String,
}

impl NmDaemonPlugin {
    // TODO(Gris Ge):
    // * Timeout
    // * Ignore failure of plugins
    pub(crate) async fn query_network_state(
        &self,
        opt: &NmstateQueryOption,
    ) -> Result<NetworkState, NmError> {
        let mut cli = NmPluginClient::new(&self.socket_path).await?;
        cli.query_network_state(opt.clone()).await
    }

    // TODO(Gris Ge):
    // * Timeout
    // * Ignore failure of plugins
    pub(crate) async fn apply_network_state(
        &self,
        apply_state: &NetworkState,
        opt: &NmstateApplyOption,
    ) -> Result<(), NmError> {
        let mut new_state = NetworkState::new();
        // Include only interfaces supported by plugin
        for iface in apply_state.ifaces.iter() {
            if self.plugin_info.iface_types.contains(iface.iface_type()) {
                new_state.ifaces.push(iface.clone());
            }
        }
        if new_state.is_empty() {
            log::trace!("No state require {} to apply", self.name);
            Ok(())
        } else {
            log::trace!(
                "Plugin {} apply_network_state {}",
                self.name,
                new_state
            );

            let mut cli = NmPluginClient::new(&self.socket_path).await?;
            cli.apply_network_state(new_state, opt.clone()).await
        }
    }
}

async fn connect_plugins(plugins: &mut HashMap<String, NmDaemonPlugin>) {
    for file_path in get_file_paths_in_dir(NmPluginClient::DEFAULT_SOCKET_DIR) {
        let path = std::path::Path::new(&file_path);
        if is_socket(path) {
            if let Ok(mut client) = NmPluginClient::new(&file_path).await {
                match client.query_plugin_info().await {
                    Ok(info) => {
                        log::info!(
                            "Plugin {} version {} connected",
                            info.name,
                            info.version,
                        );
                        plugins.insert(
                            info.name.to_string(),
                            NmDaemonPlugin {
                                name: info.name.to_string(),
                                plugin_info: info,
                                socket_path: file_path,
                            },
                        );
                    }
                    Err(e) => {
                        log::debug!("{e}");
                    }
                }
            }
        }
    }
}
