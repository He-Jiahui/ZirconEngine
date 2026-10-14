"""Current-input intake uses synthetic bytes, never a checkout copy or Cargo."""
from __future__ import annotations

from contextlib import nullcontext
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.state import State
from tools.jenkins.source import repository_id
from tools.jenkins.resources.paths import canonical_build_root, physical_path_under
from tools.jenkins import current_source as intake
from tools.jenkins.workflow.handler import handle as flow


class CurrentSourceTests(unittest.TestCase):
    def setUp(self):
        approved = canonical_build_root(r"E:\cargo-targets")
        temporary_root = physical_path_under(
            approved, approved.namespace() / "tmp/current-source-tests", allow_missing=True)
        temporary_root.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="current-source-", dir=temporary_root)
        self.root = Path(self.temp.name) / "checkout"
        self.root.mkdir()
        self.cas = Path(self.temp.name) / "cas"
        self.state = State(Path(self.temp.name) / "state.sqlite3")
        self.files = {"crate/main.rs": b"fn already_edited() {}\n", "deps/helper.py": b"value = 9\n",
                      "crate/new.rs": b"fn explicit_untracked() {}\n"}
        for path, data in self.files.items():
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        self.identity = {"repositoryId": repository_id(self.root), "sessionId": "current-session",
                         "requestId": "current-request", "attemptId": "current-attempt", "generation": 1,
                         "stageImplementationDigest": "b" * 64}
        self.hashes = {path: file_digest(self.root / path) for path in self.files}
        self.hashes["crate/deleted.rs"] = None
        self.payload = {"identity": self.identity, "baseHead": "a" * 40, "buildRoot": "D:\\cargo-targets",
                        "ownedPaths": ["crate/main.rs"], "currentHashes": self.hashes,
                        "untrackedPaths": ["crate/new.rs"],
                        "coverage": {"template": "editor_product_development", "requiredTests": ["focused::test"],
                                     "noDefaultFeatures": True}}
        self.state.authorize_session(self.identity["repositoryId"], self.identity["sessionId"], "synthetic-owner",
                                     ["crate", "deps"], ["validation"], {"source": "user", "task": "synthetic test"})
        self.mocks = [patch.object(intake, "_head", return_value="a" * 40),
                      patch.object(intake, "_git_selection", return_value=({"crate/main.rs", "deps/helper.py", "crate/deleted.rs"}, {"crate/new.rs"})),
                      patch.object(intake, "_lock", side_effect=lambda root: nullcontext()),
                      patch.object(intake, "canonical_build_root", return_value=SimpleNamespace(namespace=lambda: self.cas)),
                      patch("tools.jenkins.source._git_base_tree", return_value={})]
        for mock in self.mocks:
            mock.start()
        self.addCleanup(lambda: [mock.stop() for mock in reversed(self.mocks)])
        self.addCleanup(self.temp.cleanup)

    def register(self, **changes):
        return intake.register_current_source(self.state, self.root, {**self.payload, **changes},
                                              trusted_driver_digest="b" * 64)

    def control(self, ref):
        return {"identity": self.identity, "currentSourceRef": ref, "buildRoot": "D:\\cargo-targets"}

    def prepare(self):
        ref = self.register()["currentSourceRef"]
        with patch.object(intake, "_ensure_patch_metadata") as metadata:
            result = intake.prepare_current_source(self.state, self.root, self.control(ref), trusted_driver_digest="b" * 64)
        return ref, result, metadata

    def test_same_repository_worktree_keeps_canonical_identity_and_seals_its_bytes(self):
        worktree = Path(self.temp.name) / "selected-worktree"
        worktree_bytes = dict(self.files)
        worktree_bytes["crate/main.rs"] = b"fn worktree_version() {}\n"
        for relative, data in worktree_bytes.items():
            target = worktree / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        worktree_hashes = {
            relative: None if checksum is None else file_digest(worktree.joinpath(*relative.split("/")))
            for relative, checksum in self.hashes.items()
        }
        canonical_before = {relative: (self.root / relative).read_bytes() for relative in self.files}

        with patch("tools.jenkins.current_source.subprocess.run",
                   return_value=SimpleNamespace(returncode=0, stdout=str(self.root / ".git"))):
            registered = self.register(sourceRoot=str(worktree), currentHashes=worktree_hashes)
            reference = registered["currentSourceRef"]
            intent = self.state.get("current_source_intent", reference)["payload"]
            with patch.object(intake, "_ensure_patch_metadata"):
                prepared = intake.prepare_current_source(
                    self.state, self.root, self.control(reference), trusted_driver_digest="b" * 64)

        self.assertEqual(str(self.root), intent["repositoryRoot"])
        self.assertEqual(str(worktree), intent["sourceRoot"])
        self.assertEqual(self.identity["repositoryId"], intent["identity"]["repositoryId"])
        sealed = self.state.get("sealed_input", prepared["sealedInputRef"])["payload"]
        entry = next(item for item in sealed["sourceManifest"]["entries"]
                     if item["path"] == "crate/main.rs")
        self.assertEqual(file_digest(worktree / "crate/main.rs"), entry["sha256"])
        self.assertNotEqual(self.hashes["crate/main.rs"], entry["sha256"])
        self.assertEqual(canonical_before,
                         {relative: (self.root / relative).read_bytes() for relative in self.files})
        operation = self.state.get_operation(prepared["sourcePreparationRef"])
        self.assertFalse(operation["result"]["canonicalSourceModified"])

    def test_foreign_git_common_directory_is_rejected_before_intake(self):
        foreign = Path(self.temp.name) / "foreign-worktree"
        foreign.mkdir()
        with patch("tools.jenkins.current_source.subprocess.run",
                   return_value=SimpleNamespace(returncode=0, stdout=str(foreign / ".git"))):
            with self.assertRaises(JenkinsError) as error:
                self.register(sourceRoot=str(foreign))

        self.assertEqual("current_source_root_mismatch", error.exception.code)
        self.assertEqual([], self.state.list("current_source_intent"))
        self.assertEqual([], self.state.list("current_source_request"))
        self.assertFalse(self.cas.exists())

    def test_registration_is_side_effect_free_and_replay_exact(self):
        result = self.register()
        self.assertEqual(self.register()["currentSourceRef"], result["currentSourceRef"])
        self.assertFalse(self.cas.exists())
        self.assertEqual(self.state.list("path_claims"), [])
        self.assertIsNone(self.state.get_request(*[self.identity[name] for name in ("repositoryId", "sessionId", "requestId")]))
        self.assertEqual({path: (self.root / path).read_bytes() for path in self.files}, self.files)
        with self.assertRaises(JenkinsError):
            self.register(ownedPaths=["deps/helper.py"])

    def test_coordinator_prepares_existing_bytes_and_preserves_foreign_claim(self):
        self.state.put("path_claims", "foreign", {"owner": "foreign", "path": "deps/helper.py", "status": "claimed"})
        foreign = self.state.list("path_claims")
        ref, result, metadata = self.prepare()
        self.assertEqual(result["status"], "prepared")
        self.assertEqual(self.state.list("path_claims"), foreign)
        self.assertEqual({path: (self.root / path).read_bytes() for path in self.files}, self.files)
        operation = self.state.get_operation(result["sourcePreparationRef"])
        self.assertEqual(operation["kind"], "source_preparation")
        self.assertFalse(operation["result"]["canonicalSourceModified"])
        self.assertEqual(operation["result"]["ownedPaths"], ["crate/main.rs"])
        sealed = self.state.get("sealed_input", result["sealedInputRef"])["payload"]
        self.assertEqual(sealed["patchOperationRefs"], [])
        self.assertEqual(len(list((self.cas / "inputs/objects").iterdir())), len(self.files))
        metadata.assert_called_once()
        replay = intake.prepare_current_source(self.state, self.root, self.control(ref), trusted_driver_digest="b" * 64)
        self.assertEqual(replay, result)

    def test_current_hash_or_base_drift_fails_before_sealing(self):
        ref = self.register()["currentSourceRef"]
        (self.root / "crate/main.rs").write_text("fn concurrent_edit() {}\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as error:
            intake.prepare_current_source(self.state, self.root, self.control(ref), trusted_driver_digest="b" * 64)
        self.assertEqual(error.exception.code, "current_source_hash_mismatch")
        self.assertFalse(self.cas.exists())
        with patch.object(intake, "_head", return_value="d" * 40), self.assertRaises(JenkinsError) as error:
            intake.prepare_current_source(self.state, self.root, self.control(ref), trusted_driver_digest="b" * 64)
        self.assertEqual(error.exception.code, "current_source_base_changed")

    def test_untracked_and_null_deletion_scope_are_exact(self):
        with self.assertRaises(JenkinsError):
            self.register(untrackedPaths=[])
        wrong = {**self.hashes, "crate/new.rs": None}
        with self.assertRaises(JenkinsError):
            self.register(currentHashes=wrong, untrackedPaths=[])
        with self.assertRaises(JenkinsError):
            self.register(ownedPaths=["foreign/missing.rs"])

    def test_revoke_and_identity_change_cannot_replay(self):
        ref = self.register()["currentSourceRef"]
        with self.assertRaises(JenkinsError):
            intake.handle("query", {**self.control(ref), "identity": {**self.identity, "attemptId": "other"}}, self.state, self.root)
        self.state.revoke_session(self.identity["repositoryId"], self.identity["sessionId"])
        with self.assertRaises(JenkinsError):
            intake.handle("query", self.control(ref), self.state, self.root)

    def test_runtime_control_and_corrupt_prepared_object_cannot_replay(self):
        ref, prepared, _ = self.prepare()
        with self.assertRaises(JenkinsError):
            intake.handle("query", {**self.control(ref), "runtimeOperationId": "foreign-runtime"}, self.state, self.root)
        objects = list((self.cas / "inputs/objects").iterdir())
        objects[0].write_bytes(b"corrupt synthetic object")
        with self.assertRaises(JenkinsError) as error:
            intake.prepare_current_source(self.state, self.root, self.control(ref), trusted_driver_digest="b" * 64)
        self.assertEqual(error.exception.code, "current_source_object_corrupt")

    def test_identity_only_query_reconciles_original_slot_without_registration(self):
        control = {"identity": self.identity, "buildRoot": "D:\\cargo-targets"}
        missing = intake.handle("query", control, self.state, self.root)
        self.assertEqual(missing["status"], "not_found")
        ref = self.register()["currentSourceRef"]
        with patch.object(intake, "register_current_source", side_effect=AssertionError("must not register")):
            queried = intake.handle("query", control, self.state, self.root)
        self.assertEqual(queried["currentSourceRef"], ref)
        self.assertEqual(queried["status"], "registered")

    def test_prepared_query_and_flow_use_typed_source_receipt(self):
        ref, prepared, _ = self.prepare()
        query = intake.handle("query", self.control(ref), self.state, self.root)
        self.assertEqual(query["preparation"]["payload"]["result"], prepared)
        payload = {"identity": prepared["identity"], "sealedInputRef": prepared["sealedInputRef"],
                   "sourcePreparationRef": prepared["sourcePreparationRef"], "buildRoot": "D:\\cargo-targets"}
        flow("register-flow", payload, self.state, self.root)
        composed = flow("compose-flow", payload, self.state, self.root)
        self.assertEqual(composed["stages"][0], "source")
        self.assertNotIn("patch", composed["stages"])
        with patch("tools.jenkins.workflow.handler._light_receipt", return_value={"status": "passed"}):
            stage = flow("run-stage", {**payload, "stage": "source"}, self.state, self.root)
        self.assertEqual(stage["status"], "ready")
        with self.assertRaises(JenkinsError):
            flow("register-flow", {**payload, "sourcePreparationRef": "wrong"}, self.state, self.root)

    def test_git_inventory_has_bounded_argv_and_validates_only_selected(self):
        self.mocks[1].stop()
        output = b"crate/main.rs\0unselected/foreign-link\0"
        with patch.object(intake.subprocess, "run", return_value=SimpleNamespace(stdout=output)) as git:
            tracked, untracked = intake._git_selection(self.root, ["crate/main.rs"] * 10000)
        self.assertEqual(tracked, {"crate/main.rs"})
        self.assertEqual(untracked, {"crate/main.rs"})
        self.assertTrue(all(len(call.args[0]) < 10 for call in git.call_args_list))


if __name__ == "__main__":
    unittest.main()
