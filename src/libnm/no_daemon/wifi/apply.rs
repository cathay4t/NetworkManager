// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use super::{NmWpaConn, dbus::NmWpaSupDbus, network::WpaSupNetwork};
use crate::{
    ErrorKind, Interface, InterfaceType, MergedInterfaces, NmError,
    NmstateInterface, WifiConfig,
};

impl NmWpaConn {
    pub(crate) async fn apply(
        ifaces: &[&Interface],
        merged_ifaces: &MergedInterfaces,
    ) -> Result<(), NmError> {
        let dbus = NmWpaSupDbus::new().await?;
        let mut ssids_to_delete: HashSet<&str> = HashSet::new();
        let mut iface_names_to_delete: HashSet<&str> = HashSet::new();
        for iface in ifaces {
            let wifi_cfg = match iface {
                Interface::WifiCfg(iface) => iface.wifi.as_ref(),
                Interface::WifiPhy(iface) => iface.wifi.as_ref(),
                _ => {
                    continue;
                }
            };
            if iface.is_absent() || iface.is_down() {
                if iface.iface_type() == &InterfaceType::WifiPhy {
                    iface_names_to_delete.insert(iface.name());
                } else {
                    let ssid = if let Some(s) =
                        wifi_cfg.as_ref().map(|w| w.ssid.as_str())
                    {
                        s
                    } else {
                        iface.name()
                    };
                    ssids_to_delete.insert(ssid);
                }
            } else if iface.is_up() {
                let Some(wifi_cfg) = wifi_cfg else {
                    continue;
                };
                log::trace!("Applying {wifi_cfg}");
                if iface.iface_type() == &InterfaceType::WifiPhy {
                    add_wifi_cfg(iface.name(), wifi_cfg, &dbus).await?;
                } else if let Some(iface_name) = wifi_cfg.base_iface.as_ref() {
                    add_wifi_cfg(iface_name, wifi_cfg, &dbus).await?;
                } else {
                    // Bind to any WIFI NICs
                    for merged_iface in
                        merged_ifaces.kernel_ifaces.values().filter(|i| {
                            i.for_apply
                                .as_ref()
                                .map(|i| i.is_absent() || i.is_down())
                                != Some(true)
                                && i.merged.iface_type()
                                    == &InterfaceType::WifiPhy
                        })
                    {
                        add_wifi_cfg(
                            merged_iface.merged.name(),
                            wifi_cfg,
                            &dbus,
                        )
                        .await?;
                    }
                }
            } else {
                return Err(NmError::new(
                    ErrorKind::Bug,
                    format!(
                        "NmWpaConn::apply(): Got invalid interface state: \
                         {iface}"
                    ),
                ));
            }
        }

        del_interfaces(&dbus, &iface_names_to_delete).await?;
        del_networks(&dbus, &ssids_to_delete).await?;

        Ok(())
    }
}

async fn del_interfaces(
    dbus: &NmWpaSupDbus<'_>,
    iface_names: &HashSet<&str>,
) -> Result<(), NmError> {
    let ifaces = dbus.get_ifaces().await?;
    let existing_ifaces: Vec<&str> =
        ifaces.iter().map(|i| i.iface_name.as_str()).collect();

    for iface_name in iface_names {
        if existing_ifaces.contains(iface_name) {
            dbus.del_iface(iface_name).await?;
        }
    }
    Ok(())
}

async fn del_networks(
    dbus: &NmWpaSupDbus<'_>,
    ssids: &HashSet<&str>,
) -> Result<(), NmError> {
    let wpa_ifaces = dbus.get_ifaces().await?;
    for wpa_iface in wpa_ifaces {
        let networks = dbus.get_networks(wpa_iface.obj_path.as_str()).await?;
        for network in networks {
            if ssids.contains(network.ssid.as_str()) {
                dbus.del_network(
                    wpa_iface.obj_path.as_str(),
                    network.obj_path.as_str(),
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn add_wifi_cfg(
    iface_name: &str,
    wifi_cfg: &WifiConfig,
    dbus: &NmWpaSupDbus<'_>,
) -> Result<(), NmError> {
    let ssid = wifi_cfg.ssid.as_str();
    let iface_obj_path = match dbus.get_iface_obj_path(iface_name).await? {
        None => dbus.add_iface(iface_name).await?,
        Some(iface_obj_path) => {
            let networks = dbus.get_networks(&iface_obj_path).await?;
            for network in networks {
                if network.ssid == ssid {
                    log::debug!(
                        "Deactivating existing WIFI network {ssid} on \
                         interface {}",
                        iface_name
                    );
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
    log::debug!("Adding WIFI network {ssid} to interface {}", iface_name);
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
    // Wait 2 second for BSS to show up
    // TODO(Gris Ge): Should trigger a scan and wait scan to finish.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // For WPA3, we need to set ieee80211w explicitly
    for bss in dbus.get_bsses(iface_obj_path.as_str()).await? {
        if bss.ssid.as_deref() == Some(wifi_cfg.ssid.as_str()) && bss.is_wpa3()
        {
            log::debug!("Enable WPA3");
            dbus.del_network(
                iface_obj_path.as_str(),
                network_obj_path.as_str(),
            )
            .await?;
            let network_obj_path = dbus
                .add_network(
                    iface_obj_path.as_str(),
                    &WpaSupNetwork {
                        ssid: ssid.to_string(),
                        psk: wifi_cfg.password.clone(),
                        ieee80211w: Some(2),
                        key_mgmt: Some("SAE FT-SAE".to_string()),
                        ..Default::default()
                    },
                )
                .await?;
            dbus.enable_network(network_obj_path.as_str()).await?;
        }
    }

    Ok(())
}
