from __future__ import annotations

from contextlib import ExitStack
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment import manager as module
from tools.jenkins.deployment.start_transaction import RecoveryStartTransaction


class Capability:
    def __init__(self, manager):
        self.actor = SimpleNamespace(keeper={"pid": 77}, release_ref={"path": "release", "sha256": "a" * 64})
        self.request = {"path": "request", "sha256": "b" * 64}
        self.consume = Mock()
        self.target_driver = Mock(return_value={"driverDigest": "c" * 64, "root": "candidate"})
        self.creation_flags = Mock(return_value=0)
        self.record_spawn = Mock(side_effect=JenkinsError("child_job_unproven", "actual containment failed"))


class RecoveryStartTransactionTests(unittest.TestCase):
    def fixture(self, root):
        state = root / "state"
        (state / "deployment").mkdir(parents=True)
        (state / "deployment/driver.json").write_text(json.dumps({"driverDigest": "a" * 64, "root": "previous"}))
        (state / "deployment/host.json").write_text("{}")
        spec = SimpleNamespace(path=root / "spec.json", controller={"listenAddress": "127.0.0.1", "httpPort": 1})
        paths = SimpleNamespace(state=state, logs=root / "logs", repo=root, home=root / "home")
        return module.DeploymentManager(spec, paths, root / "java", root / "war")

    def exercise(self, *, spawn_error=None, identity_error=None):
        with tempfile.TemporaryDirectory() as tmp:
            manager = self.fixture(Path(tmp))
            cap = Capability(manager)
            child = SimpleNamespace(pid=812)
            actual = {"pid": 812, "creationTime": "1234", "executable": "python", "commandDigest": "e" * 64}
            identity = Mock(return_value=SimpleNamespace(to_dict=lambda: actual), side_effect=identity_error)
            with ExitStack() as stack:
                stack.enter_context(patch("tools.jenkins.deployment.recovery_keeper.RecoveryKeeperStart", Capability))
                stack.enter_context(patch.object(module, "check_port_available"))
                stack.enter_context(patch.object(manager, "_driver_env", return_value={"ZIRCON_DRIVER_LAUNCHER": "launcher",
                    "ZIRCON_DRIVER_DIGEST": "c" * 64, "JENKINS_PYTHON": "python"}))
                stack.enter_context(patch.object(module.subprocess, "Popen", return_value=child, side_effect=spawn_error))
                stack.enter_context(patch.object(module, "current_identity", identity))
                with self.assertRaises(JenkinsError):
                    manager._start_locked(manager.paths.state / "deployment-start.lock", recovery_keeper_start=cap)
            paths = list((manager.paths.state / "recovery-lifecycle/start-transactions").glob("*.json"))
            self.assertEqual(1, len(paths))
            journal = json.loads(paths[0].read_text())
            self.assertEqual("unknown", journal["state"])
            self.assertEqual("a" * 64, journal["previousSelector"]["driverDigest"])
            self.assertTrue(journal["spawnAttempted"])
            for field in ("automaticRetry", "runtimeAcceptance", "cargoAcceptance", "nativeTerminationProven", "historicalHoldsReleased"):
                self.assertIs(journal[field], False)
            if spawn_error is not None:
                self.assertNotIn("createdPid", journal)
                self.assertIsNone(manager._read())
                self.assertNotIn(812, module._LIVE_JOBS)
            else:
                self.assertEqual(812, journal["createdPid"])
                self.assertIs(module._LIVE_JOBS[812][0], child)
                if identity_error is None:
                    self.assertEqual(actual, journal["createdIdentity"])
                    self.assertEqual(812, manager._read()["pid"])
                    self.assertEqual("starting", manager._read()["state"])
                else:
                    self.assertNotIn("createdIdentity", journal)
                    cap.record_spawn.assert_not_called()
                module._LIVE_JOBS.pop(812)[1].close()

    def test_spawn_refusal_preserves_original_intent_without_claiming_a_child(self):
        self.exercise(spawn_error=OSError("CreateProcess refused"))

    def test_job_validation_failure_keeps_child_and_birth_in_durable_journal(self):
        self.exercise()

    def test_identity_query_failure_retains_created_pid_without_invented_birth(self):
        self.exercise(identity_error=JenkinsError("process_identity_inconclusive", "query denied"))

    def test_existing_start_transaction_cannot_be_overwritten_or_replayed(self):
        with tempfile.TemporaryDirectory() as tmp:
            manager = self.fixture(Path(tmp))
            first = RecoveryStartTransaction(manager, Capability(manager), "start-1")
            original = first.path.read_bytes()
            with self.assertRaises(JenkinsError):
                RecoveryStartTransaction(manager, Capability(manager), "start-1")
            self.assertEqual(original, first.path.read_bytes())
