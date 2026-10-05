from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.manager import DeploymentManager
from tools.jenkins.deployment.host import LifecycleHost
from tools.jenkins.deployment import host as host_module
from tools.jenkins.state.locks import process_lock


class LifecycleRuntimeTests(unittest.TestCase):
    def test_failed_or_stopped_labels_do_not_authorize_duplicate_start_without_proofs(self):
        for label in ('failed', 'stopped'):
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                paths = type('Paths', (), {'state': root / 'state', 'home': root / 'home',
                    'logs': root / 'logs', 'repo': root})()
                spec = type('Spec', (), {'controller': {'listenAddress': '127.0.0.1', 'httpPort': 1},
                    'agent': {'name': 'agent'}})()
                manager = DeploymentManager(spec, paths, root / 'java', root / 'war')
                manager._write({'operationId': 'op', 'generation': 'op', 'state': label,
                    'hostPid': 1, 'pid': 1, 'creationTime': '100', 'homePath': str(paths.home.absolute())})
                host_path = paths.state / 'deployment/host.json'
                host_path.parent.mkdir()
                host_path.write_text(json.dumps({'operationId': 'op', 'generation': 'op', 'status': label,
                    'hostPid': 1, 'controllerIdentity': {'pid': 2, 'creationTime': '120'},
                    'agentIdentity': {'pid': 3, 'creationTime': '130'}}))
                with patch('tools.jenkins.deployment.manager.identity', return_value=None), \
                     patch.object(manager, 'health', return_value={'controller': False, 'ready': False}), \
                     patch('tools.jenkins.deployment.recovery.windows_boot_filetime', return_value=50), \
                     patch('tools.jenkins.deployment.manager.subprocess.Popen') as spawn:
                    with self.assertRaises(JenkinsError) as error:
                        manager.start()
                    self.assertEqual('deployment_termination_unproven', error.exception.code)
                    spawn.assert_not_called()
                self.assertEqual(label, manager._read()['state'])
                self.assertFalse((paths.state / 'deployment/driver.json').exists())

    def test_controller_exit_stops_owned_agent_and_records_both_real_proofs(self):
        import sys
        from tools.jenkins.processes import NativeJob
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.root = root
            lifecycle.commands = root / "commands"
            lifecycle.commands.mkdir()
            lifecycle.operation_id = "op"
            lifecycle._start = lambda: None
            lifecycle._execution_broker_tick = lambda: None
            lifecycle.job = NativeJob.launch((sys.executable, "-B", "-c", "print('controller exited')"),
                cwd=root, log_dir=root / "controller")
            lifecycle.agent_job = NativeJob.launch((sys.executable, "-B", "-c", "import time; time.sleep(30)"),
                cwd=root, log_dir=root / "agent")
            saved = []
            lifecycle._save = saved.append
            try:
                lifecycle.job.process.wait(timeout=10)
                lifecycle.run()
                self.assertEqual("stopped", saved[-1]["status"])
                self.assertTrue(saved[-1]["nativeTerminationProof"]["complete"])
                self.assertTrue(saved[-1]["agentTerminationProof"]["complete"])
            finally:
                lifecycle.job.close()
                lifecycle.agent_job.close()

    def test_start_preserves_lost_same_boot_deployment_without_spawning(self):
        from tools.jenkins.processes.boot import windows_boot_filetime
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("Paths", (), {"state": root / "state", "home": root / "home",
                "logs": root / "logs", "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            operation = {"operationId": "op", "generation": "op", "state": "running",
                "hostPid": 1, "pid": 1, "creationTime": "100", "homePath": str(paths.home.absolute())}
            manager._write(operation)
            host_path = paths.state / "deployment/host.json"
            host_path.parent.mkdir()
            host_path.write_text(json.dumps({"operationId": "op", "generation": "op", "status": "running",
                "hostPid": 1, "controllerIdentity": {"pid": 2, "creationTime": "120"},
                "agentIdentity": {"pid": 3, "creationTime": "130"}}))
            with patch("tools.jenkins.deployment.manager.identity", return_value=None), \
                 patch.object(manager, "health", return_value={"controller": False, "ready": False}), \
                 patch("tools.jenkins.deployment.recovery.windows_boot_filetime", return_value=50), \
                 patch("tools.jenkins.deployment.manager.subprocess.Popen") as spawn:
                with self.assertRaises(JenkinsError) as error:
                    manager.start()
                self.assertEqual("deployment_termination_unproven", error.exception.code)
                spawn.assert_not_called()
            self.assertEqual("100", manager._read()["creationTime"])
            self.assertEqual([], list((paths.state / "deployment").glob("history/*")))

    def test_broker_records_real_completed_native_host_without_losing_lifecycle(self):
        import sys
        from types import SimpleNamespace
        from tools.jenkins.state import State
        from tools.jenkins.processes import NativeJob, ProcessRegistry
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            runtime_python = root / ".jenkins/runtime/python/python.exe"
            runtime_python.parent.mkdir(parents=True)
            runtime_python.touch()
            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.paths = SimpleNamespace(repo=root, state=root / "state")
            lifecycle.spec = object()
            lifecycle.java = root / "java"
            lifecycle.war = root / "war"
            lifecycle.launch_state = root / "state/coordination.sqlite3"
            state = State(lifecycle.launch_state)
            job = NativeJob.launch((sys.executable, "-B", "-c", "print('host complete')"),
                                   cwd=root, log_dir=root / "logs")
            registry = ProcessRegistry(state)
            guard = registry.register(job, execution_id="e", owner="e", role="execution_host_guard")
            lifecycle._launches = {"e": (job, registry)}
            launcher = root / "launcher.py"
            state.put("execution_launch", "e", {"status": "started", "executionId": "e",
                "operationId": "op", "generation": "1", "driverDigest": "d",
                "guardNativeJobId": guard.native_job_id})
            state.put("execution", "e", {"status": "running", "executionId": "e",
                "operationId": "op", "generation": "1", "driverDigest": "d",
                "driver": {"runtimeOperationId": "op", "generation": "1", "digest": "d"}})
            state.put("execution_host", "e", {"status": "terminal", "operationId": "op", "generation": "1"})
            try:
                job.wait(timeout_seconds=10)
                with patch.object(host_module, "DeploymentManager") as manager, \
                     patch.object(host_module, "active_driver_binding", return_value={
                         "digest": "d", "generation": "op", "launcher": str(launcher)}):
                    manager.return_value._driver_env.return_value = {
                        "ZIRCON_DRIVER_LAUNCHER": str(launcher), "JENKINS_PYTHON": str(runtime_python)}
                    lifecycle._execution_broker_tick()
                result = state.get("execution_launch", "e")["payload"]
                self.assertEqual("exited", result["status"])
                self.assertEqual(0, result["exitCode"])
                self.assertTrue(state.get("native_job", guard.native_job_id)["payload"]["completeProof"]["complete"])
                self.assertEqual({}, lifecycle._launches)
            finally:
                job.close()

    def test_broker_error_preserves_lifecycle_until_an_owned_stop(self):
        import os
        from types import SimpleNamespace
        from tools.jenkins.state import State
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.root = root
            lifecycle.commands = root / "commands"
            lifecycle.commands.mkdir()
            lifecycle.operation_id = "op"
            lifecycle.launch_state = root / "state.sqlite3"
            identity = {"pid": 7, "creationTime": "birth"}
            lifecycle.job = SimpleNamespace(identity=SimpleNamespace(to_dict=lambda: identity),
                process=SimpleNamespace(poll=lambda: None))
            lifecycle._start = lambda: None
            def stop():
                lifecycle.job = None
                return {"status": "stopped"}
            lifecycle._stop = stop
            command = {"action": "stop", "operationId": "op", "hostPid": os.getpid(),
                "generation": "op", "controllerIdentity": identity}
            def next_tick(_delay):
                (lifecycle.commands / "stop.json").write_text(json.dumps(command))
            with patch.object(lifecycle, "_execution_broker_tick", side_effect=AttributeError("bad queue record")), \
                 patch.object(host_module.time, "sleep", side_effect=next_tick):
                lifecycle.run()
            self.assertEqual("stopped", json.loads((lifecycle.commands / "stop.result.json").read_text())["status"])
            self.assertIsNone(lifecycle.job)

    def test_successful_stop_returns_without_polling_destroyed_job(self):
        import os
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.root = root
            lifecycle.commands = root / "commands"
            lifecycle.commands.mkdir()
            lifecycle.operation_id = "stop-operation"
            lifecycle.job = SimpleNamespace(identity=SimpleNamespace(to_dict=lambda: {"pid": 7, "creationTime": "birth"}))
            command = {"action": "stop", "operationId": lifecycle.operation_id,
                       "hostPid": os.getpid(), "generation": lifecycle.operation_id,
                       "controllerIdentity": {"pid": 7, "creationTime": "birth"}}
            (lifecycle.commands / "stop.json").write_text(json.dumps(command), encoding="utf-8")
            lifecycle._start = lambda: None
            def stop():
                lifecycle.job = None
                return {"status": "stopped", "nativeTerminationProof": {"complete": True}}
            lifecycle._stop = stop
            lifecycle._execution_broker_tick = lambda: self.fail("stopped host must not dispatch more work")
            lifecycle.run()
            self.assertEqual("stopped", json.loads((lifecycle.commands / "stop.result.json").read_text())["status"])

    def test_native_components_use_their_deployment_directories(self):
        from tools.jenkins.deployment.lifecycle import spawn_owned
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("Paths", (), {"root": root / ".jenkins",
                "agent": root / ".jenkins" / "agent", "logs": root / ".jenkins" / "logs",
                "state": root / ".jenkins" / "state"})()
            with patch("tools.jenkins.deployment.lifecycle.NativeJob.launch") as launch:
                spawn_owned(paths, ["java.exe"], {}, component="agent")
                self.assertEqual(launch.call_args.kwargs["cwd"], paths.agent)
                spawn_owned(paths, ["java.exe"], {}, component="controller")
                self.assertEqual(launch.call_args.kwargs["cwd"], paths.root)

    def test_start_persists_running_identity_and_ready_health_proof(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            state = root / "state"
            state.mkdir()
            paths = type("Paths", (), {
                "state": state, "logs": root / "logs", "home": root / "home",
                "repo": root / "repo", "tmp": root / "tmp",
            })()
            spec = type("Spec", (), {
                "controller": {"listenAddress": "127.0.0.1", "httpPort": 53748},
                "agent": {"name": "zircon-agent"},
            })()

            class FakeIdentity:
                def __init__(self, pid, birth):
                    self.pid = pid
                    self.creation_time = birth

                def to_dict(self):
                    return {"pid": self.pid, "creationTime": self.creation_time,
                            "executable": "E:\\Git\\ZirconEngine\\.jenkins\\runtime\\python\\python.exe",
                            "commandDigest": "identity-digest"}

            class FakeProcess:
                def poll(self):
                    return None

            class FakeJob:
                def __init__(self, identity):
                    self.identity = identity
                    self.process = FakeProcess()

            controller = FakeJob(FakeIdentity(101, "controller-birth"))
            agent = FakeJob(FakeIdentity(202, "agent-birth"))

            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.spec = spec
            lifecycle.paths = paths
            lifecycle.java = root / "java.exe"
            lifecycle.war = root / "jenkins.war"
            lifecycle.root = state / "deployment"
            lifecycle.state = lifecycle.root / "host.json"
            lifecycle.commands = lifecycle.root / "commands"
            lifecycle.job = None
            lifecycle.agent_job = None
            lifecycle.operation_id = "start-test-operation"
            lifecycle.launch_state = state / "coordination.sqlite3"
            lifecycle._launches = {}

            manager = type("Manager", (), {
                "__init__": lambda self, *args: None,
                "bootstrap": lambda self: None,
                "register_jobs": lambda self: None,
            })

            def spawn(_paths, _args, _env, *, component):
                self_job = controller if component == "controller" else agent
                return self_job

            with patch.object(host_module, "launch_command", return_value=(
                    [str(lifecycle.java), "-jar", str(lifecycle.war)], {})), \
                 patch.object(host_module, "check_port_available"), \
                 patch.object(host_module, "spawn_owned", side_effect=spawn), \
                 patch.object(host_module, "wait_for_health", return_value=True), \
                 patch.object(host_module, "DeploymentManager", manager), \
                 patch.object(lifecycle, "_auth_headers", return_value={}), \
                 patch.object(lifecycle, "_start_agent", side_effect=lambda: setattr(lifecycle, "agent_job", agent)):
                result = lifecycle._start()

            self.assertEqual(result["status"], "running")
            self.assertEqual(result["operationId"], "start-test-operation")
            self.assertEqual(result["generation"], "start-test-operation")
            self.assertEqual(result["controllerIdentity"]["pid"], 101)
            self.assertEqual(result["controllerIdentity"]["creationTime"], "controller-birth")
            self.assertEqual(result["agentIdentity"]["pid"], 202)
            saved = json.loads(lifecycle.state.read_text(encoding="utf-8"))
            self.assertEqual(saved["status"], "running")
            self.assertEqual(saved["operationId"], "start-test-operation")
            self.assertEqual(saved["generation"], "start-test-operation")
            self.assertEqual(saved["controllerIdentity"]["creationTime"], "controller-birth")
            self.assertEqual(saved["agentIdentity"]["creationTime"], "agent-birth")
            self.assertTrue(saved["healthProof"]["controller"])
            self.assertTrue(saved["healthProof"]["agent"])
            self.assertTrue(saved["healthProof"]["ready"])

    def test_execution_launch_binding_requires_current_driver_generation(self):
        payload = {"operationId": "op-2", "generation": "1", "driverDigest": "d"}
        execution = {**payload, "driver": {"generation": "1", "runtimeOperationId": "op-2"}}
        self.assertTrue(LifecycleHost._execution_launch_binding(payload, execution, "d", "op-2"))
        self.assertFalse(LifecycleHost._execution_launch_binding(payload, execution, "other", "op-2"))
        self.assertFalse(LifecycleHost._execution_launch_binding(payload, {**execution, "generation": "old"}, "d", "op-2"))

    def test_execution_launch_binding_rejects_missing_execution(self):
        self.assertFalse(LifecycleHost._execution_launch_binding({}, {}, "d", "op"))

    def test_process_lock_rejects_second_owner_and_releases(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "deployment.lock"
            with process_lock(path):
                with self.assertRaises(JenkinsError):
                    with process_lock(path):
                        pass
            with process_lock(path):
                pass

    def test_reconcile_rejects_stale_host_generation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            paths = type("Paths", (), {"state": state, "logs": root, "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            manager._write({"operationId": "current", "generation": "current", "hostPid": 1,
                            "pid": 999999, "creationTime": "x", "state": "running"})
            (state / "deployment").mkdir()
            (state / "deployment" / "host.json").write_text(json.dumps({
                "operationId": "old", "generation": "old", "hostPid": 2, "status": "running"
            }), encoding="utf-8")
            with patch.object(manager, "health", return_value={"ready": False}):
                result = manager.reconcile()
            self.assertEqual(result["state"], "unknown-owner")
            self.assertFalse(result["hostIdentityProof"]["operationMatches"])

    def test_reconcile_does_not_promote_unknown_owner_when_health_is_ready(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            paths = type("Paths", (), {"state": state, "logs": root, "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            manager._write({"operationId": "current", "generation": "current", "hostPid": 1,
                            "pid": 999999, "creationTime": "x", "state": "running"})
            (state / "deployment").mkdir()
            (state / "deployment" / "host.json").write_text(json.dumps({
                "operationId": "other", "generation": "other", "hostPid": 2, "status": "running"
            }), encoding="utf-8")
            with patch.object(manager, "health", return_value={"ready": True}):
                result = manager.reconcile()
            self.assertEqual(result["state"], "unknown-owner")

    def test_reconcile_does_not_treat_same_birth_different_executable_as_owned(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            paths = type("Paths", (), {"state": state, "logs": root, "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            manager._write({"operationId": "current", "generation": "current", "hostPid": 1,
                            "pid": 999, "creationTime": "birth", "executable": str(root / "python.exe"),
                            "state": "running"})
            (state / "deployment").mkdir()
            (state / "deployment" / "host.json").write_text(json.dumps({
                "operationId": "current", "generation": "current", "hostPid": 1, "status": "running"
            }), encoding="utf-8")
            with patch("tools.jenkins.deployment.manager.identity", return_value={
                    "pid": 999, "creationTime": "birth", "executable": str(root / "other.exe")}), \
                 patch.object(manager, "health", return_value={"controller": True, "ready": True}):
                result = manager.reconcile()
            self.assertFalse(result["observedAlive"])
            self.assertEqual(result["state"], "unknown-owner")
            self.assertFalse(result["serviceReady"])

    def test_start_blocks_healthy_endpoint_without_owned_host_binding(self):
        """A foreign Jenkins on the configured port must not be adopted."""
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            paths = type("Paths", (), {"state": state, "home": root / "home",
                "logs": root, "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            manager._write({"operationId": "current", "generation": "current", "state": "running",
                            "hostPid": 1, "pid": 999999, "creationTime": "old", "homePath": str(paths.home.absolute())})
            (state / "deployment").mkdir()
            (state / "deployment" / "host.json").write_text(json.dumps({
                "operationId": "other", "generation": "other", "hostPid": 2, "status": "running"
            }), encoding="utf-8")
            with patch("tools.jenkins.deployment.manager.identity", return_value=None), \
                 patch.object(manager, "health", return_value={"controller": True, "agent": False,
                                                                  "plugins": True, "ready": False}), \
                 patch("tools.jenkins.deployment.manager.subprocess.Popen") as spawn:
                with self.assertRaises(JenkinsError) as error:
                    manager.start()
            self.assertEqual("deployment_owner_unproven", error.exception.code)
            self.assertTrue(error.exception.retryable)
            spawn.assert_not_called()

    def test_stopped_control_plane_restarts_without_enabling_agent(self):
        import sys
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            deployment = state / "deployment"; deployment.mkdir()
            paths = type("Paths", (), {"state": state, "home": root / "home",
                "logs": root / "logs", "repo": root, "tmp": root / "tmp"})()
            spec = type("Spec", (), {"path": root / "spec.json",
                "controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            home = str(paths.home.absolute())
            controller = {"pid": 2, "creationTime": "120", "executable": str(root / "java"),
                          "commandDigest": "b" * 64}
            complete = {"identity": controller, "complete": True, "childrenGone": True,
                        "activeProcesses": 0, "stdoutEof": True, "stderrEof": True,
                        "processExitCode": 0}
            old = {"schemaVersion": 2, "operationId": "old", "generation": "old",
                   "state": "stopped", "component": "lifecycle-host", "hostPid": 1,
                   "pid": 1, "creationTime": "100", "executable": str(root / "python"),
                   "homePath": home, "controlPlaneOnly": True}
            manager._write(old)
            (deployment / "host.json").write_text(json.dumps({
                "operationId": "old", "generation": "old", "hostPid": 1,
                "status": "stopped", "controllerIdentity": controller,
                "agentLaunchAttempted": False, "nativeTerminationProof": complete
            }), encoding="utf-8")
            (deployment / "driver.json").write_text(json.dumps({"driverDigest": "d"}), encoding="utf-8")
            captured = {}

            class FakeProcess:
                pid = 303
                def poll(self): return None

            def fake_popen(args, **kwargs):
                captured.update(kwargs)
                captured["args"] = args
                return FakeProcess()

            def fake_health_wait(*args, **kwargs):
                command = captured.get("args", [])
                operation_id = command[command.index("--operation-id") + 1]
                (deployment / "host.json").write_text(json.dumps({
                    "operationId": operation_id, "generation": operation_id, "hostPid": 303,
                    "status": "running", "controlPlaneOnly": True}), encoding="utf-8")
                return True

            identity_value = SimpleNamespace(to_dict=lambda: {
                "pid": 303, "creationTime": "3030", "executable": str(root / "python"),
                "commandDigest": "c" * 64})
            with patch("tools.jenkins.deployment.startup_observations.observe_departed_identity",
                       return_value={"pid": 1, "expectedCreationTime": "100", "status": "absent"}), \
                 patch("tools.jenkins.deployment.manager.check_port_available"), \
                 patch.object(manager, "_driver_env", return_value={
                     "ZIRCON_DRIVER_LAUNCHER": str(root / "launcher.py"),
                     "ZIRCON_DRIVER_DIGEST": "d", "JENKINS_PYTHON": sys.executable}), \
                 patch("tools.jenkins.deployment.manager.subprocess.Popen", side_effect=fake_popen), \
                 patch("tools.jenkins.deployment.manager.current_identity", return_value=identity_value), \
                 patch("tools.jenkins.deployment.manager.wait_for_health", side_effect=fake_health_wait), \
                 patch.object(manager, "health", return_value={"controller": True, "plugins": True,
                                                                  "agent": False, "ready": False}), \
                 patch.object(manager, "_credentials", return_value=("admin", "token")):
                result = manager._start_locked(state / "deployment-start.lock")
            try:
                self.assertTrue(captured["env"]["ZIRCON_JENKINS_RECOVERY_CONTROL_ONLY"] == "1")
                self.assertTrue(result["controlPlaneOnly"])
                self.assertTrue(result["state"] == "running")
            finally:
                entry = __import__("tools.jenkins.deployment.manager", fromlist=["_LIVE_JOBS"])._LIVE_JOBS.pop(303, None)
                if entry:
                    entry[1].close()

    def test_control_plane_drain_ignores_proven_historical_execution(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("Paths", (), {"state": root / "state", "home": root / "home", "logs": root / "logs", "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1}, "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            operation = {"operationId": "current", "generation": "current", "controlPlaneOnly": True, "hostPid": 11, "pid": 11, "creationTime": "host", "executable": str(root / "host.exe")}
            controller = {"pid": 12, "creationTime": "controller", "executable": str(root / "java.exe")}
            host = {"operationId": "current", "generation": "current", "hostPid": 11, "controlPlaneOnly": True, "agentLaunchAttempted": False, "controllerIdentity": controller, "status": "running"}
            identities = {11: {"creationTime": "host", "executable": str(root / "host.exe")}, 12: {"creationTime": "controller", "executable": str(root / "java.exe")}}
            with patch("tools.jenkins.deployment.manager.identity", side_effect=lambda pid: identities[pid]), \
                 patch.object(manager, "_active_execution_records", return_value=[{"key": "old", "payload": {"operationId": "old", "generation": "old"}}]), \
                 patch.object(manager, "_request", side_effect=[(302, b""), (200, json.dumps({"computer": [{"displayName": "agent", "busyExecutors": 0, "offline": True}]}).encode()), (200, json.dumps({"jobs": [{"name": "old-job", "builds": [{"building": True}]}]}).encode())]):
                result = manager._control_plane_drain(operation, host)
            self.assertEqual("drained", result["status"])
            self.assertTrue(result["proof"]["agentOffline"])

    def test_control_plane_drain_blocks_current_generation_execution(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("Paths", (), {"state": root / "state", "home": root / "home", "logs": root / "logs", "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1}, "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            operation = {"operationId": "current", "generation": "current", "controlPlaneOnly": True, "hostPid": 11, "pid": 11, "creationTime": "host", "executable": str(root / "host.exe")}
            controller = {"pid": 12, "creationTime": "controller", "executable": str(root / "java.exe")}
            host = {"operationId": "current", "generation": "current", "hostPid": 11, "controlPlaneOnly": True, "agentLaunchAttempted": False, "controllerIdentity": controller}
            identities = {11: {"creationTime": "host", "executable": str(root / "host.exe")}, 12: {"creationTime": "controller", "executable": str(root / "java.exe")}}
            with patch("tools.jenkins.deployment.manager.identity", side_effect=lambda pid: identities[pid]), patch.object(manager, "_active_execution_records", return_value=[{"key": "current", "payload": {"operationId": "current", "generation": "1", "status": "started"}}]):
                with self.assertRaises(JenkinsError) as error:
                    manager._control_plane_drain(operation, host)
            self.assertEqual("control_plane_execution_active", error.exception.code)

    def test_stop_rejects_journal_bound_to_different_home(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            home = root / "home"
            paths = type("Paths", (), {"state": state, "logs": root, "repo": root, "home": home})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            manager._write({"operationId": "op", "generation": "op", "homePath": str(root / "other")})
            with self.assertRaises(JenkinsError) as raised:
                manager._stop_locked()
            self.assertEqual(raised.exception.code, "home_path_mismatch")

    def test_driver_record_missing_fails_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); state = root / "state"; state.mkdir()
            paths = type("Paths", (), {"state": state, "repo": root})()
            spec = type("Spec", (), {"controller": {"listenAddress": "127.0.0.1", "httpPort": 1},
                                      "agent": {"name": "agent"}})()
            manager = DeploymentManager(spec, paths, root / "java", root / "war")
            with self.assertRaises(JenkinsError):
                manager._driver_env()


if __name__ == "__main__":
    unittest.main()
