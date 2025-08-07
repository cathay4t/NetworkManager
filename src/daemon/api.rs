// SPDX-License-Identifier: Apache-2.0

use nm::{NmClientCmd, NmError, NmIpcConnection};

pub(crate) async fn process(mut conn: NmIpcConnection) -> Result<(), NmError> {
    loop {
        let cmd = conn.recv::<NmClientCmd>().await?;
        match cmd {
            NmClientCmd::Ping => conn.send("pong".to_string()).await?,
        }
    }
}
