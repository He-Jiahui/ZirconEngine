from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.manager import DeploymentManager


class RecoveryEntryTests(unittest.TestCase):
    def test_recovery_digest_is_mandatory_and_not_a_mode_switch(self):
        spec = SimpleNamespace(controller={"listenAddress": "127.0.0.1", "httpPort": 1})
        manager = DeploymentManager(spec, SimpleNamespace(state=Path("unused")), Path("java"), Path("war"))
        with patch("tools.jenkins.deployment.execution_recovery.recover_execution_activation") as recover:
            for value in (None, True, "", "0" * 63, "G" * 64):
                with self.subTest(value=value), self.assertRaises(JenkinsError):
                    manager.recover_activation(transition_digest=value)
            recover.assert_not_called()
            manager.recover_activation(transition_digest="a" * 64)
            recover.assert_called_once_with(manager, transition_digest="a" * 64)

    def test_protocol_entry_forwards_only_saved_digest(self):
        from tools.jenkins.deployment import handle
        spec = SimpleNamespace(controller={"java": {"executable": "java"}, "warPath": "war"})
        with patch("tools.jenkins.deployment.load_spec", return_value=spec), \
             patch("tools.jenkins.deployment.resolve_paths", return_value=object()), \
             patch("tools.jenkins.deployment.manager.DeploymentManager") as manager:
            manager.return_value.recover_activation.return_value = {"status": "waiting"}
            result = handle("recover-activation", {"transitionDigest": "a" * 64, "controlPlaneOnly": False},
                            None, Path.cwd())
        self.assertEqual("waiting", result["status"])
        manager.return_value.recover_activation.assert_called_once_with(transition_digest="a" * 64)

    def test_executor_returns_nonzero_for_unresolved_recovery(self):
        from tools.jenkins.deployment.__main__ import main
        spec = SimpleNamespace(controller={"java": {"executable": "java"}, "warPath": "war"})
        with patch.object(sys, "argv", ["deployment", "--spec", "fixture.json", "recover-activation",
                                       "--transition-digest", "a" * 64]), \
             patch("tools.jenkins.deployment.__main__.load_spec", return_value=spec), \
             patch("tools.jenkins.deployment.__main__.resolve_paths", return_value=object()), \
             patch("tools.jenkins.deployment.__main__.DeploymentManager") as manager, \
             contextlib.redirect_stdout(io.StringIO()) as output:
            manager.return_value.recover_activation.return_value = {"status": "waiting", "runtimeAcceptance": False}
            self.assertEqual(2, main())
        self.assertEqual("waiting", json.loads(output.getvalue())["status"])
        manager.return_value.recover_activation.assert_called_once_with(transition_digest="a" * 64)

    def test_incomplete_start_directive_fails_before_lifecycle_read(self):
        spec = SimpleNamespace(controller={"listenAddress": "127.0.0.1", "httpPort": 1})
        manager = DeploymentManager(spec, SimpleNamespace(state=Path("unused")), Path("java"), Path("war"))
        with patch("tools.jenkins.deployment.execution_activation.assert_activation_start_allowed",
                   return_value={"controlPlaneOnly": True}), \
             patch.object(manager, "_read", side_effect=AssertionError("No lifecycle read")), \
             patch("tools.jenkins.deployment.manager.subprocess.Popen") as spawn:
            with self.assertRaises(JenkinsError):
                manager._start_locked(Path("unused.lock"))
        spawn.assert_not_called()

    def test_recovered_start_creates_new_controller_only_generation_and_preserves_failed_history(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; deployment = state / "deployment"
            deployment.mkdir(parents=True)
            paths = SimpleNamespace(state=state, home=root / "home", logs=root / "logs", repo=root)
            spec = SimpleNamespace(path=root / "spec.json", controller={"listenAddress": "127.0.0.1", "httpPort": 1})
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            old = {"operationId": "failed-op", "generation": "failed-op", "state": "failed",
                   "pid": 101, "hostPid": 101, "creationTime": "1000", "executable": str(root / "python"),
                   "homePath": str(paths.home.absolute()), "controlPlaneOnly": False}
            controller = {"pid": 102, "creationTime": "1100", "executable": str(root / "java"), "commandDigest": "c" * 64}
            native = {"identity": controller, "complete": True, "childrenGone": True, "activeProcesses": 0,
                      "stdoutEof": True, "stderrEof": True, "processExitCode": 1}
            host = {"operationId": "failed-op", "generation": "failed-op", "hostPid": 101, "status": "failed",
                    "controllerIdentity": controller, "nativeTerminationProof": native, "agentLaunchAttempted": False}
            manager._write(old)
            (deployment / "host.json").write_text(json.dumps(host))
            (deployment / "driver.json").write_text(json.dumps({"driverDigest": "d"}))
            receipt = {"path": str(deployment / "history/recovery.json"), "sha256": "a" * 64}
            captured = {}

            def spawn(args, **kwargs):
                self.assertEqual(old, manager._read())
                captured.update(kwargs); captured["args"] = args
                return SimpleNamespace(pid=707, poll=lambda: None)

            def health_wait(*args, **kwargs):
                command = captured["args"]; op = command[command.index("--operation-id") + 1]
                (deployment / "host.json").write_text(json.dumps(
                    {"operationId": op, "generation": op, "hostPid": 707, "status": "running", "controlPlaneOnly": True}))
                return True

            observation = {"pid": 101, "expectedCreationTime": "1000", "status": "absent"}
            with patch("tools.jenkins.deployment.execution_activation.assert_activation_start_allowed",
                       return_value={"controlPlaneOnly": True, "recoveryReceipt": receipt}), \
                 patch("tools.jenkins.deployment.startup_observations.observe_departed_identity", return_value=observation), \
                 patch("tools.jenkins.deployment.manager.check_port_available"), \
                 patch.object(manager, "_driver_env", return_value={"ZIRCON_DRIVER_LAUNCHER": str(root / "launcher.py"),
                          "ZIRCON_DRIVER_DIGEST": "d", "JENKINS_PYTHON": sys.executable}), \
                 patch("tools.jenkins.deployment.manager.subprocess.Popen", side_effect=spawn), \
                 patch("tools.jenkins.deployment.manager.current_identity", return_value=SimpleNamespace(to_dict=lambda:
                       {"pid": 707, "creationTime": "7070", "executable": str(root / "python"), "commandDigest": "b" * 64})), \
                 patch("tools.jenkins.deployment.manager.wait_for_health", side_effect=health_wait), \
                 patch.object(manager, "health", return_value={"controller": True, "plugins": True, "ready": False}), \
                 patch.object(manager, "_credentials", return_value=("admin", "token")):
                result = manager._start_locked(state / "deployment-start.lock")
            try:
                self.assertEqual("running", result["state"])
                self.assertTrue(result["controlPlaneOnly"])
                self.assertEqual(receipt, result["executionActivationRecoveryReceipt"])
                self.assertNotEqual(old["operationId"], result["operationId"])
                self.assertEqual("1", captured["env"]["ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY"])
                archive = json.loads(next((deployment / "history").glob("*.json")).read_text())
                self.assertEqual(old, archive["operation"])
                self.assertEqual(observation, archive["deploymentRecoveryEvidence"]["hostObservation"])
            finally:
                from tools.jenkins.deployment.manager import _LIVE_JOBS
                owned = _LIVE_JOBS.pop(707, None)
                if owned: owned[1].close()

    def test_complete_native_proof_with_live_supervisor_cannot_spawn(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"
            paths = SimpleNamespace(state=state, home=root / "home", logs=root, repo=root)
            spec = SimpleNamespace(controller={"listenAddress": "127.0.0.1", "httpPort": 1})
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            old = {"operationId": "old", "generation": "old", "state": "stopped", "pid": 101, "hostPid": 101,
                   "creationTime": "1000", "homePath": str(paths.home.absolute())}
            manager._write(old)
            host = state / "deployment/host.json"; host.parent.mkdir()
            host.write_text("{}")
            with patch("tools.jenkins.deployment.recovery.has_terminal_deployment_proofs", return_value=True), \
                 patch("tools.jenkins.deployment.startup_observations.observe_departed_identity",
                       side_effect=JenkinsError("deployment_owner_live", "Owner remains alive", retryable=True)), \
                 patch("tools.jenkins.deployment.manager.subprocess.Popen") as spawn:
                with self.assertRaises(JenkinsError) as error:
                    manager._start_locked(state / "deployment-start.lock")
            self.assertEqual("deployment_owner_live", error.exception.code)
            spawn.assert_not_called()
            self.assertEqual(old, manager._read())
