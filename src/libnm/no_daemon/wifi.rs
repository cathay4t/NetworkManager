// SPDX-License-Identifier: Apache-2.0

use crate::{BaseInterface, WifiPhyInterface, WifiState};

impl WifiPhyInterface {
    pub(crate) fn new_from_nispor(
        base: BaseInterface,
        np_iface: &nispor::Iface,
    ) -> Self {
        Self {
            base,
            wifi_state: get_wifi_state(np_iface),
        }
    }
}

fn get_wifi_state(np_iface: &nispor::Iface) -> Option<WifiState> {
    let np_wifi = np_iface.wifi.as_ref()?;
    let mut ret = WifiState {
        rx_bitrate_mb: np_wifi.rx_bitrate.map(|r| r / 10),
        tx_bitrate_mb: np_wifi.tx_bitrate.map(|r| r / 10),
        frequency_mhz: np_wifi.frequency,
        generation: np_wifi.generation,
        ssid: np_wifi.ssid.clone(),
        signal_dbm: np_wifi.signal.map(|s| s.into()),
        ..Default::default()
    };
    ret.sanitize_signal();

    Some(ret)
}
