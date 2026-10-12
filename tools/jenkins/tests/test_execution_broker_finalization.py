from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import host as host_module
from tools.jenkins.deployment.host import LifecycleHost
from tools.jenkins.processes import NativeJob, ProcessRegistry
from tools.jenkins.state import State


class ExecutionBrokerFinalizationTests(unittest.TestCase):
    def setUp(self):
        test_root = Path.cwd() / ".jenkins/tmp/execution-broker-finalization-tests"
        test_root.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=test_root)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.state = State(self.root / "state/coordination.sqlite3")
        self.lifecycle = LifecycleHost.__new__(LifecycleHost)
        self.lifecycle.paths = SimpleNamespace(repo=self.root, state=self.root / "state")
        self.lifecycle.spec, self.lifecycle.java, self.lifecycle.war = object(), self.root / "java", self.root / "war"
        self.lifecycle.operation_id = "op"
        self.lifecycle.launch_state = self.state.path
        self.python = self.root / ".jenkins/runtime/python/python.exe"
        self.python.parent.mkdir(parents=True)
        self.python.touch()
        self.launcher = self.root / "launcher.py"
        self.job = NativeJob.launch((sys.executable, "-B", "-c", "print('guard completed')"),
            cwd=self.root, log_dir=self.root / "logs")
        self.addCleanup(self.job.close)
        self.registry = ProcessRegistry(self.state)
        self.guard = self.registry.register(self.job, execution_id="e", owner="e", role="execution_host_guard")
        self.job.wait(timeout_seconds=10)
        self.lifecycle._launches = {"e": (self.job, self.registry)}
        self.state.put("execution", "e", {"executionId": "e", "status": "running", "operationId": "op",
            "generation": "1", "driverDigest": "d", "driver": {"runtimeOperationId": "op", "generation": "1", "digest": "d"}})
        self.state.put("execution_launch", "e", {"status": "started", "executionId": "e", "operationId": "op",
            "generation": "1", "driverDigest": "d", "launcherIdentity": self.job.identity.to_dict(),
            "guardNativeJobId": self.guard.native_job_id, "nativeHostUUID": None})
        self.state.put("execution_host", "e", {"status": "terminal", "executionId": "e", "operationId": "op",
            "generation": "1", "hostPid": self.job.identity.pid, "hostBirthToken": self.job.identity.creation_time,
            "nativeJobId": "final-phase"})
        manager = patch.object(host_module, "DeploymentManager").start()
        self.addCleanup(patch.stopall)
        manager.return_value._driver_env.return_value = {"ZIRCON_DRIVER_LAUNCHER": str(self.launcher), "JENKINS_PYTHON": str(self.python)}
        patch.object(host_module, "active_driver_binding", return_value={
            "digest": "d", "generation": "op", "launcher": str(self.launcher)}).start()
        patch.dict(host_module.os.environ, {"ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY": "0"}).start()

    def test_native_identity_update_and_exit_share_the_new_cas_version(self):
        with patch.object(self.job, "close", wraps=self.job.close) as close:
            self.lifecycle._execution_broker_tick()
            close.assert_called_once_with()
        launch = self.state.get("execution_launch", "e")
        self.assertEqual(3, launch["version"])
        self.assertEqual("exited", launch["payload"]["status"])
        self.assertEqual(0, launch["payload"]["exitCode"])
        self.assertEqual("final-phase", launch["payload"]["nativeHostUUID"])
        self.assertEqual({"hostPid": self.job.identity.pid, "hostBirthToken": self.job.identity.creation_time},
                         launch["payload"]["nativeHostIdentity"])
        self.assertEqual({}, self.lifecycle._launches)
        self.assertIsNone(self.state.get("execution_abort", "e"))
        self.assertTrue(self.state.get("native_job", self.guard.native_job_id)["payload"]["completeProof"]["complete"])

    def test_cas_race_retains_actual_guard_for_a_successful_next_tick(self):
        record_terminal = self.registry.record_terminal
        def race_after_proof(job):
            terminal = record_terminal(job)
            current = self.state.get("execution_launch", "e")
            self.state.put("execution_launch", "e", {**current["payload"], "racingNote": "preserve"},
                           expected_version=current["version"])
            return terminal
        with patch.object(self.job, "close", wraps=self.job.close) as close, \
             patch.object(self.registry, "record_terminal", side_effect=race_after_proof):
            with self.assertRaises(JenkinsError) as caught:
                self.lifecycle._execution_broker_tick()
            self.assertEqual("state_conflict", caught.exception.code)
            close.assert_not_called()
        self.assertIs(self.job, self.lifecycle._launches["e"][0])
        self.assertTrue(self.job.proof().complete)
        self.assertEqual("started", self.state.get("execution_launch", "e")["payload"]["status"])
        self.assertIsNone(self.state.get("execution_abort", "e"))
        self.lifecycle._execution_broker_tick()
        launch = self.state.get("execution_launch", "e")["payload"]
        self.assertEqual("exited", launch["status"])
        self.assertEqual("preserve", launch["racingNote"])
        self.assertIn("nativeHostIdentity", launch)
        self.assertEqual({}, self.lifecycle._launches)

    def test_abort_failure_rolls_back_launch_finalization_and_retains_guard(self):
        self.state.delete("execution_host", "e")
        original_put = State.put
        def reject_abort(state, domain, key, payload, **kwargs):
            if domain == "execution_abort":
                raise JenkinsError("state_busy", "test abort persistence failure", retryable=True)
            return original_put(state, domain, key, payload, **kwargs)
        with patch.object(State, "put", new=reject_abort), patch.object(self.job, "close", wraps=self.job.close) as close:
            with self.assertRaises(JenkinsError) as caught:
                self.lifecycle._execution_broker_tick()
            self.assertEqual("state_busy", caught.exception.code)
            close.assert_not_called()
        self.assertEqual("started", self.state.get("execution_launch", "e")["payload"]["status"])
        self.assertIsNone(self.state.get("execution_abort", "e"))
        self.assertIs(self.job, self.lifecycle._launches["e"][0])
        self.lifecycle._execution_broker_tick()
        self.assertEqual("failed", self.state.get("execution_launch", "e")["payload"]["status"])
        self.assertFalse(self.state.get("execution_abort", "e")["payload"]["artifactPublicationAllowed"])
        self.assertEqual({}, self.lifecycle._launches)

    def test_wrong_host_tuple_preserves_owned_handle_and_denies_finalization(self):
        original = self.state.get("execution_host", "e")["payload"]
        for field, value in (("operationId", "other"), ("generation", 1), ("executionId", "other"),
                             ("hostPid", self.job.identity.pid + 1), ("hostBirthToken", "wrong")):
            with self.subTest(field=field):
                self.state.put("execution_host", "e", {**original, field: value})
                before = self.state.get("execution_launch", "e")
                with patch.object(self.job, "close", wraps=self.job.close) as close:
                    with self.assertRaises(JenkinsError) as caught:
                        self.lifecycle._execution_broker_tick()
                    self.assertEqual("host_binding_mismatch", caught.exception.code)
                    close.assert_not_called()
                self.assertEqual(before, self.state.get("execution_launch", "e"))
                self.assertIsNone(self.state.get("execution_abort", "e"))
                self.assertEqual([], self.state.list("artifact"))
                self.assertIs(self.job, self.lifecycle._launches["e"][0])

    def test_wrong_guard_tuple_cannot_finalize_or_publish_abort_evidence(self):
        current = self.state.get("execution_launch", "e")
        self.state.put("execution_launch", "e", {**current["payload"], "guardNativeJobId": "other"})
        with patch.object(self.job, "close", wraps=self.job.close) as close:
            with self.assertRaises(JenkinsError) as caught:
                self.lifecycle._execution_broker_tick()
            self.assertEqual("execution_guard_identity_mismatch", caught.exception.code)
            close.assert_not_called()
        self.assertEqual("started", self.state.get("execution_launch", "e")["payload"]["status"])
        self.assertIsNone(self.state.get("execution_abort", "e"))
        self.assertEqual([], self.state.list("artifact"))
        self.assertIs(self.job, self.lifecycle._launches["e"][0])


if __name__ == "__main__":
    unittest.main()
