// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    BaseInterface, InterfaceType, JsonDisplayHideSecrets, NmError,
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
    pub wifi: Option<WifiConfig>,
}

impl WifiPhyInterface {
    pub fn new(base: BaseInterface) -> Self {
        Self {
            base,
            ..Default::default()
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
            wifi: None,
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
    pub bind_to_iface: Option<String>,
    /// WiFi generation, e.g. 6 for WiFi-6. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u32>,
    /// WiFi frequency, For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<u32>,
    /// Receive bitrate in 1mb/s. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_bitrate_mb: Option<u32>,
    /// Transmit bitrate in 1mb/s. For query only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_bitrate_mb: Option<u32>,
}

impl WifiConfig {
    pub(crate) fn sanitize(&mut self) {
        self.rx_bitrate_mb = None;
        self.tx_bitrate_mb = None;
        self.generation = None;
        self.frequency = None;
    }

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
    bind_to_iface: Option<String>,
    generation: Option<u32>,
    frequency: Option<u32>,
    rx_bitrate_mb: Option<u32>,
    tx_bitrate_mb: Option<u32>,
}

impl From<&WifiConfig> for WifiConfigHideSecrets {
    fn from(v: &WifiConfig) -> Self {
        let WifiConfig {
            ssid,
            password,
            bind_to_iface,
            generation,
            frequency,
            rx_bitrate_mb,
            tx_bitrate_mb,
        } = v.clone();
        Self {
            password: if password.is_some() {
                Some(crate::NetworkState::HIDE_PASSWORD_STR.to_string())
            } else {
                None
            },
            ssid,
            bind_to_iface,
            generation,
            frequency,
            rx_bitrate_mb,
            tx_bitrate_mb,
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
