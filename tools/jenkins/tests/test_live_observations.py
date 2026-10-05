from __future__ import annotations

import ctypes
import os
import socket
import subprocess
import sys
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.live_observations import (
    _TcpRowOwnerPid,
    _digest_args,
    observe_endpoint_owner,
    observe_live_identity,
)


class LiveObservationTests(unittest.TestCase):
    def test_observes_this_process_without_exposing_command_line(self):
        from tools.jenkins.deployment.live_observations import _windows_command_line
        from tools.jenkins.processes.identity import current_identity
        expected = current_identity(os.getpid()).to_dict()
        expected["commandDigest"] = _digest_args(_windows_command_line(os.getpid()))
        observed = observe_live_identity(expected)
        self.assertEqual(observed["pid"], os.getpid())
        self.assertEqual(observed["creationTime"], expected["creationTime"])
        self.assertEqual(observed["commandDigest"], expected["commandDigest"])
        self.assertNotIn("argv", observed)

    def test_birth_reuse_and_command_mismatch_fail_closed(self):
        base = {"pid": 42, "creationTime": "100", "executable": r"E:\python.exe",
                "commandDigest": "a" * 64}
        actual = {"pid": 42, "creationTime": "101", "executable": r"E:\python.exe"}
        with patch("tools.jenkins.deployment.live_observations.current_identity",
                   side_effect=[type("I", (), {"to_dict": lambda self: actual})(),
                                type("I", (), {"to_dict": lambda self: actual})()]), \
             patch("tools.jenkins.deployment.live_observations._windows_command_line", return_value=("python",)):
            with self.assertRaises(JenkinsError):
                observe_live_identity(base)
        actual["creationTime"] = "100"
        with patch("tools.jenkins.deployment.live_observations.current_identity",
                   side_effect=[type("I", (), {"to_dict": lambda self: actual})(),
                                type("I", (), {"to_dict": lambda self: actual})()]), \
             patch("tools.jenkins.deployment.live_observations._windows_command_line", return_value=("other",)):
            with self.assertRaises(JenkinsError):
                observe_live_identity(base)

    def test_foreign_listener_owner_is_rejected(self):
        row = _TcpRowOwnerPid(2, int.from_bytes(b"\x7f\x00\x00\x01", "little"),
                              int.from_bytes((53748).to_bytes(2, "big"), "little"), 0, 0, 999)
        raw = ctypes.string_at(ctypes.byref(row), ctypes.sizeof(row))

        class FakeIp:
            def __init__(self):
                self.GetExtendedTcpTable = FakeTable(raw)

            def _get(self, ptr, size, order, af, table_class, reserved):
                if not ptr:
                    size._obj.value = 4 + len(raw)
                    return 122
                ctypes.memmove(ctypes.addressof(ptr._obj), (1).to_bytes(4, "little") + raw, 4 + len(raw))
                return 0

        class FakeTable:
            def __init__(self, data):
                self.data = data

            def __call__(self, ptr, size, order, af, table_class, reserved):
                if not ptr:
                    size._obj.value = 4 + len(self.data)
                    return 122
                ctypes.memmove(ctypes.addressof(ptr._obj), (1).to_bytes(4, "little") + self.data,
                               4 + len(self.data))
                return 0

        with patch("tools.jenkins.deployment.live_observations.os.name", "nt"), \
             patch("tools.jenkins.deployment.live_observations.ctypes.WinDLL", return_value=FakeIp()):
            with self.assertRaises(JenkinsError):
                observe_endpoint_owner("127.0.0.1", 53748, os.getpid())

    def test_observes_child_process_and_real_listener_owner(self):
        from tools.jenkins.processes.identity import current_identity
        command = (sys.executable, "-B", "-c", "import time; time.sleep(3)", "quoted space", "back\\slash")
        child = subprocess.Popen(command, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        try:
            expected = current_identity(child.pid, executable=sys.executable, command_line=command).to_dict()
            observed = observe_live_identity(expected)
            self.assertEqual(observed["pid"], child.pid)
        finally:
            child.terminate()
            child.wait(timeout=5)
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        try:
            if os.name == "nt":
                owner = observe_endpoint_owner("127.0.0.1", listener.getsockname()[1], os.getpid())
                self.assertEqual(owner["pid"], os.getpid())
        finally:
            listener.close()


if __name__ == "__main__":
    unittest.main()
