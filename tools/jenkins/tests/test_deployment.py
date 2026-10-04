from __future__ import annotations

import json
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

from tools.jenkins.deployment.migration import migrate_home
from tools.jenkins.deployment.paths import resolve_paths
from tools.jenkins.resources.paths import ApprovedBuildRoot
from tools.jenkins.deployment.plugins import load_checksum_manifest, verify_staged_plugins
from tools.jenkins.deployment.spec import DeploymentSpec, JenkinsError, load_spec


class DeploymentTests(unittest.TestCase):
    def spec(self, root: Path) -> DeploymentSpec:
        payload = {"schemaVersion": 1, "repositoryRoot": str(root), "controller": {"version": "2.580.1", "java": {"version": "21.0.12.1"}, "executors": 0, "listenAddress": "127.0.0.1", "jenkinsHome": str(root / ".jenkins" / "jenkins_home")}, "agent": {}, "storage": {"allowedPhysicalRoots": [r"D:\cargo-targets"], "buildRoot": r"D:\cargo-targets"}}
        path = root / "deployment.json"; path.write_text(json.dumps(payload), encoding="utf-8")
        return load_spec(path)

    def test_paths_keep_home_in_repo_and_build_root_external(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / "cargo-targets").mkdir()
            with patch("tools.jenkins.deployment.paths.canonical_build_root", return_value=ApprovedBuildRoot(root / "cargo-targets")), patch("tools.jenkins.deployment.paths.build_namespace", return_value=root / "cargo-targets" / "zircon-jenkins"), patch("tools.jenkins.deployment.paths._physical", return_value=root / "cargo-targets"):
                paths = resolve_paths(self.spec(root))
            self.assertEqual(paths.home, root / ".jenkins" / "jenkins_home")
            self.assertEqual(paths.build_root, root / "cargo-targets")
            self.assertNotEqual(paths.home.drive, paths.build_root.drive) if paths.home.drive != paths.build_root.drive else None

    def test_rejects_unapproved_build_root(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); (root / "cargo-targets").mkdir()
            with patch("tools.jenkins.deployment.paths.canonical_build_root", side_effect=JenkinsError("invalid_build_root", "test")):
                with self.assertRaises(JenkinsError): resolve_paths(self.spec(root), root / "other")

    def test_plugin_checksum_manifest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); plugin = root / "git.jpi"; plugin.write_bytes(b"plugin")
            import hashlib
            digest = hashlib.sha256(b"plugin").hexdigest()
            manifest = {"schemaVersion": 1, "plugins": {"git": {"file": "git.jpi", "sha256": digest}}}
            self.assertEqual(verify_staged_plugins(root, load_checksum_manifest(self.write_manifest(root, manifest))), {"git": digest})

    def write_manifest(self, root: Path, value: dict) -> Path:
        path = root / "checksums.json"; path.write_text(json.dumps(value), encoding="utf-8"); return path

    def test_home_migration_verifies_tree_digest(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); source = root / "old"; destination = root / "new"; source.mkdir(); (source / "config.xml").write_text("x")
            proof = root / "stop.json"; proof.write_text(json.dumps({"state":"stopped","activeJobs":0,"queuedJobs":0,"terminalVerified":True}))
            result = migrate_home(source, destination, stop_proof=proof)
            self.assertEqual(result["sourceDigest"], result["destinationDigest"])


if __name__ == "__main__": unittest.main()
