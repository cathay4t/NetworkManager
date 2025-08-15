// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};

use nm::{ErrorKind, NmClientCmd, NmError, NmIpcConnection};
use nmstate::NetworkState;

use super::{
    net_state::{apply_network_state, query_network_state},
    plugin::NmDaemonPlugins,
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
                let result =
                    query_network_state(&mut conn, &plugins, *opt).await;
                conn.send(result).await?;
            }
            Ok(NmClientCmd::ApplyNetworkState(opt)) => {
                let (desired_state, opt) = *opt;
                let result = apply_network_state(
                    &mut conn,
                    &plugins,
                    desired_state,
                    opt,
                )
                .await;
                conn.send(result).await?;
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
