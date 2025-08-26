# SPDX-License-Identifier: Apache-2.0

from contextlib import contextmanager

from libnm import NmClient

from .cmdlib import exec_cmd
from .apply import libnm_apply


@contextmanager
def veth_interface(ifname, peer):
    libnm_apply(
        f"""---
        interfaces:
        - name: {ifname}
          type: veth
          veth:
            peer: {peer}
        """
    )
    try:
        yield
    finally:
        libnm_apply(
            f"""---
            interfaces:
            - name: {ifname}
              type: veth
              state: absent
            - name: {peer}
              type: veth
              state: absent
            """
        )
