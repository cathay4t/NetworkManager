// SPDX-License-Identifier: Apache-2.0

use nm::{
    Interface, InterfaceState, NetworkState, NmError, NmIpcConnection,
    NmstateInterface, WifiPhyInterface,
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
            if iface.is_absent() {
                dbus.del_iface(iface.name()).await?;
                self.del_iface_from_store(iface.name(), iface.iface_type())?;
            } else if iface.is_down() {
                dbus.del_iface(iface.name()).await?;
                self.set_iface_state_in_store(
                    iface.name(),
                    InterfaceState::Down,
                )?;
            } else if iface.is_up() {
                match iface {
                    Interface::WifiPhy(wifi_iface) => {
                        self.apply_wifiphy(wifi_iface, &dbus, conn).await?;
                        self.add_iface_to_store(iface.clone())?;
                    }
                    _ => {
                        conn.log_warn(format!(
                            "wifi plugin got unsupported interface type {} \
                             name {}",
                            iface.iface_type(),
                            iface.name()
                        ))
                        .await;
                    }
                }
            } else {
                conn.log_warn(format!(
                    "Invalid interface {}/{} state {}",
                    iface.name(),
                    iface.iface_type(),
                    iface.iface_state()
                ))
                .await;
            }
        }
        Ok(())
    }

    async fn apply_wifiphy(
        &self,
        iface: &WifiPhyInterface,
        dbus: &WpaSupDbus<'_>,
        conn: &mut NmIpcConnection,
    ) -> Result<(), NmError> {
        if let Some(wifi_cfg) = iface.wifi.as_ref()
            && let Some(ssid) = wifi_cfg.ssid.as_ref()
        {
            let iface_obj_path =
                match dbus.get_iface_obj_path(iface.name()).await? {
                    None => dbus.add_iface(iface.name()).await?,
                    Some(iface_obj_path) => {
                        let networks =
                            dbus.get_networks(&iface_obj_path).await?;
                        println!("HAHA238 {:?}", networks);
                        for network in networks {
                            if &network.ssid == ssid {
                                conn.log_debug(format!(
                                    "Removing existing WIFI network {ssid} \
                                     from interface {}",
                                    iface.name()
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
                iface.name()
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
        }
        Ok(())
    }
}
