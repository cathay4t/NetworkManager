// SPDX-License-Identifier: Apache-2.0

use crate::{BaseInterface, WifiConfig, WifiPhyInterface};

impl WifiPhyInterface {
    pub(crate) fn new_from_nispor(
        base: BaseInterface,
        np_iface: &nispor::Iface,
    ) -> Self {
        Self {
            base,
            wifi: get_wifi_conf(np_iface),
        }
    }
}

fn get_wifi_conf(np_iface: &nispor::Iface) -> Option<WifiConfig> {
    let np_wifi = np_iface.wifi.as_ref()?;
    let ret = WifiConfig {
        rx_bitrate_mb: np_wifi.rx_bitrate.map(|r| r / 10),
        tx_bitrate_mb: np_wifi.tx_bitrate.map(|r| r / 10),
        frequency: np_wifi.frequency,
        generation: np_wifi.generation,
        ssid: np_wifi.ssid.clone(),
        ..Default::default()
    };

    Some(ret)
}
