// SPDX-License-Identifier: GPL-3.0-or-later

use nm::{ErrorKind, NetworkState, NmClientCmd, NmError, NmIpcConnection};

use crate::{commander::NmCommander, lock::NmLockManager, log_debug, log_info};

pub(crate) async fn process_api_connection(
    mut conn: NmIpcConnection,
    mut commander: NmCommander,
) -> Result<(), NmError> {
    let (peer_uid, peer_pid) = get_peer_info(&conn)?;

    log_debug(
        Some(&mut conn),
        format!("Got connection from PID {peer_pid} UID {peer_uid}"),
    )
    .await;

    loop {
        let cmd = match conn.recv::<NmClientCmd>().await {
            Ok(c) => {
                if let Err(e) = permission_check(&c, peer_uid) {
                    conn.send::<Result<(), NmError>>(Err(e)).await?;
                    continue;
                } else {
                    c
                }
            }
            Err(e) => {
                if e.kind == ErrorKind::IpcClosed {
                    break Ok(());
                }
                conn.send::<Result<(), NmError>>(Err(e)).await?;
                continue;
            }
        };
        match cmd {
            NmClientCmd::Ping => conn.send(Ok("pong".to_string())).await?,
            NmClientCmd::QueryNetworkState(opt) => {
                let result =
                    commander.query_network_state(Some(&mut conn), *opt).await;
                conn.send(result).await?;
            }
            NmClientCmd::ApplyNetworkState(opt) => {
                log_info(
                    Some(&mut conn),
                    format!(
                        "Client process {peer_pid} acquiring lock before \
                         apply state"
                    ),
                )
                .await;
                if let Some(cur_locker) = NmLockManager::cur_locker_pid() {
                    log_info(
                        Some(&mut conn),
                        format!(
                            "Waiting on-going transaction by PID {cur_locker}"
                        ),
                    )
                    .await;
                }

                let lock = NmLockManager::lock(peer_pid).await;
                log_info(
                    Some(&mut conn),
                    format!("Client process {peer_pid} acquired lock"),
                )
                .await;
                let (desired_state, opt) = *opt;
                let result = commander
                    .apply_network_state(Some(&mut conn), desired_state, opt)
                    .await;
                log_info(
                    Some(&mut conn),
                    format!("Client process {peer_pid} released lock"),
                )
                .await;
                drop(lock);
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
//
// Return (uid, pid)
fn get_peer_info(conn: &NmIpcConnection) -> Result<(u32, i32), NmError> {
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

    Ok((credential.uid(), credential.pid()))
}

fn permission_check(
    command: &NmClientCmd,
    peer_uid: u32,
) -> Result<(), NmError> {
    if peer_uid == 0 {
        Ok(())
    } else {
        match command {
            NmClientCmd::Ping => Ok(()),
            NmClientCmd::QueryNetworkState(s) => {
                if s.include_secrets {
                    Err(NmError::new(
                        ErrorKind::PermissionDeny,
                        "Query with secrets included requires root permission"
                            .into(),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Err(NmError::new(
                ErrorKind::PermissionDeny,
                "Command {command} need to root permission".into(),
            )),
        }
    }
}
