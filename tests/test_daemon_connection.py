# SPDX-License-Identifier: Apache-2.0

from libnm import NmClient


def test_daemon_conn_ping():
    client = NmClient()
    assert client.ping() == "pong"
