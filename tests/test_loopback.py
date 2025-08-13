# SPDX-License-Identifier: Apache-2.0

from .testlib.statelib import show_only


def test_query_loopback():
    iface_state = show_only("lo")

    assert iface_state["name"] == "lo"
    assert iface_state["mtu"] == 65536
    assert iface_state["mac-address"] == "00:00:00:00:00:00"
    assert iface_state["ipv4"] == {
        "address": [{"ip": "127.0.0.1", "prefix-length": 8}],
        "enabled": True,
    }
    assert iface_state["ipv6"] == {
        "address": [{"ip": "::1", "prefix-length": 128}],
        "enabled": True,
    }
