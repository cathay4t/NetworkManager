# SPDX-License-Identifier: Apache-2.0

from libnm import NmClient


def show_only(iface_name):
    client = NmClient()
    state = client.query_network_state()
    for iface in state["interfaces"]:
        if iface["name"] == iface_name:
            return iface
