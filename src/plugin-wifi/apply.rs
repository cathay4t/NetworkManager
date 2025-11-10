// SPDX-License-Identifier: Apache-2.0

use nm::{
    ErrorKind, Interface, InterfaceType, NetworkState, NmError,
    NmIpcConnection, NmNoDaemon, NmstateInterface, WifiConfig,
};

use crate::{NmPluginWifi, dbus::WpaSupDbus, network::WpaSupNetwork};

impl NmPluginWifi {
    pub(crate) async fn apply(
        &self,
        desired_state: NetworkState,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        let dbus = WpaSupDbus::new().await?;
        for iface in desired_state.ifaces.iter() {
            match iface {
                Interface::WifiCfg(iface) => {
                    if iface.is_absent() || iface.is_down() {
                        dbus.del_iface(iface.name()).await?;
                        self.del_iface_from_store(iface.name()).await?;
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
                        self.add_iface_to_store(Interface::WifiCfg(
                            iface.clone(),
                        ))
                        .await?;
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
