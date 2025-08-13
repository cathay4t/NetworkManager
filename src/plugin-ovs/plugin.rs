// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use nm::{ErrorKind, NmError, NmIpcConnection};
use nm_plugin::{NmPlugin, NmPluginCmd, NmPluginInfo};

use super::show::query_network_state;

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

    async fn process(
        _plugin: &Arc<Self>,
        cmd: NmPluginCmd,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        match cmd {
            NmPluginCmd::QueryNetworkState(opt) => {
                query_network_state(*opt, conn).await?;
            }
            _ => {
                conn.send::<Result<(), NmError>>(Err(NmError::new(
                    ErrorKind::NoSupport,
                    format!("Unsupported NmPluginCmd {cmd:?}"),
                )))
                .await?;
            }
        }
        Ok(())
    }
}
