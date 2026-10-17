"""Automatic managed contexts retain the original writer and exact reader closure."""
from __future__ import annotations

from contextlib import contextmanager
import importlib.util
import json
from pathlib import Path
import sqlite3
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.deployment.driver import _plain as real_plain
from tools.jenkins.source import repository_id
from tools.jenkins.state import SCHEMA, State
from tools.jenkins import managed_worktree as formal

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "automatic_worktree_context_tests", ROOT / "tools/jenkins-management/source_preparation.py")
helper = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(helper)


class AutomaticWorktreeContextTests(unittest.TestCase):
    def setUp(self):
        self.repo, self.repository = ROOT, repository_id(ROOT)
        self.database = sqlite3.connect(":memory:", isolation_level=None)
        self.database.row_factory = sqlite3.Row
        self.database.executescript(SCHEMA)
        self.addCleanup(self.database.close)
        self.state = object.__new__(State)
        self.state.path, self.state.busy_timeout_ms = ROOT / ".jenkins/state/coordination.sqlite3", 3000

        @contextmanager
        def connected():
            yield self.database
        self.state.connect = connected
        self.owner = "windows:S-1-5-21-123-456-789-1001"
        self.config = {"driverDigest": "d" * 64, "runtimeOperationId": "start-test"}
        self.manager = SimpleNamespace(spec=SimpleNamespace(storage={"buildRoot": "E:/cargo-targets"}))
        self.owned = ["src/a.rs", "src/b.rs"]
        self.writer = self.state.authorize_session(self.repository, "old-writer", "historical-task-owner",
            self.owned, ["implementation", "validation"], {"source": "user", "task": "existing writer fixture"})
        self.reference, self.object_root = "old-writer-w", Path("E:/cargo-targets/route-test-objects")
        self.record = {"status": "ready", "repositoryId": self.repository, "sessionId": "old-writer",
            "requestId": "creation", "operationId": "creation-1", **self.config, "ownedPaths": self.owned,
            "path": str(ROOT / ".jenkins/worktrees" / self.reference), "baseHead": "b" * 40,
            "creationOperationKey": "c" * 64, "writerPrincipal": self.writer["principal"],
            "writerAuthorizationSnapshot": self.writer["snapshotDigest"]}
        self.state.put("management_worktree", self.reference, self.record)
        self.inputs = {"sourceHead": "a" * 40, "currentHashes": {"src/a.rs": "a" * 64,
            "src/b.rs": "b" * 64, "cargo.toml": "e" * 64},
            "beforeHashes": {"src/a.rs": "c" * 64, "src/b.rs": "c" * 64, "cargo.toml": "e" * 64},
            "compilerControlInputs": {"cargo.toml": "e" * 64},
            "externalInputs": [{"currentHashes": {"src/vm.rs": "f" * 64}}],
            "externalRoots": ["E:/Git/zr_vm"], "externalGitlinks": {}, "targetSourceOperationKey": "c" * 64}
        original = self.payload("inspect-worktree", "inspection")
        self.inspection_key = digest(helper._managed_route_identity(original))
        self.inventory_ref = {"path": str(self.object_root / "management/reports" / self.inspection_key / "source-inventory.json"),
                              "sha256": "f" * 64, "bytes": 123}
        self.inspection = self.state.put("management_operation", self.inspection_key, {
            "phase": "complete", "action": "inspect-worktree", "payload": original, "inputDigest": digest(original),
            "result": {"status": "complete", "operationKey": self.inspection_key, "worktreeId": self.reference,
                       "preparationInputRef": self.inventory_ref, "inventoryDigest": digest(self.inputs)}})
        def patched(patcher):
            self.addCleanup(patcher.stop)
            return patcher.start()
        patched(patch.object(helper, "local_request_principal", return_value=self.owner))
        patched(patch.object(helper, "_managed_route_object_root", return_value=self.object_root))
        self.inventory = patched(patch.object(formal, "_inventory_reference", return_value=(self.inspection, self.inputs)))

    def payload(self, action="prepare-worktree", request="prepare", session="old-writer"):
        value = {"action": action, "repositoryId": self.repository, "sessionId": session,
            "requestId": request, "operationId": request + "-1", "ownedPaths": list(self.owned),
            "worktreeId": self.reference, "buildRoot": "E:/cargo-targets", **self.config}
        if action == "prepare-worktree":
            value.update(inspectionOperationKey=self.inspection_key, preparationInputRef=self.inventory_ref)
        return value

    def route(self, value=None, **kwargs):
        return helper.route_managed_context(self.config, self.repo, self.manager,
            value or self.payload(), state=self.state, **kwargs)

    def auth_row(self, session):
        return dict(self.database.execute("SELECT * FROM authorizations WHERE repository_id=? AND session_id=?",
                                         (self.repository, session)).fetchone())

    def broad_context(self):
        helper.ensure_local_context(self.state, ROOT, "origin-chat", principal=self.owner,
            paths=["src", "cargo.toml"], actions=["implementation", "validation"], evidence={
                "source": "automatic-local-request", "policy": "local-validation-v1", "kind": "ordinary-validation",
                "repositoryId": self.repository, "repositoryRoot": str(ROOT), "ownerPrincipal": self.owner,
                "inputDigest": digest({"repositoryId": self.repository, "sessionId": "origin-chat"})})

    def completed_preparation(self, routed, route):
        key = digest(helper._managed_route_identity(routed))
        source = {"head": self.record["baseHead"], "files": {}, "sourceDigest": "e" * 64}
        manifest = {"baseHead": self.record["baseHead"]}
        prepared = {"status": "complete", "repositoryId": self.repository, **self.config,
            "worktreeId": self.reference, "baseHead": self.record["baseHead"], "sourceManifest": manifest,
            "source": source, "objectRoot": str(self.object_root), **self.inputs,
            "inspectionOperationKey": self.inspection_key, "preparationInputRef": self.inventory_ref,
            "readerSessionId": routed["sessionId"], "readerAuthorizationSnapshot": route["readerAuthorizationSnapshot"],
            "writerSessionId": "old-writer", "writerAuthorizationSnapshot": self.writer["snapshotDigest"],
            "inputDigest": digest(routed)}
        self.state.put("managed_worktree_preparation", key, prepared)
        self.state.put("management_operation", key, {"phase": "complete", "action": "prepare-worktree",
            "payload": routed, "inputDigest": digest(routed), "result": {"status": "complete",
                "worktreePreparationRef": key, "source": source, "sealedCandidateDigest": digest(manifest)}})
        return key, prepared

    def test_inspect_uses_real_plain_path_contract_and_preserves_writer(self):
        old_auth = self.auth_row("old-writer")
        old_worktree = self.state.get("management_worktree", self.reference)
        original = self.payload("inspect-worktree", "inspect-real-plain")
        expected_path = ROOT / ".jenkins/worktrees" / self.reference
        self.assertIsInstance(self.record["path"], str)
        self.assertIsNone(real_plain(expected_path))
        with patch("tools.jenkins.deployment.driver._plain", wraps=real_plain) as validator:
            routed, route = self.route(original)
        validator.assert_any_call(expected_path)
        self.assertTrue(all(isinstance(call.args[0], Path) for call in validator.call_args_list))
        self.assertEqual(original, routed)
        self.assertEqual("old-writer", route["writerSessionId"])
        self.assertEqual(old_auth, self.auth_row("old-writer"))
        self.assertEqual(old_worktree, self.state.get("management_worktree", self.reference))

    def test_prepare_derives_exact_reader_and_preserves_historical_writer(self):
        old_auth = self.auth_row("old-writer")
        old_worktree = self.state.get("management_worktree", self.reference)
        original = self.payload()
        routed, route = self.route(original)
        self.assertEqual(original["sessionId"], "old-writer")
        self.assertNotEqual("old-writer", routed["sessionId"])
        reader = self.state.check_authorization(self.repository, routed["sessionId"], "validation")
        self.assertEqual(["validation"], reader["actions"])
        self.assertEqual(sorted([*self.inputs["currentHashes"], "external/zr_vm/src/vm.rs"]), reader["paths"])
        self.assertEqual(self.writer["principal"], reader["principal"])
        evidence = json.loads(self.auth_row(routed["sessionId"])["evidence"])
        self.assertEqual(self.owner, evidence["ownerPrincipal"])
        self.assertEqual(self.writer["principal"], evidence["writerPrincipal"])
        self.assertEqual("automatic-local-request", evidence["source"])
        self.assertEqual(old_auth, self.auth_row("old-writer"))
        self.assertEqual(old_worktree, self.state.get("management_worktree", self.reference))
        self.assertEqual("old-writer", route["writerSessionId"])
        self.assertEqual(routed["sessionId"], route["readerSessionId"])
        inventory_state = self.inventory.call_args.args[0]
        self.assertIsInstance(inventory_state, helper._ScopedAuthorizationState)
        self.assertIs(inventory_state._authority, self.state)
        self.inventory.assert_called_once_with(inventory_state, self.object_root, self.inspection_key, self.inventory_ref,
            self.reference, self.repository, self.config["driverDigest"], self.config["runtimeOperationId"])

    def test_replay_alias_and_unknown_query_reuse_one_frozen_route(self):
        original = self.payload()
        routed, route = self.route(original)
        self.state.put("management_operation", digest(helper._managed_route_identity(routed)), {"phase": "unknown"})
        self.assertEqual((routed, route), self.route(original))
        self.assertEqual((routed, route), self.route(routed))
        observed, receipt = self.route({**original, "observationOnly": True}, query_only=True)
        self.assertEqual({**routed, "observationOnly": True}, observed)
        self.assertEqual(route, receipt)
        self.assertEqual(1, len(self.state.list("management_context_route")))
        with self.assertRaises(JenkinsError) as error:
            self.route({**original, "changedInput": True})
        self.assertEqual("managed_context_route_conflict", error.exception.code)

    def test_query_without_route_does_not_admit_or_verify_owner(self):
        count = self.database.execute("SELECT count(*) FROM authorizations").fetchone()[0]
        with patch.object(helper, "local_request_principal") as principal, patch.object(helper, "ensure_local_context") as ensure:
            value = self.payload(session="unregistered")
            self.assertEqual((value, None), self.route(value, query_only=True))
        principal.assert_not_called()
        ensure.assert_not_called()
        self.assertEqual(count, self.database.execute("SELECT count(*) FROM authorizations").fetchone()[0])
        self.assertEqual([], self.state.list("management_context_route"))
        with patch.object(State, "__init__", side_effect=AssertionError("query initialized schema")):
            missing_repo = ROOT / ".jenkins/no-such-routing-database"
            self.assertEqual((value, None), helper.route_managed_context(
                self.config, missing_repo, self.manager, value, query_only=True))

    def test_unproven_inventory_and_changed_writer_cannot_create_reader(self):
        self.inventory.side_effect = JenkinsError("managed_inventory_unproven", "not a completed inventory")
        with self.assertRaises(JenkinsError) as error:
            self.route()
        self.assertEqual("managed_inventory_unproven", error.exception.code)
        self.assertEqual([], self.state.list("management_context_route"))
        self.assertEqual(1, self.database.execute("SELECT count(*) FROM authorizations").fetchone()[0])
        self.inventory.side_effect = None
        row = self.state.get("management_worktree", self.reference)
        self.state.put("management_worktree", self.reference, {**self.record, "writerAuthorizationSnapshot": "0" * 64},
                       expected_version=row["version"])
        with self.assertRaises(JenkinsError) as error:
            self.route()
        self.assertEqual("managed_context_writer_changed", error.exception.code)

    def test_prepare_flow_and_submit_reuse_prepared_reader_with_writer_actor(self):
        routed, route = self.route()
        key, prepared = self.completed_preparation(routed, route)
        self.broad_context()
        flow = self.payload("prepare-flow", "flow", session="origin-chat")
        flow.update(worktreePreparationRef=key, flowIdentity={"sessionId": "old-manual-reader",
                    "requestId": "flow-request", "attemptId": "flow-attempt", "generation": 1})
        actual, flow_route = self.route(flow)
        self.assertEqual("old-writer", actual["sessionId"])
        self.assertEqual(prepared["readerSessionId"], actual["flowIdentity"]["sessionId"])
        self.assertEqual(prepared["readerAuthorizationSnapshot"], flow_route["readerAuthorizationSnapshot"])
        identity = {**actual["flowIdentity"], "repositoryId": self.repository,
                    "stageImplementationDigest": self.config["driverDigest"]}
        intent = {"identity": identity, "repositoryRoot": str(ROOT), **self.config,
            "worktreeId": self.reference, "worktreePreparationRef": key, "ownedPaths": self.owned,
            "authorizationSnapshot": prepared["readerAuthorizationSnapshot"]}
        reference = digest(intent)
        self.state.put("current_source_intent", reference, intent)
        submit = self.payload("submit-flow", "submit", session="origin-chat")
        submit["flowPayload"] = {"currentSourceRef": reference, "identity": identity}
        submitted, submission_route = self.route(submit)
        self.assertEqual(prepared["readerSessionId"], submitted["sessionId"])
        self.assertEqual(submit["flowPayload"], submitted["flowPayload"])
        self.assertEqual(prepared["readerSessionId"], submission_route["readerSessionId"])
        self.assertEqual(3, self.database.execute("SELECT count(*) FROM authorizations").fetchone()[0])

    def test_revoked_reader_is_not_recreated_and_claimed_identity_is_not_adopted(self):
        routed, route = self.route()
        self.state.revoke_session(self.repository, routed["sessionId"])
        value = self.payload(request="second-prepare")
        with self.assertRaises(JenkinsError) as error:
            self.route(value)
        self.assertEqual("local_context_conflict", error.exception.code)
        self.assertEqual(1, len(self.state.list("management_context_route")))
        self.broad_context()
        creation = self.payload("create-worktree", "create", session="origin-chat")
        creation.update(name="new", baseHead="b" * 40)
        self.state.put("management_operation", digest(helper._managed_route_identity(creation)), {"phase": "unknown"})
        with self.assertRaises(JenkinsError) as error:
            self.route(creation)
        self.assertEqual("managed_context_already_claimed", error.exception.code)

    def test_new_worktree_gets_narrow_technical_writer_without_broad_scope_change(self):
        self.broad_context()
        old = self.auth_row("origin-chat")
        for action in ("create-worktree", "prepare-merge"):
            with self.subTest(action=action):
                creation = self.payload(action, action, session="origin-chat")
                creation.update(name="new", baseHead="b" * 40)
                routed, route = self.route(creation)
                writer = self.state.check_authorization(self.repository, routed["sessionId"], "implementation")
                self.assertEqual(self.owned, writer["paths"])
                self.assertLessEqual(len(routed["sessionId"]), 11)
                self.assertEqual(64, len(route["derivationDigest"]))
                self.assertEqual(["implementation", "validation"], writer["actions"])
                self.assertEqual(self.owner, writer["principal"])
                self.assertEqual("origin-chat", route["originIdentity"]["sessionId"])
                self.assertEqual(routed["sessionId"], route["writerSessionId"])
                self.assertEqual(old, self.auth_row("origin-chat"))
                with self.assertRaises(JenkinsError):
                    self.state.check_authorization(self.repository, routed["sessionId"], "implementation", ["cargo.toml"])
                reused = helper.admit_ordinary_context(self.state, ROOT, routed, principal=self.owner,
                    driver_digest=self.config["driverDigest"], runtime_operation_id=self.config["runtimeOperationId"])
                self.assertEqual(writer, reused)

    def test_existing_exact_writer_creation_keeps_its_original_identity(self):
        creation = self.payload("create-worktree", "legacy")
        creation.update(name="w", baseHead="b" * 40)
        old = self.auth_row("old-writer")
        self.assertEqual((creation, None), self.route(creation))
        self.assertEqual(old, self.auth_row("old-writer"))
        self.assertEqual([], self.state.list("management_context_route"))


if __name__ == "__main__":
    unittest.main()
