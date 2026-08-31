from __future__ import annotations

import json
import subprocess
import tempfile
import threading
import time
import unittest
from contextlib import contextmanager, nullcontext
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator.cargo_runner import CargoJobRunner
from tools.session_coordinator.cargo_storage import (
    managed_cargo_server_port,
    managed_native_dynamic_cas_path,
)
from tools.session_coordinator.models import CoordinatorError


class CargoRunnerSourceRootTests(unittest.TestCase):
    def test_sccache_initializer_uses_the_shared_binding_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            helper = (
                repo
                / ".codex"
                / "skills"
                / "zircon-dev"
                / "scripts"
                / "managed-cargo-storage.ps1"
            )
            helper.parent.mkdir(parents=True)
            helper.write_text("# test helper", encoding="utf-8")
            path_resolver = repo / "tools" / "WindowsPathResolver.psm1"
            path_resolver.parent.mkdir(parents=True)
            path_resolver.write_text("# test path resolver", encoding="utf-8")
            runner = CargoJobRunner(
                mock.Mock(),
                mock.Mock(),
                repo_root=repo,
                log_root=root / "logs",
                compiler_cache=root / "bin" / "sccache.exe",
                powershell_executable=root / "bin" / "pwsh.exe",
            )
            stable_temporary = root / "cache" / "sccache-temporary"
            environment = {
                "SCCACHE_SERVER_PORT": "42261",
                "SCCACHE_CACHE_SIZE": "12G",
                "SCCACHE_DIR": str(root / "cache" / "sccache"),
                "TEMP": str(stable_temporary),
                "TMP": str(stable_temporary),
                "TMPDIR": str(stable_temporary),
            }
            marker_path = stable_temporary / "server-binding-v1.json"
            marker_path.parent.mkdir(parents=True)
            marker_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "server_port": 42261,
                        "server_process_id": 4242,
                        "server_started_at_utc_ticks": 638000000000000000,
                        "cache_size": "12G",
                        "cache_directory": environment["SCCACHE_DIR"],
                        "stable_temporary_directory": str(stable_temporary),
                        "compiler_cache_executable": runner.compiler_cache,
                    }
                ),
                encoding="utf-8",
            )
            completed = mock.Mock(
                returncode=0,
                stdout=json.dumps(
                    {
                        "ServerPort": 42261,
                        "ServerProcessId": 4242,
                        "BindingMarkerPath": str(marker_path),
                        "Restarted": True,
                    }
                ),
                stderr="",
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.subprocess.run",
                return_value=completed,
            ) as run:
                runner._initialize_compiler_cache_server(
                    runner.compiler_cache, environment
                )

            command = run.call_args.args[0]
            self.assertEqual(str((root / "bin" / "pwsh.exe").resolve()), command[0])
            command_index = command.index("-Command")
            self.assertIn(
                "Initialize-ManagedCompilerCacheServer", command[command_index + 1]
            )
            invocation_environment = run.call_args.kwargs["env"]
            self.assertEqual(
                str(helper.resolve()),
                invocation_environment["ZIRCON_MANAGED_SCCACHE_HELPER"],
            )
            self.assertEqual(
                str(path_resolver.resolve()),
                invocation_environment["ZIRCON_MANAGED_WINDOWS_PATH_RESOLVER"],
            )
            self.assertEqual(
                runner.compiler_cache,
                invocation_environment["ZIRCON_MANAGED_SCCACHE_EXECUTABLE"],
            )
            self.assertEqual(
                environment["SCCACHE_DIR"],
                invocation_environment["ZIRCON_MANAGED_SCCACHE_DIRECTORY"],
            )
            self.assertEqual(
                str(stable_temporary),
                invocation_environment["ZIRCON_MANAGED_SCCACHE_TEMPORARY"],
            )
            self.assertEqual(
                "42261", invocation_environment["ZIRCON_MANAGED_SCCACHE_PORT"]
            )
            self.assertEqual(
                "12G", invocation_environment["ZIRCON_MANAGED_SCCACHE_CACHE_SIZE"]
            )
            self.assertEqual(
                getattr(subprocess, "CREATE_NO_WINDOW", 0),
                run.call_args.kwargs["creationflags"],
            )

    def test_sccache_stale_binding_refuses_rebind_while_compilers_are_active(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            helper = (
                repo
                / ".codex"
                / "skills"
                / "zircon-dev"
                / "scripts"
                / "managed-cargo-storage.ps1"
            )
            helper.parent.mkdir(parents=True)
            helper.write_text("# test helper", encoding="utf-8")
            path_resolver = repo / "tools" / "WindowsPathResolver.psm1"
            path_resolver.parent.mkdir(parents=True)
            path_resolver.write_text("# test path resolver", encoding="utf-8")
            runner = CargoJobRunner(
                mock.Mock(),
                mock.Mock(),
                repo_root=repo,
                log_root=root / "logs",
                compiler_cache=root / "bin" / "sccache.exe",
                powershell_executable=root / "bin" / "pwsh.exe",
            )
            stable_temporary = root / "cache" / "sccache-temporary"
            completed = mock.Mock(
                returncode=1,
                stdout="",
                stderr=(
                    "Managed sccache binding is stale, but Cargo/rustc processes "
                    "are active (101, 202); refusing an unsafe daemon restart."
                ),
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.subprocess.run",
                return_value=completed,
            ), self.assertRaises(CoordinatorError) as rejected:
                runner._initialize_compiler_cache_server(
                    runner.compiler_cache,
                    {
                        "SCCACHE_SERVER_PORT": "42261",
                        "SCCACHE_CACHE_SIZE": "12G",
                        "SCCACHE_DIR": str(root / "cache" / "sccache"),
                        "TEMP": str(stable_temporary),
                    },
                )

            self.assertEqual("cargo_sccache_rebind_busy", rejected.exception.code)

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
            launch_order: list[str] = []
            server_initializer = mock.Mock(
                side_effect=lambda _executable, _environment: launch_order.append(
                    "sccache-server"
                )
            )

            def launch_cargo(*_args, **_kwargs):
                launch_order.append("cargo")
                return process

            runner = CargoJobRunner(
                database,
                jobs,
                repo_root=repo,
                log_root=logs,
                popen=mock.Mock(side_effect=launch_cargo),
                compiler_cache=root / "bin" / "sccache.exe",
                compiler_cache_server_initializer=server_initializer,
            )

            with mock.patch(
                "tools.session_coordinator.cargo_runner.managed_cargo_server_port",
                return_value=42261,
            ):
                runner.start(
                    session_id="session-a",
                    job_id="job-a",
                    command=("cargo", "test"),
                    working_directory=source,
                )

            self.assertEqual(source, runner.popen.call_args.kwargs["cwd"])
            child_environment = runner.popen.call_args.kwargs["env"]
            target_directory = Path(jobs.get.return_value.target_dir).resolve()
            cache_root = root / "cache"
            scratch_root = root / "scratch" / "job-a"
            self.assertEqual(str(target_directory), child_environment["CARGO_TARGET_DIR"])
            self.assertEqual(
                str(scratch_root / "cargo-home"), child_environment["CARGO_HOME"]
            )
            self.assertEqual(
                str(cache_root / "sccache"), child_environment["SCCACHE_DIR"]
            )
            self.assertEqual(
                str(cache_root / "native-dynamic"),
                child_environment["ZIRCON_NATIVE_DYNAMIC_CAS_ROOT"],
            )
            self.assertEqual(
                str(4 * 1024 * 1024 * 1024),
                child_environment["ZIRCON_NATIVE_DYNAMIC_CAS_MAX_BYTES"],
            )
            self.assertEqual(
                str((root / "bin" / "sccache.exe").resolve()),
                child_environment["RUSTC_WRAPPER"],
            )
            self.assertEqual("12G", child_environment["SCCACHE_CACHE_SIZE"])
            self.assertEqual("1", child_environment["SCCACHE_CLIENT_SIDE"])
            self.assertEqual("1", child_environment["SCCACHE_IGNORE_SERVER_IO_ERROR"])
            self.assertEqual("42261", child_environment["SCCACHE_SERVER_PORT"])
            self.assertEqual("0", child_environment["SCCACHE_IDLE_TIMEOUT"])
            self.assertEqual("0", child_environment["CARGO_INCREMENTAL"])
            self.assertEqual("0", child_environment["CARGO_PROFILE_DEV_DEBUG"])
            self.assertEqual("0", child_environment["CARGO_PROFILE_TEST_DEBUG"])
            self.assertEqual("0", child_environment["CARGO_PROFILE_RELEASE_DEBUG"])
            for name in ("TEMP", "TMP", "TMPDIR"):
                self.assertEqual(
                    str(scratch_root / "temporary"), child_environment[name]
                )
            server_initializer.assert_called_once()
            server_environment = server_initializer.call_args.args[1]
            self.assertEqual("42261", server_environment["SCCACHE_SERVER_PORT"])
            self.assertEqual(
                str(cache_root / "sccache-temporary"), server_environment["TEMP"]
            )
            self.assertEqual(["sccache-server", "cargo"], launch_order)
            self.assertTrue(scratch_root.is_dir())
            release.set()
            deadline = time.monotonic() + 2
            while scratch_root.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertFalse(scratch_root.exists())

    def test_sccache_server_port_matches_the_managed_root_contract(self) -> None:
        self.assertEqual(42263, managed_cargo_server_port(Path(r"D:\targets\job")))
        self.assertEqual(
            42261,
            managed_cargo_server_port(Path(r"E:\cargo-targets\zircon-engine")),
        )
        self.assertEqual(42268, managed_cargo_server_port(Path(r"F:\ZirconBuilds")))

    def test_native_dynamic_cas_path_matches_the_managed_root_contract(self) -> None:
        self.assertEqual(
            Path(r"E:\cargo-targets\zircon-engine\cache\native-dynamic"),
            managed_native_dynamic_cas_path(Path(r"E:\cargo-targets\zircon-engine")),
        )

    def test_scratch_survives_until_the_process_tree_release_is_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout_path = root / "stdout.log"
            stderr_path = root / "stderr.log"
            stdout_path.write_text("", encoding="utf-8")
            stderr_path.write_text("", encoding="utf-8")
            scratch_root = root / "scratch" / "job-a"
            (scratch_root / "temporary").mkdir(parents=True)
            process = mock.Mock()
            process.pid = 4242
            process.wait.return_value = 0
            jobs = mock.Mock()
            release_observations: list[bool] = []

            def release(_job_id: str, *, session_id: str) -> None:
                self.assertEqual("session-a", session_id)
                release_observations.append(scratch_root.exists())
                if len(release_observations) == 1:
                    raise CoordinatorError(
                        "cargo_process_tree_alive",
                        "A descendant still owns the target",
                    )

            jobs.release.side_effect = release
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            runner = CargoJobRunner(
                SimpleNamespace(transaction=transaction),
                jobs,
                repo_root=root,
                log_root=root / "logs",
            )

            with mock.patch("tools.session_coordinator.cargo_runner.time.sleep"):
                runner._finish(
                    "run-a",
                    "job-a",
                    "session-a",
                    process,
                    None,
                    None,
                    stdout_path,
                    stderr_path,
                    scratch_root,
                )

            self.assertEqual([True, True], release_observations)
            self.assertFalse(scratch_root.exists())

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
