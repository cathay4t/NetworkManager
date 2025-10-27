# SPDX-License-Identifier: Apache-2.0

from ..client import NmClient
from .state_option import NmstateQueryOption
from .state_option import NmstateStateKind


def show():
    client = NmClient()
    opt = NmstateQueryOption(kind=NmstateStateKind.RUNNING)
    return client.query_network_state(opt)
