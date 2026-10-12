from __future__ import annotations

import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins import already_applied as applied
from tools.jenkins.contracts import JenkinsError
from tools.jenkins.resources.paths import build_root_containing
from tools.jenkins.source import get_sealed_manifest, repository_id
from tools.jenkins.state import State


def sha(data):
    return hashlib.sha256(data).hexdigest()


def change(path, before, after):
    old, new = before.decode().splitlines(), after.decode().splitlines()
    return (f"diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n"
            f"@@ -1,{len(old)} +1,{len(new)} @@\n"
            + "".join("-" + line + "\n" for line in old)
            + "".join("+" + line + "\n" for line in new))


class AlreadyAppliedTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "src").mkdir()
        label = self.root.name.replace("_", "-").lower()
        self.before = f"// {label}\npub fn value() -> u32 {{ 1 }}\n".encode()
        self.after = f"// {label}\npub fn value() -> u32 {{ 2 }}\n".encode()
        self.manifest = f'[package]\nname="fixture-{label}"\nversion="0.1.0"\nedition="2021"\n'.encode()
        self.data = f"immutable dependency {label}\n".encode()
        (self.root / "src/lib.rs").write_bytes(self.before)
        (self.root / "Cargo.toml").write_bytes(self.manifest)
        (self.root / "data.txt").write_bytes(self.data)
        (self.root / "Cargo.lock").write_bytes(b"version = 4\n")
        self.git("init", "-q")
        self.git("config", "core.autocrlf", "false")
        self.git("add", "src/lib.rs", "Cargo.toml", "Cargo.lock", "data.txt")
        self.git("commit", "-q", "-m", "trusted baseline")
        self.head = self.git("rev-parse", "HEAD").decode().strip()
        (self.root / "src/lib.rs").write_bytes(self.after)
        self.state = State(self.root / ".jenkins/state/support.sqlite3")
        self.repo = repository_id(self.root)
        self.identity = dict(repositoryId=self.repo, sessionId="session", requestId="request",
                             attemptId="attempt", generation=1, stageImplementationDigest="d" * 64)
        self.authorize("session")
        self.payload = dict(
            inputMode="already-applied", identity=self.identity, baseHead=self.head,
            unifiedPatch=change("src/lib.rs", self.before, self.after),
            beforeHashes={"src/lib.rs": sha(self.before)}, afterHashes={"src/lib.rs": sha(self.after)},
            declaredDependencies=["Cargo.toml", "Cargo.lock", "data.txt"],
            dependencyHashes={"Cargo.toml": sha(self.manifest), "Cargo.lock": sha(b"version = 4\n"),
                              "data.txt": sha(self.data)},
            coverage={"template": "module_unit"}, buildRoot=str(build_root_containing(self.root).path))

    def authorize(self, session):
        self.state.authorize_session(self.repo, session, "user",
                                     ["src", "Cargo.toml", "Cargo.lock", "data.txt", "build.rs"],
                                     ["implementation", "validation"], {"source": "user", "task": "bounded patch"})

    def git(self, *args):
        # These writes create only this disposable, approved test fixture.
        env = {key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")}
        env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
        return subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Fixture",
                               "-c", "user.email=fixture@example.invalid", *args],
                              env=env, capture_output=True, check=True).stdout

    def register(self, payload=None):
        return applied.register_already_applied_intent(self.state, self.root, payload or self.payload,
                                                       trusted_driver_digest="d" * 64)

    def control(self, ref, identity=None):
        return {"identity": identity or self.identity, "patchRequestRef": ref}

    def prepare(self, ref=None, identity=None):
        ref = ref or self.register()["patchRequestRef"]
        return applied.prepare_already_applied(self.state, self.root, self.control(ref, identity),
                                               trusted_driver_digest="d" * 64)

    def require(self, prepared):
        return applied.require_already_applied_preparation(
            self.state, prepared["sourcePreparationRef"], identity=prepared["identity"],
            source_digest=prepared["sealedInputRef"])

    def test_registration_uses_git_before_without_source_or_cas_writes(self):
        index = (self.root / ".git/index").read_bytes()
        with patch.object(applied, "_atomic_write") as write, patch.object(applied, "seal_candidate") as seal:
            registered = self.register()
            self.assertTrue(self.register()["reused"])
            write.assert_not_called()
            seal.assert_not_called()
        intent = self.state.get("patch_intent", registered["patchRequestRef"])["payload"]
        self.assertNotIn("renderedPatch", intent)
        self.assertNotIn("beforeBytes", intent)
        self.assertNotIn("currentBytes", intent)
        self.assertEqual(sha(self.before), intent["baselineProvenance"]["paths"][0]["sha256"])
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
        self.assertEqual(index, (self.root / ".git/index").read_bytes())
        self.assertEqual([], self.state.list("sealed_input"))
        self.assertEqual([], self.state.list("path_claims"))

    def test_prepare_owns_true_git_before_cas_and_preserves_canonical_and_foreign_claims(self):
        self.state.put("path_claims", "foreign", {"repositoryId": self.repo, "path": "src/lib.rs",
                                                "owner": "foreign", "generation": 9, "beforeHash": "0" * 64})
        foreign = self.state.get("path_claims", "foreign")
        prepared = self.prepare()
        operation = self.require(prepared)
        objects = Path(operation["payload"]["objectRoot"]) / "inputs/objects"
        self.assertEqual(self.before, (objects / sha(self.before)).read_bytes())
        self.assertEqual(self.after, (objects / sha(self.after)).read_bytes())
        self.assertEqual(sha(self.before), operation["payload"]["beforeObjects"]["src/lib.rs"])
        self.assertEqual(sha(self.after), operation["payload"]["expectedAfterHashes"]["src/lib.rs"])
        self.assertIs(operation["result"]["canonicalSourceModified"], False)
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
        self.assertEqual(foreign, self.state.get("path_claims", "foreign"))
        shared = self.state.get("sealed_input", prepared["sealedInputRef"])["payload"]
        for key in ("owner", "sessionId", "ownedPaths", "claims", "patchOperationRefs", "sourcePreparationRefs"):
            self.assertNotIn(key, shared)
        self.assertEqual("already-applied-patch", shared["preparationKind"])
        self.assertEqual({"src/lib.rs", "cargo.toml", "cargo.lock", "data.txt"},
                         set(operation["result"]["currentHashes"]))
        self.assertEqual("already-applied", self.state.get_request(self.repo, "session", "request")["payload"]["inputMode"])

    def test_two_real_sessions_reuse_identical_source_and_cas_without_rewriting_first_record(self):
        first = self.prepare()
        record = self.state.get("sealed_input", first["sealedInputRef"])
        operation = self.require(first)
        object_root = Path(operation["payload"]["objectRoot"]) / "inputs/objects"
        own_hashes = (set(operation["payload"]["beforeObjects"].values())
                      | set(operation["result"]["currentHashes"].values())) - {None}
        own_objects = {checksum: (object_root / checksum).read_bytes() for checksum in own_hashes}
        other = {**self.identity, "sessionId": "session-2", "requestId": "request-2", "attemptId": "attempt-2"}
        self.authorize("session-2")
        second_ref = self.register({**self.payload, "identity": other})["patchRequestRef"]
        with patch("tools.jenkins.source._atomic_write", wraps=applied._atomic_write) as write:
            with patch.object(applied, "_atomic_write", new=write):
                second = self.prepare(second_ref, other)
        self.assertEqual([], [call.args[0] for call in write.call_args_list
                              if Path(call.args[0]).parent == object_root])
        self.assertEqual(first["sealedInputRef"], second["sealedInputRef"])
        self.assertNotEqual(first["sourcePreparationRef"], second["sourcePreparationRef"])
        self.assertEqual(record, self.state.get("sealed_input", first["sealedInputRef"]))
        self.assertEqual(own_objects, {checksum: (object_root / checksum).read_bytes() for checksum in own_hashes})
        self.assertEqual(1, len(self.state.list("sealed_input")))
        self.assertEqual(2, len(self.state.list("sealed_consumer")))
        self.require(first)
        self.require(second)
        get_sealed_manifest(self.state, repository_id_value=self.repo, session_id="session-2",
                            owner="attempt-2", source_digest=second["sealedInputRef"])

    def test_multi_file_postimage_mismatch_rejects_intake_atomically(self):
        expected_data = self.data.replace(b"immutable", b"edited")
        payload = {**self.payload,
                   "unifiedPatch": self.payload["unifiedPatch"] + change("data.txt", self.data, expected_data),
                   "beforeHashes": {**self.payload["beforeHashes"], "data.txt": sha(self.data)},
                   "afterHashes": {**self.payload["afterHashes"], "data.txt": sha(expected_data)},
                   "declaredDependencies": ["Cargo.toml", "Cargo.lock"],
                   "dependencyHashes": {key: value for key, value in self.payload["dependencyHashes"].items()
                                        if key != "data.txt"}}
        with self.assertRaises(JenkinsError):
            self.register(payload)
        self.assertEqual([], self.state.list("patch_intent"))
        self.assertEqual([], self.state.list("sealed_input"))
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
        self.assertEqual(self.data, (self.root / "data.txt").read_bytes())

    def test_forged_baseline_and_inexact_scopes_fail_closed(self):
        changes = [{"baseHead": "HEAD"}, {"baselineRef": "unissued"},
                   {"identity": {**self.identity, "repositoryId": "0" * 32}},
                   {"beforeHashes": {"src/lib.rs": sha(self.after)}},
                   {"afterHashes": {"data.txt": sha(self.data)}}, {"dependencyHashes": None},
                   {"dependencyHashes": {**self.payload["dependencyHashes"], "data.txt": None}},
                   {"declaredDependencies": [*self.payload["declaredDependencies"], "DATA.txt"]},
                   {"afterHashes": {**self.payload["afterHashes"], "SRC/LIB.rs": sha(self.after)}}]
        for value in changes:
            with self.subTest(value=value), self.assertRaises(JenkinsError):
                self.register({**self.payload, **value})
        self.assertEqual([], self.state.list("patch_intent"))

    def test_current_owned_or_dependency_drift_does_not_rewrite_source(self):
        ref = self.register()["patchRequestRef"]
        for path in ("src/lib.rs", "data.txt"):
            target = self.root / path
            original = target.read_bytes()
            target.write_bytes(b"foreign later bytes\n")
            try:
                with self.assertRaises(JenkinsError):
                    self.prepare(ref)
                self.assertEqual(b"foreign later bytes\n", target.read_bytes())
            finally:
                target.write_bytes(original)
        self.assertEqual([], self.state.list("sealed_input"))

    def test_prepared_getter_and_retry_allow_future_head_and_dirty_source(self):
        registered = self.register()
        prepared = self.prepare(registered["patchRequestRef"])
        (self.root / "later.txt").write_bytes(b"later unrelated Git change\n")
        self.git("add", "later.txt")
        self.git("commit", "-q", "-m", "future head")
        (self.root / "src/lib.rs").write_bytes(b"future canonical source\n")
        (self.root / "data.txt").write_bytes(b"future dependency\n")
        self.require(prepared)
        self.assertEqual(prepared, self.prepare(registered["patchRequestRef"]))
        self.assertEqual(b"future canonical source\n", (self.root / "src/lib.rs").read_bytes())

    def test_head_drift_before_prepare_is_rejected(self):
        ref = self.register()["patchRequestRef"]
        (self.root / "later.txt").write_bytes(b"new Git head\n")
        self.git("add", "later.txt")
        self.git("commit", "-q", "-m", "drift")
        with self.assertRaises(JenkinsError):
            self.prepare(ref)
        self.assertEqual([], self.state.list("sealed_input"))
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())

    def test_getter_checks_all_closure_objects_and_original_git_before(self):
        prepared = self.prepare()
        objects = Path(self.require(prepared)["payload"]["objectRoot"]) / "inputs/objects"
        for checksum in (sha(self.before), sha(self.after), sha(self.data)):
            target = objects / checksum
            original = target.read_bytes()
            target.write_bytes(b"corrupt test-only object\n")
            try:
                with self.subTest(checksum=checksum), self.assertRaises(JenkinsError):
                    self.require(prepared)
            finally:
                target.write_bytes(original)
        self.require(prepared)

    def test_wrong_consumer_revocation_and_typed_result_corruption_are_rejected(self):
        prepared = self.prepare()
        with self.assertRaises(JenkinsError):
            applied.require_already_applied_preparation(
                self.state, prepared["sourcePreparationRef"],
                identity={**prepared["identity"], "sessionId": "other"}, source_digest=prepared["sealedInputRef"])
        operation = self.require(prepared)
        bad = {**operation["result"], "canonicalSourceModified": True}
        self.state.transition_operation(operation["operationId"], "complete", "complete", bad)
        with self.assertRaises(JenkinsError):
            self.require(prepared)
        self.state.transition_operation(operation["operationId"], "complete", "complete", operation["result"])
        self.state.revoke_session(self.repo, "session")
        with self.assertRaises(JenkinsError):
            self.require(prepared)

    def test_partial_preparation_reconciles_forward_without_canonical_rollback(self):
        ref = self.register()["patchRequestRef"]
        with patch("tools.jenkins.patches._ensure_patch_metadata", side_effect=OSError("metadata interrupted")):
            with self.assertRaises(OSError):
                self.prepare(ref)
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
        recovered = applied.reconcile_already_applied(
            self.state, self.root, self.control(ref), trusted_driver_digest="d" * 64)
        self.assertEqual("prepared", recovered["status"])
        self.require(recovered)
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())

    def test_cancel_closes_only_own_request_and_preserves_source_claims_and_history(self):
        prepared = self.prepare()
        ref = prepared["patchRequestRef"]
        self.state.put("native_job", "historic", {"identity": {**self.identity, "requestId": "historic"},
                                                "status": "unknown", "pid": 12})
        history = self.state.get("native_job", "historic")
        cancelled = applied.reconcile_already_applied(self.state, self.root, self.control(ref), cancel=True)
        self.assertEqual("cancelled", cancelled["status"])
        self.assertEqual("cancelled", self.state.get_request(self.repo, "session", "request")["status"])
        self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
        self.assertEqual(history, self.state.get("native_job", "historic"))
        with self.assertRaises(JenkinsError):
            self.require(prepared)
        with self.assertRaises(JenkinsError):
            self.prepare(ref)

    def test_unproven_native_or_execution_or_writer_blocks_cancellation(self):
        ref = self.register()["patchRequestRef"]
        records = [("native_job", {"patchRequestRef": ref, "status": "terminal"}),
                   ("native_job", {"status": "unknown", "pid": 123}),
                   ("execution", {"identity": self.identity, "status": "running"}),
                   ("execution", {"identity": self.identity, "status": "failed"}),
                   ("writer_reservation", {"patchRequestRef": ref, "status": "active"})]
        for index, (domain, payload) in enumerate(records):
            key = "active-" + str(index)
            self.state.put(domain, key, payload)
            try:
                with self.subTest(domain=domain, payload=payload), self.assertRaises(JenkinsError):
                    applied.reconcile_already_applied(self.state, self.root, self.control(ref), cancel=True)
                self.assertEqual(self.after, (self.root / "src/lib.rs").read_bytes())
                self.assertNotEqual("cancelled", (self.state.get("patch_preparation", ref) or {"payload": {}})["payload"].get("status"))
            finally:
                self.state.delete(domain, key)

    def test_missing_module_and_dynamic_or_generated_inputs_are_not_client_completeness_proofs(self):
        (self.root / "src/hidden.rs").write_bytes(b"pub fn hidden() {}\n")
        ref = self.register({**self.payload, "coverage": {"template": "module_unit", "complete": True}})["patchRequestRef"]
        with self.assertRaises(JenkinsError) as error:
            self.prepare(ref)
        self.assertEqual("source_scope_incomplete", error.exception.code)
        self.assertEqual([], self.state.list("sealed_input"))
        (self.root / "src/hidden.rs").unlink()
        for source in (b'include!("generated.rs");\n', b'const DATA: &str = include_str!("data.txt");\n',
                       b'const OUT: &str = env!("OUT_DIR");\n'):
            with self.subTest(source=source), self.assertRaises(JenkinsError) as error:
                applied._cargo_closure(self.root, {"cargo.toml": self.manifest, "src/lib.rs": source,
                                                 "cargo.lock": b"version = 4\n", "data.txt": self.data})
            self.assertEqual("baseline_required", error.exception.code)

    def test_build_script_and_external_dependency_require_authoritative_closure(self):
        for package in (self.manifest + b'build="build.rs"\n',
                        self.manifest + b'[dependencies]\nforeign="1"\n'):
            with self.subTest(package=package), self.assertRaises(JenkinsError) as error:
                applied._cargo_closure(self.root, {"cargo.toml": package, "src/lib.rs": self.after,
                                                 "cargo.lock": b"version = 4\n", "data.txt": self.data})
            self.assertEqual("baseline_required", error.exception.code)

    def test_rust_addition_and_deletion_keep_exact_before_objects_and_check_other_cas_entries(self):
        new = b"pub fn added() {}\n"
        (self.root / "src/new.rs").write_bytes(new)
        (self.root / "data.txt").unlink()
        delete = ("diff --git a/data.txt b/data.txt\n--- a/data.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n"
                  + "-" + self.data.decode())
        addition = "diff --git a/src/new.rs b/src/new.rs\n--- /dev/null\n+++ b/src/new.rs\n@@ -0,0 +1 @@\n+" + new.decode()
        payload = {**self.payload, "unifiedPatch": self.payload["unifiedPatch"] + delete + addition,
                   "beforeHashes": {**self.payload["beforeHashes"], "data.txt": sha(self.data), "src/new.rs": None},
                   "afterHashes": {**self.payload["afterHashes"], "data.txt": None, "src/new.rs": sha(new)},
                   "declaredDependencies": ["Cargo.toml", "Cargo.lock"],
                   "dependencyHashes": {key: value for key, value in self.payload["dependencyHashes"].items()
                                        if key != "data.txt"}}
        prepared = self.prepare(self.register(payload)["patchRequestRef"])
        operation = self.require(prepared)
        self.assertIsNone(operation["payload"]["beforeObjects"]["src/new.rs"])
        self.assertEqual(sha(self.data), operation["payload"]["beforeObjects"]["data.txt"])
        self.assertIsNone(operation["result"]["currentHashes"]["data.txt"])
        self.assertFalse((self.root / "data.txt").exists())
        target = Path(operation["payload"]["objectRoot"]) / "inputs/objects" / sha(self.manifest)
        original = target.read_bytes()
        target.write_bytes(b"corrupt unrelated closure entry\n")
        try:
            with self.assertRaises(JenkinsError):
                self.require(prepared)
        finally:
            target.write_bytes(original)


if __name__ == "__main__":
    unittest.main()
