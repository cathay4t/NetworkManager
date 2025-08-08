# SPDX-License-Identifier: Apache-2.0

import json


class NmClientCmdPing:
    IPC_KIND = "ping"

    def to_json(self):
        return json.dumps(
            {
                "kind": NmClientCmdPing.IPC_KIND,
                "data": NmClientCmdPing.IPC_KIND,
            }
        )
