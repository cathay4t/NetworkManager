// SPDX-License-Identifier: Apache-2.0

use nm::{NmClient, NmError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), NmError> {
    enable_logging();

    let mut cli = NmClient::new().await?;
    println!("HAHA {:?}", cli.ping().await?);
    Ok(())
}

fn enable_logging() {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter(None, log::LevelFilter::Debug);
    log_builder.init();
}
