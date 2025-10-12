// SPDX-License-Identifier: GPL-3.0-or-later

mod api;
mod dhcp;
mod apply;
mod config;
mod daemon;
mod plugin;
mod query;
mod share_data;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), nm::NmError> {
    enable_logging();

    let mut daemon = self::daemon::NmDaemon::new().await?;

    daemon.run().await?;

    Ok(())
}

fn enable_logging() {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter(Some("nm"), log::LevelFilter::Trace);
    log_builder.filter(Some("NetworkManager"), log::LevelFilter::Trace);
    log_builder.init();
}
