// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError};
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupInterface {
    pub(crate) obj_path: OwnedObjectPath,
    pub(crate) iface_name: String,
}

impl WpaSupInterface {
    pub(crate) fn new(iface_name: String) -> Self {
        Self {
            iface_name,
            obj_path: OwnedObjectPath::default(),
        }
    }

    pub(crate) fn to_value(&self) -> HashMap<&str, zvariant::Value<'_>> {
        let mut ret = HashMap::new();
        ret.insert("Ifname", zvariant::Value::new(self.iface_name.clone()));
        ret
    }

    pub(crate) fn from_value(
        mut map: HashMap<String, zvariant::OwnedValue>,
        obj_path: OwnedObjectPath,
    ) -> Result<Self, NmError> {
        Ok(Self {
            iface_name: _from_map!(map, "Ifname", String::try_from)?
                .ok_or_else(|| {
                    NmError::new(
                        ErrorKind::Bug,
                        format!(
                            "Ifname does not exist in wpa_spplicant DBUS \
                             interface reply in {:?}",
                            map
                        ),
                    )
                })?,
            obj_path,
        })
    }
}
