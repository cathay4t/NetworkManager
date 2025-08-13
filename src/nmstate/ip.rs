// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{ErrorKind, NmstateError};

const IPV4_ADDR_LEN: usize = 32;
const IPV6_ADDR_LEN: usize = 128;
const FOREVER: &str = "forever";

/// IPv4 configuration of interface.
/// Example YAML output of interface holding static IPv4:
/// ```yaml
/// ---
/// interfaces:
/// - name: eth1
///   state: up
///   mtu: 1500
///   ipv4:
///     address:
///     - ip: 192.0.2.252
///       prefix-length: 24
///     - ip: 192.0.2.251
///       prefix-length: 24
///     dhcp: false
///     enabled: true
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct InterfaceIpv4 {
    /// Whether IPv4 stack is enabled. When set to false, all IPv4 address will
    /// be removed from this interface.
    /// Undefined means true.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub enabled: Option<bool>,
    /// Whether DHCPv4 is enabled.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub dhcp: Option<bool>,
    /// IPv4 addresses.
    /// When applying with `None`, current IP address will be preserved.
    /// When applying with `Some(Vec::new())`, all IP address will be removed.
    /// When switch from DHCP on to off with `addresses` set to None or all
    /// `addresses` are dynamic, nmstate will convert current dynamic IP
    /// address to static.
    /// The IP addresses will apply to kernel with the same order specified
    /// which result the IP addresses after first one holding the `secondary`
    /// flag.
    #[serde(skip_serializing_if = "Option::is_none", rename = "address")]
    pub addresses: Option<Vec<InterfaceIpAddr>>,
}

impl Default for InterfaceIpv4 {
    /// Create [InterfaceIpv4] with IP disabled.
    fn default() -> Self {
        Self {
            enabled: Some(false),
            dhcp: None,
            addresses: None,
        }
    }
}

impl InterfaceIpv4 {
    /// Create [InterfaceIpv4] with IP disabled.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled != Some(false)
    }

    pub(crate) fn is_auto(&self) -> bool {
        self.is_enabled() && self.dhcp == Some(true)
    }

    pub fn is_static(&self) -> bool {
        self.is_enabled()
            && !self.is_auto()
            && !self.addresses.as_deref().unwrap_or_default().is_empty()
    }

    pub(crate) fn merge(&mut self, new: &Self) {
        if new.enabled.is_some() {
            self.enabled = new.enabled;
        }
        if self.dhcp.is_none() && self.is_enabled() {
            self.dhcp = new.dhcp;
        }
        // Normally, we expect backend to preserve configuration which not
        // mentioned in desire or all auto ip address, but when DHCP switch from
        // ON to OFF, the design of nmstate is expecting dynamic IP address goes
        // static. This should be done by top level code.
        if new.is_auto()
            && new.addresses.is_some()
            && self.is_enabled()
            && !self.is_auto()
            && is_ip_addrs_none_or_all_auto(self.addresses.as_deref())
        {
            self.addresses.clone_from(&new.addresses);
            if let Some(addrs) = self.addresses.as_mut() {
                addrs.as_mut_slice().iter_mut().for_each(|a| {
                    a.valid_life_time = None;
                    a.preferred_life_time = None;
                });
            }
        }
    }

    // * Remove auto IP address.
    // * Disable DHCP and remove address if enabled: false
    pub(crate) fn sanitize(
        &mut self,
        is_desired: bool,
    ) -> Result<(), NmstateError> {
        if self.is_auto() {
            if let Some(addrs) = self.addresses.as_ref() {
                if is_desired {
                    for addr in addrs {
                        log::info!(
                            "Static addresses {addr} defined when dynamic IP \
                             is enabled"
                        );
                    }
                }
            }
        }

        if let Some(addrs) = self.addresses.as_mut() {
            if is_desired {
                for addr in addrs.as_slice().iter().filter(|a| a.is_auto()) {
                    log::info!("Ignoring Auto IP address {addr}");
                }
                if let Some(addr) =
                    addrs.as_slice().iter().find(|a| a.ip.is_ipv6())
                {
                    return Err(NmstateError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Got IPv6 address {addr} in ipv4 config section"
                        ),
                    ));
                }
                if let Some(addr) = addrs
                    .iter()
                    .find(|a| a.prefix_length as usize > IPV4_ADDR_LEN)
                {
                    return Err(NmstateError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Invalid IPv4 network prefix length '{}', should \
                             be in the range of 0 to {IPV4_ADDR_LEN}",
                            addr.prefix_length
                        ),
                    ));
                }
            }
            addrs.retain(|a| !a.is_auto());
            addrs.iter_mut().for_each(|a| {
                a.valid_life_time = None;
                a.preferred_life_time = None
            });
        }

        if !self.is_enabled() {
            self.dhcp = None;
            self.addresses = None;
        }
        Ok(())
    }
}

/// IPv6 configurations of interface.
/// Example output of interface holding automatic IPv6 settings:
/// ```yaml
/// ---
/// interfaces:
/// - name: eth1
///   state: up
///   mtu: 1500
///   ipv4:
///     enabled: false
///   ipv6:
///     address:
///       - ip: 2001:db8:2::1
///         prefix-length: 64
///       - ip: 2001:db8:1::1
///         prefix-length: 64
///       - ip: fe80::1ec1:cff:fe32:3bd3
///         prefix-length: 64
///     autoconf: true
///     dhcp: true
///     enabled: true
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct InterfaceIpv6 {
    /// Whether IPv6 stack is enable. When set to false, the IPv6 stack is
    /// disabled with IPv6 link-local address purged also.
    /// Undefined means true.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub enabled: Option<bool>,
    /// Whether DHCPv6 enabled.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub dhcp: Option<bool>,
    /// Whether autoconf via IPv6 router announcement enabled.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "crate::deserializer::option_bool_or_string"
    )]
    pub autoconf: Option<bool>,
    /// IPv6 addresses. Will be ignored when applying with
    /// DHCPv6 or autoconf is enabled.
    /// When applying with `None`, current IP address will be preserved.
    /// When applying with `Some(Vec::new())`, all IP address will be removed.
    /// The IP addresses will apply to kernel with the same order specified.
    #[serde(skip_serializing_if = "Option::is_none", rename = "address")]
    pub addresses: Option<Vec<InterfaceIpAddr>>,
}

impl Default for InterfaceIpv6 {
    /// Create [InterfaceIpv6] with IP disabled.
    fn default() -> Self {
        Self {
            enabled: Some(false),
            dhcp: None,
            autoconf: None,
            addresses: None,
        }
    }
}

impl InterfaceIpv6 {
    /// New [InterfaceIpv6] with IP disabled.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled != Some(false)
    }

    pub(crate) fn is_auto(&self) -> bool {
        self.is_enabled()
            && (self.dhcp == Some(true) || self.autoconf == Some(true))
    }

    pub fn is_static(&self) -> bool {
        self.is_enabled()
            && !self.is_auto()
            && !self.addresses.as_deref().unwrap_or_default().is_empty()
    }

    // * Disable DHCP and remove address if enabled: false
    // * Set DHCP options to None if DHCP is false
    pub(crate) fn sanitize(
        &mut self,
        is_desired: bool,
    ) -> Result<(), NmstateError> {
        if let Some(addrs) = self.addresses.as_mut() {
            if is_desired {
                for addr in addrs.as_slice().iter().filter(|a| a.is_auto()) {
                    log::info!("Ignoring Auto IP address {addr}");
                }
                if let Some(addr) = addrs.iter().find(|a| a.ip.is_ipv4()) {
                    return Err(NmstateError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Got IPv4 address {addr} in ipv6 config section"
                        ),
                    ));
                }
                if let Some(addr) = addrs
                    .iter()
                    .find(|a| a.prefix_length as usize > IPV6_ADDR_LEN)
                {
                    return Err(NmstateError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Invalid IPv6 network prefix length '{}', should \
                             be in the range of 0 to {IPV6_ADDR_LEN}",
                            addr.prefix_length
                        ),
                    ));
                }
            }
            addrs.retain(|a| !a.is_auto());
            addrs.iter_mut().for_each(|a| {
                a.valid_life_time = None;
                a.preferred_life_time = None
            });
        }

        if self.is_auto() {
            if let Some(addrs) = self.addresses.as_ref() {
                if is_desired {
                    for addr in addrs {
                        log::info!(
                            "Static addresses {addr} defined when dynamic IP \
                             is enabled"
                        );
                    }
                }
            }
        }

        if let Some(addrs) = self.addresses.as_mut() {
            addrs.retain(|addr| {
                if let IpAddr::V6(ip_addr) = addr.ip {
                    if is_ipv6_unicast_link_local(&ip_addr) {
                        if is_desired {
                            log::warn!(
                                "Ignoring IPv6 link local address {}/{}",
                                &addr.ip,
                                addr.prefix_length
                            );
                        }
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            })
        };

        if !self.is_enabled() {
            self.dhcp = None;
            self.autoconf = None;
            self.addresses = None;
        }

        Ok(())
    }

    pub(crate) fn merge(&mut self, new: &Self) {
        if new.enabled.is_some() {
            self.enabled = new.enabled;
        }
        if self.dhcp.is_none() && self.is_enabled() {
            self.dhcp = new.dhcp;
        }
        if self.autoconf.is_none() && self.is_enabled() {
            self.autoconf = new.autoconf;
        }
        // Normally, we expect backend to preserve configuration which not
        // mentioned in desire, but when DHCP switch from ON to OFF, the design
        // of nmstate is expecting dynamic IP address goes static. This should
        // be done by top level code.
        if new.is_auto()
            && new.addresses.is_some()
            && self.is_enabled()
            && !self.is_auto()
            && is_ip_addrs_none_or_all_auto(self.addresses.as_deref())
        {
            self.addresses.clone_from(&new.addresses);
            if let Some(addrs) = self.addresses.as_mut() {
                addrs.as_mut_slice().iter_mut().for_each(|a| {
                    a.valid_life_time = None;
                    a.preferred_life_time = None;
                });
            }
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct InterfaceIpAddr {
    /// IP address.
    pub ip: IpAddr,
    #[serde(deserialize_with = "crate::deserializer::u8_or_string")]
    /// Prefix length.
    /// Serialize and deserialize to/from `prefix-length`.
    pub prefix_length: u8,
    /// Remaining time for IP address been valid. The output format is
    /// "32sec" or "forever".
    /// This property is query only, it will be ignored when applying.
    /// Serialize to `valid-life-time`.
    /// Deserialize from `valid-life-time` or `valid-left` or `valid-lft`.
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "valid-left",
        alias = "valid-lft"
    )]
    pub valid_life_time: Option<String>,
    /// Remaining time for IP address been preferred. The output format is
    /// "32sec" or "forever".
    /// This property is query only, it will be ignored when applying.
    /// Serialize to `preferred-life-time`.
    /// Deserialize from `preferred-life-time` or `preferred-left` or
    /// `preferred-lft`.
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "preferred-left",
        alias = "preferred-lft"
    )]
    pub preferred_life_time: Option<String>,
}

impl Default for InterfaceIpAddr {
    fn default() -> Self {
        Self {
            ip: IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            prefix_length: 128,
            valid_life_time: None,
            preferred_life_time: None,
        }
    }
}

impl std::fmt::Display for InterfaceIpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_auto() {
            write!(
                f,
                "{}/{} valid_life_time {} preferred_lft {}",
                self.ip,
                self.prefix_length,
                self.valid_life_time.as_deref().unwrap_or(FOREVER),
                self.preferred_life_time.as_deref().unwrap_or(FOREVER)
            )
        } else {
            write!(f, "{}/{}", self.ip, self.prefix_length)
        }
    }
}

impl InterfaceIpAddr {
    pub(crate) fn is_auto(&self) -> bool {
        self.valid_life_time.is_some()
            && self.valid_life_time.as_deref() != Some(FOREVER)
    }
}

// Copy from Rust official std::net::Ipv6Addr::is_unicast_link_local() which
// is experimental.
pub(crate) fn is_ipv6_unicast_link_local(ip: &Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

impl std::convert::TryFrom<&str> for InterfaceIpAddr {
    type Error = NmstateError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut addr: Vec<&str> = value.split('/').collect();
        addr.resize(2, "");
        let ip = IpAddr::from_str(addr[0]).map_err(|e| {
            let e = NmstateError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid IP address {}: {e}", addr[0]),
            );
            log::error!("{e}");
            e
        })?;

        let prefix_length = if addr[1].is_empty() {
            if ip.is_ipv6() { 128 } else { 32 }
        } else {
            addr[1].parse::<u8>().map_err(|parse_error| {
                let e = NmstateError::new(
                    ErrorKind::InvalidArgument,
                    format!("Invalid IP address {value}: {parse_error}"),
                );
                log::error!("{e}");
                e
            })?
        };
        Ok(Self {
            ip,
            prefix_length,
            valid_life_time: None,
            preferred_life_time: None,
        })
    }
}

fn is_ip_addrs_none_or_all_auto(addrs: Option<&[InterfaceIpAddr]>) -> bool {
    addrs.is_none_or(|addrs| {
        addrs.iter().all(|a| {
            if let IpAddr::V6(ip_addr) = a.ip {
                is_ipv6_unicast_link_local(&ip_addr) || a.is_auto()
            } else {
                a.is_auto()
            }
        })
    })
}
