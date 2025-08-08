// SPDX-License-Identifier: Apache-2.0

mod api;
mod listener;
mod net_state;

use nm::{NmClient, NmError};

use self::listener::NmIpcListener;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), NmError> {
    enable_logging();

    let listener = NmIpcListener::new(NmClient::DEFAULT_SOCKET_PATH)?;

    loop {
        let conn = listener.accept().await?;
        tokio::spawn(async move {
            // Process each socket concurrently.
            self::api::process(conn).await
        });
    }
}

fn enable_logging() {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter(Some("nm"), log::LevelFilter::Trace);
    log_builder.init();
}
