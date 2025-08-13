# SPDX-License-Identifier: Apache-2.0

import json
import struct
import socket

from .error import NmError
from .log import NmLogEntry
from .cmd import NmCmdPing
from .cmd import NmCmdQueryNetworkState
from .nmstate import NmstateQueryOption

U32_MAX = 0xFFFFFFFF


class NmIpcConnection:
    def __init__(self, path):
        self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.socket.connect(path)

    def send(self, json_str):
        data_raw = json_str.encode("utf-8")
        length_raw = struct.pack(">I", len(data_raw) & U32_MAX)
        self.socket.sendall(length_raw)
        self.socket.sendall(data_raw)

    def recv(self):
        # TODO(Gris Ge): handle timeout here
        while True:
            length_raw = self.socket.recv(4)
            if not length_raw:
                raise NmError("BUG", "Got empty reply from daemon")
            length = struct.unpack(">I", length_raw)[0]
            reply = json.loads(self.socket.recv(length).decode("utf-8"))
            match reply["kind"]:
                case NmError.IPC_KIND:
                    raise NmError.from_dict(reply["data"])
                case NmLogEntry.IPC_KIND:
                    log_entry = NmLogEntry.from_dict(reply["data"])
                    log_entry.emit()
                case _:
                    return reply["data"]

    def exec(self, cmd):
        self.send(cmd.to_json())
        return self.recv()


DAEMON_SOCKET_PATH = "/var/run/NetworkManager/sockets/daemon"


class NmClient:
    def __init__(self):
        self._conn = NmIpcConnection(DAEMON_SOCKET_PATH)

    def ping(self):
        return self._conn.exec(NmCmdPing())

    def query_network_state(self, opt=None):
        if not opt:
            opt = NmstateQueryOption()
        return self._conn.exec(NmCmdQueryNetworkState(opt))
