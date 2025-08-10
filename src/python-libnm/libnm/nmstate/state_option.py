# SPDX-License-Identifier: Apache-2.0

import enum

from ..version import LATEST_SCHEMA_VERSION


class NmstateStateKind(enum.StrEnum):
    RUNNING_NETWORK_STATE = "running-network-state"
    SAVED_NETWORK_STATE = "saved-network-state"
    DEFAULT = RUNNING_NETWORK_STATE


class NmstateQueryOption:
    def __init__(
        self, version=LATEST_SCHEMA_VERSION, kind=NmstateStateKind.DEFAULT
    ):
        self.version = version
        self.kind = kind

    def to_dict(self):
        return {"version": self.version, "kind": self.kind}
