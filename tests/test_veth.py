# SPDX-License-Identifier: Apache-2.0

import pytest

import libnm
from libnm import NmValueError

from .testlib.statelib import load_yaml
from .testlib.veth import veth_interface


def test_veth_exceeded_max_mtu():
    with veth_interface("veth-test1", "veth-test1-ep"):
        with pytest.raises(NmValueError):
            libnm.apply(
                load_yaml(
                    """---
                        version: 1
                        interfaces:
                        - name: veth-test1
                          type: ethernet
                          mtu: 99999999
                     """
                )
            )


def test_veth_exceeded_min_mtu():
    with veth_interface("veth-test1", "veth-test1-ep"):
        with pytest.raises(NmValueError):
            libnm.apply(
                load_yaml(
                    """---
                        version: 1
                        interfaces:
                        - name: veth-test1
                          type: ethernet
                          mtu: 1
                     """
                )
            )
