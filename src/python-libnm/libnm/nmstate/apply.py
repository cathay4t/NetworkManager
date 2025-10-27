# SPDX-License-Identifier: Apache-2.0

from ..client import NmClient
from .state_option import NmstateApplyOption


def apply(desired_state, *, verify_change=True):
    cli = NmClient()
    opt = NmstateApplyOption(verify_change=verify_change)
    cli.apply_network_state(desired_state, opt)
