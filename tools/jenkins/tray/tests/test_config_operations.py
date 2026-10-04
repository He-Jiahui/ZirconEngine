"""Physical configuration and durable operation contracts on Windows."""
from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray.config import TrayError, absolute_plain_path, load_config, validate_runtime
from tools.jenkins.tray.operations import execute, operation_lock, read_operation
from tools.jenkins.tray.persistence import read_json, write_json, safe_error


@unittest.skipUnless(os.name == "nt", "native Windows storage and process identity")
class ConfigAndOperationsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        repo = Path(__file__).resolve().parents[3]
        cls.config = load_config(repo / ".codex/state/jenkins-tray/main.json")

    def setUp(self):
        directory = Path(r"E:\cargo-targets\jenkins-tray-unit-tests")
        directory.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="contract-", dir=directory)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def test_current_profile_matches_physical_root_and_sealed_python(self):
        value = validate_runtime(self.config)
        self.assertEqual(str(self.config.python_executable), value["pythonExecutable"])

    def test_changed_physical_identity_is_refused(self):
        config = replace(self.config, root_identity={"fileId": "other"})
        with self.assertRaisesRegex(TrayError, "身份"):
            validate_runtime(config)

    def test_python_hash_change_is_refused_without_touching_executable(self):
        with self.assertRaisesRegex(TrayError, "封存"):
            validate_runtime(replace(self.config, python_sha256="0" * 64))

    def test_unsafe_relative_and_traversal_paths_are_refused(self):
        for path in ("python.exe", "E:\\cargo-targets\\..\\other", 'E:\\bad"path', "E:\\bad\npath"):
            with self.subTest(path=path), self.assertRaises(TrayError):
                absolute_plain_path(path)

    def test_atomic_record_and_corrupt_record_detection(self):
        path = self.root / "record.json"
        write_json(path, {"operationId": "one", "state": "active"})
        write_json(path, {"operationId": "one", "state": "completed"})
        self.assertEqual("completed", read_json(path)["state"])
        path.write_text("incomplete", encoding="utf-8")
        with self.assertRaises(TrayError):
            read_json(path)

    def test_unknown_exception_cannot_expose_credentials(self):
        message = safe_error(RuntimeError("password=topsecret"))
        self.assertNotIn("topsecret", message)

    def test_native_operation_mutex_rejects_competing_thread(self):
        import threading
        results = []
        def competing():
            try:
                with operation_lock(self.config):
                    results.append("acquired")
            except TrayError:
                results.append("busy")
        with operation_lock(self.config):
            thread = threading.Thread(target=competing)
            thread.start()
            thread.join(timeout=5)
        self.assertEqual(["busy"], results)

    def test_active_original_worker_prevents_duplicate_start(self):
        from tools.jenkins.pilot.native.process_identity import process_creation_time
        config = replace(self.config, state_dir=self.root)
        write_json(self.root / "operation.json", {
            "operationId": "original", "state": "active", "action": "start", "workerPid": os.getpid(),
            "workerCreationTime": process_creation_time(os.getpid()),
        })
        with patch("tools.jenkins.tray.service.read_status", return_value={"state": "ready"}), \
             patch("tools.jenkins.tray.service.start") as start:
            with self.assertRaisesRegex(TrayError, "仍在运行"):
                execute(config, "start")
            start.assert_not_called()
        self.assertEqual("original", read_operation(config)["operationId"])


if __name__ == "__main__":
    unittest.main()
