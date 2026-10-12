"""Checkout-postimage handoff contracts; generated fixtures and no native build."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from tools.dev import jenkins_validate as entry


def checksum(data):
    return hashlib.sha256(data).hexdigest()


class ValidationEntryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "src").mkdir()
        self.before, self.after = b"before\n", b"after\n"
        self.source = self.root / "src/lib.rs"
        self.source.write_bytes(self.after)
        self.dependency = self.root / "cargo.toml"
        self.dependency.write_bytes(b"[workspace]\n")
        self.patch = self.root / "change.patch"
        self.patch.write_text("diff --git a/src/lib.rs b/src/lib.rs\n"
                              "--- a/src/lib.rs\n+++ b/src/lib.rs\n"
                              "@@ -1 +1 @@\n-before\n+after\n", encoding="utf-8")
        self.driver = {"digest": "d" * 64, "runtimeOperationId": "runtime-one",
                       "manifestFiles": sorted(entry.ALREADY_APPLIED_DRIVER_FILES)}
        self.calls = []

    def args(self, *, mode=True, **changes):
        args = entry.parser().parse_args(["--repository", str(self.root),
                                         "--session-id", "session", "--request-id", "request"])
        defaults = {"patch": str(self.patch), "before_hashes": json.dumps({"src/lib.rs": checksum(self.before)}),
                    "coverage_template": "module_unit", "build_root": r"E:\cargo-targets",
                    "already_applied": mode}
        if mode:
            defaults.update(base_head="a" * 40, after_hashes=json.dumps({"src/lib.rs": checksum(self.after)}),
                            dependency_hashes="{}")
        for name, value in {**defaults, **changes}.items():
            setattr(args, name, value)
        return args

    def payload(self, args=None):
        return entry.intent_payload(args or self.args(), self.root, {"requestId": "request"})

    def call(self, root, driver, folder, domain, action, payload, timeout):
        self.calls.append((domain, action, payload))
        if action == "register":
            return {"patchRequestRef": "patch-one"}
        if action == "submit":
            return {"status": "submitted", "queueId": 17}
        return {"status": "pending"}

    def execute(self, args=None):
        return entry.execute(args or self.args(), call=self.call, driver_loader=lambda root: self.driver)

    def rejected(self, code, action, *args):
        with self.assertRaises(entry.EntryError) as caught:
            action(*args)
        self.assertEqual(code, caught.exception.code)

    def test_unapplied_mode_preserves_original_payload_and_needs_no_support_modules(self):
        args = self.args(mode=False)
        payload = self.payload(args)
        self.assertEqual({"identity", "unifiedPatch", "beforeHashes", "declaredDependencies", "coverage",
                          "externalInputs", "baseHead", "buildRoot", "ownedPaths"}, set(payload))
        self.assertIsNone(payload["baseHead"])
        self.assertEqual(checksum(self.before), payload["beforeHashes"]["src/lib.rs"])
        self.driver.pop("manifestFiles")
        result = self.execute(self.args(mode=False, action="submit"))
        self.assertEqual("submitted", result["status"])
        self.assertEqual([("patch", "register"), ("patch", "submit")], [item[:2] for item in self.calls])

    def test_postimage_does_not_compare_before_hash_with_current_bytes(self):
        payload = self.payload()
        self.assertEqual("already-applied", payload["inputMode"])
        self.assertEqual(checksum(self.before), payload["beforeHashes"]["src/lib.rs"])
        self.assertEqual(checksum(self.after), payload["afterHashes"]["src/lib.rs"])
        self.assertEqual({}, payload["dependencyHashes"])

    def test_public_cli_parses_mode_metadata_and_optional_baseline(self):
        args = entry.parser().parse_args(["submit", "--session-id", "session", "--request-id", "request",
                                         "--already-applied", "--base-head", "a" * 40,
                                         "--after-hashes", "after.json", "--dependency-hashes", "{}",
                                         "--baseline-ref", "baseline-one"])
        self.assertTrue(args.already_applied)
        self.assertEqual("after.json", args.after_hashes)
        self.assertEqual("{}", args.dependency_hashes)
        self.assertEqual("baseline-one", args.baseline_ref)

    def test_unapplied_parser_keeps_addition_and_deletion_path_contract(self):
        text = ("diff --git a/src/added.rs b/src/added.rs\n"
                "--- /dev/null\n+++ b/src/added.rs\n@@ -0,0 +1 @@\n+added\n"
                "diff --git a/src/deleted.rs b/src/deleted.rs\n"
                "--- a/src/deleted.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-deleted\n")
        self.assertEqual({"src/added.rs": True, "src/deleted.rs": False}, entry.patch_paths(self.root, text))

    def test_already_applied_only_flags_are_rejected_without_mode_even_on_query(self):
        for field, value in (("after_hashes", "{}"), ("dependency_hashes", "{}"),
                             ("baseline_ref", "baseline-one")):
            with self.subTest(field=field):
                args = self.args(mode=False, action="query", **{field: value})
                self.rejected("already_applied_flags_conflict", self.execute, args)
        self.assertEqual([], self.calls)

    def test_immutable_base_is_required(self):
        for base in (None, "", "HEAD", "main", "a" * 39, "A" * 40):
            with self.subTest(base=base):
                self.rejected("patch_base_invalid", self.payload, self.args(base_head=base))
        self.assertEqual("b" * 64, self.payload(self.args(base_head="b" * 64))["baseHead"])

    def test_after_and_explicit_dependency_hashes_are_required(self):
        for field in ("after_hashes", "dependency_hashes"):
            with self.subTest(field=field):
                self.rejected("already_applied_inputs_missing", self.payload, self.args(**{field: None}))

    def test_json_file_inputs_and_coordinator_baseline_identifier(self):
        after = self.root / "after.json"
        dependencies = self.root / "dependencies.json"
        after.write_text(json.dumps({"src/lib.rs": checksum(self.after)}), encoding="utf-8")
        dependencies.write_text("{}", encoding="utf-8")
        payload = self.payload(self.args(after_hashes=str(after), dependency_hashes=str(dependencies),
                                        baseline_ref="baseline-one"))
        self.assertEqual("baseline-one", payload["baselineRef"])
        self.rejected("invalid_identity", self.payload, self.args(baseline_ref="../baseline"))

    def test_after_hash_scope_and_canonical_duplicates_are_rejected(self):
        for after in ({}, {"src/lib.rs": checksum(self.after), "src/extra.rs": "b" * 64},
                      {"src/other.rs": checksum(self.after)}):
            with self.subTest(after=after):
                code = "after_hashes_invalid" if not after else "patch_scope_mismatch"
                self.rejected(code, self.payload, self.args(after_hashes=json.dumps(after)))
        self.rejected("after_hashes_invalid", self.payload,
                      self.args(after_hashes=json.dumps({"src/lib.rs": checksum(self.after), "SRC\\LIB.RS": checksum(self.after)})))

    def test_invalid_after_hash_values_are_rejected(self):
        for value in ("short", "A" * 64, 7, False, ["a" * 64]):
            with self.subTest(value=value):
                self.rejected("after_hashes_invalid", self.payload,
                              self.args(after_hashes=json.dumps({"src/lib.rs": value})))

    def test_changed_missing_and_nonordinary_postimages_are_rejected(self):
        self.source.write_bytes(b"foreign\n")
        self.rejected("patch_after_hash_mismatch", self.payload)
        self.source.unlink()
        self.rejected("patch_after_hash_mismatch", self.payload)
        self.source.mkdir()
        self.rejected("patch_after_hash_mismatch", self.payload)

    def test_addition_uses_null_before_and_present_after(self):
        added = self.root / "src/new.rs"
        added.write_bytes(b"new\n")
        self.patch.write_text("diff --git a/src/new.rs b/src/new.rs\n"
                              "--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+new\n", encoding="utf-8")
        args = self.args(before_hashes=json.dumps({"src/new.rs": None}),
                         after_hashes=json.dumps({"src/new.rs": checksum(b"new\n")}))
        payload = self.payload(args)
        self.assertIsNone(payload["beforeHashes"]["src/new.rs"])
        self.assertEqual(checksum(b"new\n"), payload["afterHashes"]["src/new.rs"])
        args.before_hashes = json.dumps({"src/new.rs": "a" * 64})
        self.rejected("patch_scope_mismatch", self.payload, args)

    def test_deletion_uses_explicit_header_and_absent_postimage(self):
        self.patch.write_text("diff --git a/src/lib.rs b/src/lib.rs\n"
                              "--- a/src/lib.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-before\n", encoding="utf-8")
        args = self.args(after_hashes=json.dumps({"src/lib.rs": None}))
        self.rejected("patch_after_hash_mismatch", self.payload, args)
        self.source.unlink()
        self.assertIsNone(self.payload(args)["afterHashes"]["src/lib.rs"])
        args.before_hashes = json.dumps({"src/lib.rs": None})
        self.rejected("patch_scope_mismatch", self.payload, args)

    def test_null_after_hash_is_only_valid_for_a_deletion(self):
        self.rejected("patch_scope_mismatch", self.payload,
                      self.args(after_hashes=json.dumps({"src/lib.rs": None})))
        self.patch.write_text("diff --git a/src/lib.rs b/src/lib.rs\n"
                              "--- a/src/lib.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-before\n", encoding="utf-8")
        self.rejected("patch_scope_mismatch", self.payload)

    def test_dependency_hashes_match_exact_scope_after_removing_patch_paths(self):
        args = self.args(dependency=["Cargo.toml", "src/lib.rs"],
                         dependency_hashes=json.dumps({"CARGO.TOML": checksum(self.dependency.read_bytes())}))
        payload = self.payload(args)
        self.assertEqual(["cargo.toml"], payload["declaredDependencies"])
        self.assertEqual({"cargo.toml": checksum(self.dependency.read_bytes())}, payload["dependencyHashes"])
        for values in ({}, {"cargo.toml": checksum(self.dependency.read_bytes()), "src/lib.rs": checksum(self.after)},
                       {"cargo.toml": None}, {"cargo.toml": "short"},
                       {"cargo.toml": "b" * 64, "CARGO.TOML": "b" * 64}):
            with self.subTest(values=values):
                args.dependency_hashes = json.dumps(values)
                self.rejected("dependency_hashes_invalid", self.payload, args)

    def test_dependency_bytes_are_checked(self):
        args = self.args(dependency=["Cargo.toml"],
                         dependency_hashes=json.dumps({"cargo.toml": checksum(self.dependency.read_bytes())}))
        self.dependency.write_bytes(b"foreign\n")
        self.rejected("patch_dependency_hash_mismatch", self.payload, args)
        self.dependency.unlink()
        self.rejected("patch_dependency_invalid", self.payload, args)

    def test_missing_driver_capability_never_submits_or_claims_dry_check(self):
        for manifest in (None, [], ["tools/jenkins/already_applied.py"], ["tools/jenkins/source/baselines.py"]):
            with self.subTest(manifest=manifest):
                if manifest is None:
                    self.driver.pop("manifestFiles", None)
                else:
                    self.driver["manifestFiles"] = manifest
                for action in ("dry-check", "submit"):
                    self.rejected("patch_handoff_unavailable", self.execute, self.args(action=action))
        self.assertEqual([], self.calls)
        self.assertFalse((self.root / ".jenkins/state/requests").exists())

    def test_submission_preserves_original_identity_and_queries_existing_pending_request(self):
        submitted = self.execute(self.args(action="submit"))
        self.assertFalse(submitted["formalAcceptance"])
        identity = submitted["identity"]
        self.assertEqual("request", identity["requestId"])
        self.assertEqual("request-attempt-1", identity["attemptId"])
        registration = self.calls[0][2]
        self.assertEqual(identity, registration["identity"])
        self.assertEqual("already-applied", registration["inputMode"])
        self.execute(self.args(action="submit"))
        # Query requires no patch, hashes, mode or capability resubmission.
        self.driver.pop("manifestFiles")
        query = self.execute(self.args(mode=False, action="query", patch=None, before_hashes=None))
        self.assertEqual(identity, query["identity"])
        self.assertEqual("pending", query["status"])
        self.assertFalse(query["formalAcceptance"])
        self.assertEqual([("patch", "register"), ("patch", "submit"), ("patch", "query"), ("patch", "query")],
                         [item[:2] for item in self.calls])
        self.assertTrue(all(item[2]["identity"] == identity for item in self.calls))

    def test_existing_request_rejects_changed_immutable_input_and_attempt(self):
        self.execute(self.args(action="submit"))
        for changes in ({"baseline_ref": "other-baseline"}, {"base_head": "b" * 40},
                        {"dependency": ["Cargo.toml"],
                         "dependency_hashes": json.dumps({"cargo.toml": checksum(self.dependency.read_bytes())})}):
            with self.subTest(changes=changes):
                self.rejected("request_input_conflict", self.execute, self.args(action="submit", **changes))
        self.rejected("request_identity_conflict", self.execute,
                      self.args(mode=False, action="query", attempt_id="other-attempt"))
        self.assertEqual(2, len(self.calls))

    def test_unknown_dispatch_reconciles_same_request_without_retrying(self):
        def interrupted(root, driver, folder, domain, action, payload, timeout):
            if action == "submit":
                self.calls.append((domain, action, payload))
                raise entry.EntryError("protocol_delivery_unknown", "interrupted", retryable=True)
            return self.call(root, driver, folder, domain, action, payload, timeout)
        result = entry.execute(self.args(action="submit"), call=interrupted, driver_loader=lambda root: self.driver)
        self.assertEqual("delivery_unknown", result["status"])
        self.execute(self.args(mode=False, action="query", patch=None, before_hashes=None))
        self.assertEqual([("patch", "register"), ("patch", "submit"), ("patch", "query"), ("patch", "query")],
                         [item[:2] for item in self.calls])
        self.assertTrue(all(item[2]["identity"] == result["identity"] for item in self.calls))

    def test_unknown_registration_queries_original_identity_without_registering_again(self):
        def interrupted(root, driver, folder, domain, action, payload, timeout):
            self.calls.append((domain, action, payload))
            raise entry.EntryError("protocol_delivery_unknown", "interrupted", retryable=True)
        args = self.args(action="submit")
        with self.assertRaises(entry.EntryError) as caught:
            entry.execute(args, call=interrupted, driver_loader=lambda root: self.driver)
        self.assertEqual("protocol_delivery_unknown", caught.exception.code)
        result = self.execute(self.args(mode=False, action="query", patch=None, before_hashes=None))
        self.execute(self.args(mode=False, action="submit", patch=None, before_hashes=None))
        self.assertEqual("pending", result["status"])
        self.assertFalse(result["formalAcceptance"])
        self.assertEqual([("patch", "register"), ("request", "query"), ("request", "query")],
                         [item[:2] for item in self.calls])
        self.assertTrue(all(item[2]["identity"] == result["identity"] for item in self.calls))

    def test_selected_driver_exposes_only_verified_manifest_declarations(self):
        files = {"launcher.py": b"# synthetic launcher\n", "pipeline/steps.groovy": b"// synthetic pipeline\n",
                 **{name: b"# synthetic support module\n" for name in entry.ALREADY_APPLIED_DRIVER_FILES}}
        records = {name: {"sha256": checksum(data), "bytes": len(data)} for name, data in files.items()}
        driver_digest = entry.digest({"schemaVersion": 1, "files": records})
        driver_root = self.root / ".jenkins/runtime/drivers" / driver_digest
        for name, data in files.items():
            path = driver_root.joinpath(*name.split("/"))
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        entry.save_json(driver_root / "driver-manifest.json",
                        {"schemaVersion": 1, "files": records, "driverDigest": driver_digest})
        entry.save_json(self.root / ".jenkins/state/deployment/driver.json",
                        {"driverDigest": driver_digest, "runtimeOperationId": "runtime-one"})
        python = self.root / ".jenkins/runtime/python/python.exe"
        python.parent.mkdir(parents=True)
        python.write_bytes(b"synthetic runtime fixture")
        driver = entry.selected_driver(self.root)
        self.assertEqual(tuple(sorted(files)), driver["manifestFiles"])
        entry.require_already_applied_driver(driver)
        (driver_root / "tools/jenkins/already_applied.py").write_bytes(b"changed support")
        self.rejected("driver_integrity_failed", entry.selected_driver, self.root)


if __name__ == "__main__":
    unittest.main()
