"""Current physical build-root policy; historical repository stores stay read-only."""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.paths import resolve_paths
from tools.jenkins.deployment.spec import load_spec
from tools.jenkins.resources.paths import canonical_build_root, physical_path_under

REPO = Path(__file__).absolute().parents[3]
BUILD_ROOT = Path(r"D:\cargo-targets")


@unittest.skipUnless(os.name == "nt", "Windows physical path admission")
class RepositoryBuildRootTests(unittest.TestCase):
    def test_default_and_explicit_root_have_same_physical_identity(self):
        explicit = canonical_build_root(BUILD_ROOT)
        self.assertEqual(explicit.path, BUILD_ROOT)
        self.assertEqual(canonical_build_root(None), explicit)
        self.assertTrue(explicit.volume_identity)
        self.assertEqual(explicit.namespace(), BUILD_ROOT / "zircon-local" / "zircon-jenkins")

    def test_repository_paths_and_lookalike_roots_are_rejected(self):
        for path in (REPO / "target", REPO / ".jenkins/jenkins_home",
                     REPO / ".jenkins/builds", REPO / "nested/.jenkins/builds",
                     r"D:\targets", r"E:\ZirconBuilds", r"D:\nested\cargo-targets",
                     str(BUILD_ROOT) + r"\..\cargo-targets"):
            with self.subTest(path=path), self.assertRaises(JenkinsError):
                canonical_build_root(path)

    def test_build_paths_cannot_escape_into_home(self):
        approved = canonical_build_root(BUILD_ROOT)
        candidate = approved.namespace() / "preparations/new/tmp"
        self.assertEqual(physical_path_under(approved, candidate), candidate)
        with self.assertRaises(JenkinsError):
            physical_path_under(approved, REPO / ".jenkins/jenkins_home/output")

    def test_spec_rejects_repository_and_policy_mismatches(self):
        original = json.loads((REPO / ".jenkins/deployment-spec.json").read_text(encoding="utf-8"))
        cases = []
        for field in ("buildRoot", "allowedPhysicalRoots"):
            raw = copy.deepcopy(original)
            raw["storage"][field] = (str(REPO / ".jenkins/builds") if field == "buildRoot"
                                    else [str(REPO / ".jenkins/builds")])
            cases.append(raw)
        raw = copy.deepcopy(original)
        raw["storage"]["resourcePolicy"]["buildRoot"] = r"E:\cargo-targets"
        cases.append(raw)
        raw = copy.deepcopy(original)
        raw["controller"]["jenkinsHome"] = str(REPO.parent / "Other/.jenkins/jenkins_home")
        cases.append(raw)
        with tempfile.TemporaryDirectory() as directory:
            spec_path = Path(directory) / "deployment.json"
            for raw in cases:
                spec_path.write_text(json.dumps(raw), encoding="utf-8")
                with self.subTest(storage=raw["storage"]), self.assertRaises(JenkinsError):
                    load_spec(spec_path)

    def test_formal_spec_uses_external_namespace_and_preserves_explicit_root(self):
        spec = load_spec(REPO / ".jenkins/deployment-spec.json")
        paths = resolve_paths(spec)
        self.assertEqual(paths.build_root, BUILD_ROOT)
        self.assertEqual(paths.build_namespace, BUILD_ROOT / "zircon-local/zircon-jenkins")
        explicit = resolve_paths(spec, r"E:\cargo-targets")
        self.assertEqual(explicit.build_root, Path(r"E:\cargo-targets"))
        self.assertEqual(paths.home, REPO / ".jenkins/jenkins_home")

    def test_original_root_string_is_rejected_before_path_normalization(self):
        spec = load_spec(REPO / ".jenkins/deployment-spec.json")
        for raw in (r"D:\cargo-targets\.", r"D:\x\..\cargo-targets",
                    "D:\\\\cargo-targets", str(BUILD_ROOT) + "\\"):
            with self.subTest(raw=raw), patch("tools.jenkins.deployment.paths.canonical_build_root") as native:
                with self.assertRaises(JenkinsError):
                    resolve_paths(spec, raw)
                native.assert_not_called()

    def test_new_cli_writes_use_policy_and_recovery_keeps_original_identity(self):
        from tools.jenkins.cli import configured_build_payload
        self.assertEqual(str(BUILD_ROOT), configured_build_payload("candidate", "apply", {}, REPO)["buildRoot"])
        with self.assertRaises(JenkinsError):
            configured_build_payload("request", "submit", {"buildRoot": str(REPO / ".jenkins/builds")}, REPO)
        historical = {"buildRoot": str(REPO / ".jenkins/builds")}
        self.assertEqual(historical, configured_build_payload("flow", "reconcile-flow", historical, REPO))
