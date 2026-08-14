from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.processes import (
    popen_process_creation_time,
    process_creation_time,
    process_is_alive,
    terminate_process_tree,
)
from tools.session_coordinator.windows_job_process import (
    close_process_job,
    create_atomic_kill_on_close_process,
    resume_popen_process,
    terminate_and_close_process_job,
)


@unittest.skipUnless(os.name == "nt", "Windows process-handle semantics only")
class ProcessLivenessTests(unittest.TestCase):
    def test_exited_process_with_an_open_parent_handle_is_not_alive(self) -> None:
        child = subprocess.Popen([sys.executable, "-c", "import sys; sys.exit(255)"])
        self.addCleanup(child.wait)
        self.assertEqual(255, child.wait())

        self.assertFalse(process_is_alive(child.pid))

    def test_terminate_process_tree_rejects_reused_pid_on_retained_handle(self) -> None:
        with (
            mock.patch(
                "tools.session_coordinator.processes._open_windows_process_handle",
                return_value=9001,
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time",
                return_value="new-process",
            ),
            mock.patch(
                "tools.session_coordinator.processes._terminate_windows_handle"
            ) as terminate,
            mock.patch(
                "tools.session_coordinator.processes._close_windows_process_handle"
            ) as close,
            self.assertRaises(ProcessLookupError),
        ):
            terminate_process_tree(4242, "original-process")

        terminate.assert_not_called()
        close.assert_called_once_with(9001)

    def test_terminate_process_tree_holds_root_and_confirmed_descendant_handles(self) -> None:
        with (
            mock.patch(
                "tools.session_coordinator.processes._open_windows_process_handle",
                side_effect=lambda pid: {4242: 9001, 4243: 9002}[pid],
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time",
                return_value="111222",
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_process_parent_ids",
                side_effect=(
                    {4242: 1, 4243: 4242},
                    {4242: 1, 4243: 4242},
                    {4242: 1},
                    {},
                ),
            ),
            mock.patch(
                "tools.session_coordinator.processes._suspend_windows_handle_if_alive",
                side_effect=(False, True),
            ) as suspend_if_alive,
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_is_alive",
                return_value=False,
            ),
            mock.patch(
                "tools.session_coordinator.processes._suspend_windows_handle"
            ) as suspend,
            mock.patch(
                "tools.session_coordinator.processes._terminate_windows_handle"
            ) as terminate,
            mock.patch(
                "tools.session_coordinator.processes._close_windows_process_handle"
            ) as close,
        ):
            terminate_process_tree(4242, "111222")

        suspend.assert_not_called()
        self.assertEqual(
            [mock.call(9001), mock.call(9002)],
            suspend_if_alive.call_args_list,
        )
        self.assertEqual(
            [mock.call(9002), mock.call(9001)],
            terminate.call_args_list,
        )
        self.assertEqual([mock.call(9002), mock.call(9001)], close.call_args_list)

    def test_terminate_process_tree_stops_a_real_parent_and_child(self) -> None:
        parent = subprocess.Popen(
            [
                sys.executable,
                "-c",
                (
                    "import subprocess,sys,time; "
                    "child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
                    "print(child.pid, flush=True); time.sleep(60)"
                ),
            ],
            stdout=subprocess.PIPE,
            text=True,
        )
        self.addCleanup(lambda: parent.kill() if parent.poll() is None else None)
        assert parent.stdout is not None
        child_pid = int(parent.stdout.readline().strip())

        terminate_process_tree(parent.pid, process_creation_time(parent.pid))

        parent.wait(timeout=5)
        parent.stdout.close()
        self.assertFalse(process_is_alive(parent.pid))
        self.assertFalse(process_is_alive(child_pid))

    def test_kill_on_close_job_stops_grandchild_after_intermediate_exits(self) -> None:
        process, job_handle = create_atomic_kill_on_close_process(
            (
                sys.executable,
                "-c",
                (
                    "import subprocess,sys,time; "
                    "child=subprocess.Popen([sys.executable,'-c',"
                    "'import subprocess,sys; grandchild=subprocess.Popen([sys.executable,\"-c\",\"import time; time.sleep(60)\"]); print(grandchild.pid,flush=True)'],"
                    "stdout=subprocess.PIPE,text=True); "
                    "grandchild_pid=int(child.stdout.readline().strip()); "
                    "child.wait(); print(child.pid,grandchild_pid,flush=True); time.sleep(60)"
                ),
            ),
            cwd=Path.cwd(),
            env=dict(os.environ),
        )
        self.addCleanup(lambda: process.kill() if process.poll() is None else None)
        resume_popen_process(process)
        assert process.stdout is not None
        child_pid, grandchild_pid = (
            int(value) for value in process.stdout.readline().strip().split()
        )
        self.assertFalse(process_is_alive(child_pid))
        self.assertTrue(process_is_alive(grandchild_pid))

        close_process_job(job_handle)
        process.wait(timeout=5)
        process.stdout.close()
        process.stderr.close()

        self.assertFalse(process_is_alive(process.pid))
        self.assertFalse(process_is_alive(child_pid))
        self.assertFalse(process_is_alive(grandchild_pid))

    def test_atomic_job_process_is_bound_before_resume_and_waits_for_tree_exit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "resumed.txt"
            process, job_handle = create_atomic_kill_on_close_process(
                (
                    sys.executable,
                    "-c",
                    (
                        "from pathlib import Path; import subprocess,sys,time; "
                        f"Path({str(marker)!r}).write_text('resumed'); "
                        "subprocess.Popen([sys.executable,'-c','import time; time.sleep(60)']); "
                        "time.sleep(60)"
                    ),
                ),
                cwd=Path(directory),
                env=dict(os.environ),
            )
            self.addCleanup(
                lambda: terminate_and_close_process_job(job_handle)
                if process.poll() is None
                else None
            )
            self.assertFalse(marker.exists())

            resume_popen_process(process)
            for _ in range(100):
                if marker.exists():
                    break
                threading.Event().wait(0.02)
            self.assertTrue(marker.exists())

            terminate_and_close_process_job(job_handle)
            process.wait(timeout=5)
            process.stdout.close()
            process.stderr.close()
            self.assertFalse(process_is_alive(process.pid))

    def test_atomic_job_process_dies_when_launcher_crashes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "child.txt"
            helper_code = (
                "import os,sys; from pathlib import Path; "
                "from tools.session_coordinator.windows_job_process import "
                "create_atomic_kill_on_close_process,resume_popen_process; "
                f"marker=Path({str(marker)!r}); "
                "process,job=create_atomic_kill_on_close_process("
                "(sys.executable,'-c','import time; time.sleep(60)'),"
                f"cwd=Path({directory!r}),env=dict(os.environ)); "
                "marker.write_text(str(process.pid)); "
                "resume_popen_process(process); "
                "os._exit(0)"
            )
            helper = subprocess.Popen(
                [sys.executable, "-c", helper_code],
                cwd=Path(__file__).resolve().parents[3],
            )
            self.assertEqual(0, helper.wait(timeout=10))
            child_pid = int(marker.read_text(encoding="utf-8"))
            for _ in range(100):
                if not process_is_alive(child_pid):
                    break
                threading.Event().wait(0.02)
            self.assertFalse(process_is_alive(child_pid))

    def test_root_exit_during_suspend_is_a_benign_recovery_race(self) -> None:
        with (
            mock.patch(
                "tools.session_coordinator.processes._open_windows_process_handle",
                return_value=9001,
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time",
                return_value="111222",
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_is_alive",
                side_effect=(True, False, False),
            ),
            mock.patch(
                "tools.session_coordinator.processes._suspend_windows_handle",
                side_effect=OSError("root exited"),
            ) as suspend,
            mock.patch(
                "tools.session_coordinator.processes._windows_process_parent_ids",
                return_value={},
            ),
            mock.patch(
                "tools.session_coordinator.processes._terminate_windows_handle"
            ) as terminate,
            mock.patch(
                "tools.session_coordinator.processes._close_windows_process_handle"
            ),
        ):
            terminate_process_tree(4242, "111222")

        suspend.assert_called_once_with(9001)
        terminate.assert_called_once_with(9001)

    def test_descendant_exit_during_suspend_is_a_benign_recovery_race(self) -> None:
        with (
            mock.patch(
                "tools.session_coordinator.processes._open_windows_process_handle",
                side_effect=lambda pid: {4242: 9001, 4243: 9002}[pid],
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time",
                return_value="111222",
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_is_alive",
                side_effect=(True, True, False, False),
            ),
            mock.patch(
                "tools.session_coordinator.processes._suspend_windows_handle",
                side_effect=lambda handle: (
                    None if handle == 9001 else (_ for _ in ()).throw(OSError("descendant exited"))
                ),
            ) as suspend,
            mock.patch(
                "tools.session_coordinator.processes._windows_process_parent_ids",
                side_effect=(
                    {4242: 1, 4243: 4242},
                    {4242: 1, 4243: 4242},
                    {4242: 1},
                ),
            ),
            mock.patch(
                "tools.session_coordinator.processes._terminate_windows_handle"
            ) as terminate,
            mock.patch(
                "tools.session_coordinator.processes._resume_windows_handle"
            ) as resume,
            mock.patch(
                "tools.session_coordinator.processes._close_windows_process_handle"
            ),
        ):
            terminate_process_tree(4242, "111222")

        self.assertEqual([mock.call(9001), mock.call(9002)], suspend.call_args_list)
        terminate.assert_called_once_with(9001)
        resume.assert_not_called()

    def test_popen_creation_time_uses_retained_handle_not_pid_reopen(self) -> None:
        process = mock.Mock(pid=4242, _handle=9001)
        with (
            mock.patch.object(os, "name", "nt"),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time",
                return_value="111222",
            ) as handle_creation_time,
            mock.patch(
                "tools.session_coordinator.processes.process_creation_time"
            ) as pid_creation_time,
            mock.patch(
                "tools.session_coordinator.processes.ctypes.windll.kernel32.GetProcessId",
                return_value=4242,
            ),
        ):
            self.assertEqual("111222", popen_process_creation_time(process))

        handle_creation_time.assert_called_once_with(9001)
        pid_creation_time.assert_not_called()

    def test_popen_creation_time_survives_immediate_process_exit(self) -> None:
        process = subprocess.Popen([sys.executable, "-c", "pass"])
        self.assertEqual(0, process.wait(timeout=5))

        creation_time = popen_process_creation_time(process)

        self.assertTrue(creation_time.isdecimal())

    def test_popen_creation_time_rejects_handle_pid_mismatch(self) -> None:
        process = mock.Mock(pid=4242, _handle=9001)
        with (
            mock.patch.object(os, "name", "nt"),
            mock.patch(
                "tools.session_coordinator.processes.ctypes.windll.kernel32.GetProcessId",
                return_value=9999,
            ),
            mock.patch(
                "tools.session_coordinator.processes._windows_handle_creation_time"
            ) as handle_creation_time,
            self.assertRaises(ProcessLookupError),
        ):
            popen_process_creation_time(process)

        handle_creation_time.assert_not_called()


if __name__ == "__main__":
    unittest.main()
