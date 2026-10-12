"""Consumer acceptance uses actual shared claims, collectors and publication.

Native collector records are controlled fixtures; these tests launch no Cargo
or native process. All fixture outputs stay in the approved build namespace.
"""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from pathlib import Path
from threading import Event
import tempfile
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.artifacts import publish_execution_outputs
from tools.jenkins.contracts import JenkinsError, digest, file_digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.state import State
from tools.jenkins.validation import TrustedReceiptAuthority, closure_manifest
from tools.jenkins.workflow.execution import ExecutionEngine, execution_lock
from tools.jenkins.workflow.handler import handle


BUILD_ROOT = r"E:\cargo-targets"


class WorkflowConsumerAcceptanceTests(unittest.TestCase):
    def setUp(self):
        self.approved = canonical_build_root(BUILD_ROOT)
        temporary_root = self.approved.namespace() / "tmp"
        temporary_root.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="consumer-acceptance-", dir=temporary_root)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.state = State(self.root / "state.sqlite3")
        self.state.put("resource_policy", "default", {"cpuBudget": 2, "memoryBudget": 1024,
                                                       "diskBudget": 2048})
        self.source, self.coverage, self.driver, self.preparation = (char * 64 for char in "abcd")
        self.driver_binding = {"digest": self.driver, "generation": "1",
                               "runtimeOperationId": "fixture-" + self.root.name}
        recipe = {"kind": "cargo", "buildRoot": BUILD_ROOT,
                  "preparationKey": self.preparation,
                  "preparationRoot": str(self.approved.namespace() / "preparations" / self.preparation /
                                         "inputs" / "source"),
                  "resourceEstimates": {"cpu": 1, "memoryBytes": 256, "diskBytes": 64},
                  "phases": ["compile", "list", "test"], "requiredTestStages": ["unit_test"]}
        self.recipe = digest(recipe)
        self.state.put("recipe_plan", self.recipe, {"recipe": recipe, "recipeDigest": self.recipe,
            "sourceRef": self.source, "coverageRef": self.coverage, "driver": self.driver_binding,
            "generation": "1"})
        self.state.put("sealed_input", self.source, {"status": "sealed"})
        self.resources = Mock()
        self.engine = ExecutionEngine(self.state, repo_root=self.root, build_root=BUILD_ROOT,
                                      resource_manager=self.resources)
        self.authority = TrustedReceiptAuthority(self.state, repo_root=self.root)
        self.identities, self.consumers = {}, {}
        for name in ("a", "b"):
            identity = {"sessionId": "session-" + name, "requestId": "request-" + name,
                        "attemptId": "attempt-1", "generation": 1, "sourceInputDigest": self.source,
                        "coverageDigest": self.coverage, "stageImplementationDigest": self.driver}
            consumer = f"{identity['sessionId']}:{identity['requestId']}:{identity['attemptId']}"
            self.identities[name], self.consumers[name] = identity, consumer
            stages = ["build", "unit_test", "acceptance"]
            self.state.put("workflow", consumer, {"identity": identity, "sourceDigest": self.source,
                "coverageDigest": self.coverage, "driverDigest": self.driver, "generation": 1,
                "buildRoot": BUILD_ROOT, "status": "ready", "requiredStages": stages[:-1],
                "stageRecipeDigests": {stage: self.recipe for stage in stages[:-1]},
                "stageReceipts": {}, "dagDigest": digest(stages), "requiredOutputs": [],
                "requiredOutputsByStage": {stage: ["stdout.log", "stderr.log"] for stage in stages[:-1]},
                "stages": [{"name": stage, "status": "pending"} for stage in stages]})
            claimed = self.engine.claim(source=self.source, recipe=self.recipe,
                                        driver=self.driver_binding, consumer_id=consumer)
            if name == "a":
                self.execution_id = claimed["executionId"]
            self.assertEqual(self.execution_id, claimed["executionId"])
            row = self.state.get("workflow", consumer)
            self.state.put("workflow", consumer, {**row["payload"], "executionId": self.execution_id})

    def _call(self, action, name="a"):
        return handle(action, {"identity": self.identities[name], "executionId": self.execution_id},
                      self.state, self.root)

    def _collect_host(self):
        """Create trusted phase receipts, leaving completion to the real engine."""
        eid = self.execution_id
        output_base = self.root / "receipts" / eid
        row = self.state.get("execution", eid)
        self.state.put("execution", eid, {**row["payload"], "status": "running",
                                         "phaseOutputRoot": str(output_base)})
        records, receipts = [], {}
        for index, phase in enumerate(("compile", "list", "test")):
            native, birth = "native-" + phase, "birth-" + phase
            stage = {"compile": "build", "list": "discovery", "test": "unit_test"}[phase]
            proof = {"complete": True, "processExitCode": 0, "stdoutEof": True,
                     "stderrEof": True, "childrenGone": True}
            self.state.put("native_job", native, {"nativeJobId": native, "status": "terminal",
                "owner": eid, "executionId": eid, "birthToken": birth, "pid": 12000 + index,
                "completeProof": proof})
            record = {"phase": phase, "stage": stage, "nativeJobId": native, "terminalProof": proof}
            if phase == "list":
                record["discovery"] = {"testCount": 1, "source": "cargo-test-list"}
            else:
                attempt = f"{eid}:compile" if phase == "compile" else f"{eid}:test:unit_test"
                output = output_base / phase
                output.mkdir(parents=True)
                for stream in ("stdout", "stderr"):
                    (output / f"{stream}.log").write_text(f"{self.root.name} {phase} {stream}\n", encoding="utf-8")
                self.authority.register_attempt({"executionId": attempt, "stage": stage,
                    "sourceDigest": self.source, "recipeDigest": self.recipe,
                    "coverageDigest": self.coverage, "driverDigest": self.driver,
                    "nativeJobId": native, "birthToken": birth, "phase": phase,
                    "outputRoot": str(output), "buildRoot": str(output),
                    "requiredOutputs": ["stdout.log", "stderr.log"]})
                self.authority.record_terminal(attempt, {**proof, "nativeJobId": native,
                                                       "birthToken": birth}, closure_manifest(output))
                receipts[stage] = attempt
            records.append(record)
        self.state.put("execution_host", eid, {"status": "terminal", "phaseRecords": records,
                                               "stageReceipts": receipts})

    def _finish(self):
        self._collect_host()
        self.assertEqual("passed", self.engine.observe(self.execution_id)["status"])
        for name in ("a", "b"):
            self._call("observe-flow", name)
        return self.state.get("artifact_bundle", self.execution_id)["payload"]

    def _reject(self, code, name="a"):
        before = self.state.list("acceptance")
        with self.assertRaises(JenkinsError) as raised:
            self.authority.accept_flow({"flowId": self.consumers[name]})
        self.assertEqual(code, raised.exception.code)
        self.assertEqual(before, self.state.list("acceptance"))

    def test_cancelled_consumer_cannot_accept_after_its_peer_finishes(self):
        self._call("cancel-flow", "a")
        self._finish()
        cancelled = self._call("observe-flow", "a")
        self.assertEqual("cancelled", cancelled["status"])
        self.assertTrue(all(item["status"] == "pending" for item in cancelled["stages"]))
        self._reject("workflow_terminal", "a")
        with self.assertRaises(JenkinsError) as raised:
            self._call("accept-flow", "a")
        self.assertEqual("workflow_terminal", raised.exception.code)
        accepted = self._call("accept-flow", "b")["receipt"]
        self.assertEqual(self.consumers["b"], accepted["executionEvidence"]["consumerId"])
        self.assertTrue(all(key.startswith(self.consumers["b"] + ":")
                            for key in accepted["executionEvidence"]["artifactRefs"]))
        self.assertEqual("cancelled", self.state.get("workflow", self.consumers["a"])["payload"]["status"])

    def test_validated_during_publication_is_neither_ready_nor_acceptable(self):
        self._collect_host()
        publishing, resume = Event(), Event()

        def paused_publish(*args, **kwargs):
            publishing.set()
            if not resume.wait(10):
                raise RuntimeError("publication fixture timed out")
            return publish_execution_outputs(*args, **kwargs)

        with ThreadPoolExecutor(max_workers=1) as workers, \
                patch("tools.jenkins.artifacts.publish_execution_outputs", side_effect=paused_publish):
            finishing = workers.submit(self.engine.observe, self.execution_id)
            try:
                self.assertTrue(publishing.wait(10))
                self.assertEqual("validated", self.state.get("execution", self.execution_id)["payload"]["status"])
                observed = self._call("observe-flow")
                self.assertEqual("validated", observed["status"])
                self.assertTrue(all(item["status"] == "pending" for item in observed["stages"]))
                self._reject("execution_busy")
            finally:
                resume.set()
            self.assertEqual("passed", finishing.result(timeout=10)["status"])
        self._call("observe-flow")
        self.assertEqual("accepted", self._call("accept-flow")["status"])

    def test_validated_with_stage_receipts_cannot_bypass_publication_gate(self):
        self._finish()
        row = self.state.get("execution", self.execution_id)
        self.state.put("execution", self.execution_id, {**row["payload"], "status": "validated"})
        with execution_lock(self.state, self.execution_id):
            self.assertEqual("validated", self._call("observe-flow")["status"])
        self._reject("workflow_execution_pending")

    def test_missing_or_foreign_consumer_refs_cannot_borrow_peer_refs(self):
        bundle = self._finish()
        object_digest = bundle["files"][0]["digest"]
        key = self.consumers["a"] + ":" + object_digest
        original = self.state.get("artifact_ref", key)["payload"]
        self.state.delete("artifact_ref", key)
        self._reject("workflow_artifact_ref_missing")
        for change in ({"status": "released"}, {"owner": self.consumers["b"]},
                       {"executionId": "foreign-execution"}, {"digest": "f" * 64}):
            with self.subTest(change=change):
                self.state.put("artifact_ref", key, {**original, **change})
                self._reject("workflow_artifact_ref_missing")
        self.state.put("artifact_ref", key, original)
        self.assertEqual("accepted", self._call("accept-flow")["status"])

    def test_inactive_consumer_and_missing_binding_are_rejected_independently(self):
        self._finish()
        consumer_key = self.execution_id + ":" + self.consumers["a"]
        original = self.state.get("execution_consumer", consumer_key)["payload"]
        self.state.put("execution_consumer", consumer_key, {**original, "status": "released"})
        self._reject("workflow_consumer_inactive")
        self.state.put("execution_consumer", consumer_key, original)
        execution = self.state.get("execution", self.execution_id)["payload"]
        binding = digest({"consumer": self.consumers["a"], "executionKey": execution["executionKeyDigest"]})
        self.state.delete("execution_binding", binding)
        self._reject("workflow_consumer_mismatch")

    def test_missing_or_wrong_published_identity_cannot_accept(self):
        bundle = self._finish()
        self.state.delete("artifact_bundle", self.execution_id)
        self._reject("workflow_artifact_bundle_missing")
        for field in ("sourceDigest", "recipeDigest", "coverageDigest", "driverDigest"):
            with self.subTest(field=field):
                changed = deepcopy(bundle)
                changed["identity"][field] = "f" * 64
                changed["bundleDigest"] = digest({name: changed[name] for name in ("executionKey", "identity", "files")})
                self.state.put("artifact_bundle", self.execution_id, changed)
                row = self.state.get("execution", self.execution_id)
                self.state.put("execution", self.execution_id, {**row["payload"], "artifacts": changed})
                self._reject("workflow_artifact_bundle_missing")

    def test_missing_canonical_execution_identity_fails_closed(self):
        self._finish()
        original = self.state.get("execution", self.execution_id)["payload"]
        for field in ("executionKey", "executionKeyDigest", "ordinal", "sourceDigest", "recipeDigest"):
            with self.subTest(field=field):
                changed = deepcopy(original)
                changed.pop(field)
                self.state.put("execution", self.execution_id, changed)
                self._reject("execution_identity_corrupt")
        self.state.put("execution", self.execution_id, original)
        self.state.delete("execution", self.execution_id)
        self._reject("execution_not_found")

    def test_changed_physical_cas_bytes_cannot_accept_valid_metadata(self):
        bundle = self._finish()
        object_digest = bundle["files"][0]["digest"]
        path = self.approved.namespace() / "artifacts" / "artifacts" / "objects" / object_digest
        original = path.read_bytes()
        try:
            path.write_bytes(bytes([original[0] ^ 1]) + original[1:])
            self._reject("workflow_artifact_changed")
        finally:
            path.write_bytes(original)

    def test_acceptance_holds_shared_lock_until_workflow_and_receipt_commit(self):
        self._finish()
        verifying, resume = Event(), Event()

        def paused_digest(path):
            verifying.set()
            if not resume.wait(10):
                raise RuntimeError("acceptance fixture timed out")
            return file_digest(path)

        with ThreadPoolExecutor(max_workers=1) as workers, \
                patch("tools.jenkins.validation.receipts.file_digest", side_effect=paused_digest):
            accepting = workers.submit(self.authority.accept_flow, {"flowId": self.consumers["a"]})
            try:
                self.assertTrue(verifying.wait(10))
                with self.assertRaises(JenkinsError) as raised:
                    self.engine.cancel(self.execution_id, self.consumers["a"])
                self.assertEqual("execution_busy", raised.exception.code)
                self.assertEqual([], self.state.list("acceptance"))
            finally:
                resume.set()
            accepted = accepting.result(timeout=10)
        self.assertEqual("accepted", self.state.get("workflow", self.consumers["a"])["payload"]["status"])
        saved = self.state.get("acceptance", accepted["acceptanceId"])
        self.engine.cancel(self.execution_id, self.consumers["a"])
        self.assertEqual("released", self.state.get("execution_consumer",
            self.execution_id + ":" + self.consumers["a"])["payload"]["status"])
        self.assertEqual(accepted, self._call("accept-flow")["receipt"])
        self.assertEqual("accepted", self._call("observe-flow")["status"])
        self.assertEqual(saved, self.state.get("acceptance", accepted["acceptanceId"]))


if __name__ == "__main__":
    unittest.main()
