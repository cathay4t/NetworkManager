# SPDX-License-Identifier: Apache-2.0

from contextlib import contextmanager

from .cmdlib import exec_cmd


@contextmanager
def veth_interface(ifname, peer):
    try:
        exec_cmd(f"ip link add {ifname} type veth peer {peer}".split())
        exec_cmd(f"ip link set {ifname} up".split())
        exec_cmd(f"ip link set {peer} up".split())
        yield
    finally:
        exec_cmd(f"ip link del {ifname}".split(), check=False)
