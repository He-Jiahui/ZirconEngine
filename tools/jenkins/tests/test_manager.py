from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.deployment.manager import DeploymentManager


class ManagerTests(unittest.TestCase):
    def test_reconcile_marks_identity_unknown_without_killing(self):
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            m = DeploymentManager(S(), paths, root / "java", root / "war")
            m._write({"pid": 42, "birth": "old", "executable": "x", "state": "starting"})
            with patch("tools.jenkins.deployment.manager.identity", return_value={"pid":42,"birth":"new","executable":"x"}), patch.object(m, "health", return_value={"ready":False}):
                result = m.reconcile()
            self.assertFalse(result["observedAlive"])


    def test_archive_preserves_each_operation_and_termination_evidence(self):
        import json
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            manager = DeploymentManager(S(), paths, root / "java", root / "war")
            operation = {"operationId": "op", "generation": "op", "state": "stopped"}
            host = {"nativeTerminationProof": {"complete": True}}
            evidence = {"kind": "owned-native-terminal-proof"}
            manager._archive_deployment(operation, host, evidence)
            manager._archive_deployment(operation, host, evidence)
            records = list((root / "deployment/history").glob("*.json"))
            self.assertEqual(1, len(records))
            saved = json.loads(records[0].read_text(encoding="utf-8"))
            self.assertEqual(operation, saved["operation"])
            self.assertEqual(host, saved["host"])
            manager._archive_deployment({**operation, "operationId": "other"}, host, evidence)
            self.assertEqual(2, len(list((root / "deployment/history").glob("*.json"))))

    def test_matching_stale_host_record_cannot_promote_dead_owner(self):
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            manager = DeploymentManager(S(), paths, root / "java", root / "war")
            operation = {"operationId": "op", "generation": "op", "hostPid": 42, "pid": 42,
                         "creationTime": "100", "executable": str(root / "python"), "state": "running"}
            manager._write(operation)
            (root / "deployment").mkdir()
            (root / "deployment/host.json").write_text(__import__("json").dumps({
                "operationId": "op", "generation": "op", "hostPid": 42, "status": "running"}))
            with patch("tools.jenkins.deployment.manager.identity", return_value=None), \
                 patch.object(manager, "health", return_value={"controller": True, "plugins": True, "ready": True}):
                result = manager.reconcile()
            self.assertFalse(result["observedAlive"])
            self.assertFalse(result["serviceReady"])
            self.assertEqual("unknown-owner", result["state"])

    def test_owned_control_plane_reconcile_keeps_service_ready_and_builds_paused(self):
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            manager = DeploymentManager(S(), paths, root / "java", root / "war")
            operation = {"operationId": "op", "generation": "op", "hostPid": 42, "pid": 42,
                         "creationTime": "100", "executable": str(root / "python"), "state": "running",
                         "controlPlaneOnly": True}
            manager._write(operation)
            (root / "deployment").mkdir()
            (root / "deployment/host.json").write_text(__import__("json").dumps({
                "operationId": "op", "generation": "op", "hostPid": 42, "status": "running"}))
            with patch("tools.jenkins.deployment.manager.identity", return_value=operation), \
                 patch.object(manager, "health", return_value={"controller": True, "plugins": True, "agent": False, "ready": False}):
                result = manager.reconcile()
            self.assertTrue(result["observedAlive"])
            self.assertTrue(result["serviceReady"])
            self.assertFalse(result["healthProof"]["ready"])
            self.assertEqual("running", result["state"])




    def test_health_binds_agent_to_current_runtime_operation(self):
        import json
        from tools.jenkins.deployment.driver import runtime_agent_name
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
            agent = {"name": "zircon-agent"}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("P", (), {"state": root / "state", "logs": root, "repo": root})()
            manager = DeploymentManager(S(), paths, root / "java", root / "war")
            manager._write({"operationId": "runtime-1", "generation": "runtime-1"})
            selector = paths.state / "deployment/driver.json"
            selector.parent.mkdir()
            selector.write_text(json.dumps({"runtimeOperationId": "runtime-1"}))
            plugins = root / ".jenkins/plugin-checksums.json"
            plugins.parent.mkdir()
            plugins.write_text(json.dumps({"plugins": {"git": {"version": "1"}}}))
            runtime_name = runtime_agent_name(S.agent["name"], "runtime-1")
            nodes = [{"displayName": S.agent["name"], "offline": False},
                     {"displayName": runtime_name, "offline": True}]
            def request(path):
                body = {"computer": nodes} if path.startswith("computer/") else (
                    {"plugins": [{"shortName": "git", "version": "1", "active": True}]}
                    if path.startswith("pluginManager/") else {"mode": "NORMAL"})
                return 200, json.dumps(body).encode()
            with patch.object(manager, "_request", side_effect=request):
                self.assertFalse(manager.health()["agent"])
                nodes[1]["offline"] = False
                self.assertTrue(manager.health()["ready"])
                selector.write_text(json.dumps({"runtimeOperationId": "runtime-2"}))
                self.assertFalse(manager.health()["agent"])
                selector.write_text(json.dumps({}))
                self.assertFalse(manager.health()["agent"])

    def test_normal_start_refuses_unresolved_activation_before_launch(self):
        import json
        from tools.jenkins.contracts import JenkinsError
        class S:
            controller = {"listenAddress": "127.0.0.1", "httpPort": 1}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            paths = type("P", (), {"state": root, "logs": root, "repo": root})()
            manager = DeploymentManager(S(), paths, root / "java", root / "war")
            journal = root / "deployment/execution-activation.json"
            journal.parent.mkdir()
            journal.write_text(json.dumps({"status": "pending"}))
            with patch.object(manager, "_read", side_effect=AssertionError("lifecycle must not be read")):
                with self.assertRaises(JenkinsError) as caught:
                    manager._start_locked(root / "deployment-start.lock")
            self.assertEqual("execution_transition_unresolved", caught.exception.code)

if __name__ == "__main__": unittest.main()
