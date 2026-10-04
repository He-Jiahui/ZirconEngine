from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.plugins import PluginMaintenance
from tools.jenkins.deployment.offline import rollback as legacy_rollback
from tools.jenkins.processes.identity import current_identity


class PluginMaintenanceTests(unittest.TestCase):
    def _roots(self):
        root = Path(tempfile.mkdtemp())
        home = root / ".jenkins" / "jenkins_home"; home.mkdir(parents=True)
        state = root / ".jenkins" / "state"; state.mkdir()
        return root, home, state

    def _receipt(self, state: Path, *, live=False):
        if live:
            identity = current_identity(__import__("os").getpid()).to_dict()
        else:
            process = subprocess.Popen(["cmd.exe", "/c", "exit", "0"])
            process.wait()
            identity = {"pid": process.pid, "creationTime": "dead-process-proof", "executable": "java.exe"}
        proof = {"complete": True, "activeProcesses": 0, "stdoutEof": True,
                 "stderrEof": True, "identity": identity}
        value = {"operationId": "maintenance-1", "state": "stopped",
                 "homePath": str(state.parent / "jenkins_home"),
                 "nativeTerminationProof": proof, "agentTerminationProof": proof}
        (state / "deployment-operation.json").write_text(json.dumps(value), encoding="utf-8")
        (state / "deployment").mkdir()
        (state / "deployment" / "host.json").write_text(json.dumps({
            "status": "stopped", "operationId": "maintenance-1", "generation": None,
            "nativeTerminationProof": proof, "agentTerminationProof": proof}), encoding="utf-8")

    def _stage(self, root: Path, content=b"plugin"):
        stage = root / "stage"; stage.mkdir()
        archive = stage / "git.jpi"; archive.write_bytes(content)
        digest = hashlib.sha256(content).hexdigest()
        return stage, {"schemaVersion": 1, "jenkinsVersion": "2.580.1",
                       "operationId": "maintenance-1", "plugins": {
                           "git": {"file": "git.jpi", "sha256": digest,
                                   "requiredCore": "2.400", "dependencies": []}}}

    def test_install_requires_current_native_stop_receipt_and_rolls_back(self):
        root, home, state = self._roots(); self._receipt(state)
        stage, manifest = self._stage(root)
        (home / "plugins").mkdir(); (home / "plugins" / "old.jpi").write_bytes(b"old")
        result = PluginMaintenance(home, state).install(stage, manifest)
        self.assertEqual(result["state"], "installed")
        self.assertEqual((home / "plugins" / "git.jpi").read_bytes(), b"plugin")
        self.assertTrue((home / "plugin-backups" / "maintenance-1" / "previous-plugins" / "old.jpi").exists())

    def test_running_or_forged_proof_is_rejected(self):
        root, home, state = self._roots(); self._receipt(state, live=True)
        stage, manifest = self._stage(root)
        with self.assertRaises(JenkinsError) as error:
            PluginMaintenance(home, state).install(stage, manifest)
        self.assertEqual(error.exception.code, "stop_proof_invalid")

    def test_checksum_or_dependency_tamper_is_rejected_before_home_change(self):
        root, home, state = self._roots(); self._receipt(state)
        stage, manifest = self._stage(root, b"tampered")
        manifest["plugins"]["git"]["sha256"] = "0" * 64
        with self.assertRaises(JenkinsError): PluginMaintenance(home, state).install(stage, manifest)
        self.assertFalse((home / "plugins").exists())

    def test_activation_failure_restores_previous_set_and_journals_rollback(self):
        root, home, state = self._roots(); self._receipt(state)
        stage, manifest = self._stage(root)
        (home / "plugins").mkdir(); (home / "plugins" / "old.jpi").write_bytes(b"old")
        with patch("tools.jenkins.deployment.plugins.shutil.copy2", side_effect=OSError("disk fault")):
            with self.assertRaises(JenkinsError) as error:
                PluginMaintenance(home, state).install(stage, manifest)
        self.assertEqual(error.exception.code, "plugin_install_rolled_back")
        self.assertEqual((home / "plugins" / "old.jpi").read_bytes(), b"old")
        journal = json.loads((state / "plugin-maintenance" / "maintenance-1.json").read_text())
        self.assertEqual(journal["state"], "rolled-back")

    def test_legacy_rollback_wrapper_rejects_live_receipt(self):
        root, home, state = self._roots(); self._receipt(state, live=True)
        backup = home / "plugin-backups" / "maintenance-1"; (backup / "plugins").mkdir(parents=True)
        (backup / "plugins" / "old.jpi").write_bytes(b"old")
        (home / "plugins").mkdir(); (home / "plugins" / "live.jpi").write_bytes(b"live")
        with self.assertRaises(JenkinsError) as error:
            legacy_rollback(home, backup)
        self.assertEqual(error.exception.code, "stop_proof_invalid")
        self.assertTrue((home / "plugins" / "live.jpi").exists())

    def test_legacy_rollback_rejects_receipt_from_another_home(self):
        root, home, state = self._roots(); self._receipt(state)
        other = root / "other" / ".jenkins" / "jenkins_home"; other.mkdir(parents=True)
        value = json.loads((state / "deployment-operation.json").read_text())
        value["homePath"] = str(other)
        (state / "deployment-operation.json").write_text(json.dumps(value))
        backup = home / "plugin-backups" / "maintenance-1"; (backup / "plugins").mkdir(parents=True)
        with self.assertRaises(JenkinsError) as error:
            legacy_rollback(home, backup)
        self.assertEqual(error.exception.code, "stop_proof_home_mismatch")


if __name__ == "__main__": unittest.main()
