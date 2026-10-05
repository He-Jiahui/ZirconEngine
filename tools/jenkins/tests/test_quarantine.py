from __future__ import annotations

import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.jenkins.deployment.quarantine import QuarantineBinding, classify_launches


class QuarantineTests(unittest.TestCase):
    def setUp(self):
        self.binding = QuarantineBinding("new-operation", "new-generation", "new-driver")
        self.execution = {
            "operationId": "new-operation", "generation": "new-generation",
            "driverDigest": "new-driver",
            "driver": {"runtimeOperationId": "new-operation",
                        "generation": "new-generation", "digest": "new-driver"},
        }

    def test_only_fully_bound_fresh_request_is_eligible(self):
        rows = [{"key": "fresh", "payload": {"status": "requested",
            "executionId": "fresh", "operationId": "new-operation",
            "generation": "new-generation", "driverDigest": "new-driver"}}]
        self.assertEqual({"eligible": ["fresh"], "preserve": []},
                         classify_launches(rows, {"fresh": self.execution}, self.binding))

    def test_historical_requested_row_is_preserved_without_rewrite(self):
        rows = [{"key": "old", "payload": {"status": "requested",
            "executionId": "old", "operationId": "old-operation",
            "generation": "1", "driverDigest": "old-driver"}}]
        old = {"operationId": "old-operation", "generation": "1", "driverDigest": "old-driver",
               "driver": {"runtimeOperationId": "old-operation", "generation": "1", "digest": "old-driver"}}
        self.assertEqual({"eligible": [], "preserve": ["old"]},
                         classify_launches(rows, {"old": old}, self.binding))

    def test_missing_or_nonrequested_rows_are_preserved(self):
        rows = [
            {"key": "started", "payload": {"status": "started", "executionId": "started"}},
            {"key": "unknown", "payload": {"status": "requested", "executionId": "unknown"}},
            {"key": "malformed", "payload": None},
        ]
        self.assertEqual({"eligible": [], "preserve": ["started", "unknown", "malformed"]},
                         classify_launches(rows, {}, self.binding))

    def test_started_historical_row_is_preserved_even_when_shape_matches(self):
        payload = {"status": "started", "executionId": "fresh",
                   "operationId": "new-operation", "generation": "new-generation",
                   "driverDigest": "new-driver"}
        self.assertEqual("preserve", __import__(
            "tools.jenkins.deployment.quarantine", fromlist=["classify_launch"]
        ).classify_launch(payload, self.execution, self.binding))

    def test_generation_and_driver_fields_cannot_be_omitted(self):
        payload = {"status": "requested", "executionId": "fresh",
                   "operationId": "new-operation", "generation": "new-generation",
                   "driverDigest": "new-driver"}
        execution = {**self.execution, "driver": {"runtimeOperationId": "new-operation"}}
        self.assertEqual({"eligible": [], "preserve": ["fresh"]},
                         classify_launches([{"key": "fresh", "payload": payload}],
                                           {"fresh": execution}, self.binding))

    def test_attempt_generation_is_distinct_from_runtime_operation(self):
        binding = QuarantineBinding("start-runtime", "1", "driver")
        payload = {"status": "requested", "executionId": "fresh",
                   "operationId": "start-runtime", "generation": "1",
                   "driverDigest": "driver"}
        execution = {"operationId": "start-runtime", "generation": "1",
                     "driverDigest": "driver",
                     "driver": {"runtimeOperationId": "start-runtime",
                                 "generation": "1", "digest": "driver"}}
        self.assertEqual({"eligible": ["fresh"], "preserve": []},
                         classify_launches([{"key": "fresh", "payload": payload}],
                                           {"fresh": execution}, binding))

    def test_broker_preserves_mixed_historical_and_malformed_rows_without_launch_or_write(self):
        """A new broker must quarantine every non-exact durable binding."""
        import tempfile
        from tools.jenkins.deployment.host import LifecycleHost
        from tools.jenkins.deployment import host as host_module
        from tools.jenkins.state import State

        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            runtime_python = root / ".jenkins/runtime/python/python.exe"
            runtime_python.parent.mkdir(parents=True)
            runtime_python.touch()
            launch_state = root / "state/coordination.sqlite3"
            state = State(launch_state)

            def row(status, operation, execution_id, generation="1", digest="old-driver"):
                return {"status": status, "executionId": execution_id,
                        "operationId": operation, "generation": generation,
                        "driverDigest": digest}

            state.put("execution_launch", "old-requested",
                      row("requested", "old-op", "old-requested"))
            state.put("execution_launch", "old-started",
                      row("started", "old-op", "old-started"))
            state.put("execution_launch", "malformed",
                      row("requested", "new-op", "malformed", digest="new-driver"))
            old_driver = {"runtimeOperationId": "old-op", "generation": "1",
                          "digest": "old-driver"}
            state.put("execution", "old-requested",
                      {"operationId": "old-op", "generation": "1",
                       "driverDigest": "old-driver", "driver": old_driver})
            state.put("execution", "old-started",
                      {"operationId": "old-op", "generation": "1",
                       "driverDigest": "old-driver", "driver": old_driver})
            state.put("execution", "malformed",
                      {"operationId": "new-op", "generation": "1",
                       "driverDigest": "new-driver", "driver": {}})
            before = {r["key"]: r["version"] for r in state.list("execution_launch")}

            lifecycle = LifecycleHost.__new__(LifecycleHost)
            lifecycle.paths = SimpleNamespace(repo=root, state=root / "state",
                                              logs=root / "logs", home=root / "home")
            lifecycle.spec = object()
            lifecycle.java = root / "java"
            lifecycle.war = root / "war"
            lifecycle.launch_state = launch_state
            lifecycle.operation_id = "new-op"
            lifecycle._launches = {}
            launcher = root / "launcher.py"
            with patch.object(host_module, "DeploymentManager") as manager, \
                 patch.object(host_module, "active_driver_binding", return_value={
                     "digest": "new-driver", "generation": "new-op",
                     "launcher": str(launcher)}), \
                 patch.object(host_module.NativeJob, "launch") as launch:
                manager.return_value._driver_env.return_value = {
                    "ZIRCON_DRIVER_LAUNCHER": str(launcher),
                    "JENKINS_PYTHON": str(runtime_python)}
                lifecycle._execution_broker_tick()

            launch.assert_not_called()
            self.assertEqual(before,
                             {r["key"]: r["version"] for r in state.list("execution_launch")})
            self.assertEqual(before["old-requested"],
                             state.get("execution_launch", "old-requested")["version"])


if __name__ == "__main__":
    unittest.main()
