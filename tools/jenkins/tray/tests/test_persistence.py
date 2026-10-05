from __future__ import annotations

import ctypes
import os
import tempfile
import threading
import unittest
from pathlib import Path

from tools.jenkins.tray.persistence import read_json, write_json


class PersistenceTests(unittest.TestCase):
    def setUp(self):
        root = Path(r"E:/cargo-targets/zircon-local/jenkins-support-tests/tmp")
        root.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=str(root))
        self.path = Path(self.temp.name) / "record.json"

    def tearDown(self):
        self.temp.cleanup()

    def test_atomic_writer_preserves_complete_reader_bytes(self):
        write_json(self.path, {"version": 1, "state": "old"})
        observed = []
        stop = threading.Event()

        def reader():
            while not stop.is_set():
                value = read_json(self.path)
                if value is not None:
                    observed.append(value)

        thread = threading.Thread(target=reader)
        thread.start()
        try:
            write_json(self.path, {"version": 2, "state": "new"})
        finally:
            stop.set(); thread.join(timeout=1)
        self.assertEqual({"version": 2, "state": "new"}, read_json(self.path))
        self.assertTrue(all(value in ({"version": 1, "state": "old"}, {"version": 2, "state": "new"}) for value in observed))

    @unittest.skipUnless(os.name == "nt", "requires Windows sharing semantics")
    def test_bounded_retry_with_non_delete_sharing_reader(self):
        write_json(self.path, {"state": "old"})
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.CreateFileW.restype = ctypes.c_void_p
        kernel.CreateFileW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p]
        kernel.CloseHandle.argtypes = [ctypes.c_void_p]
        handle = kernel.CreateFileW(str(self.path), 0x80000000, 0, None, 3, 0x80, None)
        self.assertNotEqual(handle, ctypes.c_void_p(-1).value)
        done = []
        def writer():
            write_json(self.path, {"state": "new"}); done.append(True)
        thread = threading.Thread(target=writer); thread.start()
        import time; time.sleep(0.05); kernel.CloseHandle(handle); thread.join(timeout=2)
        self.assertEqual([True], done)
        self.assertEqual({"state": "new"}, read_json(self.path))

    @unittest.skipUnless(os.name == "nt", "requires Windows sharing semantics")
    def test_conflict_exhaustion_preserves_old_bytes_and_cleans_temp(self):
        write_json(self.path, {"state": "old", "marker": "keep"})
        original = self.path.read_bytes()
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.CreateFileW.restype = ctypes.c_void_p
        kernel.CreateFileW.argtypes = [ctypes.c_wchar_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p]
        kernel.CloseHandle.argtypes = [ctypes.c_void_p]
        handle = kernel.CreateFileW(str(self.path), 0x80000000, 0, None, 3, 0x80, None)
        self.assertNotEqual(handle, ctypes.c_void_p(-1).value)
        try:
            with self.assertRaises(PermissionError):
                write_json(self.path, {"state": "must-not-publish"})
        finally:
            kernel.CloseHandle(handle)
        self.assertEqual(original, self.path.read_bytes())
        self.assertEqual([], list(self.path.parent.glob("." + self.path.name + ".*.tmp")))


if __name__ == "__main__": unittest.main()
