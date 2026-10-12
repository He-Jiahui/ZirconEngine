"""Execution admission, reference sharing and terminal authority regressions."""
from pathlib import Path
from copy import deepcopy
from concurrent.futures import ThreadPoolExecutor
from contextlib import contextmanager
from threading import Event
import tempfile
import unittest
from unittest.mock import Mock, patch
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.resources.paths import repository_build_root, APPROVED_NAMES
from tools.jenkins.state import State
from tools.jenkins.validation import TrustedReceiptAuthority, closure_manifest
from tools.jenkins.workflow.execution import ExecutionEngine, execution_lock

# Use a drive-root approved path so the test does not depend on the physical
# project tree being present.  The approved drive roots are always accepted.
_APPROVED_ROOT = next(iter(sorted(APPROVED_NAMES)))


class ExecutionEngineTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name)
        self.state = State(self.repo / "state.db")
        self.state.put("resource_policy", "default",
                       {"cpuBudget": 2, "memoryBudget": 1024, "diskBudget": 2048})
        self.source, self.coverage, self.driver = "a" * 64, "b" * 64, "c" * 64
        self.prep = "d" * 64
        self.driver_binding = {"digest": self.driver, "generation": "1",
                               "runtimeOperationId": "runtime-1"}
        recipe = {"kind": "cargo", "buildRoot": _APPROVED_ROOT,
                  "preparationKey": self.prep,
                  "preparationRoot": str(canonical_build_root(_APPROVED_ROOT).namespace() / "preparations" /
                                         self.prep / "inputs" / "source"),
                  "resourceEstimates": {"cpu": 1, "memoryBytes": 256, "diskBytes": 64},
                  "phases": ["compile", "list", "test"], "requiredTestStages": ["unit_test"]}
        self.ref = digest(recipe)
        self.state.put("recipe_plan", self.ref, {"recipe": recipe, "recipeDigest": self.ref,
            "sourceRef": self.source, "coverageRef": self.coverage,
            "driver": self.driver_binding, "generation": "1"})
        self.state.put("sealed_input", self.source, {"status": "sealed"})
        self.resources = Mock()
        self.engine = ExecutionEngine(self.state, repo_root=self.repo, build_root=_APPROVED_ROOT,
                                      resource_manager=self.resources)
        for consumer in ("a", "b", "c"):
            self.state.put("workflow", consumer, {"sourceDigest": self.source,
                "coverageDigest": self.coverage, "driverDigest": self.driver,
                "generation": "1", "buildRoot": _APPROVED_ROOT})

    def tearDown(self):
        self.temp.cleanup()

    def claim(self, consumer="a"):
        return self.engine.claim(source=self.source, recipe=self.ref,
                                 driver=self.driver_binding, consumer_id=consumer)

    @contextmanager
    def in_flight(self, action, entered, resume):
        with ThreadPoolExecutor(max_workers=1) as worker:
            future = worker.submit(action)
            try:
                self.assertTrue(entered.wait(5), "The operation did not reach its checkpoint")
                yield future
            finally:
                resume.set()
                future.result(timeout=5)

    @contextmanager
    def publication_in_flight(self, eid):
        from tools.jenkins.artifacts import publish_execution_outputs
        from tools.jenkins.artifacts.store import ArtifactStore

        entered, resume = Event(), Event()

        def publish(*args, **kwargs):
            entered.set()
            self.assertTrue(resume.wait(5), "Publication was not resumed")
            return publish_execution_outputs(*args, **kwargs)

        with patch("tools.jenkins.artifacts.store._store_for_build_root",
                   return_value=ArtifactStore(self.repo / "artifacts")), \
                patch("tools.jenkins.artifacts.publish_execution_outputs", side_effect=publish):
            with self.in_flight(lambda: self.engine.observe(eid), entered, resume) as future:
                yield future

    def test_concurrent_same_input_shares_but_later_consumer_gets_fresh_test_attempt(self):
        first = self.claim()
        second = self.claim("b")
        self.assertEqual(first["executionId"], second["executionId"])
        self.assertEqual(second["refs"], 2)
        row = self.state.get("execution", first["executionId"])
        self.state.put("execution", row["key"], {**row["payload"], "status": "passed"},
                       expected_version=row["version"])
        later = self.claim("c")
        self.assertNotEqual(later["executionId"], first["executionId"])
        self.assertEqual(later["ordinal"], 2)
        self.assertEqual(self.claim()["executionId"], first["executionId"])

    def test_claim_during_publication_retries_then_gets_a_fresh_attempt(self):
        eid, _, _, _ = self.completed_host()
        with self.publication_in_flight(eid) as future:
            self.assertEqual("validated", self.state.get("execution", eid)["payload"]["status"])
            with self.assertRaises(JenkinsError) as raised:
                self.claim("b")
            self.assertEqual("execution_busy", raised.exception.code)
            self.assertTrue(raised.exception.retryable)
            self.assertIsNone(self.state.get("execution_consumer", f"{eid}:b"))
        self.assertEqual("passed", future.result()["status"])
        refs = self.state.list("artifact_ref")
        self.assertTrue(refs)
        self.assertEqual({"a"}, {row["payload"]["owner"] for row in refs})
        self.assertTrue(all(row["payload"]["status"] == "active" for row in refs))
        later = self.claim("b")
        self.assertNotEqual(eid, later["executionId"])
        self.assertEqual(2, later["ordinal"])
        self.assertEqual("claimed", later["status"])
        self.assertNotIn("stageReceipts", later)
        self.assertNotIn("artifacts", later)
        self.assertEqual(["a"], self.state.get("execution", eid)["payload"]["consumers"])

    def test_cancel_during_publication_retries_then_releases_its_artifact_refs(self):
        eid, _, _, _ = self.completed_host()
        workflow = self.state.get("workflow", "a")
        self.state.put("workflow", "a", {**workflow["payload"], "executionId": eid, "status": "ready"})
        with self.publication_in_flight(eid) as future:
            with self.assertRaises(JenkinsError) as raised:
                self.engine.cancel(eid, "a")
            self.assertEqual("execution_busy", raised.exception.code)
            self.assertTrue(raised.exception.retryable)
            self.assertEqual("active", self.state.get("execution_consumer", f"{eid}:a")["payload"]["status"])
            self.assertEqual("ready", self.state.get("workflow", "a")["payload"]["status"])
        self.assertEqual("passed", future.result()["status"])
        result = self.engine.cancel(eid, "a")
        self.assertEqual(0, result["refs"])
        self.assertEqual("passed", result["status"])
        self.assertEqual("cancelled", self.state.get("workflow", "a")["payload"]["status"])
        self.assertTrue(self.state.get("workflow", "a")["payload"]["consumerCancelled"])
        refs = self.state.list("artifact_ref")
        self.assertTrue(refs)
        self.assertTrue(all(row["payload"]["status"] == "released" for row in refs))

    def test_claim_cannot_join_the_last_consumer_cancellation_transaction(self):
        eid = self.claim()["executionId"]
        workflow = self.state.get("workflow", "a")
        self.state.put("workflow", "a", {**workflow["payload"], "executionId": eid, "status": "ready"})
        entered, resume = Event(), Event()
        put = self.state.put

        def pause_release(domain, key, payload, **kwargs):
            result = put(domain, key, payload, **kwargs)
            if domain == "execution_consumer" and key == f"{eid}:a" and payload["status"] == "released":
                entered.set()
                self.assertTrue(resume.wait(5), "Cancellation was not resumed")
            return result

        with patch.object(self.state, "put", side_effect=pause_release):
            with self.in_flight(lambda: self.engine.cancel(eid, "a"), entered, resume) as future:
                with self.assertRaises(JenkinsError) as raised:
                    self.claim("b")
                self.assertEqual("execution_busy", raised.exception.code)
                self.assertIsNone(self.state.get("execution_consumer", f"{eid}:b"))
        self.assertEqual("cancelled", future.result()["status"])
        self.assertEqual("cancelled", self.state.get("workflow", "a")["payload"]["status"])
        self.assertTrue(self.state.get("workflow", "a")["payload"]["consumerCancelled"])
        later = self.claim("b")
        self.assertNotEqual(eid, later["executionId"])
        self.assertEqual(2, later["ordinal"])

    def test_cancel_cannot_remove_a_consumer_during_another_claim_transaction(self):
        eid = self.claim()["executionId"]
        entered, resume = Event(), Event()
        put = self.state.put

        def pause_claim(domain, key, payload, **kwargs):
            result = put(domain, key, payload, **kwargs)
            if domain == "execution_consumer" and key == f"{eid}:b" and payload["status"] == "active":
                entered.set()
                self.assertTrue(resume.wait(5), "Claim was not resumed")
            return result

        with patch.object(self.state, "put", side_effect=pause_claim):
            with self.in_flight(lambda: self.claim("b"), entered, resume) as future:
                with self.assertRaises(JenkinsError) as raised:
                    self.engine.cancel(eid, "a")
                self.assertEqual("execution_busy", raised.exception.code)
        self.assertEqual(eid, future.result()["executionId"])
        result = self.engine.cancel(eid, "a")
        self.assertEqual("claimed", result["status"])
        self.assertEqual(["b"], result["consumers"])
        self.assertEqual(1, result["refs"])
        self.assertEqual("active", self.state.get("execution_consumer", f"{eid}:b")["payload"]["status"])

    def test_execution_lock_rejects_unknown_or_corrupt_identity_without_mutation(self):
        with self.assertRaises(JenkinsError) as raised:
            with execution_lock(self.state, "missing"):
                self.fail("An unknown execution must not be locked")
        self.assertEqual("execution_not_found", raised.exception.code)
        eid = self.claim()["executionId"]
        original = self.state.get("execution", eid)["payload"]
        for field, value in (("executionKey", {}), ("executionKeyDigest", "f" * 64),
                             ("ordinal", 2), ("sourceDigest", "e" * 64)):
            with self.subTest(field=field):
                self.state.put("execution", eid, {**original, field: value})
                before = self.state.get("execution", eid)
                for action in (lambda: self.claim("b"), lambda: self.engine.run(eid),
                               lambda: self.engine.observe(eid), lambda: self.engine.cancel(eid, "a")):
                    with self.assertRaises(JenkinsError) as raised:
                        action()
                    self.assertEqual("execution_identity_corrupt", raised.exception.code)
                    self.assertEqual(before, self.state.get("execution", eid))
                self.assertIsNone(self.state.get("execution_consumer", f"{eid}:b"))
        self.state.put("execution", eid, original)

    def test_source_and_consumer_coverage_cannot_change_plan(self):
        with self.assertRaisesRegex(JenkinsError, "same sealed source"):
            self.engine.claim(source="e"*64, recipe=self.ref, consumer_id="a")
        row = self.state.get("workflow", "a")
        self.state.put("workflow", "a", {**row["payload"], "coverageDigest": "f"*64})
        with self.assertRaisesRegex(JenkinsError, "coverageDigest"):
            self.claim()

    def test_waiting_admission_does_not_materialize_or_launch(self):
        value = self.claim()
        self.resources.scan_inventory.return_value = ({}, "fresh")
        self.resources.admit.return_value = None
        result = self.engine.run(value["executionId"])
        self.assertEqual(result["status"], "pending")
        self.assertIsNone(self.state.get("execution_launch", value["executionId"]))
        self.assertEqual(self.state.list("preparation_update"), [])
        self.engine.reconcile(value["executionId"])
        self.assertEqual(self.resources.scan_inventory.call_count, 2)

    def test_running_reconcile_does_not_repeat_admission_or_launch(self):
        value = self.claim()
        row = self.state.get("execution", value["executionId"])
        self.state.put("execution", row["key"], {**row["payload"], "status": "running"})
        self.state.put("execution_launch", row["key"], {"status": "requested"})
        self.engine.run(row["key"])
        self.resources.admit.assert_not_called()

    def test_cancel_releases_only_own_consumer(self):
        first = self.claim()
        self.claim("b")
        result = self.engine.cancel(first["executionId"], "a")
        self.assertEqual(result["refs"], 1)
        self.assertEqual(result["status"], "claimed")
        self.resources.release.assert_not_called()

    def test_cancel_last_pending_consumer_is_terminal_without_launch(self):
        value = self.claim()
        result = self.engine.cancel(value["executionId"], "a")
        self.assertEqual(result["status"], "cancelled")
        self.assertIsNone(self.state.get("execution_launch", value["executionId"]))
        self.assertEqual(self.engine.reconcile(value["executionId"])["status"], "cancelled")
        self.resources.admit.assert_not_called()

    def test_last_running_consumer_creates_control_for_existing_host(self):
        value = self.claim()
        row = self.state.get("execution", value["executionId"])
        self.state.put("execution", row["key"], {**row["payload"], "status": "running"})
        self.state.put("execution_host", row["key"], {"status": "running", "hostPid": 123})
        result = self.engine.cancel(row["key"], "a")
        self.assertEqual(result["status"], "cancel_requested")
        self.assertEqual(self.state.get("execution_control", row["key"])["payload"]["kind"], "cancel")

    def test_failed_host_exit_zero_cannot_pass_and_incomplete_proof_keeps_reservation(self):
        value = self.claim()
        eid = value["executionId"]
        proof = {"complete": True, "processExitCode": 0, "stdoutEof": True,
                 "stderrEof": True, "childrenGone": True}
        self.state.put("native_job", "native-1", {"status": "terminal", "owner": eid,
            "executionId": eid, "completeProof": proof})
        self.state.put("execution_host", eid, {"status": "failed",
            "phaseRecords": [{"phase": "compile", "nativeJobId": "native-1",
                              "terminalProof": proof}]})
        self.assertEqual(self.engine.observe(eid)["status"], "failed")
        other = self.claim("c")
        other_id = other["executionId"]
        self.state.put("execution_host", other_id, {"status": "failed",
            "phaseRecords": [{"nativeJobId": "missing", "terminalProof": proof}]})
        self.assertEqual(self.engine.observe(other_id)["status"], "blocked")
        self.resources.release.assert_not_called()

    def completed_host(self):
        value = self.claim()
        eid = value["executionId"]
        output_parent = canonical_build_root(_APPROVED_ROOT).namespace() / "tmp"
        output_parent.mkdir(parents=True, exist_ok=True)
        temporary = tempfile.TemporaryDirectory(prefix="stage-receipt-test-", dir=str(output_parent))
        self.addCleanup(temporary.cleanup)
        output_base = Path(temporary.name) / "receipts" / eid
        row = self.state.get("execution", eid)
        self.state.put("execution", eid, {**row["payload"], "status": "running",
                                         "reservationKey": "reservation", "phaseOutputRoot": str(output_base)})
        authority = TrustedReceiptAuthority(self.state)
        records, stage_receipts = [], {}
        for index, phase in enumerate(("compile", "list", "test")):
            native_id, birth, pid = f"native-{phase}", f"birth-{phase}", 11000 + index
            job = {"nativeJobId": native_id, "status": "running", "owner": eid,
                   "executionId": eid, "birthToken": birth, "pid": pid}
            self.state.put("native_job", native_id, job)
            stage = {"compile": "build", "list": "discovery", "test": "unit_test"}[phase]
            attempt_id = f"{eid}:compile" if phase == "compile" else f"{eid}:test:unit_test"
            output_root = output_base / phase
            output_root.mkdir(parents=True)
            for stream in ("stdout", "stderr"):
                (output_root / f"{stream}.log").write_text(f"{phase} {stream}\n", encoding="utf-8")
            if phase != "list":
                authority.register_attempt({"executionId": attempt_id, "stage": stage,
                    "sourceDigest": self.source, "recipeDigest": self.ref,
                    "coverageDigest": self.coverage, "driverDigest": self.driver,
                    "nativeJobId": native_id, "birthToken": birth, "phase": phase,
                    "outputRoot": str(output_root), "buildRoot": str(output_root),
                    "requiredOutputs": ["stdout.log", "stderr.log"]})
            proof = {"complete": True, "processExitCode": 0, "stdoutEof": True,
                     "stderrEof": True, "childrenGone": True}
            self.state.put("native_job", native_id, {**job, "status": "terminal", "completeProof": proof})
            record = {"phase": phase, "stage": stage, "nativeJobId": native_id, "terminalProof": proof}
            if phase != "list":
                authority.record_terminal(attempt_id, {**proof, "nativeJobId": native_id,
                                          "birthToken": birth}, closure_manifest(output_root))
                stage_receipts[stage] = attempt_id
            else:
                record["discovery"] = {"testCount": 1, "source": "cargo-test-list"}
            records.append(record)
        self.state.put("execution_host", eid, {"status": "terminal", "phaseRecords": records,
                                               "stageReceipts": stage_receipts})
        return eid, records, stage_receipts, output_base

    def test_phase_receipts_finish_the_parent_execution_without_changing_attempt_identity(self):
        eid, records, receipt_ids, _ = self.completed_host()
        attempts = {ref: self.state.get("validation_attempt", ref) for ref in receipt_ids.values()}
        with patch("tools.jenkins.artifacts.publish_execution_outputs", return_value={"status": "published"}) as publish, \
                patch("tools.jenkins.artifacts.attach_execution_outputs") as attach:
            result = self.engine.observe(eid)
        self.assertEqual("passed", result["status"])
        self.assertEqual(records[-1]["terminalProof"], result["terminalProof"])
        self.assertEqual(receipt_ids, result["stageReceipts"])
        self.assertEqual(attempts, {ref: self.state.get("validation_attempt", ref) for ref in receipt_ids.values()})
        publish.assert_called_once()
        attach.assert_called_once()
        self.resources.release.assert_called_once()

    def test_foreign_child_attempt_cannot_satisfy_the_parent_stage(self):
        eid, _, _, _ = self.completed_host()
        host = self.state.get("execution_host", eid)
        self.state.put("execution_host", eid, {**host["payload"],
            "stageReceipts": {**host["payload"]["stageReceipts"], "build": "other-execution:compile"}})
        with self.assertRaises(JenkinsError) as raised:
            self.engine.observe(eid)
        self.assertEqual("stage_receipt_identity_mismatch", raised.exception.code)
        self.resources.release.assert_not_called()

    def test_phase_attempt_must_bind_its_actual_native_job_and_process_birth(self):
        eid, _, receipt_ids, _ = self.completed_host()
        ref = receipt_ids["build"]
        original = self.state.get("validation_attempt", ref)["payload"]
        for field, wrong in (("nativeJobId", "native-test"), ("birthToken", "foreign-birth")):
            with self.subTest(field=field):
                self.state.put("validation_attempt", ref, {**original, field: wrong})
                with self.assertRaises(JenkinsError) as raised:
                    self.engine.observe(eid)
                self.assertEqual("stage_receipt_runner_mismatch", raised.exception.code)
        self.resources.release.assert_not_called()

    def test_receipt_must_bind_the_actual_native_job_birth_and_pid(self):
        eid, _, receipt_ids, _ = self.completed_host()
        ref = receipt_ids["build"]
        original = self.state.get("validation_attempt", ref)["payload"]
        for field, wrong in (("nativeJobId", "native-test"), ("birthToken", "foreign-birth"), ("pid", 99999)):
            with self.subTest(field=field):
                changed = deepcopy(original)
                changed["receipt"]["terminalProof"][field] = wrong
                self.state.put("validation_attempt", ref, changed)
                with self.assertRaises(JenkinsError) as raised:
                    self.engine.observe(eid)
                self.assertEqual("stage_receipt_runner_mismatch", raised.exception.code)
        self.resources.release.assert_not_called()

    def test_phase_receipt_preserves_source_coverage_recipe_and_driver_checks(self):
        eid, _, receipt_ids, _ = self.completed_host()
        ref = receipt_ids["build"]
        original = self.state.get("validation_attempt", ref)["payload"]
        for field in ("sourceDigest", "coverageDigest", "recipeDigest", "driverDigest", "executionId", "stage"):
            with self.subTest(field=field):
                changed = deepcopy(original)
                changed["receipt"][field] = "f" * 64 if field.endswith("Digest") else "foreign"
                self.state.put("validation_attempt", ref, changed)
                with self.assertRaises(JenkinsError) as raised:
                    self.engine.observe(eid)
                self.assertIn(raised.exception.code, {"receipt_identity_mismatch", "receipt_stage_invalid"})
        self.resources.release.assert_not_called()

    def test_changed_phase_output_cannot_be_accepted(self):
        eid, _, _, output_base = self.completed_host()
        (output_base / "compile/stdout.log").write_text("foreign later output\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as raised:
            self.engine.observe(eid)
        self.assertEqual("output_closure_changed", raised.exception.code)
        self.resources.release.assert_not_called()

    def test_phase_closure_cannot_claim_a_path_outside_the_physical_build_root(self):
        eid, _, receipt_ids, _ = self.completed_host()
        ref = receipt_ids["build"]
        attempt = self.state.get("validation_attempt", ref)
        changed = deepcopy(attempt["payload"])
        changed["receipt"]["outputClosure"]["root"] = r"C:\outside-build-root"
        self.state.put("validation_attempt", ref, changed)
        with self.assertRaises(JenkinsError) as raised:
            self.engine.observe(eid)
        self.assertEqual("namespace_escape", raised.exception.code)
        self.resources.release.assert_not_called()

    def test_cancelled_last_consumer_retains_real_termination_without_acceptance(self):
        eid, records, receipt_ids, _ = self.completed_host()
        attempts = {ref: self.state.get("validation_attempt", ref) for ref in receipt_ids.values()}
        self.engine.cancel(eid, "a")
        with patch("tools.jenkins.artifacts.publish_execution_outputs") as publish, \
                patch("tools.jenkins.artifacts.attach_execution_outputs") as attach:
            result = self.engine.observe(eid)
        self.assertEqual("cancelled", result["status"])
        self.assertEqual(0, result["refs"])
        self.assertEqual(records, result["phaseRecords"])
        self.assertEqual(records[-1]["terminalProof"], result["terminalProof"])
        self.assertEqual(attempts, {ref: self.state.get("validation_attempt", ref) for ref in receipt_ids.values()})
        publish.assert_not_called()
        attach.assert_not_called()
        self.resources.release.assert_called_once()

    def test_one_cancelled_consumer_does_not_discard_an_active_consumer_validation(self):
        eid, _, _, _ = self.completed_host()
        self.claim("b")
        self.engine.cancel(eid, "a")
        with patch("tools.jenkins.artifacts.publish_execution_outputs", return_value={"status": "published"}), \
                patch("tools.jenkins.artifacts.attach_execution_outputs") as attach:
            result = self.engine.observe(eid)
        self.assertEqual("passed", result["status"])
        self.assertEqual(["b"], result["consumers"])
        self.assertEqual("b", attach.call_args.args[-1])


if __name__ == "__main__":
    unittest.main()
