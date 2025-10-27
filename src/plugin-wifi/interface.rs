// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WpaSupInterface {
    pub(crate) iface_name: String,
}

impl WpaSupInterface {
    pub(crate) fn new(iface_name: String) -> Self {
        Self { iface_name }
    }
}

impl TryFrom<zvariant::OwnedValue> for WpaSupInterface {
    type Error = NmError;

    fn try_from(v: zvariant::OwnedValue) -> Result<Self, NmError> {
        let mut map: HashMap<String, zvariant::OwnedValue> =
            v.try_into().map_err(|e| {
                NmError::new(
                    ErrorKind::PluginFailure,
                    format!("Invalid DBUS reply, expecting map, error: {e}"),
                )
            })?;

        let iface_name: String = match map.remove("Ifname") {
            Some(s) => String::try_from(s).map_err(|e| {
                NmError::new(
                    ErrorKind::PluginFailure,
                    format!(
                        "Invalid wpa_supplicant DBUS reply of interface: \
                         expecting `Ifname` property as string: {e}"
                    ),
                )
            })?,
            None => {
                return Err(NmError::new(
                    ErrorKind::PluginFailure,
                    "Invalid wpa_supplicant DBUS reply of network: not found \
                     `Ifname` property"
                        .into(),
                ));
            }
        };
        Ok(Self { iface_name })
    }
}

impl WpaSupInterface {
    pub(crate) fn to_value(&self) -> HashMap<&str, zvariant::Value<'_>> {
        let mut ret = HashMap::new();
        ret.insert("Ifname", zvariant::Value::new(self.iface_name.clone()));
        ret
    }
}
