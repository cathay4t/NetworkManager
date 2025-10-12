// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex};

use nm::{ErrorKind, NetworkState, NmClientCmd, NmError, NmIpcConnection};

use super::{
    apply::apply_network_state, plugin::NmDaemonPlugins,
    query::query_network_state, share_data::NmDaemonShareData,
};

pub(crate) async fn process_api_connection(
    mut conn: NmIpcConnection,
    share_data: Arc<Mutex<NmDaemonShareData>>,
    plugins: NmDaemonPlugins,
) -> Result<(), NmError> {
    loop {
        match conn.recv::<NmClientCmd>().await {
            Ok(NmClientCmd::Ping) => conn.send(Ok("pong".to_string())).await?,
            Ok(NmClientCmd::QueryNetworkState(opt)) => {
                let result = query_network_state(
                    &mut conn,
                    &plugins,
                    *opt,
                    share_data.clone(),
                )
                .await;
                conn.send(result).await?;
            }
            Ok(NmClientCmd::ApplyNetworkState(opt)) => {
                let (desired_state, opt) = *opt;
                let result = apply_network_state(
                    &mut conn,
                    &plugins,
                    desired_state,
                    opt,
                    share_data.clone(),
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
