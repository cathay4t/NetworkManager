// SPDX-License-Identifier: Apache-2.0

mod api;
mod daemon;
mod net_state;
mod plugin;
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
