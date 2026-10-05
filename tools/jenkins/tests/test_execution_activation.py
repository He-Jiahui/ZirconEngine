from __future__ import annotations
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.driver import seal_driver
from tools.jenkins.deployment.execution_activation import (ExecutionActivation, assert_activation_start_allowed,
                                                           protected_holds, support_sources)
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.state import State


class ActivationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        home = self.repo / ".jenkins/jenkins_home"
        home.mkdir(parents=True)
        state = self.repo / ".jenkins/state"
        (state / "deployment").mkdir(parents=True)
        (state / "migration-baseline").mkdir()
        self.spec_path = self.repo / ".jenkins/deployment-spec.json"
        self.spec_path.write_text("{}")
        (self.repo / ".jenkins/workflow-spec.json").write_text("{}")
        (self.repo / "tools/jenkins").mkdir(parents=True)
        (self.repo / "tools/jenkins/__main__.py").write_text("print('fixture runtime')\n")
        (self.repo / ".jenkins/pipeline").mkdir()
        (self.repo / ".jenkins/pipeline/steps.groovy").write_text("return this\n")
        self.old = seal_driver(self.repo)
        (self.repo / "tools/jenkins/__main__.py").write_text("print('new fixture runtime')\n")
        self.candidate = seal_driver(self.repo)
        self.old.update(runtimeOperationId="old", state="selected-for-runtime-start")
        (state / "deployment/driver.json").write_text(json.dumps(self.old))
        self.db = State(state / "coordination.sqlite3")
        self.db.put("resource_reservation", "legacy", {"status": "active", "owner": "legacy"})
        self.db.put("pool_writer_hold", "legacy", {"status": "active", "owner": "legacy"})
        self.previous = {"operationId": "old", "generation": "old", "homePath": str(home),
                         "hostPid": 100, "pid": 100, "creationTime": "1000", "executable": "python.exe",
                         "state": "stopped", "controlPlaneOnly": True}
        identity = {"pid": 200, "creationTime": "1100", "executable": "java.exe"}
        proof = {"identity": identity, "complete": True, "childrenGone": True,
                 "activeProcesses": 0, "stdoutEof": True, "stderrEof": True, "processExitCode": 0}
        self.host = {"operationId": "old", "generation": "old", "hostPid": 100, "status": "stopped",
                     "controllerIdentity": identity, "nativeTerminationProof": proof, "agentLaunchAttempted": False}
        (state / "deployment/host.json").write_text(json.dumps(self.host))
        spec = SimpleNamespace(path=self.spec_path, controller={"listenAddress": "127.0.0.1", "httpPort": 1})
        paths = SimpleNamespace(repo=self.repo, home=home, state=state)
        self.manager = DeploymentManager(spec, paths, self.repo / "java", self.repo / "war")
        self.manager._write(self.previous)
        self.activation = ExecutionActivation(self.manager)
        log = state / "migration-baseline/regression.log"
        log.write_text("fixture regression passed\n")
        self.receipt = state / "migration-baseline/regression.json"
        self.receipt.write_text(json.dumps({"scope": "complete-current-Jenkins-support-pytest-discovery",
            "status": "passed", "sourceIdentityStable": True, "testsRun": 1, "failures": 0, "errors": 0,
            "exitCode": 0, "collectionErrors": [], "sourceFiles": support_sources(self.repo),
            "log": str(log), "logSha256": hashlib.sha256(log.read_bytes()).hexdigest()}))
        self.reference = {"path": str(self.receipt), "sha256": hashlib.sha256(self.receipt.read_bytes()).hexdigest()}
        self.observation = {"pid": 100, "expectedCreationTime": "1000", "status": "absent"}
        mocked = patch("tools.jenkins.deployment.execution_activation.observe_departed_identity", return_value=self.observation)
        mocked.start()
        self.addCleanup(mocked.stop)

    def prepare(self):
        return self.activation.prepare(candidate_driver=self.candidate, regression_ref=self.reference)

    def test_complete_transition_selects_candidate_and_preserves_holds(self):
        before = protected_holds(self.manager.paths.state)
        plan = self.prepare()
        def start(lock, **kwargs):
            kwargs["execution_activation"].validate(self.manager, candidate_selected=True)
            self.assertEqual(self.old, kwargs["recovery_driver"])
            return {"state": "running", "controlPlaneOnly": False, "operationId": "new-op"}
        with patch.object(self.manager, "_start_locked", side_effect=start) as start_mock:
            result = self.activation.activate(plan)
        self.assertEqual("running", result["state"])
        start_mock.assert_called_once()
        self.assertEqual(before, protected_holds(self.manager.paths.state))
        self.assertEqual(self.candidate["driverDigest"], json.loads(
            (self.manager.paths.state / "deployment/driver.json").read_text())["driverDigest"])

    def test_live_generation_and_missing_supervisor_proof_are_rejected(self):
        self.manager._write({**self.previous, "state": "running"})
        with self.assertRaises(JenkinsError):
            self.prepare()
        self.manager._write(self.previous)
        self.observation["status"] = "inconclusive"
        with self.assertRaises(JenkinsError):
            self.prepare()

    def test_source_regression_and_native_proof_mutations_are_rejected(self):
        plan = self.prepare()
        (self.repo / "tools/jenkins/__main__.py").write_text("print('foreign edit')\n")
        with self.assertRaises(JenkinsError) as error:
            self.activation.activate(plan)
        self.assertEqual("execution_transition_source_changed", error.exception.code)

    def test_changed_holds_and_durable_journal_are_rejected_before_selection(self):
        plan = self.prepare()
        self.db.put("resource_reservation", "legacy", {"status": "released"})
        with self.assertRaises(JenkinsError) as error:
            self.activation.activate(plan)
        self.assertEqual("execution_transition_holds", error.exception.code)
        self.db.put("resource_reservation", "legacy", {"status": "active", "owner": "legacy"})
        plan = self.prepare()
        self.manager._write({**self.previous, "creationTime": "999"})
        with self.assertRaises(JenkinsError) as error:
            self.activation.activate(plan)
        self.assertEqual("execution_transition_changed", error.exception.code)

    def test_manifest_and_receipt_hash_mutations_are_rejected(self):
        self.reference["sha256"] = "0" * 64
        with self.assertRaises(JenkinsError):
            self.prepare()
        self.reference["sha256"] = hashlib.sha256(self.receipt.read_bytes()).hexdigest()
        (Path(self.candidate["root"]) / "tools/jenkins/__main__.py").write_text("print('tampered')\n")
        with self.assertRaises(JenkinsError):
            self.prepare()

    def test_empty_protected_set_is_valid_and_bare_mode_cannot_bypass_plan(self):
        self.db.put("resource_reservation", "legacy", {"status": "released"})
        self.db.put("pool_writer_hold", "legacy", {"status": "released"})
        self.prepare()
        with self.assertRaises(JenkinsError):
            self.manager._start_locked(self.manager.paths.state / "deployment-start.lock",
                                       execution_activation=SimpleNamespace(previous_operation="old"))

    def test_missing_coordination_database_is_not_an_empty_hold_set(self):
        self.db.path.unlink()
        with self.assertRaises(JenkinsError) as error:
            protected_holds(self.manager.paths.state)
        self.assertEqual("execution_transition_holds", error.exception.code)

    def test_selected_candidate_metadata_is_bound_to_transition(self):
        plan = self.prepare()
        selector = self.manager.paths.state / "deployment/driver.json"
        selected = {**json.loads(selector.read_text()), "runtimeAcceptance": True}
        selector.write_text(json.dumps(selected))
        with self.assertRaises(JenkinsError) as error:
            plan.validate(self.manager, candidate_selected=True)
        self.assertEqual("execution_transition_selection", error.exception.code)

    def test_unresolved_activation_journal_blocks_normal_start(self):
        journal = self.manager.paths.state / "deployment/execution-activation.json"
        journal.write_text(json.dumps({"status": "unknown"}))
        with self.assertRaises(JenkinsError) as error:
            assert_activation_start_allowed(self.manager)
        self.assertEqual("execution_transition_unresolved", error.exception.code)
        journal.write_text(json.dumps({"status": "succeeded", "transitionDigest": "0" * 64}))
        with self.assertRaises(JenkinsError) as error:
            assert_activation_start_allowed(self.manager)
        self.assertEqual("execution_transition_unresolved", error.exception.code)

    def test_corrupt_or_incomplete_success_journal_is_blocked(self):
        journal = self.manager.paths.state / "deployment/execution-activation.json"
        journal.write_text("not-json")
        with self.assertRaises(JenkinsError) as error:
            assert_activation_start_allowed(self.manager)
        self.assertEqual("execution_transition_unresolved", error.exception.code)

    def test_selector_swap_failure_leaves_unknown_journal(self):
        plan = self.prepare()
        original_replace = Path.replace
        def replace(source, target):
            if source.name.startswith("driver.activation-"):
                raise OSError("selector exchange failed")
            return original_replace(source, target)
        with patch.object(Path, "replace", new=replace):
            with self.assertRaises(OSError):
                self.activation.activate(plan)
        journal = json.loads((self.manager.paths.state / "deployment/execution-activation.json").read_text())
        self.assertEqual("unknown", journal["status"])
        self.assertEqual("selector-swap", journal["phase"])

    def test_non_running_start_result_is_failed_journal(self):
        plan = self.prepare()
        with patch.object(self.manager, "_start_locked", return_value={"state": "failed", "operationId": "failed-op"}):
            result = self.activation.activate(plan)
        self.assertEqual("failed", result["state"])
        journal = json.loads((self.manager.paths.state / "deployment/execution-activation.json").read_text())
        self.assertEqual("failed", journal["status"])


if __name__ == "__main__":
    unittest.main()
