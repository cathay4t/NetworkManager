## SPDX-License-Identifier: Apache-2.0

from libnm import NmClient

from .statelib import load_yaml


def libnm_apply(yaml_srt):
    cli = NmClient()
    cli.apply_network_state(load_yaml(yaml_srt))
