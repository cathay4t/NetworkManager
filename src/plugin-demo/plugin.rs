// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use nm::{
    NetworkState, NmError, NmIpcConnection, NmstateApplyOption,
    NmstateQueryOption,
};
use nm_plugin::{NmPlugin, NmPluginInfo};

#[derive(Debug)]
pub(crate) struct NmPluginDemo;

impl NmPlugin for NmPluginDemo {
    const PLUGIN_NAME: &'static str = "demo";

    async fn init() -> Result<Self, NmError> {
        Ok(Self {})
    }

    async fn plugin_info(_plugin: &Arc<Self>) -> Result<NmPluginInfo, NmError> {
        Ok(NmPluginInfo::new(
            "demo".to_string(),
            "0.1.0".to_string(),
            vec![],
        ))
    }

    async fn query_network_state(
        _plugin: &Arc<Self>,
        opt: NmstateQueryOption,
        conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        conn.log_trace(format!(
            "Demo plugin got query_network_state request with option {opt}"
        ))
        .await;
        Ok(NetworkState::default())
    }

    async fn apply_network_state(
        _plugin: &Arc<Self>,
        desired_state: NetworkState,
        opt: NmstateApplyOption,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        conn.log_trace(format!(
            "Demo plugin got apply_network_state request with state \
             {desired_state} and option {opt}"
        ))
        .await;
        Ok(())
    }
}
