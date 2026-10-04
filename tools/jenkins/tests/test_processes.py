from __future__ import annotations

import sys
import tempfile
import time
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.jenkins.contracts import JenkinsError
from tools.jenkins.processes import NativeJob, ProcessIdentity, ProcessRegistry, TerminationProof, identity_matches
from tools.jenkins.state import State


class ProcessIdentityTests(unittest.TestCase):
    def test_requires_positive_pid_and_birth_time(self):
        with self.assertRaises(JenkinsError):
            ProcessIdentity(0, "birth")
        with self.assertRaises(JenkinsError):
            ProcessIdentity(4, "")

    def test_pid_reuse_is_rejected(self):
        identity = ProcessIdentity(41, "old")
        with patch("tools.jenkins.processes.identity.process_alive", return_value=True), patch(
            "tools.jenkins.processes.identity._creation_time", return_value="new"
        ):
            self.assertFalse(identity_matches(identity))

    def test_identity_does_not_store_raw_command_line(self):
        identity = ProcessIdentity(41, "old", "python", "digest")
        self.assertNotIn("secret", str(identity.to_dict()))


class NativeJobTests(unittest.TestCase):
    @unittest.skipUnless(__import__("os").name == "nt", "requires Windows Job Object")
    def test_phase_cleanup_proves_tree_and_eof_after_root_exit(self):
        with tempfile.TemporaryDirectory() as temp:
            script = "import subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); print('complete',flush=True)"
            job = NativeJob.launch((sys.executable, "-B", "-c", script),
                                   cwd=Path(temp), log_dir=Path(temp) / "logs")
            try:
                self.assertEqual(0, job.wait(timeout_seconds=10, cleanup_descendants=True))
                self.assertTrue(job.descendant_cleanup["requested"])
                self.assertGreater(job.descendant_cleanup["activeProcessesAfterRootExit"], 0)
                self.assertTrue(job.proof().complete)
                self.assertEqual(0, job.proof().process_exit_code)
                self.assertIn(b"complete", (Path(temp) / "logs/stdout.log").read_bytes())
            finally:
                job.close()

    def test_logs_are_visible_before_the_owned_process_exits(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            job = NativeJob.launch((sys.executable, "-B", "-c", "import time; print('live-log',flush=True); time.sleep(3)"),
                                   cwd=root, log_dir=root / "logs")
            try:
                log = root / "logs/stdout.log"
                deadline = time.monotonic() + 2
                while time.monotonic() < deadline and b"live-log" not in log.read_bytes():
                    time.sleep(.02)
                self.assertIn(b"live-log", log.read_bytes())
                self.assertIsNone(job.poll())
                job.wait(timeout_seconds=10)
                self.assertTrue(job.proof().complete)
            finally:
                job.close()

    @unittest.skipUnless(__import__("os").name == "nt", "requires Windows Job Object")
    def test_terminate_retained_job_after_root_exits(self):
        with tempfile.TemporaryDirectory() as temp:
            script = "import subprocess,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); print('root-exit', flush=True)"
            job = NativeJob.launch((sys.executable, "-c", script), cwd=Path(temp), log_dir=Path(temp) / "logs")
            try:
                job.process.wait(timeout=10)
                with self.assertRaises(TimeoutError):
                    job.wait(timeout_seconds=.2)
                self.assertFalse(job.proof().complete)
                self.assertTrue(job.terminate(timeout_seconds=10).complete)
                registry = ProcessRegistry(State(Path(temp) / "state.sqlite3"))
                registry.register(job, execution_id="residual-child", owner="test")
                terminal = registry.record_terminal(job)
                self.assertEqual(terminal.payload["status"], "terminal")
            finally:
                job.close()

    def test_launch_wait_and_proof(self):
        with tempfile.TemporaryDirectory() as temp:
            with NativeJob.launch((sys.executable, "-c", "print('ok')"), cwd=Path(temp)) as job:
                self.assertGreater(job.identity.pid, 0)
                self.assertEqual(job.wait(timeout_seconds=10), 0)
                proof = job.proof(requested=True)
                self.assertTrue(proof.children_gone)
                self.assertTrue(proof.complete)

    def test_invalid_working_directory_is_rejected(self):
        with self.assertRaises(JenkinsError):
            NativeJob.launch((sys.executable, "-c", "pass"), cwd="does-not-exist")

    def test_registry_only_accepts_job_produced_identity_and_proof(self):
        with tempfile.TemporaryDirectory() as temp:
            with NativeJob.launch((sys.executable, "-c", "pass"), cwd=Path(temp)) as job:
                registry = ProcessRegistry(State(Path(temp) / "state.sqlite3"))
                record = registry.register(job, execution_id="exec-1", owner="test")
                self.assertEqual(record.payload["pid"], job.identity.pid)
                job.wait(timeout_seconds=10)
                updated = registry.record_terminal(job, job.proof())
                self.assertIn("terminalProof", updated.payload)

    @unittest.skipUnless(__import__("os").name == "nt", "requires Windows Job Object")
    def test_root_exit_does_not_prove_job_terminal_while_child_lives(self):
        with tempfile.TemporaryDirectory() as temp:
            script = "import subprocess,sys,time; subprocess.Popen([sys.executable,'-c','import time; time.sleep(3)']); print('root-exit', flush=True)"
            job = NativeJob.launch((sys.executable, "-c", script), cwd=Path(temp))
            try:
                with self.assertRaises(subprocess.TimeoutExpired):
                    job.wait(timeout_seconds=0.1)
                proof = job.proof(requested=False)
                self.assertFalse(proof.complete)
                self.assertNotEqual(proof.active_processes, 0)
                job.terminate(timeout_seconds=10)
                self.assertTrue(job.proof(requested=True).complete)
            finally:
                job.close()


if __name__ == "__main__":
    unittest.main()
