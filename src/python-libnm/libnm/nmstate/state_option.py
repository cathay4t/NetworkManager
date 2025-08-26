# SPDX-License-Identifier: Apache-2.0

import enum

from ..version import LATEST_SCHEMA_VERSION


class NmstateStateKind(enum.StrEnum):
    RUNNING = "running-network-state"
    SAVED = "saved-network-state"
    DEFAULT = RUNNING


class NmstateQueryOption:
    def __init__(
        self, version=LATEST_SCHEMA_VERSION, kind=NmstateStateKind.DEFAULT
    ):
        self.version = version
        self.kind = kind

    def to_dict(self):
        return {"version": self.version, "kind": self.kind}

    def running():
        return NmstateQueryOption(kind=NmstateStateKind.RUNNING)

    def saved():
        return NmstateQueryOption(kind=NmstateStateKind.SAVED)


class NmstateApplyOption:
    def __init__(self, version=LATEST_SCHEMA_VERSION, no_verify=False):
        self.version = version
        self.no_verify = no_verify

    def to_dict(self):
        return {"version": self.version, "no-verify": self.no_verify}
