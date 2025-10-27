# SPDX-License-Identifier: Apache-2.0

import json
import os
import re
import signal

import libnm
import pytest

from .cmdlib import exec_cmd
from .retry import retry_till_true_or_timeout


HWSIM0_PERM_MAC = "02:00:00:00:00:00"
HWSIM1_PERM_MAC = "02:00:00:00:01:00"
TEST_NET_NS = "wifi-test"
TEST_WIFI_SSID = "Test-WIFI"
TEST_WIFI_PSK = "12345678"
HOSTAPD_PID_PATH = "/tmp/nm_test_hostapd.pid"
HOSTAPD_CONF_PATH = "/tmp/nm_test_hostapd.conf"
HOSTAPD_CONF_FMT = """
interface={IFACE_NAME}
driver=nl80211

hw_mode=g
channel=1
ssid=Test-WIFI

wpa=2
wpa_key_mgmt=WPA-PSK
wpa_pairwise=CCMP
wpa_passphrase=12345678
"""
TIMEOUT_SECS_SIM_WIFI_NICS = 30
PEER_IPV4 = "203.0.113.1"


@pytest.fixture
def wifi_env():
    exec_cmd(f"ip netns del {TEST_NET_NS}".split(), check=False)
    exec_cmd(f"ip netns add {TEST_NET_NS}".split())

    exec_cmd("modprobe mac80211_hwsim radios=2".split())
    assert retry_till_true_or_timeout(
        TIMEOUT_SECS_SIM_WIFI_NICS, has_sim_wifi_nics
    )

    state = libnm.show()
    wlan1 = get_nic_name_by_perm_mac(state, HWSIM0_PERM_MAC)
    wlan2 = get_nic_name_by_perm_mac(state, HWSIM1_PERM_MAC)
    start_hostapd(wlan2)
    yield wlan1
    exec_cmd(f"ip netns del {TEST_NET_NS}".split())
    exec_cmd("modprobe -r mac80211_hwsim".split(), check=False)
    os.remove(HOSTAPD_CONF_PATH)
    if os.path.exists(HOSTAPD_PID_PATH):
        with open(HOSTAPD_PID_PATH) as fd:
            pid = fd.read()
        os.kill(int(pid), signal.SIGTERM)


def get_nic_name_by_perm_mac(state, mac):
    for iface in state["interfaces"]:
        if iface.get("permanent-mac-address") == mac:
            return iface["name"]


def get_wifi_phy_name(nic_name):
    # TODO(Gris Ge): use libnm instead of iw here
    (output, _) = exec_cmd(f"iw dev {nic_name} info".split())
    match = re.search(r"[^a-zA-Z]wiphy ([0-9]+)", output)
    assert match
    if match:
        return match.group(1)


def has_sim_wifi_nics():
    exec_cmd("udevadm settle".split())
    state = libnm.show()
    wlan1 = get_nic_name_by_perm_mac(state, HWSIM0_PERM_MAC)
    wlan2 = get_nic_name_by_perm_mac(state, HWSIM1_PERM_MAC)
    return wlan1 and wlan2


def start_hostapd(nic_name):
    phy_id = get_wifi_phy_name(nic_name)
    assert phy_id
    # Move phy2 to namespace with hostpad
    exec_cmd(f"iw phy#{phy_id} set netns name {TEST_NET_NS}".split())
    exec_cmd(f"ip netns exec {TEST_NET_NS} ip link set {nic_name} up".split())
    exec_cmd(
        f"ip netns exec {TEST_NET_NS} "
        f"ip addr add {PEER_IPV4}/24 dev {nic_name}".split()
    )

    with open(HOSTAPD_CONF_PATH, "w") as fd:
        fd.write(HOSTAPD_CONF_FMT.format(IFACE_NAME=nic_name))

    exec_cmd(
        f"ip netns exec {TEST_NET_NS} "
        f"hostapd -B -d {HOSTAPD_CONF_PATH} -P {HOSTAPD_PID_PATH}".split(),
    )
