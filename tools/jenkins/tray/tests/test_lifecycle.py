import time
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from tools.jenkins.tray import lifecycle
from tools.jenkins.tray import service
from tools.jenkins.tray.config import TrayError


class Config:
    repo_root = Path("E:/Git/ZirconEngine")
    pilot_root = Path("D:/cargo-targets/jenkins-pilot-tray-test")
    state_dir = Path("E:/Git/ZirconEngine/.codex/state/jenkins-tray/test")
    root_identity = {"fileId": "root"}


def preview(**changes):
    value = {"confirmationRequired": True, "createdAt": time.time(),
             "rootIdentity": dict(Config.root_identity), "activeBuilds": [],
             "controller": {"pid": 1, "creationTime": "c"},
             "agent": {"pid": 2, "creationTime": "a"},
             "queuedCount": 0,
             "status": {"state": "busy", "canStop": True, "quietDown": False}}
    value.update(changes)
    return value


class LifecycleGuardTests(unittest.TestCase):
    def test_completed_shutdown_allows_matching_pending_binding_marker(self):
        bindings = [{"job": "zircon-pilot", "buildNumber": 1}]
        lifecycle_record = {"state": "completed", "operationId": "original-stop", "rootIdentity": dict(Config.root_identity),
                           "bindings": bindings, "terminal": {"verified": True}}
        pending_marker = {"state": "pending", "operationId": "original-stop", "rootIdentity": dict(Config.root_identity),
                          "bindings": bindings}
        config = Config()
        for marker_id, expected in (("original-stop", "ready"), ("different-stop", "error"), (None, "error")):
            with self.subTest(operationId=marker_id):
                marker = {**pending_marker, "operationId": marker_id}
                api = Mock()
                api.api.side_effect = [{"quietingDown": False}, {"offline": False}]
                with patch.object(service, "validate_runtime", return_value={}), \
                     patch.object(service, "_runtime", return_value=({}, {}, "http://127.0.0.1:1/")), \
                     patch.object(service, "_owner", return_value={"live": True}), \
                     patch.object(service, "read_json", side_effect=[marker, lifecycle_record]), \
                     patch.object(service, "_runtime_evidence", return_value=True), \
                     patch.object(service, "_client", return_value=api), \
                     patch.object(service, "_builds", return_value=([], 0)):
                    result = service.read_status(config)
                self.assertEqual(result["state"], expected)
                self.assertEqual(result["canStop"], expected == "ready")

    def test_confirmation_rejects_future_timestamp(self):
        p = preview(createdAt=time.time() + 30)
        with self.assertRaises(service.ConfirmationRequired):
            lifecycle._check_confirmation(Config(), p, p)

    def test_confirmation_rejects_stale_snapshot(self):
        p = preview()
        confirmation = dict(p)
        confirmation["createdAt"] = time.time() - 121
        with self.assertRaises(service.ConfirmationRequired):
            lifecycle._check_confirmation(Config(), p, confirmation)

    def test_live_keeper_does_not_enter_recovery(self):
        owner = {"live": True, "authorityId": "a"}
        with patch.object(service, "_owner", return_value=owner), patch.object(lifecycle, "read_json", return_value=None), \
             patch("tools.jenkins.pilot.governance_recovery.recover_storage") as recover:
            self.assertEqual(lifecycle.ensure_owner(Config()), owner)
            recover.assert_not_called()

    def test_runtime_terminal_rejects_live_process(self):
        record = {"pid": 5, "creationTime": "birth", "keeperPid": 7, "keeperCreationTime": "kb", "keeperJobHandle": 1}
        with patch.object(service, "_live", return_value=True):
            with self.assertRaises(TrayError):
                lifecycle._runtime_terminal({"controller": record})

    def test_runtime_terminal_requires_job_zero_and_log_release(self):
        record = {"pid": 5, "creationTime": "birth", "keeperPid": 7, "keeperCreationTime": "kb", "keeperJobHandle": 1}
        with patch.object(service, "_live", return_value=False), patch("tools.jenkins.tray.terminal._job_zero", side_effect=TrayError("active job")):
            with self.assertRaises(TrayError):
                lifecycle._runtime_terminal({"controller": record})

    def test_failed_start_reconcile_does_not_close_without_terminal_proof(self):
        previous = {"action": "start", "rootIdentity": dict(Config.root_identity)}
        status = {"controller": {"ready": False, "live": False}, "agent": {"online": False, "live": False}}
        runtime = {"controller": {"pid": 5}}
        with patch.object(lifecycle, "validate_runtime"), patch.object(service, "read_status", return_value=status), \
             patch.object(service, "_runtime", return_value=(runtime, {}, "")), patch.object(lifecycle, "read_json", return_value=None), \
             patch.object(lifecycle, "_runtime_terminal", side_effect=TrayError("missing EOF")):
            with self.assertRaises(TrayError):
                lifecycle.reconcile_runtime(Config(), previous)


if __name__ == "__main__":
    unittest.main()
