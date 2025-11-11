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

        Ok(Self {
            obj_path,
            psk: None,
            ssid: _from_map!(map, "ssid", parse_ssid)?.ok_or_else(|| {
                NmError::new(
                    ErrorKind::Bug,
                    "ssid does not exist in wpa_spplicant DBUS network query \
                     reply"
                        .to_string(),
                )
            })?,
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

fn parse_ssid(value: zvariant::OwnedValue) -> Result<String, NmError> {
    let quoted = String::try_from(value).map_err(|e| {
        NmError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid SSID in wpa_supplicant network DBUS reply: {e}"),
        )
    })?;

    if let Some(s) = quoted.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
    {
        Ok(s.to_string())
    } else {
        Ok(quoted.to_string())
    }
}
