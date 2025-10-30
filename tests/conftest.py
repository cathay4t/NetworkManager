# SPDX-License-Identifier: Apache-2.0

import pathlib
import subprocess
import sys
import time

import pytest

from .testlib.retry import retry_till_true_or_timeout

project_dir = pathlib.Path(__file__).parent.parent.resolve()
sys.path.insert(0, f"{project_dir}/src/python-libnm")

from libnm import NmClient

DAEMON_LOG = "/tmp/nm_test_daemon.log"


@pytest.fixture(scope="session", autouse=True)
def test_env_setup(run_daemon):
    yield


@pytest.fixture(scope="session")
def run_daemon():
    bin_path = pathlib.Path(
        f"{project_dir}/target/debug/NetworkManager"
    ).resolve()
    process = subprocess.Popen(
        bin_path, stdout=sys.stdout, stderr=open(DAEMON_LOG, "w")
    )
    # Wait daemon to start up
    time.sleep(1)
    retry_till_true_or_timeout(30, check_daemon_connection)
    yield
    if process:
        process.terminate()


def check_daemon_connection():
    try:
        client = NmClient()
        return client.ping() == "pong"
    except:
        return false
