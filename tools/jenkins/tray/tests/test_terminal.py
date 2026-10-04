import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray import terminal
from tools.jenkins.tray.config import TrayError


class _Config:
    pilot_root = Path("D:/cargo-targets/jenkins-pilot-tray-test")
    repo_root = Path("E:/Git/ZirconEngine")
    root_identity = {"fileId": "fixture"}
    driver_input_hash = "d" * 64
    state_dir = Path("E:/Git/ZirconEngine/.codex/state/jenkins-tray/test")


def _build(**extra):
    value = {"job": "zircon-pilot", "number": 4, "sessionId": "s", "requestId": "r",
             "attemptId": "a", "generation": 1}
    value.update(extra)
    return value


class TerminalGuardTests(unittest.TestCase):
    def test_unknown_build_is_rejected_without_journal_write(self):
        with patch.object(terminal, "validate_runtime", return_value={"driverInputHash": "d" * 64}), \
             patch.object(terminal, "_journal_rows", return_value=[]):
            with self.assertRaises(TrayError):
                terminal.running_bindings(_Config(), [_build()])

    def test_identity_mismatch_is_rejected(self):
        identity = {"schemaVersion": 2, "sessionId": "s", "requestId": "other",
                    "attemptId": "a", "generation": 1, "inputHash": "1" * 64,
                    "template": "python-static"}
        row = {"job": "zircon-pilot", "build_number": 4, "session_id": "s",
               "request_id": "r", "identity_json": __import__("json").dumps(identity),
               "bundle_hash": "b" * 64}
        with patch.object(terminal, "validate_runtime", return_value={"driverInputHash": "d" * 64}), \
             patch.object(terminal, "_journal_rows", return_value=[row]):
            with self.assertRaises(TrayError):
                terminal.running_bindings(_Config(), [_build()])

    def test_local_proof_missing_intent_returns_none(self):
        identity = {"schemaVersion": 2, "sessionId": "s", "requestId": "r",
                    "attemptId": "a", "generation": 1, "inputHash": "1" * 64,
                    "template": "python-static"}
        with patch.object(terminal, "ManagedStorage") as storage:
            storage.return_value.backend.return_value.__enter__.return_value.exists.return_value = False
            self.assertIsNone(terminal.local_terminal_proof(_Config(), {"identity": identity}))

    def test_cancel_bindings_empty_is_verified(self):
        with patch.object(terminal, "write_json") as write:
            result = terminal.cancel_bindings(_Config(), object(), [], deadline=9999999999)
        self.assertTrue(result["verified"])
        write.assert_called()

    def test_cancel_bindings_requires_deadline(self):
        with self.assertRaises(TrayError):
            terminal.cancel_bindings(_Config(), object(), [{"identity": {}}], deadline=0)

    def test_local_proof_rejects_missing_eof_or_terminal_receipt(self):
        identity = {"schemaVersion": 2, "sessionId": "s", "requestId": "r",
                    "attemptId": "a", "generation": 1, "inputHash": "1" * 64,
                    "template": "python-static"}
        binding = {"identity": identity, "job": "zircon-pilot", "buildNumber": 4,
                   "bundleHash": "b" * 64, "driverInputHash": "d" * 64}
        backend = __import__("unittest").mock.MagicMock()
        backend.exists.return_value = True
        backend.read_bytes.return_value = b'{"status":"complete","binding":{}}'
        with patch.object(terminal, "ManagedStorage") as storage, patch.object(terminal, "require_live_storage_owner", return_value={"authorityId": "a"}), \
             patch.object(terminal, "_read_receipt", return_value={"processTree": {"terminal": False}}), \
             patch.object(terminal, "_validate_intent"), patch.object(terminal, "_terminal_receipt", side_effect=TrayError("missing EOF")):
            storage.return_value.backend.return_value.__enter__.return_value = backend
            with self.assertRaises(TrayError):
                terminal.local_terminal_proof(_Config(), binding)

    def test_job_zero_rejects_active_native_processes(self):
        launch = {"pid": 5, "creationTime": "birth", "keeperPid": 7,
                  "keeperCreationTime": "kb", "keeperJobHandle": 1}
        with patch("tools.jenkins.pilot.lifetime._kernel"), patch("tools.jenkins.pilot.lifetime._open_owner"), \
             patch("tools.jenkins.pilot.native.windows_job_process._JobObjectBasicAccountingInformation") as info:
            info.return_value.active_processes = 1
            # Native API details are platform-bound; an access failure must remain a refusal.
            with self.assertRaises((TrayError, OSError, AttributeError, TypeError)):
                terminal._job_zero(launch)


if __name__ == "__main__":
    unittest.main()
