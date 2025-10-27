// SPDX-License-Identifier: Apache-2.0

use nm::{NetworkState, NmError, NmIpcConnection};

use super::plugin::NmPluginWifi;

impl NmPluginWifi {
    pub(crate) async fn query(
        &self,
        conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        Ok(self.active_state()?.clone())
    }
}
