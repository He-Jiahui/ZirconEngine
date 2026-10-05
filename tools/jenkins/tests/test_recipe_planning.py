from __future__ import annotations

import tempfile
import unittest
import json
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.state import State
from tools.jenkins.workflow.planning import RecipePlanner, get_recipe_plan
from tools.jenkins.resources.paths import repository_build_root


class RecipePlanningTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.state = State(self.root / "state.sqlite3")
        self.source = "a" * 64
        self.driver = {"digest": "b" * 64, "generation": "driver-generation-1"}
        self.build_root = repository_build_root()
        self.workspace = str(self.build_root / 'sealed-recipe-tests/workspace')
        self.metadata = {"packages": [{"id": "demo 0.1", "name": "demo",
                                        "manifest_path": self.workspace + r"\Cargo.toml",
                                        "targets": [{"name": "demo", "kind": ["lib"],
                                                     "src_path": self.workspace + r"\crates\demo\src\lib.rs"}],
                                        "dependencies": []}]}
        self.state.put("sealed_input", self.source, {
            "status": "sealed", "sourceDigest": self.source,
            "manifest": {"entries": [{"path": "crates/demo/src/lib.rs", "status": "present"}]},
            "coverage": {"sourceDigest": self.source, "coverageDigest": "c" * 64,
                         "selectedTests": 1}, "coverageDigest": "c" * 64,
            "toolchain": "stable", "toolchainDigest": "d" * 64,
            "lockDigest": "e" * 64, "compilerIdentity": "rustc-1",
        })
        self.cargo = self.root / "cargo.exe"
        self.cargo.write_bytes(b"fixture")
        (self.root / "Cargo.lock").write_text("# fixture\n", encoding="utf-8")
        self.environment_file = self.root / "msvc-environment.json"
        self.environment = {"INCLUDE": "D:\\sealed\\include", "LIB": "D:\\sealed\\lib",
                            "LIBPATH": "D:\\sealed\\libpath", "Path": "D:\\sealed\\bin",
                            "VCINSTALLDIR": "D:\\sealed\\vc", "VCToolsVersion": "14.44.35207",
                            "CARGO_NET_OFFLINE": "true", "CARGO_INCREMENTAL": "1"}
        self.environment_file.write_text(json.dumps(self.environment, sort_keys=True), encoding="utf-8")
        canonical_environment = {"CARGO_INCREMENTAL": "1", "CARGO_NET_OFFLINE": "true",
                                 "INCLUDE": self.environment["INCLUDE"], "LIB": self.environment["LIB"],
                                 "LIBPATH": self.environment["LIBPATH"], "PATH": self.environment["Path"],
                                 "VCINSTALLDIR": self.environment["VCINSTALLDIR"],
                                 "VCToolsVersion": self.environment["VCToolsVersion"]}
        self.state.put("environment_policy", "default", {
            "schemaVersion": 1, "kind": "jenkins.trusted.build-environment", "status": "verified",
            "environmentFile": str(self.environment_file), "fileDigest": file_digest(self.environment_file),
            "environment": canonical_environment, "environmentDigest": digest(canonical_environment),
            "cargoPath": str(self.cargo), "cargoSha256": file_digest(self.cargo),
        })
        self.state.put("resource_policy", "default", {
            "estimateAuthority": "state.policy/validated-profile",
            "recipeEstimates": {"default": {"cpu": 1, "memoryBytes": 1024, "diskBytes": 2048},
                                 "tiny-fixture": {"cpu": 1, "memoryBytes": 256, "diskBytes": 64}},
        })
        self.state.put("sealed_cargo_evidence", self.source, {
            "sourceDigest": self.source, "manifestDigest": self.source,
            "cargoMetadata": self.metadata, "metadataDigest": digest(self.metadata),
            "toolchain": {"cargo": "cargo 1.80", "rustc": "rustc 1.80"},
            "toolchainDigest": "f" * 64, "lockDigest": "e" * 64,
            "compilerIdentity": "rustc-1", "workspaceRoot": self.workspace,
        })

    def tearDown(self):
        self.temp.cleanup()

    def planner(self):
        return RecipePlanner(self.state, repo_root=self.root, cargo_path=self.cargo)

    def test_plan_derives_commands_and_persists_immutable_reference(self):
        plan = self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                   changed_paths=["crates/demo/src/lib.rs"])
        self.assertEqual(plan.recipe["kind"], "cargo")
        self.assertEqual(plan.recipe["commands"]["compile"][-1], "--no-run")
        self.assertEqual(plan.recipe["commands"]["list"][-2:], ["--", "--list"])
        self.assertEqual(plan.recipe["requiredPhases"], ["compile", "list", "test"])
        self.assertEqual(plan.recipe["workspaceRoot"], self.workspace)
        self.assertEqual(plan.recipe["cargoMetadata"]["packages"][0]["targets"][0]["src_path"],
                         "crates/demo/src/lib.rs")
        record = get_recipe_plan(self.state, plan.recipe_ref)
        self.assertEqual(record["recipeDigest"], plan.recipe_ref)
        self.state.put("recipe_plan", plan.recipe_ref,
                       {**record, "recipe": {**record["recipe"], "command": ["del", "C:\\"]}},
                       expected_version=self.state.get("recipe_plan", plan.recipe_ref)["version"])
        with self.assertRaises(JenkinsError):
            get_recipe_plan(self.state, plan.recipe_ref)

    def test_unknown_impact_is_rejected(self):
        with self.assertRaisesRegex(JenkinsError, "unknown impact"):
            self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                changed_paths=["missing/unknown.rs"])

    def test_native_fixture_is_explicitly_non_cargo(self):
        plan = self.planner().plan_native_fixture(source_ref=self.source, driver=self.driver,
                                                  command=["python", "-c", "print(1)"])
        self.assertEqual(plan.recipe["kind"], "native-validation")
        self.assertTrue(get_recipe_plan(self.state, plan.recipe_ref)["fixtureOnly"])
        with self.assertRaises(JenkinsError):
            self.planner().plan_native_fixture(source_ref=self.source, driver=self.driver,
                                               command=["cargo", "test"], fixture_kind="cargo")

    def test_sealed_identity_and_driver_generation_are_required(self):
        with self.assertRaises(JenkinsError):
            self.planner().plan(sealed_input_ref="d" * 64, driver=self.driver)
        with self.assertRaises(JenkinsError):
            self.planner().plan(sealed_input_ref=self.source, driver={"digest": "b" * 64})

    def test_default_features_and_profile_are_explicit_in_every_cargo_command(self):
        plan = self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                   changed_paths=["crates/demo/src/lib.rs"], profile="release")
        for command in plan.recipe["commands"].values():
            self.assertIn("--profile", command)
            self.assertNotIn("--no-default-features", command)
        self.assertNotIn(self.source, plan.recipe["preparationKey"])

    def test_zero_discovery_is_pending_until_receipt(self):
        plan = self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                   changed_paths=["crates/demo/src/lib.rs"])
        self.assertEqual(plan.recipe["testCount"], 0)
        self.assertEqual(plan.recipe["testAllocation"]["zeroTests"], "reject")

    def test_authoritative_cargo_evidence_is_required(self):
        self.state.delete("sealed_cargo_evidence", self.source)
        with self.assertRaisesRegex(JenkinsError, "authoritative sealed Cargo evidence"):
            self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                changed_paths=["crates/demo/src/lib.rs"])

    def test_environment_policy_and_resource_profile_are_bound_to_recipe(self):
        self.state.put("validation_profile", self.source, {"profile": "tiny-fixture"})
        plan = self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                   changed_paths=["crates/demo/src/lib.rs"])
        self.assertEqual(plan.recipe["resourceProfile"], "tiny-fixture")
        self.assertEqual(plan.recipe["resource"], {"cpu": 1, "memoryBytes": 256, "diskBytes": 64})
        execution = plan.recipe["executionEnvironment"]
        self.assertEqual(execution["INCLUDE"], self.environment["INCLUDE"])
        self.assertEqual(execution["PATH"], self.environment["Path"])
        self.assertEqual(execution["RUSTC_WRAPPER"], "")
        self.assertIn(str(self.build_root / 'zircon-local/zircon-jenkins/preparations') + '\\', plan.recipe["preparationRoot"])

    def test_environment_file_change_is_rejected(self):
        self.environment_file.write_text(json.dumps({**self.environment, "LIB": "foreign"}, sort_keys=True),
                                         encoding="utf-8")
        with self.assertRaisesRegex(JenkinsError, "environment file digest"):
            self.planner().plan(sealed_input_ref=self.source, driver=self.driver,
                                changed_paths=["crates/demo/src/lib.rs"])


if __name__ == "__main__":
    unittest.main()
