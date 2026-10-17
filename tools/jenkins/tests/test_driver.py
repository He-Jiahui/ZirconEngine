"""A deployed driver is an immutable, verified input to every workflow."""
from pathlib import Path
import shutil
import tempfile
import unittest
import uuid
from unittest import mock

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.deployment.driver import (seal_driver, verify_driver, driver_environment,
                                              runtime_agent_name, runtime_agent_label)


class DriverTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        source = self.root / "tools/jenkins"
        source.mkdir(parents=True)
        (source / "__main__.py").write_text("print('sealed')\n", encoding="utf-8")
        pipeline = self.root / ".jenkins/pipeline"
        pipeline.mkdir(parents=True)
        (pipeline / "steps.groovy").write_text("return this\n", encoding="utf-8")

    def test_same_inputs_share_snapshot_but_changes_get_new_identity(self):
        first = seal_driver(self.root)
        self.assertEqual(first, seal_driver(self.root))
        self.assertTrue(Path(first["launcher"]).is_file())
        (self.root / "tools/jenkins/__main__.py").write_text("print('new')\n", encoding="utf-8")
        second = seal_driver(self.root)
        self.assertNotEqual(first["driverDigest"], second["driverDigest"])
        self.assertEqual("print('sealed')\n", (Path(first["root"]) / "tools/jenkins/__main__.py").read_text())

    def _simulated_windows_rename_error(self, code):
        error = OSError("simulated Windows rename contention")
        error.winerror = code
        return error

    def test_windows_rename_contention_retries_only_until_publication(self):
        import os
        failures = [self._simulated_windows_rename_error(code) for code in (5, 32, 33)]
        real_rename = os.rename

        def contend_then_publish(source, destination):
            if failures:
                raise failures.pop(0)
            return real_rename(source, destination)

        with mock.patch("tools.jenkins.deployment.driver.os.rename",
                        side_effect=contend_then_publish) as rename, \
             mock.patch("tools.jenkins.deployment.driver.time.sleep"):
            snapshot = seal_driver(self.root)
        self.assertEqual(4, rename.call_count)
        self.assertEqual(snapshot["driverDigest"], verify_driver(
            snapshot["root"], expected_digest=snapshot["driverDigest"])["driverDigest"])

    def test_retryable_windows_rename_error_stops_at_deadline_and_cleans_stage(self):
        error = self._simulated_windows_rename_error(5)
        clock = [0.0, 1.0, 1.5, 2.0]

        with mock.patch("tools.jenkins.deployment.driver.os.rename", side_effect=error) as rename, \
             mock.patch("tools.jenkins.deployment.driver.time.monotonic",
                        side_effect=lambda: clock.pop(0)), \
             mock.patch("tools.jenkins.deployment.driver.time.sleep") as sleep:
            with self.assertRaises(OSError):
                seal_driver(self.root)
        self.assertEqual(2, rename.call_count)
        self.assertEqual(1, sleep.call_count)
        self.assertEqual(0.025, sleep.call_args.args[0])
        staging = Path(rename.call_args_list[0].args[0])
        self.assertFalse(staging.exists())
        self.assertEqual([], list((self.root / ".jenkins/runtime/drivers").glob(".incoming-*")))

    def test_nonretryable_windows_rename_error_fails_without_another_attempt(self):
        error = self._simulated_windows_rename_error(87)
        with mock.patch("tools.jenkins.deployment.driver.os.rename", side_effect=error) as rename, \
             mock.patch("tools.jenkins.deployment.driver.time.sleep") as sleep:
            with self.assertRaises(OSError):
                seal_driver(self.root)
        self.assertEqual(1, rename.call_count)
        sleep.assert_not_called()
        staging = Path(rename.call_args.args[0])
        self.assertFalse(staging.exists())

    def test_missing_staging_without_destination_fails_closed(self):
        def vanish_then_fail(source, destination):
            shutil.rmtree(source)
            raise self._simulated_windows_rename_error(5)

        with mock.patch("tools.jenkins.deployment.driver.os.rename",
                        side_effect=vanish_then_fail) as rename:
            with self.assertRaises(JenkinsError) as rejected:
                seal_driver(self.root)
        self.assertEqual("driver_publish_source_missing", rejected.exception.code)
        rename.assert_called_once()
        self.assertFalse(Path(rename.call_args.args[0]).exists())

    def test_same_digest_parallel_publisher_is_verified_and_owned_stage_is_cleaned(self):
        import os
        real_rename = os.rename

        def competing_publisher(source, destination):
            peer = Path(source).parent / (".incoming-" + uuid.uuid4().hex)
            shutil.copytree(source, peer)
            real_rename(peer, destination)
            raise self._simulated_windows_rename_error(5)

        with mock.patch("tools.jenkins.deployment.driver.os.rename",
                        side_effect=competing_publisher):
            snapshot = seal_driver(self.root)
        self.assertEqual(snapshot["driverDigest"], verify_driver(
            snapshot["root"], expected_digest=snapshot["driverDigest"])["driverDigest"])
        incoming = list((self.root / ".jenkins/runtime/drivers").glob(".incoming-*"))
        self.assertEqual([], incoming)

    def test_invalid_parallel_destination_fails_closed_and_is_not_overwritten(self):
        def publish_invalid_destination(source, destination):
            destination.mkdir()
            marker = destination / "foreign-marker.txt"
            marker.write_text("preserve", encoding="utf-8")
            raise self._simulated_windows_rename_error(5)

        with mock.patch("tools.jenkins.deployment.driver.os.rename",
                        side_effect=publish_invalid_destination):
            with self.assertRaises(JenkinsError):
                seal_driver(self.root)
        snapshots = self.root / ".jenkins/runtime/drivers"
        destination = next(path for path in snapshots.iterdir()
                           if len(path.name) == 64 and not path.name.startswith("."))
        self.assertEqual("preserve", (destination / "foreign-marker.txt").read_text(encoding="utf-8"))
        self.assertEqual([], list(snapshots.glob(".incoming-*")))

    def test_corrupt_deployed_source_cannot_be_reused(self):
        snapshot = seal_driver(self.root)
        (Path(snapshot["root"]) / "tools/jenkins/__main__.py").write_text("print('tampered')\n")
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])
        with self.assertRaises(JenkinsError):
            seal_driver(self.root)

    def test_manifest_cannot_redirect_verification_outside_snapshot(self):
        import json
        snapshot = seal_driver(self.root)
        path = Path(snapshot["root"]) / "driver-manifest.json"
        manifest = json.loads(path.read_text())
        manifest["files"]["../outside.py"] = {"sha256": "0" * 64, "bytes": 0}
        path.write_text(json.dumps(manifest))
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])

    def test_unlisted_import_cannot_enter_a_verified_snapshot(self):
        snapshot = seal_driver(self.root)
        (Path(snapshot["root"]) / "tools/jenkins/extra.py").write_text("pass\n")
        with self.assertRaises(JenkinsError):
            verify_driver(snapshot["root"], expected_digest=snapshot["driverDigest"])

    def test_driver_environment_requires_dedicated_python(self):
        import sys
        snapshot = seal_driver(self.root)
        with self.assertRaises(JenkinsError):
            driver_environment(snapshot, sys.executable, self.root)
        executable = self.root / ".jenkins/runtime/python/python.exe"
        executable.parent.mkdir(parents=True)
        executable.write_bytes(b"fixture")
        environment = driver_environment(snapshot, executable, self.root)
        self.assertEqual(str(executable), environment["JENKINS_PYTHON"])

    def test_invalid_python_cannot_be_selected_as_a_sealed_driver(self):
        (self.root / "tools/jenkins/broken.py").write_text("value = )\n", encoding="utf-8")
        with self.assertRaises(JenkinsError) as rejected:
            seal_driver(self.root)
        self.assertEqual("driver_syntax_invalid", rejected.exception.code)
        self.assertFalse((self.root / ".jenkins/runtime/drivers").exists())


class RuntimeAgentIdentityTests(unittest.TestCase):
    def test_configured_node_name_and_selector_are_stable_across_operations(self):
        for base in ("zircon-windows-agent", "custom_agent.v2"):
            for operation in ("start-123456789012345", "start-9999912345", "restart_owned.v2"):
                with self.subTest(base=base, operation=operation):
                    self.assertEqual(base, runtime_agent_name(base, operation))
                    self.assertEqual(base, runtime_agent_label(base, operation))

    def test_stable_name_preserves_all_original_input_rejections(self):
        for helper in (runtime_agent_name, runtime_agent_label):
            for base in (None, 7, "", " ", "bad label", "../agent", "agent/child", "agent:port", "agent\n"):
                with self.subTest(helper=helper.__name__, base=base), self.assertRaises(JenkinsError) as rejected:
                    helper(base, "start-1")
                self.assertEqual("agent_name_invalid", rejected.exception.code)
            for operation in (None, 7, "", " ", "runtime op with spaces", "start/child", "start:1", "start\n"):
                with self.subTest(helper=helper.__name__, operation=operation), self.assertRaises(JenkinsError) as rejected:
                    helper("zircon-windows-agent", operation)
                self.assertEqual("runtime_operation_invalid", rejected.exception.code)

    def test_same_node_name_cannot_authorize_another_execution_runtime(self):
        from tools.jenkins.deployment.host import LifecycleHost

        selected_operation, selected_digest = "start-current", "a" * 64
        name = runtime_agent_name("zircon-windows-agent", selected_operation)
        self.assertEqual(name, runtime_agent_name("zircon-windows-agent", "start-old"))
        launch = {"agentName": name, "operationId": selected_operation, "generation": "7", "driverDigest": selected_digest}
        execution = {**launch, "driver": {"digest": selected_digest, "runtimeOperationId": selected_operation, "generation": "7"}}
        self.assertTrue(LifecycleHost._execution_launch_binding(launch, execution, selected_digest, selected_operation))
        for old_execution in (
            {**execution, "operationId": "start-old"},
            {**execution, "generation": "6"},
            {**execution, "driverDigest": "b" * 64},
            {**execution, "driver": {**execution["driver"], "runtimeOperationId": "start-old"}},
        ):
            with self.subTest(execution=old_execution):
                self.assertEqual(name, old_execution["agentName"])
                self.assertFalse(LifecycleHost._execution_launch_binding(launch, old_execution, selected_digest, selected_operation))


if __name__ == "__main__":
    unittest.main()
