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
        match data["kind"]:
            case "invalid-argument":
                return NmValueError(data["kind"], data["msg"])
            case _:
                return NmError(data["kind"], data["msg"])


class NmValueError(NmError, ValueError):
    pass
