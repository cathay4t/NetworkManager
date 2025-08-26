// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{
    InterfaceIpv4, InterfaceIpv6, InterfaceState, InterfaceType, JsonDisplay,
    NmError,
};

#[derive(
    Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonDisplay,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
/// Information shared among all interface types
pub struct BaseInterface {
    pub name: String,
    #[serde(default, rename = "type")]
    pub iface_type: InterfaceType,
    #[serde(default)]
    pub state: InterfaceState,
    /// In which order should this interface been activated. The smallest
    /// number will be activated first.
    /// Undefined or set to 0 when applying desire state means automatically
    /// decide the correct value.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub up_priority: u32,
    /// Controller interface name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    /// Controller interface type.
    /// Optional to define when applying.
    /// Serialize and deserialize to/from `controller-type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_type: Option<InterfaceType>,
    /// MAC address in the format: upper case hex string separated by `:` on
    /// every two characters. Case insensitive when applying.
    /// Serialize and deserialize to/from `mac-address`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    /// MAC address never change after reboots(normally stored in firmware of
    /// network interface). Using the same format as `mac_address` property.
    /// Ignored during apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permanent_mac_address: Option<String>,
    /// Maximum transmission unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u64>,
    /// Minimum MTU allowed. Ignored during apply.
    /// Serialize and deserialize to/from `min-mtu`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mtu: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Maximum MTU allowed. Ignored during apply.
    /// Serialize and deserialize to/from `max-mtu`.
    pub max_mtu: Option<u64>,
    /// IPv4 information.
    /// Hided if interface is not allowed to hold IP information(e.g. port of
    /// bond is not allowed to hold IP information).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<InterfaceIpv4>,
    /// IPv4 information.
    /// Hided if interface is not allowed to hold IP information(e.g. port of
    /// bond is not allowed to hold IP information).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<InterfaceIpv6>,
}

impl BaseInterface {
    pub fn hide_secrets(&mut self) {}

    pub fn merge(&mut self, other: &Self) {
        // Do not allow unknown interface type overriding existing
        // Do not allow ethernet interface type overriding veth
        if !(other.iface_type.is_unknown()
            || (other.iface_type == InterfaceType::Ethernet
                && self.iface_type == InterfaceType::Veth))
        {
            self.iface_type = other.iface_type.clone();
        }
        if other.state != InterfaceState::Unknown {
            self.state = other.state;
        }
        if other.controller.is_some() {
            self.controller.clone_from(&other.controller);
        }
        if other.controller_type.is_some() {
            self.controller_type.clone_from(&other.controller_type);
        }
        if other.mac_address.is_some() {
            self.mac_address.clone_from(&other.mac_address);
        }
        if other.permanent_mac_address.is_some() {
            self.permanent_mac_address
                .clone_from(&other.permanent_mac_address);
        }
        if other.mtu.is_some() {
            self.mtu = other.mtu;
        }
        if other.min_mtu.is_some() {
            self.min_mtu = other.min_mtu;
        }
        if other.max_mtu.is_some() {
            self.max_mtu = other.max_mtu;
        }
        match (self.ipv4.as_mut(), other.ipv4.as_ref()) {
            (None, Some(other_ipv4)) => self.ipv4 = Some(other_ipv4.clone()),
            (Some(self_ipv4), Some(other_ipv4)) => self_ipv4.merge(other_ipv4),
            _ => (),
        }

        match (self.ipv6.as_mut(), other.ipv6.as_ref()) {
            (None, Some(other_ipv6)) => self.ipv6 = Some(other_ipv6.clone()),
            (Some(self_ipv6), Some(other_ipv6)) => self_ipv6.merge(other_ipv6),
            _ => (),
        }
    }

    pub fn sanitize(&mut self, is_desired: bool) -> Result<(), NmError> {
        if let Some(ipv4) = self.ipv4.as_mut() {
            ipv4.sanitize(is_desired)?;
        }
        if let Some(ipv6) = self.ipv6.as_mut() {
            ipv6.sanitize(is_desired)?;
        }
        Ok(())
    }

    pub fn sanitize_for_verify(&mut self) {
        if let Some(ipv4) = self.ipv4.as_mut() {
            ipv4.sanitize_for_verify();
        }
        if let Some(ipv6) = self.ipv6.as_mut() {
            ipv6.sanitize_for_verify();
        }
    }

    pub(crate) fn clone_name_type_only(&self) -> Self {
        Self {
            name: self.name.clone(),
            iface_type: self.iface_type.clone(),
            state: InterfaceState::Up,
            ..Default::default()
        }
    }
}

impl BaseInterface {
    pub fn new(name: String, iface_type: InterfaceType) -> Self {
        Self {
            name,
            iface_type,
            state: InterfaceState::Up,
            ..Default::default()
        }
    }
}

fn is_zero(d: &u32) -> bool {
    *d == 0
}
