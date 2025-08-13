# SPDX-License-Identifier: Apache-2.0

import pathlib
import subprocess
import sys
import time

import pytest

project_dir = pathlib.Path(__file__).parent.parent.resolve()
sys.path.insert(0, f"{project_dir}/src/python-libnm")


@pytest.fixture(scope="session", autouse=True)
def test_env_setup(run_daemon):
    yield


@pytest.fixture(scope="session")
def run_daemon():
    bin_path = pathlib.Path(
        f"{project_dir}/target/debug/NetworkManager"
    ).resolve()
    process = subprocess.Popen(bin_path, stdout=sys.stdout, stderr=sys.stderr)
    # Wait daemon to start up
    time.sleep(1)
    yield
    if process:
        process.terminate()
