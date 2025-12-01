// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_core::Parseable;
use wl_nl80211::{Nl80211Element, Nl80211Elements};

use super::{bss::WpaSupBss, dbus::NmWpaSupDbus};
use crate::{NmError, NmNoDaemon, WifiConfig};

impl NmNoDaemon {
    /// Run WIFI active scan.
    /// If `iface_name` is None, will scan on all found WIFI interfaces,
    /// otherwise only scan on specified interfaces.
    pub async fn wifi_scan(
        iface_name: Option<&str>,
    ) -> Result<Vec<WifiConfig>, NmError> {
        let mut ret = Vec::new();
        let dbus = NmWpaSupDbus::new().await?;

        let mut filter = nispor::NetStateFilter::minimum();
        filter.iface = Some(nispor::NetStateIfaceFilter::minimum());
        let np_state =
            nispor::NetState::retrieve_with_filter_async(&filter).await?;

        let avaiable_wifi_phys: Vec<&str> = np_state
            .ifaces
            .values()
            .filter_map(|np_iface| {
                if np_iface.iface_type == nispor::IfaceType::Wifi {
                    Some(np_iface.name.as_str())
                } else {
                    None
                }
            })
            .collect();

        let mut bsses = if let Some(iface_name) = iface_name {
            bss_active_scan(&dbus, &[iface_name]).await?
        } else {
            bss_active_scan(&dbus, avaiable_wifi_phys.as_slice()).await?
        };

        for ((iface_name, _ssid), bss) in bsses.drain() {
            let mut wifi_cfg = WifiConfig::from(bss);
            wifi_cfg.base_iface = Some(iface_name);
            ret.push(wifi_cfg);
        }

        Ok(ret)
    }
}

pub(crate) async fn bss_active_scan(
    dbus: &NmWpaSupDbus<'_>,
    ifaces: &[&str],
) -> Result<HashMap<(String, String), WpaSupBss>, NmError> {
    // Interface name to dbus object path map
    let mut iface_obj_paths: HashMap<String, String> = HashMap::new();
    // HashMap key is (iface_name, ssid)
    let mut ret: HashMap<(String, String), WpaSupBss> = HashMap::new();

    for iface in ifaces {
        match dbus.get_iface_obj_path(iface).await? {
            Some(o) => iface_obj_paths.insert(iface.to_string(), o),
            None => iface_obj_paths
                .insert(iface.to_string(), dbus.add_iface(iface).await?),
        };
    }

    for (iface_name, iface_obj_path) in iface_obj_paths.iter() {
        log::debug!("Starting WIFI active scan on {iface_name}");
        dbus.scan(iface_obj_path.as_str()).await?;
    }

    for (iface_name, iface_obj_path) in iface_obj_paths.iter() {
        if dbus.is_iface_scanning(iface_obj_path.as_str()).await? {
            log::debug!("Waiting WIFI scan on {iface_name} to finish");
            dbus.wait_scan(iface_obj_path.as_str()).await?;
        }
        log::debug!("WIFI scan on {iface_name} finished");
    }

    for (iface_name, iface_obj_path) in iface_obj_paths.iter() {
        for mut bss in dbus.get_bsses(iface_obj_path.as_str()).await? {
            bss.iface_name = iface_name.to_string();
            let Some(ssid) = bss.ssid.clone() else {
                continue;
            };
            let key = (iface_name.to_string(), ssid);

            if let Some(ies) = bss.ies.as_ref()
                && let Ok(ies) = Nl80211Elements::parse(ies.as_slice())
            {
                let ies = ies.0;
                if ies
                    .iter()
                    .any(|ie| matches!(ie, Nl80211Element::HeCapability(_)))
                {
                    bss.generation = Some(6);
                } else if ies
                    .iter()
                    .any(|ie| matches!(ie, Nl80211Element::VhtCapability(_)))
                {
                    bss.generation = Some(5);
                } else if ies
                    .iter()
                    .any(|ie| matches!(ie, Nl80211Element::HtCapability(_)))
                {
                    bss.generation = Some(4);
                }
            }

            // only override if better signal
            if let Some(exist_bss) = ret.get_mut(&key) {
                if exist_bss.signal_dbm < bss.signal_dbm {
                    *exist_bss = bss;
                }
            } else {
                ret.insert(key, bss);
            }
        }
    }
    log::trace!("WIFI scan result {:?}", ret);

    Ok(ret)
}
