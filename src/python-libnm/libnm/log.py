# SPDX-License-Identifier: Apache-2.0

import logging

from .error import NmError


class NmLogEntry:
    IPC_KIND = "log"

    def __init__(self, source, level, message):
        self.source = source
        self.level = level
        self.message = message

    def from_dict(data):
        return NmLogEntry(data["source"], data["level"], data["message"])

    def emit(self):
        logger = logging.getLogger("libnmstate")
        match self.level:
            case "trace":
                logger.debug(self.message, extra={"source": self.source})
            case "debug":
                logger.debug(self.message, extra={"source": self.source})
            case "info":
                logger.info(self.message, extra={"source": self.source})
            case "warn":
                logger.info(self.message, extra={"source": self.source})
            case "error":
                logger.error(self.message, extra={"source": self.source})
            case _:
                raise NmError("Bug", f"unknown log level {self.level}")
