// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError, WifiState};
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum WpaSupInterfaceState {
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

impl From<String> for WpaSupInterfaceState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "disconnected" => Self::Disconnected,
            "inactive" => Self::Inactive,
            "scanning" => Self::Scanning,
            "authenticating" => Self::Authenticating,
            "associating" => Self::Associating,
            "associated" => Self::Associated,
            "4way_handshake" => Self::FourWayHandshake,
            "group_handshake" => Self::GroupHandshake,
            "completed" => Self::Completed,
            "unknown" => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

impl From<WpaSupInterfaceState> for WifiState {
    fn from(v: WpaSupInterfaceState) -> Self {
        match v {
            WpaSupInterfaceState::Disconnected => Self::Disconnected,
            WpaSupInterfaceState::Inactive => Self::Inactive,
            WpaSupInterfaceState::Scanning => Self::Scanning,
            WpaSupInterfaceState::Authenticating => Self::Authenticating,
            WpaSupInterfaceState::Associating => Self::Associating,
            WpaSupInterfaceState::Associated => Self::Associated,
            WpaSupInterfaceState::FourWayHandshake => Self::FourWayHandshake,
            WpaSupInterfaceState::GroupHandshake => Self::GroupHandshake,
            WpaSupInterfaceState::Completed => Self::Completed,
            WpaSupInterfaceState::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupInterface {
    pub(crate) obj_path: OwnedObjectPath,
    pub(crate) iface_name: String,
    pub(crate) state: WpaSupInterfaceState,
}

impl WpaSupInterface {
    pub(crate) fn new(iface_name: String) -> Self {
        Self {
            iface_name,
            obj_path: OwnedObjectPath::default(),
            state: WpaSupInterfaceState::Unknown,
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
            state: _from_map!(map, "State", String::try_from)?
                .map(WpaSupInterfaceState::from)
                .unwrap_or_default(),
            obj_path,
        })
    }
}
