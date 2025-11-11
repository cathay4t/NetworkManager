// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{
    ErrorKind, Interface, InterfaceType, NetworkState, NmError,
    NmIpcConnection, NmNoDaemon, NmstateInterface, WifiCfgInterface,
    WifiConfig,
};

use crate::{
    NmPluginWifi, dbus::WpaSupDbus, interface::WpaSupInterface,
    network::WpaSupNetwork,
};

impl NmPluginWifi {
    pub(crate) async fn apply(
        &self,
        desired_state: NetworkState,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        let cur_cfgs = self.get_activated_cfgs()?;
        let dbus = WpaSupDbus::new().await?;
        for iface in desired_state.ifaces.iter() {
            match iface {
                Interface::WifiCfg(iface) => {
                    if iface.is_absent() || iface.is_down() {
                        del_network(&dbus, iface, &cur_cfgs).await?;
                        self.del_from_store(iface.name()).await?;
                    } else if iface.is_up() {
                        let wifi_cfg =
                            if let Some(wifi_cfg) = iface.wifi.as_ref() {
                                wifi_cfg
                            } else {
                                return Err(NmError::new(
                                    ErrorKind::InvalidArgument,
                                    format!("WiFi config undefined in {iface}"),
                                ));
                            };
                        self.apply_wifi_cfg(wifi_cfg, &dbus, conn).await?;
                        self.add_to_store(Interface::WifiCfg(iface.clone()))
                            .await?;
                    }
                }
                Interface::WifiPhy(iface) => {
                    if iface.is_down() || iface.is_absent() {
                        dbus.del_iface(iface.name()).await?;
                    }
                }
                _ => {
                    conn.log_warn(format!(
                        "wifi plugin got unsupported interface type {} name {}",
                        iface.iface_type(),
                        iface.name()
                    ))
                    .await;
                }
            }
        }
        Ok(())
    }

    async fn apply_wifi_cfg(
        &self,
        wifi_cfg: &WifiConfig,
        dbus: &WpaSupDbus<'_>,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        if let Some(iface_name) = wifi_cfg.base_iface.as_ref() {
            self.apply_iface_wifi_cfg(iface_name, wifi_cfg, dbus, conn)
                .await
        } else {
            // Enable WIFI on any WIFI interface
            let cur_state =
                NmNoDaemon::query_network_state(Default::default()).await?;

            for iface in cur_state
                .ifaces
                .kernel_ifaces
                .values()
                .filter(|i| i.iface_type() == &InterfaceType::WifiPhy)
            {
                self.apply_iface_wifi_cfg(iface.name(), wifi_cfg, dbus, conn)
                    .await?;
            }
            Ok(())
        }
    }

    async fn apply_iface_wifi_cfg(
        &self,
        iface_name: &str,
        wifi_cfg: &WifiConfig,
        dbus: &WpaSupDbus<'_>,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        let ssid = match wifi_cfg.ssid.as_ref() {
            Some(s) => s,
            None => {
                return Err(NmError::new(
                    ErrorKind::InvalidArgument,
                    format!("SSID undefined in {wifi_cfg}"),
                ));
            }
        };
        let iface_obj_path = match dbus.get_iface_obj_path(iface_name).await? {
            None => dbus.add_iface(iface_name).await?,
            Some(iface_obj_path) => {
                let networks = dbus.get_networks(&iface_obj_path).await?;
                for network in networks {
                    if &network.ssid == ssid {
                        conn.log_debug(format!(
                            "Deactivating existing WIFI network {ssid} on \
                             interface {}",
                            iface_name
                        ))
                        .await;
                        dbus.del_network(
                            iface_obj_path.as_str(),
                            network.obj_path.as_str(),
                        )
                        .await?;
                    }
                }
                iface_obj_path
            }
        };
        conn.log_debug(format!(
            "Adding WIFI network {ssid} to interface {}",
            iface_name
        ))
        .await;
        let network_obj_path = dbus
            .add_network(
                iface_obj_path.as_str(),
                &WpaSupNetwork {
                    ssid: ssid.to_string(),
                    psk: wifi_cfg.password.clone(),
                    ..Default::default()
                },
            )
            .await?;
        dbus.enable_network(network_obj_path.as_str()).await?;
        Ok(())
    }
}

async fn del_network(
    dbus: &WpaSupDbus<'_>,
    iface: &WifiCfgInterface,
    cur_cfgs: &HashMap<String, WifiCfgInterface>,
) -> Result<(), NmError> {
    let iface = if let Some(i) = cur_cfgs.get(iface.name()) {
        i
    } else {
        log::debug!(
            "iface {}/{} does not exist, no need to delete",
            iface.name(),
            InterfaceType::WifiCfg
        );
        return Ok(());
    };
    let wpa_ifaces = dbus.get_ifaces().await?;
    let ssid =
        if let Some(s) = iface.wifi.as_ref().and_then(|w| w.ssid.as_ref()) {
            s
        } else {
            return Ok(());
        };
    if let Some(iface_name) =
        iface.wifi.as_ref().and_then(|w| w.base_iface.as_ref())
    {
        if let Some(wpa_iface) = wpa_ifaces
            .iter()
            .find(|wpa_iface| wpa_iface.iface_name.as_str() == iface_name)
        {
            del_wpa_network(dbus, wpa_iface, ssid).await?;
        }
    } else {
        for wpa_iface in wpa_ifaces.iter() {
            del_wpa_network(dbus, wpa_iface, ssid).await?;
        }
    }
    Ok(())
}
async fn del_wpa_network(
    dbus: &WpaSupDbus<'_>,
    wpa_iface: &WpaSupInterface,
    ssid: &str,
) -> Result<(), NmError> {
    let wpa_networks = dbus.get_networks(wpa_iface.obj_path.as_str()).await?;
    for wpa_network in wpa_networks
        .iter()
        .filter(|wpa_network| wpa_network.ssid.as_str() == ssid)
    {
        dbus.del_network(
            wpa_iface.obj_path.as_str(),
            wpa_network.obj_path.as_str(),
        )
        .await?;
    }
    Ok(())
}
