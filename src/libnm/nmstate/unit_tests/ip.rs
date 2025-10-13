// SPDX-License-Identifier: Apache-2.0

use crate::{Interface, MergedNetworkState, NetworkState};

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
