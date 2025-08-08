// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NmClientCmd, NmError, NmIpcConnection};
use nmstate::NetworkState;

use crate::net_state::query_network_state;

pub(crate) async fn process(mut conn: NmIpcConnection) -> Result<(), NmError> {
    loop {
        let cmd = conn.recv::<NmClientCmd>().await?;
        match cmd {
            NmClientCmd::Ping => conn.send(Ok("pong".to_string())).await?,
            NmClientCmd::QueryNetworkState(opt) => {
                query_network_state(&mut conn, *opt).await?
            }
            _ => {
                conn.send::<Result<NetworkState, NmError>>(Err(NmError::new(
                    ErrorKind::NoSupport,
                    format!("Unsupported request {cmd:?}"),
                )))
                .await?;
            }
        }
    }
}
