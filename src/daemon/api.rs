// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NmClientCmd, NmError, NmIpcConnection};
use nmstate::NetworkState;

use crate::net_state::query_network_state;

pub(crate) async fn process(mut conn: NmIpcConnection) -> Result<(), NmError> {
    loop {
        match conn.recv::<NmClientCmd>().await {
            Ok(NmClientCmd::Ping) => conn.send(Ok("pong".to_string())).await?,
            Ok(NmClientCmd::QueryNetworkState(opt)) => {
                query_network_state(&mut conn, *opt).await?
            }
            Ok(cmd) => {
                conn.send::<Result<NetworkState, NmError>>(Err(NmError::new(
                    ErrorKind::NoSupport,
                    format!("Unsupported request {cmd:?}"),
                )))
                .await?;
            }
            Err(e) => conn.send::<Result<(), NmError>>(Err(e)).await?,
        }
    }
}
