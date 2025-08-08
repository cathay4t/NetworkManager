# SPDX-License-Identifier: Apache-2.0

import json


class NmError(Exception):
    IPC_KIND = "error"

    def __init__(self, kind, msg):
        self.kind = kind
        self.msg = msg

    def to_json(self):
        return json.dumps(
            {
                "kind": self.kind,
                "msg": self.msg,
            }
        )

    def from_dict(data):
        return NmError(data["kind"], data["msg"])
