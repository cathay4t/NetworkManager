// SPDX-License-Identifier: Apache-2.0

use nm::{NmError, NmIpcConnection, NmLogEntry};
use nmstate::{NetworkState, NmstateQueryOption};

use super::ovsdb::{ovsdb_is_running, ovsdb_retrieve};

pub(crate) async fn query_network_state(
    _opt: NmstateQueryOption,
    conn: &mut NmIpcConnection,
) -> Result<(), NmError> {
    if !ovsdb_is_running().await {
        conn.log(NmLogEntry::new_info(
            "plugin.ovs".into(),
            "OVS daemon not running".into(),
        ))
        .await?;
        conn.send(Ok(NetworkState::default())).await?;
        return Ok(());
    }

    conn.send(ovsdb_retrieve().await).await?;

    Ok(())
}
