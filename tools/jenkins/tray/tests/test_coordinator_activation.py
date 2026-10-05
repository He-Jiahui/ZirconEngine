import json
import sqlite3
import shutil
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.jenkins_coordinator import activate, require_active
from tools.jenkins.tray import terminal
from tools.jenkins.tray.config import TrayError


ROOT = Path("D:/cargo-targets/zircon-local/evidence/jenkins-coordinator-activation-tests")


class Config:
    repo_root = Path("E:/Git/ZirconEngine")
    pilot_root = ROOT / "pilot"
    config_path = ROOT / "profile.json"
    root_identity = {"fileId": "root"}
    python_executable = Path(sys.executable)


class ActivationGateTests(unittest.TestCase):
    def setUp(self):
        ROOT.mkdir(parents=True, exist_ok=True)
        self.tray = ROOT / "tray.json"
        self.pilot = ROOT / "pilot.json"
        self.tray.write_text(json.dumps({"accepted": True, "acceptance": "jenkins-windows-tray",
                                         "driverInputHash": "d" * 64, "sourceHashes": {"tools/jenkins/tray/service.py": "s" * 64},
                                         "allRequirementsPassed": True}), encoding="utf-8")
        self.pilot.write_text(json.dumps({"accepted": True, "acceptance": "independent-jenkins-pilot",
                                          "root": str(Config.pilot_root), "rootIdentity": Config.root_identity,
                                          "driver": {"driverInputHash": "d" * 64},
                                          "gates": {f"I{i}": "passed" for i in range(1, 7)},
                                          "retirementMarkerSha256": "m" * 64}), encoding="utf-8")

    def tearDown(self):
        shutil.rmtree(ROOT, ignore_errors=True)

    def _patch_common(self):
        return patch.multiple(
            "tools.jenkins.jenkins_coordinator",
            validate_runtime=lambda config: {},
            read_status=lambda config: {"state": "ready", "owner": {"live": True}, "quietDown": False, "url": "http://127.0.0.1:1/"},
            require_live_storage_owner=lambda *args: {"authorityId": "owner"},
            verify_driver=lambda root: "d" * 64,
            file_sha256=lambda path: "m" * 64 if str(path).endswith("coordinator-retirement.json") else "s" * 64,
            write_json=unittest.mock.DEFAULT,
        )

    def test_pending_evidence_is_refused_without_writes(self):
        pending = json.loads(self.tray.read_text(encoding="utf-8")); pending.pop("accepted")
        self.tray.write_text(json.dumps(pending), encoding="utf-8")
        with patch("tools.jenkins.jenkins_coordinator.validate_runtime"), patch("tools.jenkins.jenkins_coordinator.read_status", return_value={"state": "ready", "owner": {"live": True}, "quietDown": False}), \
             patch("tools.jenkins.jenkins_coordinator.require_live_storage_owner", return_value={}), patch("tools.jenkins.jenkins_coordinator.verify_driver", return_value="d" * 64), patch("tools.jenkins.jenkins_coordinator.write_json") as write:
            with self.assertRaises(TrayError):
                activate(Config(), tray_evidence=self.tray, pilot_evidence=self.pilot)
            write.assert_not_called()

    def test_driver_or_gate_mismatch_is_refused(self):
        bad = json.loads(self.pilot.read_text(encoding="utf-8")); bad["gates"]["I3"] = "failed"
        self.pilot.write_text(json.dumps(bad), encoding="utf-8")
        with patch("tools.jenkins.jenkins_coordinator.validate_runtime"), patch("tools.jenkins.jenkins_coordinator.read_status", return_value={"state": "ready", "owner": {"live": True}, "quietDown": False}), \
             patch("tools.jenkins.jenkins_coordinator.require_live_storage_owner", return_value={}), patch("tools.jenkins.jenkins_coordinator.verify_driver", return_value="d" * 64), \
             patch("tools.jenkins.jenkins_coordinator.write_json") as write:
            with self.assertRaises(TrayError):
                activate(Config(), tray_evidence=self.tray, pilot_evidence=self.pilot)
            write.assert_not_called()

    def test_live_config_root_mismatch_is_refused(self):
        with patch("tools.jenkins.jenkins_coordinator.validate_runtime", side_effect=TrayError("root changed")), \
             patch("tools.jenkins.jenkins_coordinator.write_json") as write:
            with self.assertRaises(TrayError):
                activate(Config(), tray_evidence=self.tray, pilot_evidence=self.pilot)
            write.assert_not_called()

    def test_require_active_rechecks_tray_source_hashes(self):
        active = ROOT / "active.json"
        active.write_text(json.dumps({"enabled": True, "functionalTestsAllowed": True,
                                      "configFile": str(Config.config_path), "repoRoot": str(Config.repo_root),
                                      "root": str(Config.pilot_root), "rootIdentity": Config.root_identity,
                                      "driverInputHash": "d" * 64,
                                      "trayEvidence": {"path": str(self.tray), "sha256": "x" * 64},
                                      "pilotEvidence": {"path": str(self.pilot), "sha256": "y" * 64},
                                      "retirementMarkerSha256": "m" * 64}), encoding="utf-8")
        with patch("tools.jenkins.jenkins_coordinator.read_json", side_effect=[json.loads(active.read_text()), json.loads(self.tray.read_text()), json.loads(self.pilot.read_text())]), \
             patch("tools.jenkins.jenkins_coordinator.load_config", return_value=Config()), patch("tools.jenkins.jenkins_coordinator.validate_runtime"), \
             patch("tools.jenkins.jenkins_coordinator.require_live_storage_owner"), patch("tools.jenkins.jenkins_coordinator.verify_driver", return_value="d" * 64), \
             patch("tools.jenkins.jenkins_coordinator.file_sha256", side_effect=lambda path: "changed" if str(path).endswith("service.py") else "x" * 64):
            with self.assertRaises(TrayError):
                require_active(Config.repo_root)


class ReadonlyJournalTests(unittest.TestCase):
    def test_journal_rows_use_existing_database_readonly(self):
        root = ROOT / "journal"
        root.mkdir(parents=True, exist_ok=True)
        db = root / "submissions.sqlite3"
        with sqlite3.connect(db) as con:
            con.execute("CREATE TABLE submissions (job TEXT, build_number INTEGER, session_id TEXT, request_id TEXT, identity_json TEXT, bundle_hash TEXT)")
            con.execute("INSERT INTO submissions VALUES ('zircon-pilot', 4, 's', 'r', '{}', 'b')")
        class Windows:
            def open_relative(self, name, mode): return db.open(mode)
        class Backend:
            def exists(self, name): return name == "submissions.sqlite3"
            _windows = Windows()
        with patch.object(terminal, "WorkerStorage") as storage:
            storage.return_value.__enter__.return_value = Backend()
            rows = terminal._journal_rows(type("C", (), {"pilot_root": root})())
        self.assertEqual(rows[0]["job"], "zircon-pilot")


if __name__ == "__main__":
    unittest.main()
