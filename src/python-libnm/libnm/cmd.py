# SPDX-License-Identifier: Apache-2.0

import json

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
