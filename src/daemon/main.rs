// SPDX-License-Identifier: GPL-3.0-or-later

mod api;
mod apply;
mod conf_manager;
mod conf_worker;
mod daemon;
mod dhcp_manager;
mod dhcp_worker;
mod event;
mod monitor_manager;
mod monitor_worker;
mod plugin;
mod query;
mod share_data;
mod worker;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), nm::NmError> {
    enable_logging();

    // According to https://github.com/tokio-rs/tokio/discussions/7091
    // We should not use the main thread for heavy lifting.
    let handle = tokio::spawn(async move {
        match self::daemon::NmDaemon::new().await {
            Ok(mut daemon) => daemon.run().await,
            Err(e) => log::error!("Failed to start daemon {e}"),
        };
    });

    handle
        .await
        .map_err(|e| nm::NmError::new(nm::ErrorKind::Bug, format!("{e}")))
}

fn enable_logging() {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter(Some("nm"), log::LevelFilter::Trace);
    log_builder.filter(Some("NetworkManager"), log::LevelFilter::Trace);
    log_builder.init();
}
