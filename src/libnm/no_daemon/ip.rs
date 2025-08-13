// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use nmstate::{InterfaceIpAddr, InterfaceIpv4, InterfaceIpv6};

pub(crate) fn np_ipv4_to_nmstate(
    np_iface: &nispor::Iface,
) -> Option<InterfaceIpv4> {
    if let Some(np_ip) = &np_iface.ipv4 {
        let mut ip = InterfaceIpv4::default();
        ip.enabled = Some(!np_ip.addresses.is_empty());
        if !ip.is_enabled() {
            return Some(ip);
        }
        let mut addresses = Vec::new();
        for np_addr in &np_ip.addresses {
            if np_addr.valid_lft != "forever" {
                ip.dhcp = Some(true);
            }
            match std::net::IpAddr::from_str(np_addr.address.as_str()) {
                Ok(i) => {
                    let mut addr = InterfaceIpAddr::default();
                    addr.ip = i;
                    addr.prefix_length = np_addr.prefix_len;
                    addr.valid_life_time = if np_addr.valid_lft != "forever" {
                        Some(np_addr.valid_lft.clone())
                    } else {
                        None
                    };
                    addr.preferred_life_time =
                        if np_addr.preferred_lft != "forever" {
                            Some(np_addr.preferred_lft.clone())
                        } else {
                            None
                        };
                    addresses.push(addr);
                }
                Err(e) => {
                    log::warn!(
                        "BUG: nispor got invalid IP address {}, error {}",
                        np_addr.address.as_str(),
                        e
                    );
                }
            }
        }
        ip.addresses = Some(addresses);
        Some(ip)
    } else {
        // IP might just disabled
        Some(InterfaceIpv4::default())
    }
}

pub(crate) fn np_ipv6_to_nmstate(
    np_iface: &nispor::Iface,
) -> Option<InterfaceIpv6> {
    if let Some(np_ip) = &np_iface.ipv6 {
        let mut ip = InterfaceIpv6::default();

        ip.enabled = Some(!np_ip.addresses.is_empty());

        if !ip.is_enabled() {
            return Some(ip);
        }
        let mut addresses = Vec::new();
        for np_addr in &np_ip.addresses {
            if np_addr.valid_lft != "forever" {
                ip.autoconf = Some(true);
            }
            match std::net::IpAddr::from_str(np_addr.address.as_str()) {
                Ok(i) => {
                    let mut addr = InterfaceIpAddr::default();
                    addr.ip = i;
                    addr.prefix_length = np_addr.prefix_len;
                    addr.valid_life_time = if np_addr.valid_lft != "forever" {
                        Some(np_addr.valid_lft.clone())
                    } else {
                        None
                    };
                    addr.preferred_life_time =
                        if np_addr.preferred_lft != "forever" {
                            Some(np_addr.preferred_lft.clone())
                        } else {
                            None
                        };
                    addresses.push(addr);
                }
                Err(e) => {
                    log::warn!(
                        "BUG: nispor got invalid IP address {}, error {}",
                        np_addr.address.as_str(),
                        e
                    );
                }
            }
        }
        ip.addresses = Some(addresses);
        Some(ip)
    } else {
        // IP might just disabled
        Some(InterfaceIpv6::default())
    }
}
