// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError};
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, Default)]
pub(crate) struct WpaSupNetwork {
    pub(crate) obj_path: OwnedObjectPath,
    pub(crate) ssid: String,
    pub(crate) psk: Option<String>,
}

impl WpaSupNetwork {
    pub(crate) fn from_value(
        value: zvariant::OwnedValue,
        obj_path: OwnedObjectPath,
    ) -> Result<Self, NmError> {
        let mut map: HashMap<String, zvariant::OwnedValue> =
            value.try_into().map_err(|e| {
                NmError::new(
                    ErrorKind::PluginFailure,
                    format!("Invalid DBUS reply, expecting map, error: {e}"),
                )
            })?;

        let ssid: String = match map.remove("ssid") {
            Some(s) => {
                let ssid = String::try_from(s).map_err(|e| {
                    NmError::new(
                        ErrorKind::PluginFailure,
                        format!(
                            "Invalid wpa_supplicant DBUS reply of network: \
                             expecting `ssid` property as string: {e}"
                        ),
                    )
                })?;
                // wpa_supplicant always add quote to SSID
                match ssid.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    Some(s) => s.to_string(),
                    None => ssid,
                }
            }
            None => {
                return Err(NmError::new(
                    ErrorKind::PluginFailure,
                    "Invalid wpa_supplicant DBUS reply of network: not found \
                     `ssid` property"
                        .into(),
                ));
            }
        };
        Ok(Self {
            ssid,
            psk: None,
            obj_path,
        })
    }

    pub(crate) fn to_value(&self) -> HashMap<&str, zvariant::Value<'_>> {
        let mut ret = HashMap::new();
        ret.insert("ssid", zvariant::Value::new(self.ssid.clone()));
        if let Some(v) = &self.psk {
            ret.insert("psk", zvariant::Value::new(v.clone()));
        }
        ret
    }
}
