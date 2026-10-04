from pathlib import Path
import json
import os
import subprocess
import threading
import time
import unittest
import sys
from unittest.mock import patch

from tools.jenkins.pilot.native.process_identity import process_matches_creation_time
from tools.jenkins.pilot.contracts import PilotError, RequestIdentity
from tools.jenkins.pilot.runner import run, run_directory
from tools.jenkins.pilot.snapshot import capture
from tools.jenkins.pilot.storage import ManagedStorage
from tools.jenkins.pilot.tests.fixtures import temporary_directory


class RunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = temporary_directory("jenkins-runner-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        for args in (["init", "--quiet"], ["config", "user.name", "fixture"], ["config", "user.email", "fixture@localhost"]):
            subprocess.run(["git", "-C", str(self.repo), *args], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        subprocess.run(["git", "-C", str(self.repo), "config", "core.autocrlf", "false"], check=True)
        (self.repo / "source.py").write_bytes(b"value = 1\n")
        subprocess.run(["git", "-C", str(self.repo), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.repo), "commit", "--quiet", "-m", "fixture"], check=True)
        self.sealed = capture(self.repo, ManagedStorage(self.root), paths=["."])

    def test_static_runner_checks_materialized_inputs_and_owned_tree(self):
        identity = RequestIdentity("session", "static", "attempt", 1, self.sealed.input_hash, "python-static")
        receipt = run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle,
                      identity=identity, job="pilot", build_number=1)
        self.assertEqual(receipt["outcome"], "passed")
        self.assertEqual(receipt["sourceBefore"], receipt["sourceAfter"])
        self.assertTrue(receipt["processTree"]["terminal"])
        self.assertEqual(receipt["processTree"]["scope"], "windows_job")

    def test_run_directory_requires_full_native_file_identity(self):
        from dataclasses import replace
        from tools.jenkins.pilot.native.paths import WorkerStorage
        original = WorkerStorage.create_directory
        def different_identity(backend, relative):
            created = original(backend, relative)
            native = created.native_identity
            changed_id = bytes([native.file_id[0] ^ 1]) + native.file_id[1:]
            return replace(created, native_identity=replace(native, file_id=changed_id))
        identity = RequestIdentity("session", "replaced-directory", "attempt", 1,
                                   self.sealed.input_hash, "python-static")
        with patch.object(WorkerStorage, "create_directory", different_identity), \
                patch("tools.jenkins.pilot.runner.materialize") as restore:
            with self.assertRaises(PilotError):
                run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle,
                    identity=identity, job="pilot", build_number=1)
            restore.assert_not_called()

    def test_managed_cargo_uses_retained_context_environment(self):
        from unittest.mock import Mock
        identity = RequestIdentity("session", "cargo-env", "attempt", 1, self.sealed.input_hash, "managed-cargo-check-v2")
        child = "import os,json; keys=['CARGO_TARGET_DIR','RUSTC_WRAPPER']; print(json.dumps({k:os.environ.get(k) for k in keys}))"
        command = [sys.executable, "-B", "-c", child]
        context = Mock(command=command, environment=dict(os.environ, CARGO_TARGET_DIR=str(self.root / "managed-target"), RUSTC_WRAPPER=""))
        guards = {"CARGO_TARGET_DIR": str(self.root / "inherited-target"), "RUSTC_WRAPPER": "foreign-wrapper"}
        with patch.dict(os.environ, guards), patch("tools.jenkins.pilot.managed_cargo.prepare_execution", return_value=context), patch("tools.jenkins.pilot.managed_cargo.collect_evidence", return_value={"accepted": True}):
            receipt = run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle, identity=identity, job="pilot", build_number=4)
            self.assertEqual(os.environ["CARGO_TARGET_DIR"], guards["CARGO_TARGET_DIR"])
        observed = json.loads((run_directory(self.root, identity) / "output.log").read_text())
        self.assertEqual(observed["CARGO_TARGET_DIR"], str(self.root / "managed-target"))
        self.assertEqual(observed["RUSTC_WRAPPER"], "")
        self.assertEqual(receipt["outcome"], "passed")
        context.finalize.assert_called_once()
        context.close.assert_called_once()
        observation = context.finalize.call_args.args[0]
        self.assertEqual(observation["nativeJobEvidence"]["records"]["afterClose"]["activeProcesses"], 0)
        self.assertTrue(observation["processTree"]["pipeEOF"])

    def test_cancel_stops_owned_child_and_parent(self):
        identity = RequestIdentity("session", "cancel", "attempt", 1, self.sealed.input_hash, "fault-probe")
        cancellation = threading.Event()
        results = []
        thread = threading.Thread(target=lambda: results.append(run(
            root=self.root, repo_root=self.repo, bundle=self.sealed.bundle, identity=identity,
            job="pilot", build_number=2, cancellation=cancellation, probe_seconds=120,
        )))
        thread.start()
        probe = run_directory(self.root, identity) / "probe.json"
        try:
            deadline = time.monotonic() + 120
            while not probe.exists() and thread.is_alive() and time.monotonic() < deadline:
                time.sleep(0.1)
            self.assertTrue(probe.exists(), "owned child never started")
            processes = json.loads(probe.read_text())
        finally:
            cancellation.set()
            thread.join(timeout=120)
        self.assertFalse(thread.is_alive())
        self.assertEqual(results[0]["outcome"], "cancelled")
        self.assertTrue(results[0]["processTree"]["terminal"])
        for name in ("parent", "child"):
            self.assertFalse(process_matches_creation_time(processes[name + "Pid"], processes[name + "CreationTime"]))

    def test_metadata_write_failure_closes_already_started_tree(self):
        from tools.jenkins.pilot.native.process_supervisor import ProcessSupervisor
        original_start = ProcessSupervisor.start
        started = []
        def remember(instance, *args, **kwargs):
            process = original_start(instance, *args, **kwargs)
            started.append(process)
            return process
        identity = RequestIdentity("session", "write-failure", "attempt", 1, self.sealed.input_hash, "fault-probe")
        with patch.object(ProcessSupervisor, "start", remember), patch.object(ManagedStorage, "atomic_write", side_effect=OSError("disk full")):
            with self.assertRaises(OSError):
                run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle, identity=identity,
                    job="pilot", build_number=3, probe_seconds=120)
        self.assertEqual(len(started), 1)
        self.assertFalse(process_matches_creation_time(started[0].pid, started[0].creation_time))

    def _receipt_run(self, name, command, *, template="python-static", **kwargs):
        identity = RequestIdentity("session", name, "attempt", 1, self.sealed.input_hash, template)
        from unittest.mock import Mock
        context = Mock(command=command, environment=dict(os.environ))
        with patch("tools.jenkins.pilot.runner.template_command", return_value=command), patch("tools.jenkins.pilot.managed_cargo.prepare_execution", return_value=context):
            receipt = run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle,
                          identity=identity, job="pilot", build_number=5, **kwargs)
        persisted = json.loads((run_directory(self.root, identity) / "receipt.json").read_bytes())
        self.assertEqual(json.loads(json.dumps(receipt)), persisted)
        self.assertEqual("failed", receipt["outcome"])
        self.assertTrue(receipt["terminalErrors"])
        return receipt

    @unittest.skipUnless(os.name == "nt", "native Windows Job evidence")
    def test_root_zero_with_sleeping_grandchild_timeout_archives_failure_and_actual_zero(self):
        grandchild = "import time; time.sleep(120)"
        child = f"import subprocess,sys; subprocess.Popen([sys.executable,'-B','-c',{grandchild!r}])"
        root = f"import subprocess,sys; subprocess.run([sys.executable,'-B','-c',{child!r}],check=True)"
        started = time.monotonic()
        receipt = self._receipt_run("grandchild-timeout", [sys.executable, "-B", "-c", root])
        self.assertGreaterEqual(time.monotonic() - started, 8)
        self.assertEqual(0, receipt["exitCode"])
        self.assertEqual("owned_job_wait_failed", receipt["reason"])
        self.assertEqual("TimeoutError", receipt["terminalErrors"][0]["type"])
        self.assertTrue(receipt["processTree"]["terminal"])
        records = receipt["nativeJobEvidence"]["records"]
        self.assertGreater(records["beforeWait"]["activeProcesses"], 0)
        self.assertEqual(0, records["afterWait"]["activeProcesses"])
        self.assertEqual(0, records["afterClose"]["activeProcesses"])
        self.assertTrue(records["beforeWait"]["members"])
        self.assertTrue(all(member["member"] for member in records["beforeWait"]["members"]))
        self.assertTrue(receipt["processTree"]["pipeEOF"])

    def test_nonempty_output_has_actual_eof_and_passes(self):
        identity = RequestIdentity("session", "full-output", "attempt", 1, self.sealed.input_hash, "python-static")
        command = [sys.executable, "-B", "-c", "import sys; print('stdout complete'); print('stderr complete',file=sys.stderr)"]
        from unittest.mock import Mock
        context = Mock(command=command, environment=dict(os.environ))
        with patch("tools.jenkins.pilot.runner.template_command", return_value=command), patch("tools.jenkins.pilot.managed_cargo.prepare_execution", return_value=context):
            receipt = run(root=self.root, repo_root=self.repo, bundle=self.sealed.bundle,
                          identity=identity, job="pilot", build_number=6)
        self.assertEqual("passed", receipt["outcome"])
        self.assertTrue(receipt["processTree"]["readersFinished"])
        self.assertTrue(receipt["processTree"]["pipeEOF"])
        output = (run_directory(self.root, identity) / "output.log").read_bytes()
        self.assertIn(b"stdout complete", output)
        self.assertIn(b"stderr complete", output)

    def test_wait_exception_retains_root_exit_and_failed_receipt(self):
        from tools.jenkins.pilot.native.process_supervisor import ManagedProcess
        with patch.object(ManagedProcess, "wait", side_effect=OSError("injected wait failure")):
            receipt = self._receipt_run("wait-error", [sys.executable, "-B", "-c", "print('done')"])
        self.assertEqual(0, receipt["exitCode"])
        self.assertEqual("wait", receipt["terminalErrors"][0]["stage"])

    @unittest.skipUnless(os.name == "nt", "native Windows Job evidence")
    def test_native_wait_query_error_releases_duplicate_before_closing_inherited_pipes(self):
        from tools.jenkins.pilot.native.process_identity import popen_process_creation_time
        record = self.root / "grandchild-native-identity.json"
        grandchild = "import time; time.sleep(120)"
        child = (
            "import subprocess,sys,json; from pathlib import Path; "
            "from tools.jenkins.pilot.native.process_identity import popen_process_creation_time; "
            f"p=subprocess.Popen([sys.executable,'-B','-c',{grandchild!r}]); "
            f"Path({str(record)!r}).write_text(json.dumps({{'pid':p.pid,'creationTime':popen_process_creation_time(p)}}))"
        )
        root = f"import subprocess,sys; subprocess.run([sys.executable,'-B','-c',{child!r}],check=True)"
        started = time.monotonic()
        with patch("tools.jenkins.pilot.native.process_supervisor.wait_for_process_job_terminal",
                   side_effect=OSError("injected native wait accounting failure")):
            receipt = self._receipt_run("native-wait-query-error", [sys.executable, "-B", "-c", root])
        self.assertLess(time.monotonic() - started, 60, "receipt must not wait for the sleeping grandchild")
        self.assertEqual(0, receipt["exitCode"])
        self.assertEqual("OSError", receipt["terminalErrors"][0]["type"])
        self.assertFalse(receipt["processTree"]["terminal"])
        self.assertEqual("unverified", receipt["processTree"]["scope"])
        self.assertTrue(receipt["processTree"]["pipeEOF"])
        self.assertTrue(receipt["processTree"]["readersFinished"])
        records = receipt["nativeJobEvidence"]["records"]
        self.assertGreater(records["beforeClose"]["activeProcesses"], 0)
        self.assertIsNone(records["afterClose"]["activeProcesses"])
        identity = json.loads(record.read_bytes())
        self.assertFalse(process_matches_creation_time(identity["pid"], identity["creationTime"]))

    def test_owned_termination_error_archives_failure(self):
        from tools.jenkins.pilot.native.process_supervisor import ManagedProcess
        original = ManagedProcess.terminate
        def terminate(process, **kwargs):
            original(process, **kwargs)
            raise OSError("injected owned termination reporting failure")
        cancel = threading.Event(); cancel.set()
        with patch.object(ManagedProcess, "terminate", terminate):
            receipt = self._receipt_run("termination-error", [sys.executable, "-B", "-c", "import time; time.sleep(120)"], cancellation=cancel)
        self.assertEqual("execution", receipt["terminalErrors"][0]["stage"])

    def test_native_query_failure_is_unverified_and_archived(self):
        from tools.jenkins.pilot.process_evidence import JobEvidence
        with patch.object(JobEvidence, "_accounting", side_effect=OSError("injected accounting query failure")):
            receipt = self._receipt_run("query-error", [sys.executable, "-B", "-c", "print('done')"])
        self.assertEqual(0, receipt["exitCode"])
        self.assertFalse(receipt["processTree"]["terminal"])
        self.assertEqual("unverified", receipt["processTree"]["scope"])

    def test_native_pid_list_failure_is_unverified_and_archived(self):
        from tools.jenkins.pilot.process_evidence import JobEvidence
        with patch.object(JobEvidence, "_members", side_effect=OSError("injected PID list failure")):
            receipt = self._receipt_run("list-error", [sys.executable, "-B", "-c", "print('done')"])
        self.assertFalse(receipt["processTree"]["terminal"])

    def test_reader_exit_without_eof_does_not_pass(self):
        from tools.jenkins.pilot.process_evidence import ObservedPipe
        with patch.object(ObservedPipe, "read1", side_effect=OSError("injected pipe read failure")):
            receipt = self._receipt_run("pipe-error", [sys.executable, "-B", "-c", "print('done')"])
        self.assertTrue(receipt["processTree"]["readersFinished"])
        self.assertFalse(receipt["processTree"]["pipeEOF"])
        self.assertEqual("owned_job_wait_failed", receipt["reason"])
        self.assertTrue(any(error["stage"] == "output_capture" for error in receipt["terminalErrors"]))

    def test_collector_exception_does_not_discard_failure_receipt(self):
        with patch("tools.jenkins.pilot.managed_cargo.collect_evidence", side_effect=RuntimeError("injected collector failure")):
            receipt = self._receipt_run("collector-error", [sys.executable, "-B", "-c", "print('done')"], template="managed-cargo-check-v2")
        self.assertEqual(0, receipt["exitCode"])
        self.assertFalse(receipt["managedCargo"]["accepted"])
        self.assertFalse(receipt["managedCargo"]["formalAcceptance"])
        self.assertEqual("managed_cargo_collector", receipt["terminalErrors"][0]["stage"])


if __name__ == "__main__":
    unittest.main()
