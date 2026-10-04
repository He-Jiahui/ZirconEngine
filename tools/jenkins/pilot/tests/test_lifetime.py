from pathlib import Path
import json
import os
import subprocess
import sys
import time
import unittest
from unittest.mock import patch
from tools.jenkins.pilot.lifetime import launch_owned, stop_owned, wait_runtime_started
from tools.jenkins.pilot.native.windows_job_process import create_atomic_kill_on_close_process, terminate_and_close_process_job
from tools.jenkins.pilot.tests.test_native_process import native_directory
from tools.jenkins.pilot.native.process_identity import process_creation_time, popen_process_creation_time, process_matches_creation_time, wait_for_process_exit


@unittest.skipUnless(os.name == "nt", "requires native Windows Job Objects")
class LifetimeTests(unittest.TestCase):
    def test_missing_java_start_marker_never_consumes_readiness_budget(self):
        with native_directory() as root:
            record = {"pid": os.getpid(), "creationTime": process_creation_time(os.getpid()),
                      "runtimeStartPath": str(root / "runtime-start-missing.json"), "logPath": str(root / "runtime.log")}
            with self.assertRaisesRegex(TimeoutError, "asset preparation"):
                wait_runtime_started(record, preparation_seconds=0)

    def test_java_start_marker_requires_exact_live_child_in_retained_job(self):
        from tools.jenkins.pilot.storage import ManagedStorage
        from tools.jenkins.pilot.contracts import canonical_json
        with native_directory() as root:
            process, job = create_atomic_kill_on_close_process((sys.executable, "-B", "-c", "pass"), cwd=root, env=dict(os.environ))
            birth = process_creation_time(os.getpid())
            record = {"pid": os.getpid(), "creationTime": birth, "keeperPid": os.getpid(), "keeperCreationTime": birth,
                      "keeperJobHandle": job, "runtimeAssets": {"fixture": "fixed"},
                      "runtimeStartPath": str(root / "runtime-start-exact.json"), "logPath": str(root / "runtime.log")}
            marker = {"schemaVersion": 1, "hostPid": os.getpid(), "hostCreationTime": birth, "childPid": process.pid,
                      "childCreationTime": popen_process_creation_time(process), "runtimeAssets": record["runtimeAssets"]}
            try:
                ManagedStorage(root).atomic_write("runtime-start-exact.json", canonical_json(marker))
                self.assertEqual(wait_runtime_started(record), marker)
                marker.update(childPid=os.getpid(), childCreationTime=birth)
                ManagedStorage(root).atomic_write("runtime-start-exact.json", canonical_json(marker))
                with self.assertRaisesRegex(ProcessLookupError, "outside the exact"):
                    wait_runtime_started(record)
            finally:
                terminate_and_close_process_job(job)
                process.wait(15)
                process.close()
    def _probe(self, path):
        deadline = time.monotonic() + 120
        while time.monotonic() < deadline:
            if path.exists():
                try:
                    return json.loads(path.read_bytes())
                except ValueError:
                    pass
            time.sleep(0.1)
        self.fail("owned descendants did not produce their startup proof")

    def test_hard_launcher_exit_keeps_a_durable_suspended_job_record(self):
        with native_directory() as temporary:
            root = Path(temporary)
            keeper = subprocess.Popen([sys.executable, "-B", "-c", "import time; time.sleep(180)"],
                                      creationflags=subprocess.CREATE_NO_WINDOW)
            launcher = None
            record = None
            script = """
import json, os, sys, time
from pathlib import Path
from tools.jenkins.pilot.lifetime import launch_owned
from tools.jenkins.pilot.storage import ManagedStorage
root=Path(os.environ['TEST_RUNTIME_ROOT'])
lease={'owner_pid':int(os.environ['TEST_KEEPER_PID']), 'owner_process_creation_time':os.environ['TEST_KEEPER_BIRTH']}
def persist(record):
    ManagedStorage(root).atomic_write('runtime-record.json',json.dumps(record).encode())
    while True: time.sleep(0.1)
command=[sys.executable,'-B','-c',"import sys,time;from pathlib import Path;Path(sys.argv[1]).write_text('executed');time.sleep(120)",str(root/'executed.txt')]
with (root/'runtime.log').open('wb') as stream:
    launch_owned(command,cwd=root,env=os.environ,stream=stream,lease=lease,on_record=persist)
"""
            env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONPATH=str(Path.cwd()),
                       TEST_RUNTIME_ROOT=str(root), TEST_KEEPER_PID=str(keeper.pid),
                       TEST_KEEPER_BIRTH=popen_process_creation_time(keeper), TMP=str(root), TEMP=str(root))
            try:
                launcher = subprocess.Popen([sys.executable, "-B", "-c", script], env=env,
                                            creationflags=subprocess.CREATE_NO_WINDOW)
                record = self._probe(root / "runtime-record.json")
                launcher.kill()
                launcher.wait(timeout=15)
                self.assertFalse((root / "executed.txt").exists())
                self.assertTrue(process_matches_creation_time(record["pid"], record["creationTime"]))
                self.assertTrue(stop_owned(record))
                self.assertFalse(process_matches_creation_time(record["pid"], record["creationTime"]))
                self.assertIsNone(keeper.poll())
            finally:
                if launcher is not None and launcher.poll() is None:
                    launcher.kill()
                    launcher.wait(timeout=15)
                if record is not None:
                    stop_owned(record)
                if keeper.poll() is None:
                    keeper.kill()
                keeper.wait(timeout=15)

    def test_keeper_death_kills_runtime_and_its_nested_child(self):
        with native_directory() as temporary:
            root = Path(temporary)
            keeper = subprocess.Popen([sys.executable, "-B", "-c", "import time; time.sleep(180)"],
                                      creationflags=subprocess.CREATE_NO_WINDOW)
            lease = {"owner_pid": keeper.pid, "owner_process_creation_time": popen_process_creation_time(keeper)}
            env = dict(os.environ)
            env.update({"PYTHONDONTWRITEBYTECODE": "1", "PYTHONPATH": str(Path.cwd())})
            probe = root / "probe.json"
            try:
                with (root / "runtime.log").open("wb") as stream:
                    record = launch_owned([sys.executable, "-B", "-m", "tools.jenkins.pilot.fault_probe", str(probe), "120"],
                                          cwd=Path.cwd(), env=env, stream=stream, lease=lease)
                children = self._probe(probe)
                keeper.kill()
                keeper.wait(timeout=15)
                deadline = time.monotonic() + 15
                while process_matches_creation_time(record["pid"], record["creationTime"]) and time.monotonic() < deadline:
                    time.sleep(0.1)
                for kind in ("parent", "child"):
                    wait_for_process_exit(children[kind + "Pid"], children[kind + "CreationTime"],
                                          timeout_seconds=max(0, deadline - time.monotonic()))
                    self.assertFalse(process_matches_creation_time(children[kind + "Pid"], children[kind + "CreationTime"]))
                self.assertTrue(stop_owned(record))
            finally:
                if keeper.poll() is None:
                    keeper.kill()
                keeper.wait(timeout=15)

    def test_repeated_stop_while_keeper_lives_is_safe(self):
        with native_directory() as temporary:
            root = Path(temporary)
            env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONPATH=str(Path.cwd()))
            lease = {"owner_pid": os.getpid(), "owner_process_creation_time": None}
            from tools.jenkins.pilot.native.process_identity import process_creation_time
            lease["owner_process_creation_time"] = process_creation_time(os.getpid())
            probe = root / "probe.json"
            record = None
            try:
                with (root / "runtime.log").open("wb") as stream:
                    record = launch_owned([sys.executable, "-B", "-m", "tools.jenkins.pilot.fault_probe", str(probe), "120"],
                                          cwd=Path.cwd(), env=env, stream=stream, lease=lease)
                children = self._probe(probe)
                with self.assertRaises(OSError):
                    os.rename(root / "runtime.log", root / "runtime-renamed.log")
                self.assertTrue(stop_owned(record))
                self.assertTrue(stop_owned(record))
                for kind in ("parent", "child"):
                    self.assertFalse(process_matches_creation_time(children[kind + "Pid"], children[kind + "CreationTime"]))
            finally:
                if record is not None:
                    stop_owned(record)

    def test_stale_owner_identity_never_launches_a_runtime(self):
        with native_directory() as temporary:
            with (Path(temporary) / "runtime.log").open("wb") as stream:
                with self.assertRaises(ProcessLookupError):
                    launch_owned([sys.executable, "-B", "-c", "raise RuntimeError('must not execute')"],
                                 cwd=Path.cwd(), env=os.environ, stream=stream,
                                 lease={"owner_pid": os.getpid(), "owner_process_creation_time": "0"})

    def test_keeper_query_denied_cannot_prove_runtime_terminal(self):
        record = {"keeperPid": os.getpid(), "keeperCreationTime": "123",
                  "pid": 12345, "creationTime": "456"}
        with patch("tools.jenkins.pilot.lifetime._kernel", return_value=object()), \
             patch("tools.jenkins.pilot.lifetime._open_owner", side_effect=PermissionError("injected keeper access denied")), \
             patch("tools.jenkins.pilot.lifetime.process_matches_creation_time", return_value=False) as root_query, \
             patch("tools.jenkins.pilot.lifetime._wait_log_release") as log_query:
            with self.assertRaises(PermissionError):
                stop_owned(record)
            root_query.assert_not_called()
            log_query.assert_not_called()


if __name__ == "__main__":
    unittest.main()
