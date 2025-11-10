// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use nm::{
    ErrorKind, Interface, InterfaceType, NetworkState, NmError,
    NmIpcConnection, NmstateApplyOption, NmstateInterface, NmstateQueryOption,
    WifiCfgInterface,
};
use nm_plugin::{NmPlugin, NmPluginInfo};

#[derive(Debug, Default)]
pub(crate) struct NmPluginWifiShareData {
    activated_cfgs: HashMap<String, WifiCfgInterface>,
}

#[derive(Debug)]
pub(crate) struct NmPluginWifi {
    share_data: Mutex<NmPluginWifiShareData>,
}

impl NmPluginWifi {
    pub(crate) fn share_data<'a>(
        &'a self,
    ) -> Result<MutexGuard<'a, NmPluginWifiShareData>, NmError> {
        self.share_data.lock().map_err(|e| {
            NmError::new(
                ErrorKind::Bug,
                format!("Failed to lock share_data of NmPluginWifi: {e}"),
            )
        })
    }

    /// Cloned activated WiFi configurations.
    pub(crate) fn get_activated_cfgs(
        &self,
    ) -> Result<HashMap<String, WifiCfgInterface>, NmError> {
        Ok(self.share_data()?.activated_cfgs.clone())
    }

    pub(crate) async fn add_iface_to_store(
        &self,
        iface: Interface,
    ) -> Result<(), NmError> {
        match iface {
            Interface::WifiCfg(iface) => {
                self.share_data()?
                    .activated_cfgs
                    .insert(iface.name().to_string(), *iface);
            }
            _ => {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "NmPluginWifi::add_iface_to_activated_cfgs() got \
                         unexpected interface {iface:?}"
                    ),
                ));
            }
        };
        Ok(())
    }

    pub(crate) async fn del_iface_from_store(
        &self,
        iface_name: &str,
    ) -> Result<(), NmError> {
        self.share_data()?.activated_cfgs.retain(|_, iface| {
            iface
                .wifi
                .as_ref()
                .map(|w| w.base_iface.as_deref() == Some(iface_name))
                != Some(true)
        });
        Ok(())
    }
}

impl NmPlugin for NmPluginWifi {
    const PLUGIN_NAME: &'static str = "wifi";

    async fn init() -> Result<Self, NmError> {
        Ok(Self {
            share_data: Mutex::new(Default::default()),
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
