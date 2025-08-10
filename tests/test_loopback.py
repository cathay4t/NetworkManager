# SPDX-License-Identifier: Apache-2.0

from libnm import NmClient

from .testlib.statelib import show_only


def test_query_loopback():
    iface_state = show_only("lo")

    assert iface_state["name"] == "lo"
    assert iface_state["mtu"] == 65536
    assert iface_state["mac-address"] == "00:00:00:00:00:00"
