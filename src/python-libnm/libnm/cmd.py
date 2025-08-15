# SPDX-License-Identifier: Apache-2.0

import json

from .nmstate import NmstateApplyOption
from .nmstate import NmstateQueryOption


class NmCmdPing:
    IPC_KIND = "ping"

    def to_json(self):
        return json.dumps(
            {
                "kind": NmCmdPing.IPC_KIND,
                "data": NmCmdPing.IPC_KIND,
            }
        )


class NmCmdQueryNetworkState:
    IPC_KIND = "query-network-state"

    def __init__(self, opt: NmstateQueryOption):
        self.opt = opt

    def to_json(self):
        return json.dumps(
            {
                "kind": NmCmdQueryNetworkState.IPC_KIND,
                "data": {NmCmdQueryNetworkState.IPC_KIND: self.opt.to_dict()},
            }
        )


class NmCmdApplyNetworkState:
    IPC_KIND = "apply-network-state"

    def __init__(self, desired_state, opt: NmstateApplyOption):
        self.desired_state = desired_state
        self.opt = opt

    def to_json(self):
        return json.dumps(
            {
                "kind": NmCmdApplyNetworkState.IPC_KIND,
                "data": {
                    NmCmdApplyNetworkState.IPC_KIND: (
                        self.desired_state,
                        self.opt.to_dict(),
                    )
                },
            }
        )
