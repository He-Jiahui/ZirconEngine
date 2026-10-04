"""Native process lifetime checks without executing Cargo."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
from unittest.mock import patch

from tools.jenkins.pilot import closure
from tools.jenkins.pilot.tests.fixtures import temporary_directory
from tools.jenkins.pilot import inputs
from tools.jenkins.pilot.native.process_identity import process_matches_creation_time


@unittest.skipUnless(os.name == "nt", "native Windows Job Objects required")
class MetadataTransportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = temporary_directory("closure-native-")
        self.addCleanup(self._cleanup)
        self.root = Path(self.temporary.name)
        self.environment = dict(os.environ, TEMP=str(self.root), TMP=str(self.root),
                                PYTHONDONTWRITEBYTECODE="1", PYTHONPATH=str(Path.cwd()))

    def _cleanup(self):
        # Job accounting reaches zero before Windows necessarily releases CWD.
        deadline = time.monotonic() + 5
        while True:
            try:
                self.temporary.cleanup()
                return
            except PermissionError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.05)

    def test_complete_output_larger_than_worker_ring_and_completed_process_shape(self):
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        argv = (sys.executable, "-B", "-c", "import sys; print('x'*400000); print('warning',file=sys.stderr)")
        result = transport(argv, self.root, self.environment)
        self.assertIsInstance(result, subprocess.CompletedProcess)
        self.assertEqual(result.args, argv)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), "x" * 400000)
        self.assertEqual(result.stderr.strip(), "warning")
        evidence = transport.receipts[0]
        self.assertTrue(evidence["processTree"]["terminal"])
        self.assertEqual(evidence["processTree"]["scope"], "windows_job")
        self.assertGreater(evidence["outputBytes"], 400000)
        from tools.jenkins.pilot.managed_cargo import terminal_verified
        self.assertTrue(terminal_verified(result.native_observation))

    def test_timeout_terminates_actual_child_and_grandchild(self):
        identities = self.root / "identities.json"
        grandchild = "import time; time.sleep(120)"
        child = (
            "import json,os,subprocess,sys,time; from pathlib import Path; "
            "from tools.jenkins.pilot.native.process_identity import process_creation_time; "
            f"p=subprocess.Popen([sys.executable,'-B','-c',{grandchild!r}]); "
            f"Path({str(identities)!r}).write_text(json.dumps({{'childPid':os.getpid(),'childBirth':process_creation_time(os.getpid()),'grandchildPid':p.pid,'grandchildBirth':process_creation_time(p.pid)}})); "
            "print('descendants-ready',flush=True); time.sleep(120)"
        )
        root = f"import subprocess,sys,time; subprocess.Popen([sys.executable,'-B','-c',{child!r}]); time.sleep(120)"
        argv = (sys.executable, "-B", "-c", root)
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        with self.assertRaises(subprocess.TimeoutExpired) as caught:
            transport(argv, self.root, self.environment)
        self.assertIn("descendants-ready", caught.exception.output)
        identities = json.loads(identities.read_text())
        for name in ("child", "grandchild"):
            self.assertFalse(process_matches_creation_time(identities[name + "Pid"], identities[name + "Birth"]))
        evidence = transport.receipts[0]
        self.assertEqual(evidence["outcome"], "timed_out")
        self.assertTrue(evidence["processTree"]["terminal"])
        self.assertFalse(process_matches_creation_time(evidence["pid"], evidence["creationTime"]))

    def test_dummy_lease_owner_exit_converges_metadata_and_grandchild(self):
        from tools.jenkins.pilot.native.process_identity import popen_process_creation_time
        import threading

        owner = subprocess.Popen((sys.executable, "-B", "-c", "import time; time.sleep(120)"),
                                 cwd=self.root, env=self.environment,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.addCleanup(lambda: owner.poll() is None and owner.terminate())
        owner_birth = popen_process_creation_time(owner)
        identities_path = self.root / "owner-death-identities.json"
        descendant = "import time; time.sleep(120)"
        child = (
            "import json,os,subprocess,sys,time; from pathlib import Path; "
            "from tools.jenkins.pilot.native.process_identity import process_creation_time; "
            f"p=subprocess.Popen([sys.executable,'-B','-c',{descendant!r}]); "
            f"Path({str(identities_path)!r}).write_text(json.dumps({{'childPid':os.getpid(),'childBirth':process_creation_time(os.getpid()),'grandchildPid':p.pid,'grandchildBirth':process_creation_time(p.pid)}})); "
            "time.sleep(120)"
        )
        task = f"import subprocess,sys,time; subprocess.Popen([sys.executable,'-B','-c',{child!r}]); time.sleep(120)"
        cancellation_errors = []

        def end_dummy_owner_after_descendants_start():
            deadline = time.monotonic() + 15
            while not identities_path.exists() and time.monotonic() < deadline:
                time.sleep(0.02)
            if not identities_path.exists():
                cancellation_errors.append("native descendant did not start")
            # Popen retains this independently created owner's exact handle.
            owner.terminate()
            owner.wait(timeout=5)

        controller = threading.Thread(target=end_dummy_owner_after_descendants_start)
        controller.start()
        transport = closure.NativeMetadataTransport(timeout_seconds=25,
                                                     lease_owner=(owner.pid, owner_birth))
        try:
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", task), self.root, self.environment)
        finally:
            controller.join(timeout=20)
        self.assertFalse(controller.is_alive())
        self.assertFalse(cancellation_errors)
        identities = json.loads(identities_path.read_text())
        for name in ("child", "grandchild"):
            self.assertFalse(process_matches_creation_time(identities[name + "Pid"], identities[name + "Birth"]))
        evidence = transport.receipts[0]
        self.assertEqual(evidence["reason"], "lease_owner_exited")
        self.assertEqual(evidence["outcome"], "failed")
        self.assertFalse(evidence["processTree"]["terminal"])
        self.assertTrue(evidence["jobAccountingZero"])
        self.assertFalse(process_matches_creation_time(evidence["pid"], evidence["creationTime"]))

    def test_dead_dummy_owner_rejected_before_native_launch(self):
        from tools.jenkins.pilot.native.process_identity import popen_process_creation_time
        owner = subprocess.Popen((sys.executable, "-B", "-c", "pass"), cwd=self.root,
                                 env=self.environment, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        birth = popen_process_creation_time(owner)
        owner.wait(timeout=10)
        transport = closure.NativeMetadataTransport(lease_owner=(owner.pid, birth))
        with patch.object(closure, "create_atomic_kill_on_close_process") as launch:
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", "pass"), self.root, self.environment)
        launch.assert_not_called()
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])

    def test_live_pid_with_wrong_owner_birth_is_rejected_without_signalling_it(self):
        from tools.jenkins.pilot.native.process_identity import popen_process_creation_time
        owner = subprocess.Popen((sys.executable, "-B", "-c", "import time; time.sleep(120)"),
                                 cwd=self.root, env=self.environment,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        birth = popen_process_creation_time(owner)
        try:
            transport = closure.NativeMetadataTransport(lease_owner=(owner.pid, str(int(birth) + 1)))
            with patch.object(closure, "create_atomic_kill_on_close_process") as launch:
                with self.assertRaises(OSError):
                    transport((sys.executable, "-B", "-c", "pass"), self.root, self.environment)
            launch.assert_not_called()
            self.assertIsNone(owner.poll())
            self.assertFalse(transport.receipts[0]["processTree"]["terminal"])
        finally:
            owner.terminate()
            owner.wait(timeout=5)

    def test_success_also_converges_lingering_descendant(self):
        argv = (sys.executable, "-B", "-c",
                "import subprocess,sys; p=subprocess.Popen([sys.executable,'-B','-c','import time; time.sleep(120)'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL); "
                "from tools.jenkins.pilot.native.process_identity import process_creation_time; import json; print(json.dumps([p.pid,process_creation_time(p.pid)]),flush=True)")
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        result = transport(argv, self.root, self.environment)
        pid, birth = json.loads(result.stdout)
        self.assertEqual(result.returncode, 0)
        self.assertFalse(process_matches_creation_time(pid, birth))
        self.assertTrue(transport.receipts[0]["processTree"]["terminal"])

    def test_output_limit_fails_and_never_returns_truncated_metadata(self):
        transport = closure.NativeMetadataTransport(timeout_seconds=10, maximum_output_bytes=1024)
        with self.assertRaises(OSError):
            transport((sys.executable, "-B", "-c", "print('x'*100000)"), self.root, self.environment)
        self.assertEqual(transport.receipts[0]["outcome"], "failed")
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])

    def test_nonzero_exit_never_claims_completed_metadata(self):
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        result = transport((sys.executable, "-B", "-c", "raise SystemExit(7)"), self.root, self.environment)
        self.assertEqual(result.returncode, 7)
        receipt = transport.receipts[0]
        self.assertEqual(receipt["outcome"], "failed")
        self.assertTrue(receipt["pipeEOF"])
        self.assertTrue(receipt["jobAccountingZero"])
        self.assertFalse(receipt["processTree"]["terminal"])

    def test_receipt_callback_cannot_counterfeit_observed_native_exit(self):
        def counterfeit(receipt):
            receipt.update(exitCode=0, outcome="completed")
            receipt["processTree"]["terminal"] = True
        transport = closure.NativeMetadataTransport(timeout_seconds=10, record=counterfeit)
        result = transport((sys.executable, "-B", "-c", "raise SystemExit(7)"), self.root, self.environment)
        self.assertEqual(result.returncode, 7)
        self.assertEqual(transport.receipts[0]["exitCode"], 7)
        self.assertEqual(transport.receipts[0]["outcome"], "failed")
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])
        self.assertEqual(result.native_observation["exitCode"], 7)
        from tools.jenkins.pilot.managed_cargo import terminal_verified
        self.assertTrue(terminal_verified(result.native_observation))

    def test_resume_failure_cannot_claim_terminal_success(self):
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        with patch.object(closure, "resume_popen_process", side_effect=OSError("resume failed")):
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", "print('never')"), self.root, self.environment)
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])

    def test_create_failure_records_no_process_or_terminal_success(self):
        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        with patch.object(closure, "create_atomic_kill_on_close_process", side_effect=OSError("create failed")):
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", "pass"), self.root, self.environment)
        self.assertIsNone(transport.receipts[0]["pid"])
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])

    def test_job_termination_error_cannot_claim_terminal_success(self):
        terminate = closure.terminate_and_close_process_job

        def fail_after_native_cleanup(handle):
            terminate(handle)
            raise OSError("termination evidence unavailable")

        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        with patch.object(closure, "terminate_and_close_process_job", side_effect=fail_after_native_cleanup):
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", "print('result')"), self.root, self.environment)
        self.assertFalse(transport.receipts[0]["processTree"]["terminal"])

    def test_pipe_read_error_fails_after_owned_job_cleanup(self):
        launch = closure.create_atomic_kill_on_close_process

        class BrokenPipe:
            def __init__(self, original):
                self.original = original
                self.buffer = self

            def read1(self, _size):
                raise OSError("pipe read failed")

            def close(self):
                self.original.close()

        def broken_stdout(*args, **kwargs):
            process, job = launch(*args, **kwargs)
            process.stdout = BrokenPipe(process.stdout)
            return process, job

        transport = closure.NativeMetadataTransport(timeout_seconds=10)
        with patch.object(closure, "create_atomic_kill_on_close_process", side_effect=broken_stdout):
            with self.assertRaises(OSError):
                transport((sys.executable, "-B", "-c", "import time; time.sleep(120)"), self.root, self.environment)
        evidence = transport.receipts[0]
        self.assertFalse(evidence["processTree"]["terminal"])
        self.assertFalse(process_matches_creation_time(evidence["pid"], evidence["creationTime"]))


class ClosureBindingTests(unittest.TestCase):
    def test_declared_command_is_fixed(self):
        self.assertEqual(closure.DECLARED_COMMANDS,
                         (("cargo", "check", "--locked", "-p", "zircon_reflect_derive"),))

    def test_capture_rejects_change_between_planning_and_sealing(self):
        from types import SimpleNamespace
        planned = ({"src/lib.rs": "a" * 64}, {}, {"Cargo.toml": "b" * 64})
        sealed = SimpleNamespace(manifest={"entries": [{"path": "src/lib.rs", "sha256": "c" * 64}]})
        with self.assertRaisesRegex(closure.PilotError, "differ"):
            closure.verify_captured_inputs(sealed, planned[0])

    def test_external_capture_must_match_every_planned_sibling_file(self):
        from types import SimpleNamespace
        primary = {"Cargo.toml": "a" * 64}
        external = {Path("E:/Git/zr_vm"): {"rust/Cargo.toml": "b" * 64}}
        sealed = SimpleNamespace(manifest={
            "entries": [{"path": "Cargo.toml", "sha256": "a" * 64}],
            "external": [{"mount": "zr_vm", "entries": [
                {"path": "rust/Cargo.toml", "sha256": "c" * 64}]}]})
        with self.assertRaisesRegex(closure.PilotError, "external"):
            closure.verify_captured_inputs(sealed, primary, external)
        sealed.manifest["external"][0]["entries"][0]["sha256"] = "b" * 64
        closure.verify_captured_inputs(sealed, primary, external)
        sealed.manifest["external"].append({"mount": "extra", "entries": []})
        with self.assertRaisesRegex(closure.PilotError, "external"):
            closure.verify_captured_inputs(sealed, primary, external)

    def test_missing_external_capture_is_rejected(self):
        from types import SimpleNamespace
        sealed = SimpleNamespace(manifest={"entries": [{"path": "Cargo.toml", "sha256": "a" * 64}]})
        with self.assertRaisesRegex(closure.PilotError, "external"):
            closure.verify_captured_inputs(sealed, {"Cargo.toml": "a" * 64},
                                           {Path("E:/Git/zr_vm"): {"Cargo.toml": "b" * 64}})


class IndependentPlannerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = temporary_directory("closure-inputs-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.write("Cargo.toml", '[workspace]\nmembers=["derive", "dep", "inactive"]\n')
        self.write("Cargo.lock", "version = 4\n")
        for package in ("derive", "dep", "inactive"):
            name = "zircon_reflect_derive" if package == "derive" else package
            self.write(package + "/Cargo.toml", '[package]\nname="' + name + '"\nversion="0.1.0"\n')
            self.write(package + "/src/lib.rs", "// " + package)
        self.write("dep/data/schema.json", '{"compileInput":true}')
        self.inventory = {p.relative_to(self.repo).as_posix() for p in self.repo.rglob("*") if p.is_file()}
        self.metadata = {"packages": [
            {"id": name, "name": "zircon_reflect_derive" if name == "derive" else name,
             "source": None, "manifest_path": str(self.repo / name / "Cargo.toml")}
            for name in ("derive", "dep", "inactive")],
            "resolve": {"nodes": [{"id": "derive", "dependencies": ["dep"]},
                {"id": "dep", "dependencies": []}, {"id": "inactive", "dependencies": []}]}}

    def write(self, relative, value):
        path = self.repo / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(value, encoding="utf-8")

    def test_full_active_path_dependency_and_inactive_workspace_manifest(self):
        manifests, topology = inputs.manifest_closure(self.repo)
        with patch.object(inputs, "live_git_paths", return_value=self.inventory):
            selected = inputs.select_metadata_inputs(self.repo, self.metadata, manifests)
        names = {name for owner, name in selected}
        self.assertIn("dep/data/schema.json", names)
        self.assertIn("dep/src/lib.rs", names)
        self.assertIn("inactive/Cargo.toml", names)
        self.assertNotIn("inactive/src/lib.rs", names)
        self.assertIn("Cargo.lock", topology)

    def test_incomplete_graph_and_undeclared_external_package_fail(self):
        manifests, _ = inputs.manifest_closure(self.repo)
        self.metadata["resolve"]["nodes"][0]["dependencies"] = ["missing"]
        with self.assertRaises(closure.PilotError):
            inputs.select_metadata_inputs(self.repo, self.metadata, manifests)
        self.metadata["resolve"]["nodes"][0]["dependencies"] = ["dep"]
        self.metadata["packages"][1]["manifest_path"] = str(self.root / "escape/Cargo.toml")
        with self.assertRaises(closure.PilotError):
            inputs.select_metadata_inputs(self.repo, self.metadata, manifests)

    def test_deleted_active_source_remains_in_sealed_selection(self):
        manifests, _ = inputs.manifest_closure(self.repo)
        (self.repo / "dep/src/lib.rs").unlink()
        with patch.object(inputs, "live_git_paths", return_value=self.inventory):
            selected = inputs.select_metadata_inputs(self.repo, self.metadata, manifests)
        self.assertIn((self.repo, "dep/src/lib.rs"), selected)
        self.assertIsNone(selected[self.repo, "dep/src/lib.rs"])
        from types import SimpleNamespace
        sealed = SimpleNamespace(manifest={"entries": [{"path": "deleted.rs", "sha256": None}]})
        closure.verify_captured_inputs(sealed, {"deleted.rs": None})

    def test_public_transport_cache_and_configuration_drift(self):
        from contextlib import nullcontext
        from types import SimpleNamespace
        calls, cache = [], {}
        def transport(argv, source, environment):
            calls.append(tuple(argv))
            result = subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata), "")
            result.native_observation = {"test": "actual mocked transport"}
            return result
        with (patch.object(inputs, "live_git_paths", return_value=self.inventory),
              patch.object(inputs, "metadata_execution", side_effect=lambda *_args, **_kwargs: nullcontext(
                  SimpleNamespace(command=("fixed-cargo", "metadata", "--locked"), environment={},
                      complete=lambda observation: self.assertEqual(observation, {"test": "actual mocked transport"}))))):
            first = inputs.plan_inputs(self.repo, self.root / "metadata", transport=transport, metadata_cache=cache)
            self.assertEqual(inputs.plan_inputs(self.repo, self.root / "metadata", transport=transport, metadata_cache=cache), first)
            self.assertEqual(len(calls), 1)
            self.write("dep/Cargo.toml", '[package]\nname="dep"\nversion="0.1.1"\n')
            second = inputs.plan_inputs(self.repo, self.root / "metadata", transport=transport, metadata_cache=cache)
            self.assertNotEqual(first, second)
            self.assertEqual(len(calls), 2)

    def test_target_alias_and_command_change_fail_before_transport(self):
        transport = unittest.mock.Mock()
        with self.assertRaises(ValueError):
            inputs.plan_inputs(self.repo, Path("C:/bad-target"), transport=transport)
        with self.assertRaises(closure.PilotError):
            inputs.plan_inputs(self.repo, self.root / "metadata", (("cargo", "check"),), transport=transport)
        transport.assert_not_called()



if __name__ == "__main__":
    unittest.main()
