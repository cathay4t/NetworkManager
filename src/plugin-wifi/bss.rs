// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use nm::{ErrorKind, NmError, WifiState};
use zvariant::OwnedObjectPath;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupBss {
    pub(crate) obj_path: OwnedObjectPath,
    pub(crate) ssid: Option<String>,
    pub(crate) mode: Option<String>,
    pub(crate) frequency_mhz: Option<u16>,
    pub(crate) signal_dbm: Option<i16>,
    pub(crate) wpa1: Option<WpaSupBssWpa1>,
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
            mode: _from_map!(map, "Mode", String::try_from)?,
            frequency_mhz: _from_map!(map, "Frequency", u16::try_from)?,
            signal_dbm: _from_map!(map, "Signal", i16::try_from)?,
            wpa1: _from_map!(map, "WPA", WpaSupBssWpa1::try_from)?,
            rsn: _from_map!(map, "RSA", WpaSupBssRsn::try_from)?,
        })
    }
}

impl From<WpaSupBss> for WifiState {
    fn from(bss: WpaSupBss) -> WifiState {
        let mut ret = WifiState::default();
        ret.ssid = bss.ssid;
        ret.frequency_mhz = bss.frequency_mhz.map(|f| f.into());
        ret.signal_dbm = bss.signal_dbm;
        ret.sanitize_signal();
        ret
    }
}

fn parse_ssid(value: zvariant::OwnedValue) -> Result<String, NmError> {
    let bytes = Vec::<u8>::try_from(value).map_err(|e| {
        NmError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid SSID in BSS DBUS reply: {e}"),
        )
    })?;

    String::from_utf8(bytes).map_err(|e| {
        NmError::new(
            ErrorKind::InvalidArgument,
            format!("Invalid SSID in BSS DBUS reply, not UTF-8: {e}"),
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct WpaSupBssWpa1 {
    pub(crate) key_management_suits: Option<Vec<String>>,
    pub(crate) pairwise_cipher_suits: Option<Vec<String>>,
    pub(crate) group_cipher: Option<String>,
}

impl TryFrom<zvariant::OwnedValue> for WpaSupBssWpa1 {
    type Error = NmError;

    fn try_from(v: zvariant::OwnedValue) -> Result<Self, NmError> {
        let error_msg = format!(
            "Expecting map for WPA(for WPA1) reply of BSS: but got {v:?}"
        );
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
        })
    }
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
