"""Public workflow replay keeps registration, execution and request authority."""
from __future__ import annotations

from contextlib import ExitStack
from copy import deepcopy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.source import register_sealed_consumer, repository_id
from tools.jenkins.state import State
from tools.jenkins.workflow.handler import handle
from tools.jenkins.workflow.planning import RecipePlan


BUILD_ROOT = r"E:\cargo-targets"


class WorkflowReplayTests(unittest.TestCase):
    def setUp(self):
        self.approved = canonical_build_root(BUILD_ROOT)
        temporary_root = self.approved.namespace() / "tmp"
        temporary_root.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="workflow-replay-", dir=temporary_root)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = State(self.root / "state.sqlite3")
        self.repository = repository_id(self.root)
        self.driver = "c" * 64
        self.runtime = "runtime-replay"
        self.active = {"digest": self.driver, "generation": self.runtime, "runtimeOperationId": self.runtime}
        self.state.put("resource_policy", "default", {"cpuBudget": 2, "memoryBudget": 1024, "diskBudget": 2048})
        self.objects = self.root / "objects"
        object_files = self.objects / "inputs" / "objects"
        object_files.mkdir(parents=True)
        before, after = self.root / "before.rs", self.root / "after.rs"
        before.write_text("pub fn value() -> u32 { 1 }\n", encoding="utf-8")
        after.write_text("pub fn value() -> u32 { 2 }\n", encoding="utf-8")
        before_hash, after_hash = file_digest(before), file_digest(after)
        (object_files / before_hash).write_bytes(before.read_bytes())
        (object_files / after_hash).write_bytes(after.read_bytes())
        manifest = {"schemaVersion": 1, "repositoryId": self.repository, "baseHead": None,
                    "entries": [{"path": "src/lib.rs", "status": "present", "beforeHash": before_hash,
                                 "sha256": after_hash, "objectDigest": after_hash, "size": after.stat().st_size}],
                    "external": [], "declaredDependencies": []}
        self.source = digest(manifest)
        coverage = {"template": "module_unit", "selectedTests": 1, "sourceDigest": self.source}
        self.coverage = digest(coverage)
        self.patch_ref = "patch-fixture"
        operation = self.state.put_operation("source_patch", self.patch_ref, {
            "repositoryRoot": str(self.root), "objectRoot": str(self.objects), "owner": "source-owner",
            "paths": ["src/lib.rs"], "beforeObjects": {"src/lib.rs": before_hash},
            "expectedAfterHashes": {"src/lib.rs": after_hash}})
        self.state.transition_operation(self.patch_ref, operation["status"], "complete",
                                        {"afterHashes": {"src/lib.rs": after_hash}})
        self.state.put("sealed_input", self.source, {"status": "sealed", "sourceDigest": self.source,
            "repositoryId": self.repository, "repositoryRoot": str(self.root), "manifest": manifest,
            "sourceManifest": manifest, "coverage": coverage, "coverageDigest": self.coverage,
            "objectRoot": str(self.objects), "ownedPaths": ["src/lib.rs"], "buildRoot": BUILD_ROOT,
            "patchOperationRefs": [self.patch_ref]})
        preparation = "d" * 64
        recipe = {"kind": "cargo", "buildRoot": BUILD_ROOT, "preparationKey": preparation,
                  "preparationRoot": str(self.approved.namespace() / "preparations" / preparation / "inputs" / "source"),
                  "resourceEstimates": {"cpu": 1, "memoryBytes": 256, "diskBytes": 64},
                  "phases": ["compile", "list", "test"], "requiredTestStages": ["unit_test"]}
        self.recipe_ref = digest(recipe)
        driver = {"digest": self.driver, "generation": "1", "runtimeOperationId": self.runtime}
        self.plan = RecipePlan(self.recipe_ref, recipe, self.source, self.coverage, "1", driver)
        self.identities, self.keys = {}, {}
        for name in ("a", "b", "c"):
            identity = {"repositoryId": self.repository, "sessionId": "session-" + name,
                        "requestId": "request-" + name, "attemptId": "attempt-" + name, "generation": 1,
                        "sourceInputDigest": self.source, "coverageDigest": self.coverage,
                        "stageImplementationDigest": self.driver}
            key = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}"
            self.identities[name], self.keys[name] = identity, key
            grant = self.state.authorize_session(self.repository, identity["sessionId"], "user", ["src"],
                ["validation"], {"source": "user", "task": "workflow replay fixture"})
            self.state.submit_request({**identity, "sealedInputRef": self.source, "driverDigest": self.driver,
                "runtimeOperationId": self.runtime, "buildRoot": BUILD_ROOT,
                "authorizationSnapshot": grant["snapshotDigest"]})
            register_sealed_consumer(self.state, repository_id_value=self.repository,
                session_id=identity["sessionId"], owner=identity["attemptId"], source_digest=self.source,
                manifest_digest=self.source)
        guards = ExitStack()
        self.addCleanup(guards.close)
        guards.enter_context(patch("tools.jenkins.deployment.driver.active_driver_binding", return_value=self.active))
        self.planner = guards.enter_context(patch("tools.jenkins.workflow.planning.RecipePlanner.plan", side_effect=self._plan))
        self.dispatch = guards.enter_context(patch("tools.jenkins.workflow.handler._ensure_execution_job",
                                                  return_value={"delivery": "submitted", "formalAcceptance": False}))
        self.syntax = guards.enter_context(patch("tools.jenkins.workflow.handler.syntax_receipt",
                                                return_value={"commandDigest": digest("syntax-fixture")}))
        self.format = guards.enter_context(patch("tools.jenkins.workflow.handler.format_receipt",
                                                return_value={"commandDigest": digest("format-fixture")}))

    def _plan(self, **kwargs):
        self.assertEqual(self.source, kwargs["sealed_input_ref"])
        self.assertEqual(self.plan.driver, kwargs["driver"])
        self.assertEqual(["src/lib.rs"], kwargs["changed_paths"])
        self.assertEqual("module_unit", kwargs["template"])
        self.state.put("recipe_plan", self.recipe_ref, {"recipe": self.plan.recipe, "recipeDigest": self.recipe_ref,
            "sourceRef": self.source, "coverageRef": self.coverage, "driver": self.plan.driver, "generation": "1"})
        return self.plan

    def _call(self, action, name="a", **extra):
        return handle(action, {"identity": self.identities[name], **extra}, self.state, self.root)

    def _register(self, name="a", **extra):
        return self._call("register-flow", name, sealedInputRef=self.source, patchOperationRef=self.patch_ref,
                          buildRoot=BUILD_ROOT, **extra)

    def _pipeline_register(self, name="a"):
        identity = self.identities[name]
        return handle("register-flow", {**identity, "identity": identity,
            "driverDigest": self.driver, "runtimeOperationId": self.runtime,
            "sealedInputRef": self.source, "patchOperationRef": self.patch_ref,
            "patchRequestRef": "", "currentSourceRef": "", "sourcePreparationRef": "",
            "buildRoot": BUILD_ROOT}, self.state, self.root)

    def _request(self, name="a"):
        identity = self.identities[name]
        return self.state.get_request(self.repository, identity["sessionId"], identity["requestId"])

    def _prepared(self, name="a"):
        self._register(name)
        self._call("compose-flow", name)
        self._call("run-stage", name, stage="patch", patchOperationRef=self.patch_ref)
        self._call("run-stage", name, stage="syntax")
        self._call("run-stage", name, stage="format")
        return self._call("submit-execution", name)

    def _shared(self):
        values = [self._prepared(name) for name in ("a", "b", "c")]
        self.assertEqual(1, len({value["executionId"] for value in values}))
        eid = values[0]["executionId"]
        row = self.state.get("execution", eid)
        self.state.put("execution", eid, {**row["payload"], "status": "running"})
        self.state.put("execution_host", eid, {"status": "running", "hostPid": 123})
        return eid

    def test_register_compose_and_light_replay_preserve_the_submitted_execution_and_receipts(self):
        submitted = self._prepared()
        before = self.state.get("workflow", self.keys["a"])
        attempts = self.state.list("validation_attempt")
        self.assertNotEqual(before["payload"]["registrationExecutionId"], submitted["executionId"])
        registered = self._register()
        composed = self._call("compose-flow")
        self.assertEqual(submitted["executionId"], registered["executionId"])
        self.assertEqual(submitted["executionId"], composed["executionId"])
        for stage in ("patch", "syntax", "format"):
            self._call("run-stage", stage=stage, patchOperationRef=self.patch_ref)
        self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        self.assertEqual(attempts, self.state.list("validation_attempt"))
        self.assertEqual(self.recipe_ref, before["payload"]["recipeRef"])
        self.assertEqual(1, self.planner.call_count)
        self.assertEqual(1, self.dispatch.call_count)
        self.assertEqual(1, self.syntax.call_count)
        self.assertEqual(1, self.format.call_count)
        resubmitted = self._call("submit-execution")
        self.assertEqual(submitted["executionId"], resubmitted["executionId"])
        replayed = self.state.get("workflow", self.keys["a"])["payload"]
        for field in ("registrationExecutionId", "recipeRef", "executionId", "stageReceipts",
                      "stageRecipeDigests", "requiredOutputsByStage", "stages", "status"):
            self.assertEqual(before["payload"][field], replayed[field])
        self.assertEqual(attempts, self.state.list("validation_attempt"))
        self.assertEqual(1, self.state.get("execution", submitted["executionId"])["payload"]["refs"])
        self.assertEqual(1, self.planner.call_count)
        self.assertEqual([], self.state.list("native_job"))

    def test_compose_replay_during_partial_light_progress_does_not_clear_passed_receipts(self):
        self._register()
        self._call("compose-flow")
        self._call("run-stage", stage="patch", patchOperationRef=self.patch_ref)
        before = self.state.get("workflow", self.keys["a"])
        self.assertEqual("pending", before["payload"]["status"])
        self.assertEqual("pending", self._call("compose-flow")["status"])
        self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        self.assertEqual("passed", before["payload"]["stages"][0]["status"])

    def test_manual_registration_replays_the_actual_pipeline_empty_parameter_defaults(self):
        submitted = self._prepared()
        before = self.state.get("workflow", self.keys["a"])
        attempts = self.state.list("validation_attempt")
        self.assertIsNone(before["payload"]["sourcePreparationRef"])
        result = self._pipeline_register()
        self.assertEqual(submitted["executionId"], result["executionId"])
        self._call("compose-flow")
        for stage in ("patch", "syntax", "format"):
            self._call("run-stage", stage=stage, patchOperationRef=self.patch_ref,
                       sourcePreparationRef="")
        self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        self.assertEqual(attempts, self.state.list("validation_attempt"))
        self.assertEqual(1, self.planner.call_count)
        self.assertEqual(1, self.dispatch.call_count)

    def test_optional_preparation_absence_is_canonical_in_both_registration_directions(self):
        self._pipeline_register()
        before = self.state.get("workflow", self.keys["a"])
        self.assertIsNone(before["payload"]["sourcePreparationRef"])
        for value in (None, ""):
            with self.subTest(value=value):
                self._register(sourcePreparationRef=value)
                self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        self._register()
        self.assertEqual(before, self.state.get("workflow", self.keys["a"]))

    def test_absent_optional_preparation_cannot_hide_a_changed_or_typed_preparation(self):
        self._prepared()
        before = self.state.get("workflow", self.keys["a"])
        for value in ("foreign-preparation", 0, False, {}, []):
            with self.subTest(value=value), self.assertRaises(JenkinsError) as raised:
                self._register(sourcePreparationRef=value)
            self.assertEqual("source_preparation_mismatch", raised.exception.code)
            self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        sealed = self.state.get("sealed_input", self.source)
        for kind in ("current-source", "already-applied-patch"):
            changed = {**sealed["payload"], "patchOperationRefs": [],
                       "sourceManifest": {**sealed["payload"]["sourceManifest"], "preparationKind": kind}}
            self.state.put("sealed_input", self.source, changed)
            for value in (None, ""):
                with self.subTest(kind=kind, value=value), self.assertRaises(JenkinsError) as raised:
                    self._call("register-flow", sealedInputRef=self.source, buildRoot=BUILD_ROOT,
                               sourcePreparationRef=value)
                self.assertEqual("source_preparation_missing", raised.exception.code)
                self.assertEqual(before, self.state.get("workflow", self.keys["a"]))
        self.state.put("sealed_input", self.source, sealed["payload"])

    def test_replay_rejects_a_changed_registration_recipe_or_identity(self):
        self._prepared()
        before = self.state.get("workflow", self.keys["a"])
        with self.assertRaises(JenkinsError) as raised:
            self._register(recipe={"kind": "changed"})
        self.assertEqual("workflow_identity_mismatch", raised.exception.code)
        for field in ("sourceInputDigest", "coverageDigest", "stageImplementationDigest", "attemptId"):
            with self.subTest(field=field), self.assertRaises(JenkinsError):
                handle("register-flow", {"identity": {**self.identities["a"], field: "foreign"},
                    "sealedInputRef": self.source, "buildRoot": BUILD_ROOT}, self.state, self.root)
        self.assertEqual(before, self.state.get("workflow", self.keys["a"]))

    def test_replay_rejects_a_missing_registration_anchor_or_foreign_execution_binding(self):
        self._prepared()
        original = self.state.get("workflow", self.keys["a"])["payload"]
        missing = dict(original)
        missing.pop("registrationExecutionId")
        self.state.put("workflow", self.keys["a"], missing)
        with self.assertRaises(JenkinsError):
            self._register()
        self.state.put("workflow", self.keys["a"], {**original, "executionId": "foreign-execution"})
        with self.assertRaises(JenkinsError):
            self._register()
        self.state.put("workflow", self.keys["a"], original)
        execution = self.state.get("execution", original["executionId"])["payload"]
        binding_key = digest({"consumer": self.keys["a"], "executionKey": execution["executionKeyDigest"]})
        binding = self.state.get("execution_binding", binding_key)
        self.state.put("execution_binding", binding_key, {**binding["payload"], "consumerId": self.keys["b"]})
        with self.assertRaises(JenkinsError):
            self._register()
        self.state.put("execution_binding", binding_key, binding["payload"])
        with patch("tools.jenkins.deployment.driver.active_driver_binding",
                   return_value={**self.active, "runtimeOperationId": "foreign-runtime"}), self.assertRaises(JenkinsError):
            self._register()

    def test_replay_preserves_failed_cancelled_and_unknown_history(self):
        self._prepared()
        original = self.state.get("workflow", self.keys["a"])["payload"]
        for status in ("failed", "cancelled", "unknown", "reconciling"):
            with self.subTest(status=status):
                self.state.put("workflow", self.keys["a"], {**original, "status": status,
                    "recovery": {"status": "blocked", "reasonCode": "original-error"}})
                before = self.state.get("workflow", self.keys["a"])
                self.assertEqual(status, self._register()["status"])
                self.assertEqual(status, self._call("compose-flow")["status"])
                self.assertEqual(before, self.state.get("workflow", self.keys["a"]))

    def test_public_cancel_updates_only_its_request_ref_and_workflow(self):
        eid = self._shared()
        peers = {name: (self._request(name), self.state.get("workflow", self.keys[name])) for name in ("b", "c")}
        host = self.state.get("execution_host", eid)
        result = self._call("cancel-flow", executionId=eid)
        self.assertEqual("running", result["status"])
        self.assertEqual("cancelled", self._request()["status"])
        self.assertEqual("cancelled", self.state.get("workflow", self.keys["a"])["payload"]["status"])
        self.assertEqual("released", self.state.get("execution_consumer", f"{eid}:{self.keys['a']}")["payload"]["status"])
        execution = self.state.get("execution", eid)["payload"]
        self.assertEqual(2, execution["refs"])
        self.assertEqual({self.keys["b"], self.keys["c"]}, set(execution["consumers"]))
        self.assertFalse(execution.get("cancelRequested", False))
        self.assertIsNone(self.state.get("execution_control", eid))
        self.assertEqual(host, self.state.get("execution_host", eid))
        for name in ("b", "c"):
            self.assertEqual(peers[name], (self._request(name), self.state.get("workflow", self.keys[name])))
        self._call("cancel-execution", executionId=eid)
        self.assertEqual("cancelled", self._request()["status"])

    def test_cancel_transaction_rolls_back_request_ref_and_workflow_together(self):
        eid = self._shared()
        before = (self._request(), self.state.get("workflow", self.keys["a"]),
                  self.state.get("execution", eid), self.state.list("execution_consumer"))
        with patch.object(self.state, "transition_request", side_effect=JenkinsError("state_conflict", "fixture")):
            with self.assertRaises(JenkinsError) as raised:
                self._call("cancel-flow", executionId=eid)
        self.assertEqual("state_conflict", raised.exception.code)
        self.assertEqual(before, (self._request(), self.state.get("workflow", self.keys["a"]),
                                 self.state.get("execution", eid), self.state.list("execution_consumer")))

    def test_public_cancel_preserves_an_accepted_request_and_workflow_history(self):
        eid = self._shared()
        row = self.state.get("workflow", self.keys["a"])
        history = {"status": "accepted", "receiptRef": "historical-acceptance"}
        self.state.put("workflow", self.keys["a"], {**row["payload"], "status": "accepted", "acceptance": history})
        with self.state.transaction() as connection:
            connection.execute("UPDATE requests SET status='accepted' WHERE repository_id=? AND session_id=? AND request_id=?",
                (self.repository, self.identities["a"]["sessionId"], self.identities["a"]["requestId"]))
        before = (self._request(), self.state.get("workflow", self.keys["a"]))
        self._call("cancel-flow", executionId=eid)
        self.assertEqual(before, (self._request(), self.state.get("workflow", self.keys["a"])))
        self.assertEqual(2, self.state.get("execution", eid)["payload"]["refs"])

    def test_public_cancel_preserves_unknown_request_and_failed_workflow_history(self):
        eid = self._shared()
        row = self.state.get("workflow", self.keys["a"])
        failure = {"status": "blocked", "reasonCode": "original-registration-error"}
        self.state.put("workflow", self.keys["a"], {**row["payload"], "status": "failed", "recovery": failure})
        with self.state.transaction() as connection:
            connection.execute("UPDATE requests SET status='unknown' WHERE repository_id=? AND session_id=? AND request_id=?",
                (self.repository, self.identities["a"]["sessionId"], self.identities["a"]["requestId"]))
        request = self._request()
        self._call("cancel-flow", executionId=eid)
        self.assertEqual(request, self._request())
        flow = self.state.get("workflow", self.keys["a"])["payload"]
        self.assertEqual("failed", flow["status"])
        self.assertEqual(failure, flow["recovery"])
        self.assertTrue(flow["consumerCancelled"])
        self.assertEqual(2, self.state.get("execution", eid)["payload"]["refs"])

    def test_cancel_with_a_corrupt_runtime_binding_fails_closed_without_releasing_refs(self):
        eid = self._shared()
        execution = self.state.get("execution", eid)
        payload = dict(execution["payload"])
        payload.pop("driver")
        self.state.put("execution", eid, payload)
        before = (self._request(), self.state.get("workflow", self.keys["a"]),
                  self.state.get("execution", eid), self.state.list("execution_consumer"))
        with self.assertRaises(JenkinsError) as raised:
            self._call("cancel-flow", executionId=eid)
        self.assertEqual("execution_identity_corrupt", raised.exception.code)
        self.assertEqual(before, (self._request(), self.state.get("workflow", self.keys["a"]),
                                 self.state.get("execution", eid), self.state.list("execution_consumer")))

    def test_cancel_rejects_a_foreign_consumer_binding_without_releasing_refs(self):
        eid = self._shared()
        execution = self.state.get("execution", eid)["payload"]
        key = digest({"consumer": self.keys["a"], "executionKey": execution["executionKeyDigest"]})
        binding = self.state.get("execution_binding", key)
        self.state.put("execution_binding", key, {**binding["payload"], "consumerId": self.keys["b"]})
        before = (self._request(), self.state.get("workflow", self.keys["a"]),
                  self.state.get("execution", eid), self.state.list("execution_consumer"))
        with self.assertRaises(JenkinsError) as raised:
            self._call("cancel-flow", executionId=eid)
        self.assertEqual("execution_identity_mismatch", raised.exception.code)
        self.assertEqual(before, (self._request(), self.state.get("workflow", self.keys["a"]),
                                 self.state.get("execution", eid), self.state.list("execution_consumer")))

    def test_cancel_rejects_a_foreign_request_attempt_without_releasing_refs(self):
        eid = self._shared()
        request = self._request()
        before = (self.state.get("workflow", self.keys["a"]), self.state.get("execution", eid),
                  self.state.list("execution_consumer"))
        get_request = self.state.get_request
        for field, value in (("attemptId", "foreign"), ("generation", 2), ("sealedInputRef", "f" * 64),
                             ("sourceInputDigest", "f" * 64), ("stageImplementationDigest", "f" * 64),
                             ("coverageDigest", "f" * 64), ("driverDigest", "f" * 64),
                             ("runtimeOperationId", "foreign-runtime"), ("authorizationSnapshot", "f" * 64)):
            with self.subTest(field=field):
                changed = deepcopy(request)
                changed["payload"][field] = value

                def foreign(repository, session, request_id, **kwargs):
                    if (repository, session, request_id) == (self.repository, self.identities["a"]["sessionId"],
                                                            self.identities["a"]["requestId"]):
                        return changed
                    return get_request(repository, session, request_id, **kwargs)

                with patch.object(self.state, "get_request", side_effect=foreign), self.assertRaises(JenkinsError):
                    self._call("cancel-flow", executionId=eid)
                self.assertEqual(before, (self.state.get("workflow", self.keys["a"]), self.state.get("execution", eid),
                                          self.state.list("execution_consumer")))
                self.assertEqual(request, self._request())


if __name__ == "__main__":
    unittest.main()
