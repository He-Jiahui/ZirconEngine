from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.tray.formal import load_formal_config, status
from tools.jenkins.tray import formal_native


class FormalTrayTests(unittest.TestCase):
    def test_unknown_identity_is_not_terminable(self):
        with patch.object(formal_native, "identity", return_value=None):
            self.assertFalse(formal_native.terminate_owned({"pid": 1, "birth": "x", "executable": "x"}))

    def test_pid_reuse_is_not_terminable(self):
        record = {"pid": 1, "birth": "old", "executable": "E:/jenkins.exe"}
        with patch.object(formal_native, "identity", return_value={"pid": 1, "birth": "new", "executable": "E:/jenkins.exe"}):
            self.assertFalse(formal_native.terminate_owned(record))

    def test_formal_config_uses_repo_home_and_external_build_root(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); runtime = root / ".jenkins" / "runtime"; (runtime / "jdk" / "bin").mkdir(parents=True)
            java = runtime / "jdk" / "bin" / "java.exe"; war = runtime / "jenkins.war"; java.write_bytes(b"x"); war.write_bytes(b"x")
            spec = root / "spec.json"
            spec.write_text(json.dumps({"schemaVersion":1,"repositoryRoot":str(root),"controller":{"version":"2.580.1","java":{"version":"21.0.12.1"},"listenAddress":"127.0.0.1","httpPort":53748,"executors":0,"jenkinsHome":str(root / ".jenkins" / "jenkins_home")},"agent":{},"storage":{"allowedPhysicalRoots":[r"D:\cargo-targets"],"buildRoot":r"D:\cargo-targets"}}), encoding="utf-8")
            with patch("tools.jenkins.tray.formal.java_major", return_value=21):
                config = load_formal_config(spec, java=java, war=war)
            self.assertEqual(config.paths.home, root / ".jenkins" / "jenkins_home")
            self.assertEqual(config.paths.build_root, Path(r"D:\cargo-targets"))

    def test_status_is_stopped_without_state(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); runtime = root / ".jenkins" / "runtime"; (runtime / "jdk" / "bin").mkdir(parents=True)
            java = runtime / "jdk" / "bin" / "java.exe"; war = runtime / "jenkins.war"; java.write_bytes(b"x"); war.write_bytes(b"x")
            spec = root / "spec.json"; spec.write_text(json.dumps({"schemaVersion":1,"repositoryRoot":str(root),"controller":{"version":"2.580.1","java":{"version":"21.0.12.1"},"listenAddress":"127.0.0.1","httpPort":53748,"executors":0,"jenkinsHome":str(root / ".jenkins" / "jenkins_home")},"agent":{},"storage":{"allowedPhysicalRoots":[r"D:\cargo-targets"],"buildRoot":r"D:\cargo-targets"}}), encoding="utf-8")
            with patch("tools.jenkins.tray.formal.java_major", return_value=21):
                config = load_formal_config(spec, java=java, war=war)
            self.assertEqual(status(config)["state"], "stopped")


if __name__ == "__main__": unittest.main()
