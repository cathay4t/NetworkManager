// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use nm::{
    NmError, NmIpcConnection,
    nmstate::{NetworkState, NmstateQueryOption},
};
use nm_plugin::{NmPlugin, NmPluginInfo};

pub(crate) struct NmPluginOvs {}

impl NmPlugin for NmPluginOvs {
    const PLUGIN_NAME: &'static str = "ovs";

    async fn init() -> Result<Self, NmError> {
        Ok(Self {})
    }

    async fn plugin_info(_plugin: &Arc<Self>) -> Result<NmPluginInfo, NmError> {
        Ok(NmPluginInfo::new(
            "ovs".to_string(),
            "0.1.0".to_string(),
            Vec::new(),
        ))
    }

    async fn query_network_state(
        _plugin: &Arc<Self>,
        _opt: NmstateQueryOption,
        conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        Self::query(conn).await
    }
}
