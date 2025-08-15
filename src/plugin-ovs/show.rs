// SPDX-License-Identifier: Apache-2.0

use nm::{NmError, NmIpcConnection, NmLogEntry};
use nmstate::NetworkState;

use super::{
    ovsdb::{ovsdb_is_running, ovsdb_retrieve},
    plugin::NmPluginOvs,
};

impl NmPluginOvs {
    pub(crate) async fn query(
        conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        if !ovsdb_is_running().await {
            conn.log(NmLogEntry::new_info(
                "plugin.ovs".into(),
                "OVS daemon not running".into(),
            ))
            .await?;
            return Ok(NetworkState::default());
        }

        ovsdb_retrieve().await
    }
}
