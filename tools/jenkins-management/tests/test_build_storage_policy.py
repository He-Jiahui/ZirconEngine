"""Storage attribution and actual public/raw recipe admission; no compiler runs."""
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).absolute().parents[3]
sys.path.insert(0, str(ROOT / "tools/dev"))


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


audit = load("storage_audit_subject", ROOT / "tools/dev/build_storage_audit.py")
entry = load("storage_entry_subject", ROOT / "tools/jenkins-management/entry.py")
runner = load("storage_runner_subject", ROOT / "tools/jenkins-management/runner.py")
monitor = load("storage_monitor_subject", ROOT / "tools/dev/jenkins_transition_monitor.py")


class AdmissionTests(unittest.TestCase):
    def test_broad_public_commands_rejected_before_dispatch(self):
        for args in (["check", "-p", "zircon_runtime"],
                     ["check", "-p", "zircon_runtime", "--all-features"],
                     ["test", "-p", "zircon_runtime", "--no-default-features"],
                     ["test", "-p", "zircon_runtime", "--no-default-features", "--tests"],
                     ["check", "--workspace", "--no-default-features"],
                     ["check", "-p", "zircon_runtime", "--no-default-features", "--all-targets"]):
            with self.subTest(args=args), self.assertRaises(entry.EntryError):
                entry.parse_cargo(args)

    def test_valid_named_module_and_one_test_are_preserved(self):
        recipe = entry.parse_cargo(["test", "-p", "zircon_runtime", "--no-default-features",
            "--features", "diagnostics", "--test", "contract", "specific_case", "--", "--exact"])
        manager = runner.Management.__new__(runner.Management)
        manager.payload = recipe
        result = manager.validation_recipe()
        self.assertEqual(result["packages"], ["zircon_runtime"])
        self.assertEqual(result["tests"], ["contract"])
        self.assertIn("--exact", result["command"])
        self.assertIn("--no-default-features", result["command"])

    def test_raw_json_cannot_bypass_public_admission(self):
        manager = runner.Management.__new__(runner.Management)
        for extra in ({}, {"noDefaultFeatures": False}, {"allFeatures": True},
                      {"allTests": True}, {"lib": False, "tests": ["a", "b"]}):
            manager.payload = {"verb": "test", "packages": ["zircon_runtime"],
                "lib": True, "noDefaultFeatures": True, **extra}
            if not extra:
                self.assertTrue(manager.validation_recipe()["lib"])
            else:
                with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, "build_scope_policy"):
                    manager.validation_recipe()

    def test_historical_query_can_decode_original_recipe(self):
        recipe = entry.parse_cargo(["check", "-p", "zircon_runtime", "--all-features"], enforce_policy=False)
        self.assertTrue(recipe["allFeatures"])

    def test_compact_environment_binds_effective_cache_settings(self):
        original = {"CARGO_INCREMENTAL": "1", "CARGO_PROFILE_DEV_DEBUG": "2",
                    "CARGO_PROFILE_RELEASE_DEBUG": "2", "PATH": "trusted"}
        result = runner.compact_build_environment(original)
        self.assertEqual(original["CARGO_INCREMENTAL"], "1")
        self.assertEqual(result["CARGO_INCREMENTAL"], "0")
        self.assertEqual(result["CARGO_PROFILE_DEV_DEBUG"], "0")
        self.assertEqual(result["CARGO_PROFILE_TEST_DEBUG"], "0")
        self.assertEqual(result["CARGO_PROFILE_RELEASE_DEBUG"], "2")


class AuditTests(unittest.TestCase):
    def setUp(self):
        self.fixture = tempfile.TemporaryDirectory(prefix="storage-policy-", dir=os.environ["ZR_COMMIT_TEST_ROOT"])
        self.addCleanup(self.fixture.cleanup)
        self.repo = Path(self.fixture.name)
        (self.repo / ".codex").mkdir()
        self.requests = self.repo / ".jenkins/state/management/branch-integration/requests"
        self.requests.mkdir(parents=True)
        self.unit = "E:/cargo-targets/zircon-local/management/pools/compatible"
        self.roots_patch = patch.object(audit, "source_roots", return_value={})
        self.roots_patch.start()
        self.addCleanup(self.roots_patch.stop)

    def scan(self, size, *, complete=True, manifests=()):
        return {"outputItems": [{"path": "E:/cargo-targets/zircon-local/management", "complete": complete,
            "buildUnits": {self.unit: {"logicalBytes": size, "files": 1, "incrementalBytes": size // 2, "symbolBytes": 0}},
            "manifestPaths": [str(p) for p in manifests]}]}

    def test_full_access_keeps_inventory_scope_and_config_bytes(self):
        config = self.repo / ".codex/config.toml"
        raw = b"default_permissions=':danger-full-access'\n[permissions.zircon-strict.filesystem]\n'E:/cargo-targets'='write'\n"
        config.write_bytes(raw)
        value, grants, _ = audit.inventory.load_grants(self.repo)
        self.assertEqual(value["default_permissions"], ":danger-full-access")
        self.assertEqual(grants["E:/cargo-targets"], "write")
        self.assertEqual(config.read_bytes(), raw)
        self.assertTrue(audit.inventory.writable(Path("E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor"), grants))

    def test_growth_is_not_waste_and_partial_scan_keeps_baseline(self):
        before = audit.audit(self.repo, self.scan(100))
        self.assertIsNone(before["units"][0]["logicalDeltaBytes"])
        partial = audit.audit(self.repo, self.scan(50, complete=False), before)
        self.assertFalse(partial["complete"])
        self.assertEqual(partial["baseline"], before["baseline"])
        self.assertIsNone(partial["units"][0]["logicalDeltaBytes"])
        after = audit.audit(self.repo, self.scan(120), partial)
        self.assertEqual(after["units"][0]["logicalDeltaBytes"], 20)
        self.assertFalse(after["units"][0]["wasteProvenByGrowth"])

    def test_unknown_growth_stays_pending_and_unchanged_runs_are_quiet(self):
        before = audit.audit(self.repo, self.scan(0))
        grown = audit.audit(self.repo, self.scan(2 * 1024**3), before)
        repeated = audit.audit(self.repo, self.scan(2 * 1024**3), grown)
        self.assertEqual(grown["findings"], repeated["findings"])
        self.assertFalse(monitor.issue_delta(grown["findings"], repeated["findings"])["notify"])
        failed = audit.audit(self.repo, None, grown)
        ids = {f["id"] for f in failed["findings"]}
        self.assertTrue(all(f["id"] in ids for f in grown["findings"]))

    def test_shared_manifest_writers_link_to_original_threads_without_blame(self):
        manifests = []
        for number in (1, 2):
            manifest = self.repo / f"writer-{number}.json"
            manifest.write_text(json.dumps({"owner": f"session-{number}", "operation_id": f"operation-{number}",
                "source_root": str(self.repo), "output_roots": {"target": self.unit + "/target"},
                "processes": [{"command": ["cargo.exe", "check", "-p", "zircon_runtime", "--no-default-features"]}]}))
            manifests.append(manifest)
            (self.requests / f"operation-{number}.json").write_text(json.dumps({"request": {
                "sessionId": f"session-{number}", "sourceRoot": str(self.repo), "ownerThreadId": f"chat-{number}"},
                "sourceCommit": "a" * 40, "status": "committed"}))
        result = audit.audit(self.repo, self.scan(500, manifests=manifests))
        unit = result["units"][0]
        self.assertEqual(unit["attribution"], "shared-or-uncertain")
        self.assertEqual({w["deliveryReferences"][0]["ownerThreadId"] for w in unit["writerCandidates"]}, {"chat-1", "chat-2"})
        self.assertFalse(unit["wasteProvenByGrowth"])

    def test_actual_unsafe_command_is_reported_but_string_commands_are_not_guessed(self):
        manifest = self.repo / "unsafe.json"
        manifest.write_text(json.dumps({"owner": "session", "operation_id": "original-operation",
            "output_roots": [self.unit + "/target"], "processes": [
                {"command": ["cargo.exe", "build", "--workspace", "--all-features"]},
                {"command": "powershell secret-command"}]}))
        result = audit.audit(self.repo, self.scan(100, manifests=[manifest]))
        issue = next(f for f in result["findings"] if f["code"] == "build_scope_violation")
        self.assertEqual(issue["subject"], "original-operation")
        self.assertNotIn("secret-command", json.dumps(result))
        self.assertIsNone(audit.observed_cargo("cargo --workspace"))

    def test_profile_overrides_are_reported_with_exact_source_hash(self):
        for relative in ("Cargo.toml", "zircon_plugins/Cargo.toml", "examples/woc/native/Cargo.toml"):
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('[profile.dev]\ndebug=0\nincremental=false\n[profile.test]\ndebug=0\nincremental=false\n[profile.dev.package.zircon_runtime]\nincremental=true\n')
        errors = []
        values = audit.profile_observations({"root": {"path": str(self.repo), "sessionId": "original-owner"}}, errors)
        self.assertFalse(errors)
        self.assertTrue(all(len(row["sourceSha256"]) == 64 for row in values))
        self.assertIn("package-override:zircon_runtime", values[0]["violations"])

    def test_invalid_request_preserves_pending_evidence(self):
        (self.requests / "bad.json").write_text('{"request": []}')
        previous = {"findings": [{"id": "build_scope_violation:original", "code": "build_scope_violation",
            "subject": "original", "message": "pending", "severity": "review"}]}
        result = audit.audit(self.repo, self.scan(1), previous)
        self.assertFalse(result["complete"])
        self.assertTrue(any(f["id"] == "build_scope_violation:original" for f in result["findings"]))

    def test_verified_profile_fix_resolves_itself_during_unrelated_scan_failure(self):
        profile = {"path": str(self.repo / "Cargo.toml"), "profile": "dev", "violations": []}
        subject = profile["path"] + ":dev"
        old = {"id": "build_profile_violation:" + audit.key(subject), "code": "build_profile_violation",
               "subject": subject, "message": "pending", "severity": "review"}
        with patch.object(audit, "profile_observations", return_value=[profile]):
            result = audit.audit(self.repo, self.scan(1, complete=False), {"findings": [old]})
        self.assertFalse(result["complete"])
        self.assertNotIn(old["id"], {f["id"] for f in result["findings"]})

    def test_metadata_read_budget_is_enforced(self):
        path = self.repo / "large.json"
        path.write_bytes(b" " * (audit.MAX_METADATA_BYTES + 1))
        with self.assertRaisesRegex(ValueError, "read budget"):
            audit.read_metadata(path)

    def test_expired_directory_budget_cannot_report_zero_as_a_complete_scan(self):
        path = self.repo / "retained-output"
        path.mkdir()
        (path / "preserved.txt").write_bytes(b"retained")
        result = audit.inventory.tree_fact(path, deadline=-1)
        self.assertFalse(result["complete"])
        self.assertTrue(result["scanTimedOut"])
        self.assertEqual(result["errors"][0]["type"], "ScanBudgetExceeded")
        self.assertGreater(result["unvisitedEntriesLowerBound"], 0)
        self.assertEqual((path / "preserved.txt").read_bytes(), b"retained")

    def test_compiler_first_pass_is_not_double_counted_or_lost_to_later_timeout(self):
        scan = self.scan(100)
        scan["inventory"] = [{**scan["outputItems"][0], "exists": True,
            "path": "E:/cargo-targets/zircon-local"},
            {"path": str(self.repo), "exists": True, "complete": False}]
        scan["complete"] = False
        scan["outputItems"][0] = {**scan["outputItems"][0], "complete": False,
            "buildUnits": {self.unit: {"logicalBytes": 3, "files": 1, "incrementalBytes": 0, "symbolBytes": 0}}}
        result = audit.audit(self.repo, scan)
        self.assertFalse(result["complete"])
        self.assertEqual(result["units"][0]["logicalBytes"], 100)
        self.assertTrue(result["units"][0]["complete"])
        self.assertEqual(result["baseline"][audit.key(self.unit)]["logicalBytes"], 100)

    def test_compiler_accounting_uses_one_pool_and_excludes_source(self):
        root = Path("E:/cargo-targets/zircon-local/management/pools/identity")
        self.assertEqual(audit.inventory.compiler_unit(root / "target/debug/deps/a.rlib"), root)
        self.assertEqual(audit.inventory.compiler_unit(root / "build/debug/incremental/a.bin"), root)
        self.assertIsNone(audit.inventory.compiler_unit(Path("E:/Git/ZirconEngine/zircon_runtime/build/helper.py")))
        self.assertIsNone(audit.inventory.compiler_unit(Path("E:/nested/cargo-targets/target/a.rlib")))
        self.assertEqual(audit.inventory.compiler_unit(Path("E:/cargo-targets/zircon-local/cache/sccache/a.bin")),
                         Path("E:/cargo-targets/zircon-local/cache/sccache"))


if __name__ == "__main__":
    unittest.main()
