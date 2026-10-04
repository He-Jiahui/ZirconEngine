"""Exercise Windows ownership and terminal-proof seams without retired tooling."""
from contextlib import contextmanager
from pathlib import Path
import os
import json
import sys
import threading
import unittest
import uuid

from tools.jenkins.pilot.native.process_identity import (
    process_creation_time, popen_process_creation_time, process_matches_creation_time,
)
from tools.jenkins.pilot.native.process_supervisor import ProcessSupervisor
from tools.jenkins.pilot.native.windows_job_process import (
    create_atomic_kill_on_close_process, resume_popen_process,
    terminate_and_close_process_job,
    wait_for_process_job_terminal,
)


@contextmanager
def native_directory():
    path = Path(r"D:\cargo-targets\zircon-jenkins") / ("tests-" + uuid.uuid4().hex)
    path.mkdir(parents=True)
    yield path  # Preserve physical evidence for inspection.


@unittest.skipUnless(os.name == "nt", "requires native Windows Job Objects")
class NativeProcessTests(unittest.TestCase):
    def test_job_terminal_helpers_wait_for_exact_lingering_descendant_signal(self):
        for natural in (False, True):
            with self.subTest(natural=natural), native_directory() as root:
                duration = 0.5 if natural else 60
                code = (
                    "import subprocess,sys,json;from tools.jenkins.pilot.native.process_identity import process_creation_time;"
                    f"p=subprocess.Popen([sys.executable,'-B','-c','import time;time.sleep({duration})'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL);"
                    "print(json.dumps([p.pid,process_creation_time(p.pid)]),flush=True)"
                )
                process, job = create_atomic_kill_on_close_process((sys.executable, "-B", "-c", code), cwd=root, env=dict(os.environ))
                try:
                    resume_popen_process(process)
                    self.assertEqual(process.wait(15), 0)
                    pid, birth = json.loads(process.stdout.read())
                    if natural:
                        wait_for_process_job_terminal(job, timeout_seconds=15)
                    else:
                        terminate_and_close_process_job(job)
                        job = None
                    self.assertFalse(process_matches_creation_time(pid, birth))
                finally:
                    if job:
                        terminate_and_close_process_job(job)
                    process.close()

    def test_suspended_atomic_launch_has_exact_identity_before_first_instruction(self):
        with native_directory() as root:
            marker = root / "executed.txt"
            process, job = create_atomic_kill_on_close_process(
                (sys.executable, "-B", "-c", "from pathlib import Path;import sys;Path(sys.argv[1]).write_text('ran')", str(marker)),
                cwd=root, env=dict(os.environ),
            )
            try:
                birth = popen_process_creation_time(process)
                self.assertEqual(process_creation_time(process.pid), birth)
                self.assertFalse(marker.exists())
                self.assertFalse(process_matches_creation_time(process.pid, str(int(birth) + 1)))
                resume_popen_process(process)
                self.assertEqual(process.wait(timeout=15), 0)
                self.assertEqual(marker.read_text(), "ran")
            finally:
                terminate_and_close_process_job(job)
                process.close()

    def test_root_exit_does_not_claim_terminal_while_descendant_holds_pipes(self):
        with native_directory() as root:
            supervisor = ProcessSupervisor(job_terminal_timeout=0.2)
            script = "import subprocess,sys;subprocess.Popen([sys.executable,'-B','-c','import time;time.sleep(60)']);print('root exited',flush=True)"
            process = supervisor.start((sys.executable, "-B", "-c", script), cwd=root,
                                       env=dict(os.environ), owner=("test", "descendant", 0, 0))
            try:
                with self.assertRaises(TimeoutError):
                    process.wait(timeout=15)
                self.assertIsNone(process.terminal_scope)
                self.assertTrue(process._job_terminal_proven)
            finally:
                supervisor.close()

    def test_pipe_read_failure_cannot_be_accepted_as_eof(self):
        with native_directory() as root:
            supervisor = ProcessSupervisor()
            process = supervisor.start((sys.executable, "-B", "-c", "print('complete')"),
                                       cwd=root, env=dict(os.environ), owner=("test", "pipe-error", 0, 0))
            try:
                process._pipe_errors["stdout"] = OSError("injected read failure")
                with self.assertRaises(OSError):
                    process.wait(timeout=15)
                self.assertIsNone(process.terminal_scope)
            finally:
                supervisor.close()

    def test_cancel_then_wait_proves_root_job_and_both_eofs(self):
        with native_directory() as root:
            supervisor = ProcessSupervisor()
            process = supervisor.start((sys.executable, "-B", "-c", "import time;time.sleep(60)"),
                                       cwd=root, env=dict(os.environ), owner=("test", "cancel", 0, 0))
            try:
                process.terminate(reason="test")
                self.assertNotEqual(process.wait(timeout=15), 0)
                self.assertEqual(process.terminal_scope, "windows_job")
                self.assertTrue(all(process._pipe_eof.values()))
            finally:
                supervisor.close()

    def test_failed_durable_output_callback_denies_acceptance_after_real_eof(self):
        with native_directory() as root:
            writes = []
            def failing_write(owner, channel, data):
                writes.append((channel, data))
                raise OSError("injected disk-full durable log write")
            supervisor = ProcessSupervisor(output_callback=failing_write)
            process = supervisor.start((sys.executable, "-B", "-c", "import sys;print('stdout');print('stderr',file=sys.stderr)"),
                                       cwd=root, env=dict(os.environ), owner=("test", "write-error", 0, 0))
            try:
                with self.assertRaises(OSError):
                    process.wait(timeout=15)
                self.assertEqual(process.process.poll(), 0)
                self.assertTrue(process._job_terminal_proven)
                self.assertTrue(all(process._pipe_eof.values()))
                self.assertEqual(set(process._pipe_errors), {"stdout", "stderr"})
                self.assertTrue(writes)
                self.assertIsNone(process.terminal_scope)
            finally:
                supervisor.close()

    def test_close_waits_for_atomic_launch_registration_then_kills_owned_root(self):
        with native_directory() as root:
            launched, release = threading.Event(), threading.Event()
            close_started, closed = threading.Event(), threading.Event()
            results, failures = [], []
            def paused_launch(*args, **kwargs):
                process, job = create_atomic_kill_on_close_process(*args, **kwargs)
                launched.set()
                if not release.wait(15):
                    terminate_and_close_process_job(job)
                    process.close()
                    raise TimeoutError("test did not release suspended atomic root")
                return process, job
            supervisor = ProcessSupervisor(atomic_popen=paused_launch)
            def start():
                try:
                    results.append(supervisor.start((sys.executable, "-B", "-c", "import time;time.sleep(60)"),
                                                   cwd=root, env=dict(os.environ), owner=("test", "close-race", 0, 0)))
                except BaseException as error:
                    failures.append(error)
            def close():
                close_started.set()
                supervisor.close()
                closed.set()
            starter = threading.Thread(target=start)
            closer = threading.Thread(target=close)
            starter.start()
            try:
                self.assertTrue(launched.wait(15))
                closer.start()
                self.assertTrue(close_started.wait(15))
                self.assertFalse(closed.wait(0.2))
                release.set()
                starter.join(15)
                closer.join(15)
                self.assertFalse(failures)
                self.assertTrue(closed.is_set())
                self.assertEqual(len(results), 1)
                self.assertIsNotNone(results[0].process.poll())
                self.assertTrue(results[0]._job_terminal_proven)
            finally:
                release.set()
                starter.join(15)
                if closer.ident is not None:
                    closer.join(15)
                supervisor.close()


if __name__ == "__main__":
    unittest.main()
