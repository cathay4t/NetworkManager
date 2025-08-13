// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};

use nm::{ErrorKind, NmClientCmd, NmError, NmIpcConnection};
use nmstate::NetworkState;

use super::{
    net_state::query_network_state, plugin::NmDaemonPlugins,
    share_data::NmDaemonShareData,
};

pub(crate) async fn process_api_connection(
    mut conn: NmIpcConnection,
    _share_data: Arc<Mutex<NmDaemonShareData>>,
    plugins: NmDaemonPlugins,
) -> Result<(), NmError> {
    loop {
        match conn.recv::<NmClientCmd>().await {
            Ok(NmClientCmd::Ping) => conn.send(Ok("pong".to_string())).await?,
            Ok(NmClientCmd::QueryNetworkState(opt)) => {
                query_network_state(&mut conn, &plugins, *opt).await?
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
