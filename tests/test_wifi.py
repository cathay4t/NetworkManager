# SPDX-License-Identifier: Apache-2.0


import libnm

from .testlib.cmdlib import exec_cmd
from .testlib.statelib import load_yaml
from .testlib.wifi import wifi_env
from .testlib.wifi import PEER_IPV4
from .testlib.wifi import TEST_WIFI_SSID
from .testlib.wifi import TEST_WIFI_PSK
from .testlib.retry import retry_till_true_or_timeout


def test_wifi_iface_static_ip(wifi_env):
    nic_name = wifi_env
    libnm.apply(
        load_yaml(
            f"""---
            interfaces:
              - name: {nic_name}
                type: wifi-phy
                state: up
                wifi:
                  ssid: {TEST_WIFI_SSID}
                  password: {TEST_WIFI_PSK}
                ipv4:
                  enabled: true
                  dhcp: false
                  address:
                    - ip: 203.0.113.99
                      prefix-length: 24"""
        )
    )
    assert retry_till_true_or_timeout(5, ping_peer)


def ping_peer():
    try:
        exec_cmd(f"ping {PEER_IPV4} -c 1 -w 5".split())
    except:
        return False
    return True
