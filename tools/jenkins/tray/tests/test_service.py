import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray import service


class ServiceGuardTests(unittest.TestCase):
    def test_unknown_process_identity_fails_closed(self):
        with self.assertRaises(service.TrayError):
            service._live({"pid": 7})

    def test_reused_or_missing_process_is_stopped(self):
        with patch("tools.jenkins.tray.service.process_matches_creation_time", return_value=False):
            self.assertFalse(service._live({"pid": 7, "creationTime": "birth"}))

    def test_runtime_evidence_requires_marker(self):
        record = {"pid": 7, "creationTime": "birth"}
        with patch("tools.jenkins.tray.service._live", return_value=True), patch(
            "tools.jenkins.pilot.lifetime.wait_runtime_started", side_effect=ValueError("missing marker")
        ):
            with self.assertRaises(service.TrayError):
                service._runtime_evidence(record)

    def test_stop_preview_does_not_expose_command_or_secret(self):
        class Config:
            root_identity = {"fileId": "x"}
            pilot_root = Path("D:/cargo-targets/jenkins-pilot-test")

        status = {"activeBuilds": [], "queuedCount": 0}
        state = {"controller": {"pid": 1, "creationTime": "b", "commandLine": "secret", "agentSecret": "secret"},
                 "agent": {"pid": 2, "creationTime": "c", "secret": "secret"}}
        with patch.object(service, "read_status", return_value=status), patch.object(service, "_runtime", return_value=(state, {}, "")):
            preview = service.stop_preview(Config())
        self.assertNotIn("commandLine", preview["controller"])
        self.assertNotIn("agentSecret", preview["controller"])
        self.assertNotIn("secret", preview["agent"])


if __name__ == "__main__":
    unittest.main()
