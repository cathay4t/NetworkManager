// SPDX-License-Identifier: Apache-2.0

use nm::{ErrorKind, NmError, NmIpcConnection, NmNoDaemon};
use nmstate::{NetworkState, NmstateQueryOption, NmstateStateKind};

pub(crate) async fn query_network_state(
    conn: &mut NmIpcConnection,
    opt: NmstateQueryOption,
) -> Result<(), NmError> {
    conn.log_debug(format!("querying network state with option {opt:?}"))
        .await;
    match opt.kind {
        NmstateStateKind::RunningNetworkState => {
            let net_state = NmNoDaemon::query_network_state(opt).await?;
            // TODO: Merged with DHCP status and other daemon mode specific
            // stuff
            conn.send(Ok(net_state)).await?;
        }
        _ => {
            let e = NmError::new(
                ErrorKind::NoSupport,
                format!("Unsupported query option: {}", opt.kind),
            );
            conn.send::<Result<NetworkState, NmError>>(Err(e)).await?;
        }
    }
    Ok(())
}
