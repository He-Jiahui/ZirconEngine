"""Execution admission, reference sharing and terminal authority regressions."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock
from tools.jenkins.contracts import JenkinsError, digest
from tools.jenkins.resources import canonical_build_root
from tools.jenkins.resources.paths import repository_build_root, APPROVED_NAMES
from tools.jenkins.state import State
from tools.jenkins.workflow.execution import ExecutionEngine

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


if __name__ == "__main__":
    unittest.main()
