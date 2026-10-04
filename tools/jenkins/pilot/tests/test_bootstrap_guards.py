import os
import threading
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from unittest.mock import patch
from contextlib import nullcontext

from tools.jenkins.pilot import bootstrap
from tools.jenkins.pilot.bootstrap import BootstrapError, _require_keeper_record, _require_no_pending_launch, _terminate_record
from tools.jenkins.pilot.jenkins_config import PilotPaths
from tools.jenkins.pilot.tests.fixtures import temporary_directory


class BootstrapGuardTests(unittest.TestCase):
    def test_live_legacy_record_never_uses_ppid_tree_cleanup(self):
        with self.assertRaises(BootstrapError):
            _terminate_record({"pid": os.getpid(), "creationTime": "legacy"})

    def test_reused_keeper_pid_with_different_birth_is_not_adopted(self):
        record = {"keeperPid": 10, "keeperCreationTime": "old", "keeperJobHandle": 20}
        with self.assertRaises(BootstrapError):
            _require_keeper_record(record, {"owner_pid": 10, "owner_process_creation_time": "new"})

    def test_running_legacy_record_without_job_proof_is_not_adopted(self):
        with self.assertRaises(BootstrapError):
            _require_keeper_record({"pid": 10}, {"owner_pid": 10, "owner_process_creation_time": "new"})

    def test_incomplete_launch_intent_cannot_admit_another_runtime(self):
        with self.assertRaises(BootstrapError):
            _require_no_pending_launch({"pendingLaunch": {"kind": "agent", "launcherPid": 10}})

    def test_concurrent_controller_starts_preserve_one_recorded_job(self):
        with temporary_directory("jenkins-pilot-lifecycle-") as directory:
            root = Path(directory)
            paths = PilotPaths(root)
            paths.war_path.parent.mkdir(parents=True)
            paths.war_path.write_bytes(b"fixture")
            java = root / "java.exe"
            java.write_bytes(b"fixture")
            lease = {"owner_pid": 10, "owner_process_creation_time": "birth"}
            record = {"pid": 20, "creationTime": "child", "keeperPid": 10,
                      "keeperCreationTime": "birth", "keeperJobHandle": 30}
            barrier = threading.Barrier(2)
            launches = []

            def launch(*_arguments, **options):
                launches.append(dict(record))
                time.sleep(0.2)
                options["on_record"](dict(record))
                return dict(record)

            def start():
                barrier.wait(timeout=10)
                return bootstrap.start_controller(root)

            with patch("tools.jenkins.pilot.governance.require_live_storage_owner", return_value=lease), \
                    patch("tools.jenkins.pilot.assets.prepared_assets_context", return_value=nullcontext()), \
                    patch("tools.jenkins.pilot.lifetime.wait_runtime_started"), \
                    patch.object(bootstrap, "read_manifest", return_value={"javaPath": str(java)}), \
                    patch.object(bootstrap, "_java_major", return_value=21), \
                    patch.object(bootstrap, "_write_security_bootstrap"), \
                    patch.object(bootstrap, "ensure_credentials", return_value={}), \
                    patch.object(bootstrap, "_start_process", side_effect=launch), \
                    patch.object(bootstrap, "_record_alive", return_value=True), \
                    patch.object(bootstrap, "_wait_controller"), \
                    patch.object(bootstrap, "write_manifest"):
                with ThreadPoolExecutor(max_workers=2) as workers:
                    results = list(workers.map(lambda _: start(), range(2)))
            self.assertEqual(len(launches), 1)
            self.assertEqual(sorted(result["alreadyRunning"] for result in results), [False, True])
            self.assertEqual(bootstrap._read_state(paths)["controller"], record)

    def test_actual_java_marker_precedes_controller_readiness_budget(self):
        self._marker_boundary(failure=False)

    def test_preparation_timeout_cleans_owned_host_without_starting_http_budget(self):
        self._marker_boundary(failure=True)

    def _marker_boundary(self, *, failure):
        with temporary_directory("jenkins-pilot-marker-boundary-") as directory:
            root, events = Path(directory), []
            paths = PilotPaths(root)
            paths.war_path.parent.mkdir(parents=True)
            paths.war_path.write_bytes(b"fixture")
            java = root / "java.exe"
            java.write_bytes(b"fixture")
            record = {"pid": 20, "creationTime": "child", "keeperPid": 10,
                      "keeperCreationTime": "birth", "keeperJobHandle": 30}
            def marker(value, *, preparation_seconds):
                self.assertEqual(value, record)
                self.assertEqual(preparation_seconds, 180)
                events.append("actual-java-marker")
                if failure:
                    raise TimeoutError("asset preparation still pending")
            def ready(url):
                events.append("controller-http-readiness")
            def cleanup(value):
                self.assertEqual(value, record)
                events.append("owned-job-cleanup")
                return True
            with patch("tools.jenkins.pilot.governance.require_live_storage_owner", return_value={}), \
                 patch("tools.jenkins.pilot.assets.prepared_assets_context", return_value=nullcontext("a"*64)), \
                 patch("tools.jenkins.pilot.lifetime.wait_runtime_started", side_effect=marker), \
                 patch.object(bootstrap, "read_manifest", return_value={"javaPath": str(java)}), \
                 patch.object(bootstrap, "_java_major", return_value=21), \
                 patch.object(bootstrap, "_read_state", return_value={}), \
                 patch.object(bootstrap, "_write_security_bootstrap"), \
                 patch.object(bootstrap, "ensure_credentials", return_value={}), \
                 patch.object(bootstrap, "_start_process", return_value=record), \
                 patch.object(bootstrap, "_wait_controller", side_effect=ready), \
                 patch.object(bootstrap, "_write_state"), patch.object(bootstrap, "write_manifest"), \
                 patch.object(bootstrap, "_terminate_record", side_effect=cleanup):
                if failure:
                    with self.assertRaises(TimeoutError):
                        bootstrap.start_controller(root)
                else:
                    bootstrap.start_controller(root)
            self.assertEqual(events, ["actual-java-marker", "owned-job-cleanup"] if failure
                             else ["actual-java-marker", "controller-http-readiness"])


if __name__ == "__main__":
    unittest.main()
