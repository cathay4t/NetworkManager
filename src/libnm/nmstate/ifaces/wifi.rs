// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    BaseInterface, InterfaceType, JsonDisplay, JsonDisplayHideSecrets, NmError,
    NmstateInterface, nmstate::deserializer::option_number_as_string,
};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplayHideSecrets,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
/// WiFi physical interface
pub struct WifiPhyInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi_link: Option<WifiLink>,
}

impl WifiPhyInterface {
    pub fn new(name: String, wifi_link: WifiLink) -> Self {
        Self {
            base: BaseInterface {
                name: name.to_string(),
                iface_type: InterfaceType::WifiPhy,
                ..Default::default()
            },
            wifi_link: Some(wifi_link),
        }
    }
}

impl Default for WifiPhyInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::WifiPhy,
                ..Default::default()
            },
            wifi_link: None,
        }
    }
}

impl NmstateInterface for WifiPhyInterface {
    fn base_iface(&self) -> &BaseInterface {
        &self.base
    }

    fn base_iface_mut(&mut self) -> &mut BaseInterface {
        &mut self.base
    }

    fn is_virtual(&self) -> bool {
        false
    }

    fn sanitize_iface_specfic(
        &mut self,
        _current: Option<&Self>,
    ) -> Result<(), NmError> {
        if let Some(wifi_cfg) = self.wifi_link.as_mut() {
            *wifi_cfg = WifiLink {
                state: Some(WifiState::Completed),
                ssid: wifi_cfg.ssid.clone(),
                ..Default::default()
            }
        }
        Ok(())
    }
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    JsonDisplay,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum WifiState {
    Disconnected,
    Inactive,
    Scanning,
    Authenticating,
    Associating,
    Associated,
    FourWayHandshake,
    GroupHandshake,
    Completed,
    #[default]
    Unknown,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WifiLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WifiState>,
    /// Service Set Identifier(SSID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    /// WiFi generation, e.g. 6 for WiFi-6. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u32>,
    /// WiFi frequency in MHz. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_mhz: Option<u32>,
    /// Receive bitrate in 1mb/s. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_bitrate_mb: Option<u32>,
    /// Transmit bitrate in 1mb/s. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_bitrate_mb: Option<u32>,
    /// Signal in dBm. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_dbm: Option<i16>,
    /// Signal in percentage. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_percent: Option<u8>,
}

// Align with Microsoft `WLAN_ASSOCIATION_ATTRIBUTES`
const NOISE_FLOOR_DBM: i16 = -100;
const SIGNAL_MAX_DBM: i16 = -50;

impl WifiLink {
    /// Use `signal_dbm` to calculate out `signal_percent`
    pub fn sanitize_signal(&mut self) {
        if let Some(s) = self.signal_dbm {
            self.signal_percent = Some(Self::signal_dbm_to_percent(s));
        }
    }

    pub fn signal_dbm_to_percent(dbm: i16) -> u8 {
        let dbm = dbm.clamp(NOISE_FLOOR_DBM, SIGNAL_MAX_DBM);
        (100.0f64 * (NOISE_FLOOR_DBM - dbm) as f64
            / (NOISE_FLOOR_DBM - SIGNAL_MAX_DBM) as f64) as u8
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonDisplayHideSecrets,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
/// Pseudo Interface for WiFi Configuration
pub struct WifiCfgInterface {
    #[serde(flatten)]
    pub base: BaseInterface,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wifi: Option<WifiConfig>,
}

impl WifiCfgInterface {
    pub fn new(base: BaseInterface) -> Self {
        Self {
            base,
            ..Default::default()
        }
    }

    pub fn parent(&self) -> Option<&str> {
        self.wifi.as_ref().and_then(|w| w.base_iface.as_deref())
    }
}

impl Default for WifiCfgInterface {
    fn default() -> Self {
        Self {
            base: BaseInterface {
                iface_type: InterfaceType::WifiCfg,
                ..Default::default()
            },
            wifi: None,
        }
    }
}

impl NmstateInterface for WifiCfgInterface {
    fn base_iface(&self) -> &BaseInterface {
        &self.base
    }

    fn base_iface_mut(&mut self) -> &mut BaseInterface {
        &mut self.base
    }

    fn is_virtual(&self) -> bool {
        true
    }

    fn sanitize_iface_specfic(
        &mut self,
        _current: Option<&Self>,
    ) -> Result<(), NmError> {
        if let Some(wifi_cfg) = self.wifi.as_mut() {
            wifi_cfg.sanitize();
        }
        Ok(())
    }

    fn hide_secrets_iface_specific(&mut self) {
        if let Some(wifi_cfg) = self.wifi.as_mut() {
            wifi_cfg.hide_secrets();
        }
    }

    fn sanitize_current_for_verify_iface_specfic(&mut self) {
        self.hide_secrets_iface_specific()
    }
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    JsonDisplayHideSecrets,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WifiConfig {
    /// SSID (Service Set Identifier)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "option_number_as_string"
    )]
    pub ssid: Option<String>,
    /// Password for authentication
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "option_number_as_string"
    )]
    pub password: Option<String>,
    /// Whether this WiFi configuration only for specified interface or not.
    /// If undefined, it means any WiFi network interface can be used for
    /// connecting this WiFi.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_iface: Option<String>,
}

impl WifiConfig {
    pub(crate) fn sanitize(&mut self) {}

    pub fn hide_secrets(&mut self) {
        if self.password.is_some() {
            self.password =
                Some(crate::NetworkState::HIDE_PASSWORD_STR.to_string());
        }
    }
}

impl std::fmt::Debug for WifiConfig {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", WifiConfigHideSecrets::from(self))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct WifiConfigHideSecrets {
    ssid: Option<String>,
    password: Option<String>,
    base_iface: Option<String>,
}

impl From<&WifiConfig> for WifiConfigHideSecrets {
    fn from(v: &WifiConfig) -> Self {
        let WifiConfig {
            ssid,
            password,
            base_iface,
        } = v.clone();
        Self {
            password: if password.is_some() {
                Some(crate::NetworkState::HIDE_PASSWORD_STR.to_string())
            } else {
                None
            },
            ssid,
            base_iface,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Interface, NetworkState};

    #[test]
    fn test_hide_secrets_in_debug_wificfg() {
        let wifi_cfg = WifiConfig {
            ssid: Some("test-wifi".into()),
            password: Some("12345678".into()),
            ..Default::default()
        };
        let debug_str = format!("{:?}", wifi_cfg);
        println!("Debug string {:?}", debug_str);
        assert!(!debug_str.contains("12345678"));
    }

    #[test]
    fn test_hide_secrets_in_display_wificfg() {
        let wifi_cfg = WifiConfig {
            ssid: Some("test-wifi".into()),
            password: Some("12345678".into()),
            ..Default::default()
        };
        let debug_str = format!("{}", wifi_cfg);
        println!("Display string {}", debug_str);
        assert!(!debug_str.contains("12345678"));
    }

    #[test]
    fn test_hide_secrets_in_display_wifiiface() {
        let wifi_iface = WifiPhyInterface {
            base: Default::default(),
            wifi: Some(WifiConfig {
                ssid: Some("test-wifi".into()),
                password: Some("12345678".into()),
                ..Default::default()
            }),
        };
        let debug_str = format!("{}", wifi_iface);
        println!("Display string {}", debug_str);
        assert!(!debug_str.contains("12345678"));
    }

    #[test]
    fn test_hide_secrets_in_display_iface() {
        let iface = Interface::WifiPhy(Box::new(WifiPhyInterface {
            base: Default::default(),
            wifi: Some(WifiConfig {
                ssid: Some("test-wifi".into()),
                password: Some("12345678".into()),
                ..Default::default()
            }),
        }));
        let debug_str = format!("{}", iface);
        println!("Display string {}", debug_str);
        assert!(!debug_str.contains("12345678"));
    }

    #[test]
    fn test_hide_secrets_in_display_net_state() {
        let iface = Interface::WifiPhy(Box::new(WifiPhyInterface {
            base: Default::default(),
            wifi: Some(WifiConfig {
                ssid: Some("test-wifi".into()),
                password: Some("12345678".into()),
                ..Default::default()
            }),
        }));
        let mut net_state = NetworkState::new();
        net_state.ifaces.push(iface);
        let debug_str = format!("{}", net_state);
        println!("Display string {}", debug_str);
        assert!(!debug_str.contains("12345678"));
    }
}
