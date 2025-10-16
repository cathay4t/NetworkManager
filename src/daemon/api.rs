// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{ErrorKind, NetworkState, NmClientCmd, NmError, NmIpcConnection};

use super::{
    apply::apply_network_state, plugin::NmDaemonPlugins,
    query::query_network_state, share_data::NmDaemonShareData,
};

pub(crate) async fn process_api_connection(
    mut conn: NmIpcConnection,
    share_data: NmDaemonShareData,
    plugins: NmDaemonPlugins,
) -> Result<(), NmError> {
    let peer_uid = get_peer_uid(&conn)?;

    loop {
        let cmd = match conn.recv::<NmClientCmd>().await {
            Ok(c) => {
                if peer_uid != 0
                    && !matches!(
                        c,
                        NmClientCmd::Ping | NmClientCmd::QueryNetworkState(_)
                    )
                {
                    conn.send::<Result<(), NmError>>(Err(NmError::new(
                        ErrorKind::PermissionDeny,
                        "Need to be root for making changes".into(),
                    )))
                    .await?;
                    continue;
                } else {
                    c
                }
            }
            Err(e) => {
                conn.send::<Result<(), NmError>>(Err(e)).await?;
                continue;
            }
        };
        match cmd {
            NmClientCmd::Ping => conn.send(Ok("pong".to_string())).await?,
            NmClientCmd::QueryNetworkState(opt) => {
                // TODO(Gris Ge): Forbid non-root user to query secrets
                let result = query_network_state(
                    &mut conn,
                    &plugins,
                    *opt,
                    share_data.clone(),
                )
                .await;
                conn.send(result).await?;
            }
            NmClientCmd::ApplyNetworkState(opt) => {
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

// Once https://github.com/rust-lang/rust/issues/76915 goes stable and shipped
// to most distributions, we should use `std::os::unix::net::SocketCred`
fn get_peer_uid(conn: &NmIpcConnection) -> Result<u32, NmError> {
    let credential = nix::sys::socket::getsockopt(
        conn,
        nix::sys::socket::sockopt::PeerCredentials,
    )
    .map_err(|e| {
        NmError::new(
            ErrorKind::Bug,
            format!("Failed to getsockopt SO_PEERCRED failed: {e}"),
        )
    })?;

    Ok(credential.uid())
}
