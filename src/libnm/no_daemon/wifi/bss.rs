// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use zvariant::OwnedObjectPath;

use crate::{ErrorKind, NmError};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupBss {
    pub(crate) obj_path: OwnedObjectPath,
    pub(crate) ssid: Option<String>,
    pub(crate) bssid: Option<Vec<u8>>,
    pub(crate) mode: Option<String>,
    /// Robust Security Network defined by 802.11i, used for WPA2 and WPA3
    pub(crate) rsn: Option<WpaSupBssRsn>,
}

impl WpaSupBss {
    pub(crate) fn from_value(
        mut map: HashMap<String, zvariant::OwnedValue>,
        obj_path: OwnedObjectPath,
    ) -> Result<Self, NmError> {
        Ok(Self {
            obj_path,
            ssid: _from_map!(map, "SSID", parse_ssid)?,
            bssid: _from_map!(map, "BSSID", Vec::<u8>::try_from)?,
            mode: _from_map!(map, "Mode", String::try_from)?,
            rsn: _from_map!(map, "RSN", WpaSupBssRsn::try_from)?,
        })
    }

    pub(crate) fn is_wpa3(&self) -> bool {
        if let Some(key_mgmt_suits) = self
            .rsn
            .as_ref()
            .and_then(|r| r.key_management_suits.as_ref())
        {
            key_mgmt_suits.contains(&"sae".to_string())
                || key_mgmt_suits.contains(&"ft-sae".to_string())
        } else {
            false
        }
    }
}

fn parse_ssid(value: zvariant::OwnedValue) -> Result<String, NmError> {
    let bytes = Vec::<u8>::try_from(value).map_err(|e| {
        NmError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid SSID in wpa_supplicant BSS DBUS reply: {e}"),
        )
    })?;

    String::from_utf8(bytes).map_err(|e| {
        NmError::new(
            ErrorKind::InvalidArgument,
            format!(
                "Invalid SSID in wpa_supplicant BSS DBUS reply, not UTF-8: {e}"
            ),
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupBssRsn {
    pub(crate) key_management_suits: Option<Vec<String>>,
    pub(crate) pairwise_cipher_suits: Option<Vec<String>>,
    pub(crate) group_cipher: Option<String>,
    pub(crate) mgmt_group_cipher: Option<String>,
}

impl TryFrom<zvariant::OwnedValue> for WpaSupBssRsn {
    type Error = NmError;

    fn try_from(v: zvariant::OwnedValue) -> Result<Self, NmError> {
        let error_msg =
            format!("Expecting map for RSN reply of BSS: but got {v:?}");
        let mut map = HashMap::<String, zvariant::OwnedValue>::try_from(v)
            .map_err(|_| NmError::new(ErrorKind::Bug, error_msg))?;
        Ok(Self {
            key_management_suits: _from_map!(
                map,
                "KeyMgmt",
                Vec::<String>::try_from
            )?,
            pairwise_cipher_suits: _from_map!(
                map,
                "Pairwise",
                Vec::<String>::try_from
            )?,
            group_cipher: _from_map!(map, "Group", String::try_from)?,
            mgmt_group_cipher: _from_map!(map, "MgmtGroup", String::try_from)?,
        })
    }
}
