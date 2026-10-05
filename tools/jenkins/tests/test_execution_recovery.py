from __future__ import annotations

import copy
import json
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import execution_recovery as recovery
from tools.jenkins.deployment.execution_activation import assert_activation_start_allowed, protected_holds
from tools.jenkins.tests import test_execution_activation as activation_tests


class RecoveryTests(unittest.TestCase):
    def setUp(self):
        activation_tests.ActivationTests.setUp(self)
        self.state = self.manager.paths.state
        self.previous["executable"] = str(self.repo / ".jenkins/runtime/python/python.exe")
        self.previous["commandDigest"] = "a" * 64
        self.host["controllerIdentity"]["executable"] = str(self.repo / ".jenkins/runtime/jdk/bin/java.exe")
        self.host["controllerIdentity"]["commandDigest"] = "b" * 64
        self.manager._write(self.previous)
        self.save("host.json", self.host)
        self.plan = self.activation.prepare(candidate_driver=self.candidate, regression_ref=self.reference)
        self.transition = self.plan.transition_digest
        with patch.object(self.manager, "_start_locked", side_effect=RuntimeError("interrupted start")):
            with self.assertRaises(RuntimeError):
                self.activation.activate(self.plan)
        self.selected = self.read("driver.json")
        self.journal_path = self.state / "deployment/execution-activation.json"
        mocked = patch("tools.jenkins.deployment.execution_recovery.observe_departed_identity",
                       side_effect=lambda value: {"pid": value["pid"], "expectedCreationTime": value["creationTime"],
                                                  "status": "absent"})
        mocked.start(); self.addCleanup(mocked.stop)
        mocked = patch("tools.jenkins.deployment.execution_recovery.check_port_available")
        mocked.start(); self.addCleanup(mocked.stop)

    def save(self, name, value):
        (self.state / "deployment" / name).write_text(json.dumps(value))

    def read(self, name):
        return json.loads((self.state / "deployment" / name).read_text())

    def recover(self):
        return recovery.recover_execution_activation(self.manager, self.transition)

    def runtime(self, *, terminal=False):
        operation = {**self.previous, "operationId": "candidate-op", "generation": "candidate-op",
                     "state": "failed" if terminal else "running", "controlPlaneOnly": False,
                     "hostPid": 300, "pid": 300, "creationTime": "3000", "commandDigest": "c" * 64}
        controller = {"pid": 400, "creationTime": "4000",
                      "executable": str(self.repo / ".jenkins/runtime/jdk/bin/java.exe"), "commandDigest": "d" * 64}
        agent = {**controller, "pid": 500, "creationTime": "5000", "commandDigest": "e" * 64}
        host = {"operationId": operation["operationId"], "generation": operation["generation"], "hostPid": 300,
                "status": "failed" if terminal else "running", "controlPlaneOnly": False,
                "agentLaunchAttempted": True, "controllerIdentity": controller, "agentIdentity": agent}
        if terminal:
            for field, value in (("nativeTerminationProof", controller), ("agentTerminationProof", agent)):
                host[field] = {"identity": value, "complete": True, "childrenGone": True, "activeProcesses": 0,
                               "stdoutEof": True, "stderrEof": True, "processExitCode": 1}
        self.manager._write(operation)
        self.save("host.json", host)
        self.save("driver.json", {**self.selected, "runtimeOperationId": operation["operationId"]})
        return operation, host

    def live_mocks(self):
        stack = __import__("contextlib").ExitStack()
        stack.enter_context(patch("tools.jenkins.deployment.live_observations.observe_live_identity",
                                 side_effect=lambda value: {k: value[k] for k in ("pid", "creationTime", "executable", "commandDigest")}))
        stack.enter_context(patch("tools.jenkins.deployment.live_observations.observe_endpoint_owner",
                                 return_value={"address": "127.0.0.1", "port": 1, "pid": 400, "ownerCount": 1}))
        stack.enter_context(patch.object(self.manager, "health",
                                return_value={"controller": True, "plugins": True, "agent": True, "ready": True}))
        return stack

    def test_unknown_before_spawn_rolls_back_and_preserves_holds_and_operation(self):
        holds = protected_holds(self.state)
        result = self.recover()
        self.assertEqual("rolled_back", result["status"])
        self.assertFalse(result["runtimeAcceptance"])
        self.assertEqual(self.old, self.read("driver.json"))
        self.assertEqual(self.previous, self.manager._read())
        self.assertEqual(holds, protected_holds(self.state))
        self.assertEqual(result["recoveryReceipt"], assert_activation_start_allowed(self.manager)["recoveryReceipt"])

    def test_pre_selector_failure_rolls_back_idempotently(self):
        self.save("driver.json", self.old)
        first = self.recover()
        second = self.recover()
        self.assertEqual(first, second)
        self.assertEqual(1, len(list((self.state / "deployment/history").glob("execution-recovery-*.json"))))

    def test_foreign_selector_and_changed_holds_cannot_mutate_transition(self):
        journal = self.journal_path.read_bytes()
        self.save("driver.json", {**self.selected, "foreign": True})
        self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(journal, self.journal_path.read_bytes())
        self.save("driver.json", self.selected)
        self.db.put("resource_reservation", "legacy", {"status": "active", "owner": "foreign"})
        self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(self.selected, self.read("driver.json"))
        self.assertEqual(journal, self.journal_path.read_bytes())

    def test_archive_and_candidate_mutations_are_rejected(self):
        journal = self.read("execution-activation.json")
        journal["candidate"]["driverDigest"] = "0" * 64
        self.save("execution-activation.json", journal)
        with self.assertRaises(JenkinsError):
            self.recover()

    def test_spawned_terminal_candidate_rolls_back_without_rewriting_failed_operation(self):
        failed, host = self.runtime(terminal=True)
        result = self.recover()
        self.assertEqual("rolled_back", result["status"])
        self.assertEqual(failed, self.manager._read())
        self.assertEqual(host, self.read("host.json"))
        receipt = json.loads(Path(result["recoveryReceipt"]["path"]).read_text())
        self.assertEqual(failed, receipt["operation"])
        self.assertEqual(host, receipt["host"])
        self.assertEqual("absent", receipt["hostObservation"]["status"])

    def test_native_proofs_with_live_host_leave_selector_and_journal_unchanged(self):
        self.runtime(terminal=True)
        selector = self.read("driver.json"); journal = self.journal_path.read_bytes()
        with patch("tools.jenkins.deployment.execution_recovery.observe_departed_identity",
                   side_effect=JenkinsError("deployment_owner_live", "host alive", retryable=True)):
            result = self.recover()
        self.assertEqual("waiting", result["status"])
        self.assertEqual(selector, self.read("driver.json"))
        self.assertEqual(journal, self.journal_path.read_bytes())

    def test_incomplete_agent_native_proof_blocks_terminal_rollback(self):
        _, host = self.runtime(terminal=True)
        host["agentTerminationProof"]["stdoutEof"] = False
        self.save("host.json", host)
        selector = self.read("driver.json")
        self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(selector, self.read("driver.json"))

    def test_rollback_crash_after_selector_swap_resumes_same_intent(self):
        write = recovery._write_json_atomic

        def crash(path, value):
            if Path(path) == self.journal_path and value.get("status") == "rolled_back":
                raise OSError("crash after selector swap")
            return write(path, value)

        with patch.object(recovery, "_write_json_atomic", side_effect=crash):
            self.assertEqual("waiting", self.recover()["status"])
        intent = self.read("execution-activation.json")
        self.assertEqual("rollback_pending", intent["status"])
        self.assertEqual(self.old, self.read("driver.json"))
        result = self.recover()
        self.assertEqual("rolled_back", result["status"])
        self.assertEqual(intent["rollbackIntent"], result["recoveryReceipt"])
        self.assertEqual(self.previous, self.manager._read())

    def test_owner_change_after_intent_blocks_reentry(self):
        write = recovery._write_json_atomic

        def crash(path, value):
            if Path(path) == self.journal_path and value.get("status") == "rolled_back":
                raise OSError("interrupted")
            return write(path, value)

        with patch.object(recovery, "_write_json_atomic", side_effect=crash):
            self.recover()
        self.manager._write({**self.previous, "creationTime": "foreign"})
        journal = self.journal_path.read_bytes()
        self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(journal, self.journal_path.read_bytes())

    def test_live_candidate_reconciliation_uses_identity_and_listener_evidence(self):
        operation, host = self.runtime()
        with self.live_mocks():
            result = self.recover()
            again = self.recover()
        self.assertEqual("succeeded", result["status"])
        self.assertEqual("resolved", again["status"])
        self.assertFalse(result["runtimeAcceptance"])
        receipt = json.loads(Path(result["reconciliationReceipt"]["path"]).read_text())
        self.assertEqual(operation, receipt["operation"])
        self.assertEqual(host, receipt["host"])
        self.assertEqual(3, len(receipt["observed"]["identities"]))
        self.assertEqual(400, receipt["observed"]["endpointOwner"]["pid"])
        self.assertEqual(1, len(list((self.state / "deployment/history").glob("execution-recovery-*.json"))))

    def test_health_cannot_substitute_for_actual_identity_or_port_owner(self):
        self.runtime()
        journal = self.journal_path.read_bytes()
        with self.live_mocks(), patch("tools.jenkins.deployment.live_observations.observe_live_identity",
                                      side_effect=JenkinsError("live_observation_failed", "PID reused")):
            self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(journal, self.journal_path.read_bytes())
        with self.live_mocks(), patch("tools.jenkins.deployment.live_observations.observe_endpoint_owner",
                                      side_effect=JenkinsError("live_observation_failed", "foreign controller")):
            self.assertEqual("waiting", self.recover()["status"])
        self.assertEqual(journal, self.journal_path.read_bytes())

    def test_live_manifest_corruption_cannot_reconcile(self):
        self.runtime()
        path = Path(self.candidate["root"]) / "tools/jenkins/__main__.py"
        path.write_text("print('corrupt')\n")
        with self.live_mocks(), self.assertRaises(JenkinsError):
            self.recover()
        self.assertEqual("unknown", self.read("execution-activation.json")["status"])

    def test_rollback_guard_rejects_changed_receipt_selector_or_hold(self):
        result = self.recover()
        self.save("driver.json", self.selected)
        with self.assertRaises(JenkinsError):
            assert_activation_start_allowed(self.manager)
        self.save("driver.json", self.old)
        path = Path(result["recoveryReceipt"]["path"]); path.write_text("{}")
        with self.assertRaises(JenkinsError):
            assert_activation_start_allowed(self.manager)

    def test_controller_only_new_generation_must_retain_verified_receipt_chain(self):
        result = self.recover()
        op = "recovered-op"
        new = {**self.previous, "operationId": op, "generation": op, "state": "running", "pid": 600, "hostPid": 600,
               "creationTime": "6000", "executionActivationRecoveryReceipt": result["recoveryReceipt"]}
        host = {**self.host, "operationId": op, "generation": op, "hostPid": 600, "status": "running",
                "controlPlaneOnly": True}
        self.manager._write(new)
        self.save("host.json", host)
        self.save("driver.json", {**self.old, "runtimeOperationId": op, "state": "selected-for-runtime-start"})
        self.assertTrue(assert_activation_start_allowed(self.manager)["controlPlaneOnly"])
        new.pop("executionActivationRecoveryReceipt")
        self.manager._write(new)
        with self.assertRaises(JenkinsError):
            assert_activation_start_allowed(self.manager)


if __name__ == "__main__":
    unittest.main()
