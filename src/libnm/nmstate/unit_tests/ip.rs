// SPDX-License-Identifier: Apache-2.0

use super::super::ip::sanitize_ip_network;
use crate::{ErrorKind, Interface, MergedNetworkState, NetworkState};

#[test]
fn test_purge_auto_ip_when_apply() {
    let desired: NetworkState = serde_yaml::from_str(
        r#"
        interfaces:
        - name: eth1
          type: ethernet
          ipv4:
            enabled: true
            address:
            - ip: 192.0.2.19
              prefix-length: 24
              valid-lft: 60s
              preferred-lft: 60s
            - ip: 192.0.2.18
              prefix-length: 24
          ipv6:
            enabled: true
            address:
            - ip: 2001:db8::19
              prefix-length: 64
              valid-lft: 160s
              preferred-lft: 160s
            - ip: 2001:db8::18
              prefix-length: 64
              valid-lft: forever
              preferred-lft: forever
        "#,
    )
    .unwrap();
    let current: NetworkState = serde_yaml::from_str(
        r#"
        interfaces:
        - name: eth1
          type: ethernet
          state: up
        "#,
    )
    .unwrap();

    let merged =
        MergedNetworkState::new(desired, current, Default::default()).unwrap();

    let apply_state = merged.gen_state_for_apply();
    let apply_iface = apply_state.ifaces.kernel_ifaces.get("eth1").unwrap();

    let expected: Interface = serde_yaml::from_str(
        r#"
        name: eth1
        type: ethernet
        ipv4:
          enabled: true
          address:
          - ip: 192.0.2.18
            prefix-length: 24
        ipv6:
          enabled: true
          address:
          - ip: 2001:db8::18
            prefix-length: 64
        "#,
    )
    .unwrap();

    assert_eq!(apply_iface, &expected);
}

#[test]
fn test_sanitize_ip_network_empty_str() {
    let result = sanitize_ip_network("");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_sanitize_ip_network_invalid_str() {
    let result = sanitize_ip_network("192.0.2.1/24/");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_sanitize_ip_network_invalid_ipv4_prefix_length() {
    let result = sanitize_ip_network("192.0.2.1/33");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_sanitize_ip_network_invalid_ipv6_prefix_length() {
    let result = sanitize_ip_network("::1/129");
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), ErrorKind::InvalidArgument);
    }
}

#[test]
fn test_sanitize_ip_network_ipv4_gateway() {
    assert_eq!(sanitize_ip_network("0.0.0.1/0").unwrap(), "0.0.0.0/0");
}

#[test]
fn test_sanitize_ip_network_ipv6_gateway() {
    assert_eq!(sanitize_ip_network("::1/0").unwrap(), "::/0");
}

#[test]
fn test_sanitize_ip_network_ipv4_host_only() {
    assert_eq!(sanitize_ip_network("192.0.2.1").unwrap(), "192.0.2.1/32");
}

#[test]
fn test_sanitize_ip_network_ipv6_host_only() {
    assert_eq!(
        sanitize_ip_network("2001:db8:1::0").unwrap(),
        "2001:db8:1::/128"
    );
}

#[test]
fn test_sanitize_ip_network_ipv4_host_only_explicit() {
    assert_eq!(sanitize_ip_network("192.0.2.1/32").unwrap(), "192.0.2.1/32");
}

#[test]
fn test_sanitize_ip_network_ipv6_host_only_explicit() {
    assert_eq!(
        sanitize_ip_network("2001:db8:1::0/128").unwrap(),
        "2001:db8:1::/128"
    );
}

#[test]
fn test_sanitize_ip_network_ipv4_net() {
    assert_eq!(sanitize_ip_network("192.0.3.1/23").unwrap(), "192.0.2.0/23");
}

#[test]
fn test_sanitize_ip_network_ipv6_net() {
    assert_eq!(
        sanitize_ip_network("2001:db8:1::f/64").unwrap(),
        "2001:db8:1::/64"
    );
}
