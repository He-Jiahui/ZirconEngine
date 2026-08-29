from __future__ import annotations

import tempfile
import threading
import unittest
from contextlib import contextmanager, nullcontext
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator.cargo_runner import CargoJobRunner
from tools.session_coordinator.models import CoordinatorError


class CargoRunnerSourceRootTests(unittest.TestCase):
    def test_collector_bounds_log_reader_join_and_records_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 0
            jobs = mock.Mock()
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader = mock.Mock()
            reader.is_alive.return_value = True
            reader_group = SimpleNamespace(
                threads=(reader,),
                streams=(),
                errors=[],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner._STREAM_READER_JOIN_TIMEOUT_SECONDS",
                0.25,
            ):
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    None,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        reader.join.assert_called_once_with(timeout=0.25)
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("cargo_run_log_reader_timeout", update_parameters[4])

    def test_log_write_failure_keeps_draining_the_child_pipe(self) -> None:
        stream = mock.Mock()
        stream.read.side_effect = ["first", "second", ""]
        output = mock.Mock()
        output.write.side_effect = OSError("disk full")
        ready = threading.Event()
        errors: list[tuple[str, BaseException]] = []
        error_lock = threading.Lock()
        read_failed = threading.Event()

        with mock.patch.object(Path, "open", return_value=output):
            CargoJobRunner._drain_stream(
                stream,
                Path("stdout.log"),
                ready,
                errors,
                error_lock,
                read_failed,
            )

        self.assertTrue(ready.is_set())
        self.assertEqual(3, stream.read.call_count)
        self.assertEqual(["write"], [kind for kind, _error in errors])
        self.assertFalse(read_failed.is_set())

    def test_log_read_failure_only_signals_the_collector(self) -> None:
        stream = mock.Mock()
        stream.read.side_effect = OSError("pipe read failed")
        output = mock.Mock()
        ready = threading.Event()
        errors: list[tuple[str, BaseException]] = []
        error_lock = threading.Lock()
        read_failed = threading.Event()

        with mock.patch.object(Path, "open", return_value=output):
            CargoJobRunner._drain_stream(
                stream,
                Path("stdout.log"),
                ready,
                errors,
                error_lock,
                read_failed,
            )

        self.assertTrue(ready.is_set())
        self.assertTrue(read_failed.is_set())
        self.assertEqual(["read"], [kind for kind, _error in errors])

    def test_collector_terminates_job_before_waiting_after_log_read_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            terminated = threading.Event()
            process.poll.side_effect = lambda: 1 if terminated.is_set() else None

            def wait(*_args, **_kwargs):
                self.assertTrue(terminated.is_set())
                return 1

            process.wait.side_effect = wait
            jobs = mock.Mock()
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[("read", OSError("pipe read failed"))],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            reader_group.read_failed.set()
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                terminate_process_job=lambda handle: (
                    self.assertEqual(9001, handle),
                    terminated.set(),
                ),
            )

            runner._finish(
                "run-a",
                "job-a",
                "session-a",
                process,
                9001,
                reader_group,
                stdout_path,
                stderr_path,
            )

        self.assertTrue(terminated.is_set())
        jobs.finish.assert_called_once_with("job-a", session_id="session-a", exit_code=1)
        jobs.release.assert_called_once_with("job-a", session_id="session-a")
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("finish_blocked", update_parameters[0])
        self.assertEqual("cargo_run_log_read_failed", update_parameters[4])

    def test_collector_waits_for_job_tree_past_the_legacy_deadline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 0
            jobs = mock.Mock()
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            wait_process_job = mock.Mock(side_effect=(TimeoutError(), None))
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                wait_process_job=wait_process_job,
            )

            with (
                mock.patch("tools.session_coordinator.cargo_runner.close_process_job"),
                mock.patch(
                    "tools.session_coordinator.cargo_runner.time.monotonic",
                    side_effect=(0.0, 121.0),
                ),
            ):
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    9001,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        jobs.heartbeat.assert_called_once_with("job-a", session_id="session-a")
        jobs.finish_from_atomic_job_terminal.assert_called_once_with(
            "job-a", session_id="session-a", exit_code=0
        )
        jobs.finish.assert_not_called()
        jobs.release.assert_not_called()
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("completed", update_parameters[0])
        self.assertIsNone(update_parameters[4])

    def test_collector_uses_job_object_terminal_evidence_over_stale_pid_projection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 0
            jobs = mock.Mock()
            stale_projection = CoordinatorError(
                "cargo_process_tree_alive", "PID projection has not caught up"
            )
            jobs.finish.side_effect = stale_projection
            jobs.release.side_effect = stale_projection
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                wait_process_job=mock.Mock(return_value=None),
            )

            with (
                mock.patch("tools.session_coordinator.cargo_runner.close_process_job"),
                mock.patch("tools.session_coordinator.cargo_runner.time.sleep"),
            ):
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    9001,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        jobs.finish_from_atomic_job_terminal.assert_called_once_with(
            "job-a", session_id="session-a", exit_code=0
        )
        jobs.finish.assert_not_called()
        jobs.release.assert_not_called()
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("completed", update_parameters[0])
        self.assertIsNone(update_parameters[4])

    def test_collector_closes_job_handle_when_read_failure_termination_raises(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 1
            jobs = mock.Mock()
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[("read", OSError("pipe read failed"))],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            reader_group.read_failed.set()
            terminate_process_job = mock.Mock(side_effect=OSError("termination failed"))
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                terminate_process_job=terminate_process_job,
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.close_process_job"
            ) as close_job:
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    9001,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        terminate_process_job.assert_called_once_with(9001)
        close_job.assert_any_call(9001)

    def test_collector_closes_job_handle_when_heartbeat_termination_raises(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 1
            jobs = mock.Mock()
            jobs.heartbeat.side_effect = CoordinatorError(
                "cargo_job_heartbeat_failed", "lease heartbeat failed"
            )
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            terminate_process_job = mock.Mock(side_effect=OSError("termination failed"))
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                terminate_process_job=terminate_process_job,
                wait_process_job=mock.Mock(side_effect=TimeoutError()),
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.close_process_job"
            ) as close_job:
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    9001,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        terminate_process_job.assert_called_once_with(9001)
        close_job.assert_any_call(9001)
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("finish_blocked", update_parameters[0])
        self.assertEqual("cargo_process_job_termination_failed", update_parameters[4])

    def test_collector_retries_job_close_when_termination_and_close_raise(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 1
            jobs = mock.Mock()
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            reader_group = SimpleNamespace(
                threads=(),
                errors=[("read", OSError("pipe read failed"))],
                error_lock=threading.Lock(),
                read_failed=threading.Event(),
            )
            reader_group.read_failed.set()
            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
                terminate_process_job=mock.Mock(side_effect=OSError("termination failed")),
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.close_process_job",
                side_effect=OSError("close failed"),
            ) as close_job:
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    9001,
                    reader_group,
                    stdout_path,
                    stderr_path,
                )

        self.assertGreaterEqual(
            [call.args for call in close_job.call_args_list].count((9001,)), 2
        )
        update_parameters = connection.execute.call_args.args[1]
        self.assertEqual("finish_blocked", update_parameters[0])
        self.assertEqual("cargo_process_job_close_failed", update_parameters[4])

    def test_runner_uses_the_coordinator_selected_immutable_working_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            source = root / "verify/job/source"
            logs = root / "logs"
            repo.mkdir()
            source.mkdir(parents=True)
            process = mock.Mock()
            process.pid = 4242
            process.poll.return_value = None
            release = threading.Event()
            process.wait.side_effect = lambda *args, **kwargs: release.wait(timeout=2) or 0
            jobs = mock.Mock()
            jobs.managed_start_registration.side_effect = nullcontext
            jobs.get.return_value = SimpleNamespace(
                session_id="session-a",
                status=SimpleNamespace(value="leased"),
                target_dir=str(root / "target"),
            )
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            database = SimpleNamespace(transaction=transaction)
            runner = CargoJobRunner(
                database,
                jobs,
                repo_root=repo,
                log_root=logs,
                popen=mock.Mock(return_value=process),
            )

            runner.start(
                session_id="session-a",
                job_id="job-a",
                command=("cargo", "test"),
                working_directory=source,
            )

            self.assertEqual(source, runner.popen.call_args.kwargs["cwd"])
            child_environment = runner.popen.call_args.kwargs["env"]
            target_directory = Path(jobs.get.return_value.target_dir).resolve()
            self.assertEqual(str(target_directory), child_environment["CARGO_TARGET_DIR"])
            self.assertEqual(
                str(target_directory / "cargo-home"), child_environment["CARGO_HOME"]
            )
            self.assertEqual(
                str(target_directory / "sccache"), child_environment["SCCACHE_DIR"]
            )
            for name in ("TEMP", "TMP", "TMPDIR"):
                self.assertEqual(
                    str(target_directory / "temporary"), child_environment[name]
                )
            release.set()

    def test_runner_rejects_a_missing_source_root_before_spawn(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            jobs = mock.Mock()
            jobs.get.return_value = SimpleNamespace(
                session_id="session-a",
                status=SimpleNamespace(value="leased"),
                target_dir=str(root / "target"),
            )
            runner = CargoJobRunner(
                mock.Mock(),
                jobs,
                repo_root=repo,
                log_root=root / "logs",
                popen=mock.Mock(),
            )

            with self.assertRaises(CoordinatorError) as rejected:
                runner.start(
                    session_id="session-a",
                    job_id="job-a",
                    command=("cargo", "test"),
                    working_directory=root / "missing",
                )

        self.assertEqual("cargo_run_source_root_invalid", rejected.exception.code)
        runner.popen.assert_not_called()


if __name__ == "__main__":
    unittest.main()
