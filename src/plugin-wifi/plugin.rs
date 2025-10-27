// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex, MutexGuard};

use nm::{
    ErrorKind, Interface, InterfaceState, InterfaceType, NetworkState, NmError,
    NmIpcConnection, NmstateApplyOption, NmstateInterface, NmstateQueryOption,
};
use nm_plugin::{NmPlugin, NmPluginInfo};

#[derive(Debug)]
pub(crate) struct NmPluginWifi {
    /// Contains active state(WiFi only)
    pub(crate) active_state: Mutex<NetworkState>,
}

impl NmPluginWifi {
    /// Locked RW access to active_state
    pub(crate) fn active_state<'a>(
        &'a self,
    ) -> Result<MutexGuard<'a, NetworkState>, NmError> {
        self.active_state.lock().map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!("Failed to lock active_state of NmPluginWifi: {e}"),
            )
        })
    }

    pub(crate) fn add_iface_to_store(
        &self,
        iface: Interface,
    ) -> Result<(), NmError> {
        self.active_state()?.ifaces.push(iface);
        Ok(())
    }

    pub(crate) fn del_iface_from_store(
        &self,
        iface_name: &str,
        iface_type: &InterfaceType,
    ) -> Result<(), NmError> {
        self.active_state()?
            .ifaces
            .remove(iface_name, Some(iface_type));
        Ok(())
    }

    /// Silently ignore if such interface does not exist in store
    pub(crate) fn set_iface_state_in_store(
        &self,
        iface_name: &str,
        iface_state: InterfaceState,
    ) -> Result<(), NmError> {
        if let Some(iface) = self
            .active_state()?
            .ifaces
            .get_mut(iface_name, Some(&InterfaceType::WifiPhy))
        {
            iface.base_iface_mut().state = iface_state;
        }
        Ok(())
    }
}

impl NmPlugin for NmPluginWifi {
    const PLUGIN_NAME: &'static str = "wifi";

    async fn init() -> Result<Self, NmError> {
        Ok(Self {
            active_state: Mutex::new(NetworkState::new()),
        })
    }

    async fn plugin_info(_plugin: &Arc<Self>) -> Result<NmPluginInfo, NmError> {
        Ok(NmPluginInfo::new(
            "wifi".to_string(),
            "0.1.0".to_string(),
            vec![InterfaceType::WifiCfg, InterfaceType::WifiPhy],
        ))
    }

    async fn query_network_state(
        plugin: &Arc<Self>,
        _opt: NmstateQueryOption,
        conn: &mut NmIpcConnection,
    ) -> Result<NetworkState, NmError> {
        plugin.query(conn).await
    }

    async fn apply_network_state(
        plugin: &Arc<Self>,
        desired_state: NetworkState,
        _opt: NmstateApplyOption,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        plugin.apply(desired_state, conn).await
    }
}
