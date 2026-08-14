from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator import windows_job_process


@unittest.skipUnless(os.name == "nt", "Windows Job Object semantics only")
class WindowsJobProcessTests(unittest.TestCase):
    def test_text_stream_owns_transferred_handle_when_io_open_fails(self) -> None:
        transferred = mock.Mock()
        transferred.open_osfhandle.return_value = 77
        with (
            mock.patch.object(windows_job_process.io, "open", side_effect=OSError("open failed")),
            mock.patch.object(windows_job_process.os, "close") as close_descriptor,
            mock.patch.object(windows_job_process, "_close_handle") as close_raw_handle,
            self.assertRaises(OSError),
        ):
            windows_job_process._text_stream_from_handle(transferred, 9001)

        transferred.open_osfhandle.assert_called_once_with(9001, os.O_RDONLY)
        close_descriptor.assert_called_once_with(77)
        close_raw_handle.assert_not_called()

    def test_text_stream_closes_raw_handle_when_transfer_fails(self) -> None:
        transferred = mock.Mock()
        transferred.open_osfhandle.side_effect = OSError("transfer failed")
        with (
            mock.patch.object(windows_job_process, "_close_handle") as close_raw_handle,
            self.assertRaises(OSError),
        ):
            windows_job_process._text_stream_from_handle(transferred, 9001)

        close_raw_handle.assert_called_once_with(9001)

    def test_atomic_launch_failure_after_create_process_terminates_root_and_job(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with (
                mock.patch.object(
                    windows_job_process,
                    "_process_is_in_job",
                    side_effect=OSError("identity verification failed"),
                ),
                mock.patch.object(
                    windows_job_process, "_terminate_process_handle", wraps=windows_job_process._terminate_process_handle
                ) as terminate,
                mock.patch.object(
                    windows_job_process, "close_process_job", wraps=windows_job_process.close_process_job
                ) as close_job,
                self.assertRaises(OSError),
            ):
                windows_job_process.create_atomic_kill_on_close_process(
                    (sys.executable, "-c", "import time; time.sleep(60)"),
                    cwd=Path(directory),
                    env=dict(os.environ),
                )

        terminate.assert_called_once()
        close_job.assert_called_once()

    def test_atomic_launch_cleanup_closes_job_when_process_termination_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with (
                mock.patch.object(
                    windows_job_process,
                    "_process_is_in_job",
                    side_effect=OSError("identity verification failed"),
                ),
                mock.patch.object(
                    windows_job_process,
                    "_terminate_process_handle",
                    side_effect=OSError("terminate failed"),
                ),
                mock.patch.object(
                    windows_job_process, "_close_handle", wraps=windows_job_process._close_handle
                ) as close_handle,
                mock.patch.object(
                    windows_job_process, "close_process_job", wraps=windows_job_process.close_process_job
                ) as close_job,
                self.assertRaisesRegex(OSError, "identity verification failed"),
            ):
                windows_job_process.create_atomic_kill_on_close_process(
                    (sys.executable, "-c", "import time; time.sleep(60)"),
                    cwd=Path(directory),
                    env=dict(os.environ),
                )

        self.assertGreaterEqual(close_handle.call_count, 2)
        close_job.assert_called_once()

    def test_uninitialized_attribute_list_is_not_deleted(self) -> None:
        kernel32 = windows_job_process.ctypes.windll.kernel32
        original_initialize = kernel32.InitializeProcThreadAttributeList
        initialize_calls = 0

        def fail_second_initialize(attribute_list, count, flags, size):
            nonlocal initialize_calls
            initialize_calls += 1
            if initialize_calls == 2:
                return False
            return original_initialize(attribute_list, count, flags, size)

        with tempfile.TemporaryDirectory() as directory:
            with (
                mock.patch.object(
                    kernel32,
                    "InitializeProcThreadAttributeList",
                    side_effect=fail_second_initialize,
                ),
                mock.patch.object(
                    kernel32, "DeleteProcThreadAttributeList"
                ) as delete_attribute_list,
                self.assertRaises(OSError),
            ):
                windows_job_process.create_atomic_kill_on_close_process(
                    (sys.executable, "-c", "pass"),
                    cwd=Path(directory),
                    env=dict(os.environ),
                )

        self.assertEqual(2, initialize_calls)
        delete_attribute_list.assert_not_called()

    def test_atomic_process_wait_reports_signed_windows_exit_code(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            process, job_handle = windows_job_process.create_atomic_kill_on_close_process(
                (sys.executable, "-c", "raise SystemExit(255)"),
                cwd=Path(directory),
                env=dict(os.environ),
            )
            try:
                windows_job_process.resume_popen_process(process)
                self.assertEqual(255, process.wait(timeout=5))
            finally:
                process.stdout.close()
                process.stderr.close()
                windows_job_process.close_process_job(job_handle)

    def test_atomic_process_close_is_idempotent(self) -> None:
        stdout = mock.Mock()
        stderr = mock.Mock()
        process = windows_job_process.AtomicJobProcess(
            args=("command",),
            process_handle=9001,
            pid=42,
            stdout=stdout,
            stderr=stderr,
        )
        with mock.patch.object(windows_job_process, "_close_handle") as close_handle:
            process.close()
            process.close()

        stdout.close.assert_called_once()
        stderr.close.assert_called_once()
        close_handle.assert_called_once_with(9001)

    def test_atomic_process_wait_timeout_matches_popen_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            process, job_handle = windows_job_process.create_atomic_kill_on_close_process(
                (sys.executable, "-c", "import time; time.sleep(60)"),
                cwd=Path(directory),
                env=dict(os.environ),
            )
            windows_job_process.resume_popen_process(process)
            try:
                with self.assertRaises(subprocess.TimeoutExpired):
                    process.wait(timeout=0.01)
            finally:
                windows_job_process.terminate_and_close_process_job(job_handle)
                process.wait(timeout=5)
                process.stdout.close()
                process.stderr.close()


if __name__ == "__main__":
    unittest.main()
