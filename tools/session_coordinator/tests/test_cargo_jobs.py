from __future__ import annotations

import json
import os
import sqlite3
import subprocess
import tempfile
import threading
import time
import unittest
from contextlib import contextmanager
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.cargo_runner import CargoJobRunner
from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoJobStatus,
    CargoLaneKind,
    TargetPathPolicy,
    target_identity,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus, utc_text
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CargoJobTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/cargo-targets"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.sessions.register(session_id="session-b")
        self.policy = TargetPathPolicy([self.target_root])
        self.service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
        )
        self.process_creation_times: dict[int, str] = {}
        self.service.process_creation_time = lambda pid: self.process_creation_times.get(
            pid, f"stable:{pid}"
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    @staticmethod
    def compatibility(**overrides: str) -> CargoCompatibility:
        values = {
            "platform": "windows",
            "toolchain": "stable-x86_64-pc-windows-msvc",
            "target_architecture": "x86_64-pc-windows-msvc",
            "workspace": "Cargo.toml",
            "build_config": "profile=test;features=default;rustflags=;incremental=0;debug=0",
        }
        values.update(overrides)
        return CargoCompatibility(**values)

    def test_allocated_lane_is_unique_and_under_allowlisted_cargo_target_root(self) -> None:
        first = self.service.acquire("session-a", CargoLaneKind.CHECK)
        second = self.service.acquire("session-a", CargoLaneKind.CHECK)

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertTrue(Path(first.target_dir).is_relative_to(self.target_root))
        self.assertTrue(Path(first.target_dir).is_dir())
        self.assertEqual(CargoJobStatus.LEASED, first.status)
        self.assertNotIn(str(self.repo / "target"), first.target_dir)

    def test_explicit_target_outside_allowlist_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.TEST,
                requested_target=self.repo / "target/manual",
            )
        self.assertEqual("cargo_target_not_managed", rejected.exception.code)

    def test_active_explicit_target_cannot_have_two_writers(self) -> None:
        requested = self.target_root / "shared-check"
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        with self.assertRaises(CoordinatorError) as occupied:
            self.service.acquire(
                "session-a", CargoLaneKind.TEST, requested_target=requested
            )

        self.assertEqual(CargoJobStatus.LEASED, first.status)
        self.assertEqual("cargo_lane_occupied", occupied.exception.code)

    def test_failed_cleanup_blocks_overlapping_target_reacquire(self) -> None:
        requested = self.target_root / "failed-cleanup"
        prior = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )
        self.service.release(prior.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE cargo_jobs
                SET cleanup_status='failed', cleanup_error='delete outcome unknown'
                WHERE job_id=?
                """,
                (prior.job_id,),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.TEST,
                requested_target=requested / "child",
            )

        self.assertEqual("cargo_lane_cleanup_failed", rejected.exception.code)
        self.assertEqual(prior.job_id, rejected.exception.details["jobId"])

    def test_reconcile_scans_processes_outside_the_database_write_transaction(self) -> None:
        """A slow process-tree probe must not freeze unrelated coordinator writes."""
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test"],
            root_is_supervisor=True,
        )
        blocked_writes: list[str] = []

        def scan_live_process_tree(_pid: int) -> tuple[int, ...]:
            connection = sqlite3.connect(self.database.path, timeout=0.05, isolation_level=None)
            try:
                connection.execute("BEGIN IMMEDIATE")
                connection.rollback()
            except sqlite3.OperationalError as error:
                blocked_writes.append(str(error))
            finally:
                connection.close()
            return (4242,)

        self.service.process_tree_pids = scan_live_process_tree

        self.service.reconcile_orphans()

        self.assertEqual([], blocked_writes)

    def test_gpu_reservation_keeps_fifo_until_nominated_job_reaches_terminal_state(self) -> None:
        now = "2026-07-15T12:00:00+00:00"
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO action_requests(
                    action_id, action_kind, risk, required_role, actor,
                    daemon_instance_id, parameters_json, impact_json, warnings_json,
                    state_fingerprint, confirmation_phrase_hash, status, created_at,
                    expires_at, completed_at
                ) VALUES (?, 'service.resume', 'yellow', 'operator', 'operator',
                          'daemon', ?, '[]', '[]', 'fingerprint', 'phrase', 'succeeded',
                          ?, ?, ?)
                """,
                (
                    "gpu-resume",
                    json.dumps({"timeoutSeconds": 30, "gpuReservationSessionId": "session-a"}),
                    now,
                    now,
                    now,
                ),
            )

        with self.assertRaises(CoordinatorError) as blocked:
            self.service.acquire("session-b", CargoLaneKind.GPU)
        self.assertEqual("cargo_gpu_lane_reserved", blocked.exception.code)

        reserved = self.service.acquire("session-a", CargoLaneKind.GPU)
        self.assertEqual(CargoJobStatus.LEASED, reserved.status)

        with self.assertRaises(CoordinatorError) as still_reserved:
            self.service.acquire("session-b", CargoLaneKind.GPU)
        self.assertEqual("cargo_gpu_lane_reserved", still_reserved.exception.code)

        self.service.start(
            reserved.job_id,
            session_id="session-a",
            pid=4242,
            command=["powershell", "run-render18-product.ps1"],
            root_is_supervisor=True,
        )

        with self.assertRaises(CoordinatorError) as running_reserved:
            self.service.acquire("session-b", CargoLaneKind.GPU)
        self.assertEqual("cargo_gpu_lane_reserved", running_reserved.exception.code)

        self.service.process_alive = lambda _pid: False
        self.service.finish(reserved.job_id, session_id="session-a", exit_code=0)

        following = self.service.acquire("session-b", CargoLaneKind.GPU)
        self.assertEqual(CargoJobStatus.LEASED, following.status)

    def test_coordinator_runner_persists_output_and_releases_after_process_exit(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "run-logs",
        )

        run = runner.start(
            session_id="session-a",
            job_id=job.job_id,
            command=(
                os.fspath(Path(os.sys.executable)),
                "-c",
                "import os, sys; print('runner-out'); print(os.environ['CARGO_TARGET_DIR']); "
                "print('runner-err', file=sys.stderr); raise SystemExit(7)",
            ),
        )
        deadline = datetime.now(UTC) + timedelta(seconds=5)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual("completed", state["status"])
        self.assertEqual(7, state["exitCode"])
        self.assertIn("runner-out", state["stdoutTail"])
        self.assertIn(job.target_dir, state["stdoutTail"])
        self.assertIn("runner-err", state["stderrTail"])
        self.assertEqual(CargoJobStatus.RELEASED, self.service.get(job.job_id).status)

    def test_runner_reconciles_a_released_job_without_rewriting_raw_logs(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        log_root = Path(self.temporary_directory.name) / "terminal-run-logs"
        run_root = log_root / job.job_id / "run-terminal"
        run_root.mkdir(parents=True)
        stdout_path = run_root / "stdout.log"
        stderr_path = run_root / "stderr.log"
        stdout_path.write_text("preserved stdout\n", encoding="utf-8")
        stderr_path.write_text("preserved stderr\n", encoding="utf-8")
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=log_root,
        )
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test"],
        )
        self.service.process_alive = lambda _pid: False
        self.service.finish(job.job_id, session_id="session-a", exit_code=101)
        self.service.release(job.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cargo_job_runs(
                       run_id, job_id, session_id, command_json, environment_json, status,
                       stdout_path, stderr_path, started_at
                   ) VALUES (?, ?, ?, '[]', '{}', 'running', ?, ?, ?)""",
                (
                    "run-terminal",
                    job.job_id,
                    "session-a",
                    str(stdout_path),
                    str(stderr_path),
                    "2026-07-16T11:00:00+00:00",
                ),
            )

        self.assertEqual(("run-terminal",), runner.reconcile_terminal_runs())
        state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual("completed", state["status"])
        self.assertEqual(101, state["exitCode"])
        self.assertIn("preserved stdout", state["stdoutTail"])
        self.assertIn("preserved stderr", state["stderrTail"])
        self.assertTrue(stdout_path.exists())
        self.assertTrue(stderr_path.exists())

    def test_runner_reconciles_dead_orphaned_and_exitless_released_run_projections(self) -> None:
        log_root = Path(self.temporary_directory.name) / "reconciled-run-logs"
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=log_root,
        )
        jobs = []
        for run_id, job_status in (("run-orphaned", "orphaned"), ("run-released", "released")):
            job = self.service.acquire("session-a", CargoLaneKind.TEST)
            jobs.append((run_id, job, job_status))
            run_root = log_root / job.job_id / run_id
            run_root.mkdir(parents=True)
            with self.database.transaction() as connection:
                connection.execute(
                    """
                    UPDATE cargo_jobs
                    SET status=?, exit_code=NULL, process_tree_live_pids_json='[]',
                        released_at='2026-07-16T11:10:00+00:00'
                    WHERE job_id=?
                    """,
                    (job_status, job.job_id),
                )
                connection.execute(
                    """INSERT INTO cargo_job_runs(
                           run_id, job_id, session_id, command_json, environment_json, status,
                           stdout_path, stderr_path, started_at
                       ) VALUES (?, ?, 'session-a', '[]', '{}', 'running', ?, ?, ?)""",
                    (
                        run_id,
                        job.job_id,
                        str(run_root / "stdout.log"),
                        str(run_root / "stderr.log"),
                        "2026-07-16T11:00:00+00:00",
                    ),
                )

        self.assertEqual(
            ("run-orphaned", "run-released"), runner.reconcile_terminal_runs()
        )
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT run_id, status, exit_code, error_code FROM cargo_job_runs ORDER BY run_id"
            ).fetchall()

        self.assertEqual(
            [
                (
                    "run-orphaned",
                    "completed",
                    None,
                    "cargo_run_reconciled_from_orphaned_job",
                ),
                (
                    "run-released",
                    "completed",
                    None,
                    "cargo_run_reconciled_from_released_job_missing_exit_code",
                ),
            ],
            [tuple(row) for row in rows],
        )

    def test_runner_reconciles_uncollected_cleanup_unproven_process(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        log_root = Path(self.temporary_directory.name) / "cleanup-unproven-run-logs"
        run_root = log_root / job.job_id / "run-cleanup-unproven"
        run_root.mkdir(parents=True)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=log_root,
        )
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE cargo_jobs
                   SET status='orphaned', process_tree_live_pids_json='[]'
                   WHERE job_id=?""",
                (job.job_id,),
            )
            connection.execute(
                """INSERT INTO cargo_job_runs(
                       run_id, job_id, session_id, command_json, environment_json, status,
                       stdout_path, stderr_path, started_at
                   ) VALUES ('run-cleanup-unproven', ?, 'session-a', '[]', '{}', 'running',
                             ?, ?, '2026-07-16T11:00:00+00:00')""",
                (
                    job.job_id,
                    str(run_root / "stdout.log"),
                    str(run_root / "stderr.log"),
                ),
            )
        with runner._running_lock:
            runner._running[job.job_id] = object()

        self.assertEqual(
            ("run-cleanup-unproven",), runner.reconcile_terminal_runs(job_id=job.job_id)
        )
        self.assertEqual(
            "completed", runner.status(job.job_id, session_id="session-a")["status"]
        )

    def test_runner_rechecks_collector_registration_before_projection(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "collector-race-run-logs",
        )
        reconcile_query_entered = threading.Event()
        allow_reconcile_query = threading.Event()
        release_entered = threading.Event()
        allow_release = threading.Event()
        original_connect = runner.database.connect
        original_release = runner.cargo_jobs.release
        reconciled: list[tuple[str, ...]] = []

        @contextmanager
        def blocked_reconcile_connect():
            if threading.current_thread().name == "stale-reconcile":
                reconcile_query_entered.set()
                if not allow_reconcile_query.wait(timeout=5):
                    self.fail("reconcile query was not released")
            with original_connect() as connection:
                yield connection

        def delayed_release(*args, **kwargs):
            release_entered.set()
            if not allow_release.wait(timeout=5):
                self.fail("runner did not allow the delayed release to complete")
            return original_release(*args, **kwargs)

        def reconcile() -> None:
            reconciled.append(runner.reconcile_terminal_runs(job_id=job.job_id))

        with patch.object(runner.database, "connect", new=blocked_reconcile_connect), patch.object(
            runner.cargo_jobs, "release", side_effect=delayed_release
        ):
            reconcile_thread = threading.Thread(target=reconcile, name="stale-reconcile")
            reconcile_thread.start()
            self.assertTrue(reconcile_query_entered.wait(timeout=5))
            runner.start(
                session_id="session-a",
                job_id=job.job_id,
                command=(os.fspath(Path(os.sys.executable)), "-c", "raise SystemExit(0)"),
            )
            self.assertTrue(release_entered.wait(timeout=5))
            allow_reconcile_query.set()
            reconcile_thread.join(timeout=5)
            self.assertFalse(reconcile_thread.is_alive())
            projected_while_release_blocked = runner.status(
                job.job_id, session_id="session-a"
            )["status"]
            allow_release.set()
            deadline = datetime.now(UTC) + timedelta(seconds=5)
            state = runner.status(job.job_id, session_id="session-a")
            while state["status"] == "running" and datetime.now(UTC) < deadline:
                time.sleep(0.02)
                state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual([()], reconciled)
        self.assertEqual("running", projected_while_release_blocked)
        self.assertEqual("completed", state["status"])

    def test_runner_registers_collector_before_running_state_is_visible(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "collector-registration-logs",
        )
        registration_entered = threading.Event()
        allow_registration = threading.Event()
        original_lock = runner._running_lock
        start_result: list[object] = []
        start_errors: list[BaseException] = []
        maintenance_entered = threading.Event()
        orphaned: list[object] = []
        reconciled: list[tuple[str, ...]] = []

        class RegistrationGate:
            def __enter__(self):
                if threading.current_thread().name == "gapped-start":
                    registration_entered.set()
                    if not allow_registration.wait(timeout=5):
                        raise AssertionError("collector registration was not released")
                return original_lock.__enter__()

            def __exit__(self, *args):
                return original_lock.__exit__(*args)

        runner._running_lock = RegistrationGate()

        def start() -> None:
            try:
                start_result.append(
                    runner.start(
                        session_id="session-a",
                        job_id=job.job_id,
                        command=(
                            os.fspath(Path(os.sys.executable)),
                            "-c",
                            "raise SystemExit(0)",
                        ),
                    )
                )
            except BaseException as error:
                start_errors.append(error)

        def maintain() -> None:
            maintenance_entered.set()
            orphaned.extend(self.service.reconcile_orphans())
            reconciled.append(runner.reconcile_terminal_runs(job_id=job.job_id))

        start_thread = threading.Thread(target=start, name="gapped-start")
        start_thread.start()
        self.assertTrue(registration_entered.wait(timeout=5))
        maintenance_thread = threading.Thread(target=maintain, name="start-maintenance")
        maintenance_thread.start()
        self.assertTrue(maintenance_entered.wait(timeout=5))
        try:
            time.sleep(0.2)
            self.assertTrue(maintenance_thread.is_alive())
        finally:
            allow_registration.set()
        start_thread.join(timeout=5)
        maintenance_thread.join(timeout=5)
        self.assertFalse(start_thread.is_alive())
        self.assertFalse(maintenance_thread.is_alive())
        self.assertEqual([], start_errors)
        self.assertEqual(1, len(start_result))
        deadline = datetime.now(UTC) + timedelta(seconds=5)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual([], orphaned)
        self.assertEqual([()], reconciled)
        self.assertEqual("completed", state["status"])

    def test_runner_does_not_spawn_while_orphan_reconcile_owns_start_guard(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        reconcile_entered = threading.Event()
        allow_reconcile = threading.Event()
        popen_entered = threading.Event()
        start_result: list[object] = []
        start_errors: list[BaseException] = []
        original_reconcile = self.service._reconcile_orphans

        def blocked_reconcile(*args, **kwargs):
            reconcile_entered.set()
            if not allow_reconcile.wait(timeout=5):
                raise AssertionError("orphan reconciliation was not released")
            return original_reconcile(*args, **kwargs)

        def observed_popen(*args, **kwargs):
            popen_entered.set()
            return subprocess.Popen(*args, **kwargs)

        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "spawn-guard-run-logs",
            popen=observed_popen,
        )

        def start() -> None:
            try:
                start_result.append(
                    runner.start(
                        session_id="session-a",
                        job_id=job.job_id,
                        command=(
                            os.fspath(Path(os.sys.executable)),
                            "-c",
                            "import time; time.sleep(0.5)",
                        ),
                    )
                )
            except BaseException as error:
                start_errors.append(error)

        with patch.object(self.service, "_reconcile_orphans", side_effect=blocked_reconcile):
            maintenance_thread = threading.Thread(
                target=self.service.reconcile_orphans,
                name="guarded-reconcile",
            )
            maintenance_thread.start()
            self.assertTrue(reconcile_entered.wait(timeout=5))
            start_thread = threading.Thread(target=start, name="guarded-start")
            start_thread.start()
            try:
                time.sleep(0.2)
                spawned_while_blocked = popen_entered.is_set()
            finally:
                allow_reconcile.set()
            maintenance_thread.join(timeout=5)
            start_thread.join(timeout=5)
        self.assertFalse(spawned_while_blocked)
        self.assertFalse(maintenance_thread.is_alive())
        self.assertFalse(start_thread.is_alive())
        self.assertEqual([], start_errors)
        self.assertEqual(1, len(start_result))

        deadline = datetime.now(UTC) + timedelta(seconds=5)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")
        self.assertEqual("completed", state["status"])

    def test_runner_rejects_non_executable_owner_before_spawn(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET status=? WHERE session_id=?",
                (SessionStatus.STALE.value, "session-a"),
            )
        process = unittest.mock.Mock()
        process.pid = 4242
        process.poll.return_value = None
        spawn = unittest.mock.Mock(return_value=process)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "stale-owner-run-logs",
            popen=spawn,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            runner.start(
                session_id="session-a",
                job_id=job.job_id,
                command=("cargo", "test"),
            )

        self.assertEqual("cargo_session_not_executable", rejected.exception.code)
        spawn.assert_not_called()
        persisted = self.service.get(job.job_id)
        self.assertEqual(CargoJobStatus.LEASED, persisted.status)
        self.assertIsNone(persisted.pid)

    def test_runner_rejects_reserved_command_mismatch_before_spawn(self) -> None:
        compatibility = self.compatibility()
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=compatibility,
            command=("cargo", "test", "--lib"),
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.assertIsNone(reservation["jobId"])
        process = unittest.mock.Mock()
        process.pid = 4242
        process.poll.return_value = None
        spawn = unittest.mock.Mock(return_value=process)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "command-mismatch-run-logs",
            popen=spawn,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            runner.start(
                session_id="session-a",
                job_id=job.job_id,
                command=("cargo", "test", "--workspace"),
            )

        self.assertEqual("cargo_cpu_reservation_command_mismatch", rejected.exception.code)
        spawn.assert_not_called()
        persisted = self.service.get(job.job_id)
        self.assertEqual(CargoJobStatus.LEASED, persisted.status)
        self.assertIsNone(persisted.pid)

    def test_restart_reconciles_authorized_start_without_spawn_identity(self) -> None:
        compatibility = self.compatibility()
        command = ("cargo", "test", "--lib")
        reservation = self.service.reserve_cpu(
            "session-a", compatibility=compatibility, command=command
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )

        authorized = self.service.authorize_managed_start(
            job.job_id,
            session_id="session-a",
            command=command,
        )
        self.assertEqual(CargoJobStatus.RUNNING, authorized.status)
        self.assertIsNone(authorized.pid)

        successor = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda _pid: False,
        )
        orphaned = successor.reconcile_orphans()
        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, successor.get(job.job_id).status)
        with self.database.connect() as connection:
            persisted_reservation = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", persisted_reservation["status"])

    def test_atomic_runner_persists_pid_and_run_before_resuming_child(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        process = unittest.mock.Mock()
        process.pid = 4242
        process.stdout = unittest.mock.Mock()
        process.stdout.read.return_value = ""
        process.stderr = unittest.mock.Mock()
        process.stderr.read.return_value = ""
        released = threading.Event()

        def wait(*_args, **_kwargs):
            self.assertTrue(released.wait(timeout=5))
            return 0

        process.wait.side_effect = wait
        process.poll.return_value = None
        resume_observation: list[tuple[object, ...]] = []

        def observe_resume(_process) -> None:
            with self.database.connect() as connection:
                persisted_job = connection.execute(
                    "SELECT status, pid FROM cargo_jobs WHERE job_id=?",
                    (job.job_id,),
                ).fetchone()
                run_count = connection.execute(
                    "SELECT COUNT(*) FROM cargo_job_runs WHERE job_id=? AND status='running'",
                    (job.job_id,),
                ).fetchone()[0]
            with runner._running_lock:
                collecting = job.job_id in runner._collecting
            resume_observation.append(
                (persisted_job["status"], persisted_job["pid"], run_count, collecting)
            )
            released.set()

        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "atomic-resume-run-logs",
            atomic_popen=lambda *_args, **_kwargs: (process, 9001),
            resume_process=observe_resume,
            terminate_process_job=lambda _handle: None,
            wait_process_job=lambda _handle, **_kwargs: None,
        )
        with patch(
            "tools.session_coordinator.cargo_runner.popen_process_creation_time",
            return_value="stable:4242",
        ):
            runner.start(
                session_id="session-a",
                job_id=job.job_id,
                command=("cargo", "test"),
            )

        deadline = datetime.now(UTC) + timedelta(seconds=5)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")
        self.assertEqual([("running", 4242, 1, True)], resume_observation)
        self.assertEqual("completed", state["status"])

    @unittest.skipUnless(os.name == "nt", "Atomic Cargo runner uses Windows Job Objects")
    def test_atomic_windows_runner_resumes_captures_and_releases_job_tree(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "atomic-windows-run-logs",
        )
        run = runner.start(
            session_id="session-a",
            job_id=job.job_id,
            command=(
                os.fspath(Path(os.sys.executable)),
                "-c",
                "print('atomic cargo output')",
            ),
        )

        deadline = datetime.now(UTC) + timedelta(seconds=10)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual("completed", state["status"])
        self.assertEqual(0, state["exitCode"])
        self.assertIn("atomic cargo output", state["stdoutTail"])
        self.assertEqual(CargoJobStatus.RELEASED, self.service.get(job.job_id).status)
        self.assertTrue(Path(run.stdout_path).is_file())

    def test_restart_projects_unresumed_atomic_run_as_launch_failed(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        command = ("cargo", "test")
        run_id = "atomic-unresumed-run"
        run_root = Path(self.temporary_directory.name) / "unresumed-run"
        run_root.mkdir()
        self.service.authorize_managed_start(
            job.job_id, session_id="session-a", command=command
        )
        self.service.register_authorized_managed_run(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=command,
            run_id=run_id,
            environment={},
            stdout_path=run_root / "stdout.log",
            stderr_path=run_root / "stderr.log",
            started_at=utc_text(),
            root_process_creation_time="stable:4242",
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE cargo_jobs
                SET status='orphaned', finished_at=?,
                    process_tree_live_pids_json='[]', process_tree_exited_at=?
                WHERE job_id=?
                """,
                (utc_text(), utc_text(), job.job_id),
            )
        successor = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "unresumed-logs",
        )

        self.assertEqual((run_id,), successor.reconcile_terminal_runs())
        state = successor.status(job.job_id, session_id="session-a")
        self.assertEqual("launch_failed", state["status"])
        self.assertEqual("cargo_launch_interrupted_before_resume", state["errorCode"])

    def test_atomic_runner_never_resumes_when_log_reader_cannot_open(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        process = unittest.mock.Mock()
        process.pid = 4242
        process.stdout = unittest.mock.Mock()
        process.stderr = unittest.mock.Mock()
        process.poll.return_value = None
        resumed = unittest.mock.Mock()
        terminated: list[int | None] = []
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "reader-failure-logs",
            atomic_popen=lambda *_args, **_kwargs: (process, 9001),
            resume_process=resumed,
            terminate_process_job=terminated.append,
        )
        with patch(
            "tools.session_coordinator.cargo_runner.popen_process_creation_time",
            return_value="stable:4242",
        ), patch.object(Path, "open", side_effect=OSError("log open failed")):
            with self.assertRaises(CoordinatorError) as rejected:
                runner.start(
                    session_id="session-a",
                    job_id=job.job_id,
                    command=("cargo", "test"),
                )

        self.assertEqual("cargo_atomic_launch_log_open_failed", rejected.exception.code)
        resumed.assert_not_called()
        self.assertEqual([9001], terminated)

    def test_runner_durably_blocks_target_when_rejected_spawn_cannot_be_stopped(self) -> None:
        job = self.service.acquire(
            "session-a",
            CargoLaneKind.TEST,
            requested_target=self.target_root / "unproven-rejected-spawn",
        )

        class UnkillableProcess:
            pid = 4242

            @staticmethod
            def poll():
                return None

            @staticmethod
            def kill():
                raise OSError("simulated kill failure")

            @staticmethod
            def wait(*, timeout=None):
                raise TimeoutError(f"still alive after {timeout}")

        self.service.process_tree_pids = lambda pid: (pid,)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "rejected-spawn-run-logs",
            popen=lambda *_args, **_kwargs: UnkillableProcess(),
        )
        command = ("cargo", "test")
        with patch.object(
            self.service,
            "register_authorized_managed_run",
            side_effect=CoordinatorError("simulated_start_rejection", "rejected"),
        ), self.assertRaises(CoordinatorError) as rejected:
            runner.start(session_id="session-a", job_id=job.job_id, command=command)

        self.assertEqual("cargo_launch_cleanup_unproven", rejected.exception.code)
        persisted = self.service.get(job.job_id)
        self.assertEqual(CargoJobStatus.RUNNING, persisted.status)
        self.assertEqual(4242, persisted.pid)
        self.assertEqual("stable:4242", persisted.root_process_creation_time)
        self.assertEqual(command, persisted.command)
        self.assertEqual(
            (),
            self.service.reconcile_orphans(now=datetime.now(UTC) + timedelta(minutes=6)),
        )
        successor = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
            process_tree_pids=lambda pid: (pid,),
            supervisor_cargo_pids=lambda pid: (pid,),
            process_creation_time=lambda pid: f"stable:{pid}",
        )
        self.assertEqual(
            (),
            successor.reconcile_orphans(now=datetime.now(UTC) + timedelta(minutes=6)),
        )
        with self.assertRaises(CoordinatorError) as occupied:
            successor.acquire(
                "session-b",
                CargoLaneKind.TEST,
                requested_target=Path(job.target_dir),
            )
        self.assertEqual("cargo_lane_occupied", occupied.exception.code)

    def test_runner_releases_bound_cpu_reservation_after_owner_becomes_stale(self) -> None:
        compatibility = self.compatibility()
        command = (
            os.fspath(Path(os.sys.executable)),
            "-c",
            "import time; time.sleep(0.2); raise SystemExit(0)",
        )
        reservation = self.service.reserve_cpu(
            "session-a", compatibility=compatibility, command=command
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "bound-run-logs",
        )

        release_entered = threading.Event()
        allow_release = threading.Event()
        original_release = runner.cargo_jobs.release

        def delayed_release(*args, **kwargs):
            release_entered.set()
            if not allow_release.wait(timeout=5):
                self.fail("runner did not allow the delayed release to complete")
            return original_release(*args, **kwargs)

        with patch.object(runner.cargo_jobs, "release", side_effect=delayed_release):
            runner.start(session_id="session-a", job_id=job.job_id, command=command)
            self.sessions.set_status("session-a", SessionStatus.STALE)
            self.assertTrue(release_entered.wait(timeout=5))
            try:
                self.assertEqual(
                    "running", runner.status(job.job_id, session_id="session-a")["status"]
                )
            finally:
                allow_release.set()
            deadline = datetime.now(UTC) + timedelta(seconds=5)
            state = runner.status(job.job_id, session_id="session-a")
            while state["status"] == "running" and datetime.now(UTC) < deadline:
                time.sleep(0.02)
                state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual("completed", state["status"])
        self.assertEqual(CargoJobStatus.RELEASED, self.service.get(job.job_id).status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("released", row["status"])
        self.assertEqual(
            CargoJobStatus.RELEASED,
            self.service.release(job.job_id, session_id="session-a").status,
        )

    def test_coordinator_runner_records_allowed_environment(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        runner = CargoJobRunner(
            self.database,
            self.service,
            repo_root=self.repo,
            log_root=Path(self.temporary_directory.name) / "run-logs",
        )

        runner.start(
            session_id="session-a",
            job_id=job.job_id,
            command=(
                os.fspath(Path(os.sys.executable)),
                "-c",
                "import os; print(os.environ['RUSTFLAGS'])",
            ),
            environment={"RUSTFLAGS": "-C debuginfo=0 -C codegen-units=16"},
        )
        deadline = datetime.now(UTC) + timedelta(seconds=5)
        state = runner.status(job.job_id, session_id="session-a")
        while state["status"] == "running" and datetime.now(UTC) < deadline:
            time.sleep(0.02)
            state = runner.status(job.job_id, session_id="session-a")

        self.assertEqual("completed", state["status"])
        self.assertEqual(
            {"RUSTFLAGS": "-C debuginfo=0 -C codegen-units=16"},
            state["environment"],
        )
        self.assertIn("-C debuginfo=0 -C codegen-units=16", state["stdoutTail"])

    def test_gpu_lane_is_global_across_distinct_targets(self) -> None:
        first = self.service.acquire(
            "session-a",
            CargoLaneKind.GPU,
            requested_target=self.target_root / "gpu-a",
        )

        with self.assertRaises(CoordinatorError) as blocked:
            self.service.acquire(
                "session-b",
                CargoLaneKind.GPU,
                requested_target=self.target_root / "gpu-b",
            )

        self.assertEqual("cargo_gpu_lane_occupied", blocked.exception.code)
        self.assertEqual(first.job_id, blocked.exception.details["jobId"])

    def test_gpu_startup_audit_reports_existing_leases(self) -> None:
        first = self.service.acquire("session-a", CargoLaneKind.GPU)

        audit = self.service.audit_active_gpu_jobs()

        self.assertEqual((first.job_id,), tuple(job.job_id for job in audit))

    def test_start_rejection_records_the_job_and_error_code(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.start(
                job.job_id,
                session_id="session-b",
                pid=4242,
                command=["cargo", "test"],
            )

        self.assertEqual("cargo_job_owner_mismatch", rejected.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                """
                SELECT session_id, event_type, payload_json FROM events
                WHERE session_id=? ORDER BY event_id DESC LIMIT 1
                """,
                ("session-b",),
            ).fetchone()
        self.assertIsNotNone(row)
        self.assertEqual("cargo.start_rejected", row["event_type"])
        self.assertEqual(
            {
                "code": "cargo_job_owner_mismatch",
                "jobId": job.job_id,
                "pid": 4242,
                "rootIsSupervisor": False,
            },
            json.loads(row["payload_json"]),
        )

    def test_target_identity_is_case_and_separator_insensitive(self) -> None:
        self.assertEqual(
            target_identity(r"E:\cargo-targets\Check-A"),
            target_identity("e:/CARGO-TARGETS/check-a"),
        )

    def test_nested_lane_below_cargo_targets_is_accepted(self) -> None:
        requested = self.target_root / "zircon-shared/check"

        job = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        self.assertEqual(requested.resolve(), Path(job.target_dir))

    def test_legacy_targets_zircon_engine_root_is_rejected(self) -> None:
        legacy_root = self.target_root.parent / "targets/zircon-engine"
        legacy_root.mkdir(parents=True)

        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([legacy_root])

        self.assertEqual("invalid_target_root", rejected.exception.code)

    def test_all_three_allowlisted_root_names_are_accepted(self) -> None:
        roots = []
        for name in ("cargo-targets", "targets", "ZirconBuilds"):
            root = self.target_root.parent / name
            root.mkdir(parents=True, exist_ok=True)
            roots.append(root)

        policy = TargetPathPolicy(roots)

        self.assertEqual(
            tuple(root.resolve() for root in roots),
            policy.roots,
        )
        for root in roots:
            self.assertEqual(
                (root / "pool/example").resolve(),
                policy.validate(root / "pool/example"),
            )

    def test_production_config_exposes_three_root_names_per_available_drive(self) -> None:
        config = CoordinatorConfig.for_repo(self.repo)
        with patch("pathlib.Path.exists", return_value=True):
            roots = config.enabled_target_roots

        self.assertEqual(9, len(roots))
        self.assertEqual(
            ["cargo-targets", "targets", "zirconbuilds"] * 3,
            [root.name.casefold() for root in roots],
        )

    def test_arbitrary_root_name_is_rejected(self) -> None:
        arbitrary = self.target_root.parent / "other-builds"
        arbitrary.mkdir()

        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([arbitrary])

        self.assertEqual("invalid_target_root", rejected.exception.code)

    def test_no_configured_target_drive_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([])
        self.assertEqual("target_root_unavailable", rejected.exception.code)

    def test_symlink_lane_escape_is_rejected_when_supported(self) -> None:
        outside = self.target_root.parent / "outside"
        outside.mkdir()
        link = self.target_root / "escaped-link"
        link.parent.mkdir(parents=True, exist_ok=True)
        try:
            link.symlink_to(outside, target_is_directory=True)
        except OSError as error:
            if os.name != "nt":
                self.skipTest(f"directory symlink is unavailable: {error}")
            junction = subprocess.run(
                ["cmd.exe", "/d", "/c", "mklink", "/J", str(link), str(outside)],
                capture_output=True,
                text=True,
                check=False,
            )
            if junction.returncode != 0:
                self.skipTest(
                    f"directory symlink and junction are unavailable: {junction.stderr}"
                )

        with self.assertRaises(CoordinatorError) as rejected:
            self.policy.validate(link)
        self.assertEqual("cargo_target_not_managed", rejected.exception.code)

    def test_released_explicit_compatible_lane_can_be_reused(self) -> None:
        requested = self.target_root / "manual-check"
        first = self.service.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=requested,
            compatibility=self.compatibility(),
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            requested_target=requested,
            compatibility=self.compatibility(),
        )

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertEqual(first.target_dir, second.target_dir)
        self.assertTrue(requested.is_dir())

    def test_released_compatible_lane_is_reused_across_sessions(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertEqual(first.target_dir, second.target_dir)
        self.assertEqual(first.job_id, second.reused_from_job_id)
        self.assertEqual(first.reuse_key, second.reuse_key)

    def test_source_and_lock_changes_reuse_cargo_fingerprint_pool(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        (self.repo / "Cargo.lock").write_text("changed", encoding="utf-8")

        second = self.service.acquire(
            "session-b", CargoLaneKind.CHECK, compatibility=compatibility
        )

        self.assertEqual(first.reuse_key, second.reuse_key)
        self.assertEqual(first.target_dir, second.target_dir)

    def test_missing_primary_is_retired_before_deterministic_replacement(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        Path(first.target_dir).rmdir()

        second = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertEqual("deleted", self.service.get(first.job_id).cleanup_status.value)
        self.assertEqual(first.reuse_key, second.reuse_key)
        self.assertEqual(first.target_dir, second.target_dir)

    def test_duplicate_retained_pool_is_demoted_to_immediate_cleanup(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        newer_target = self.target_root / "newer-primary"
        newer_target.mkdir()
        newer_job_id = "newer-retained-pool"
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, dry_run,
                    created_at, last_heartbeat_at, released_at, target_key,
                    reuse_key, compatibility_json, compatibility_key, reuse_profile,
                    cleanup_policy, cleanup_status
                )
                SELECT ?, session_id, lane_kind, ?, 'released', dry_run,
                       created_at, last_heartbeat_at, ?, ?, reuse_key,
                       compatibility_json, compatibility_key, reuse_profile,
                       'retained', 'retained'
                FROM cargo_jobs WHERE job_id=?
                """,
                (
                    newer_job_id,
                    str(newer_target),
                    (datetime.now(UTC) + timedelta(minutes=1)).isoformat(),
                    target_identity(newer_target),
                    first.job_id,
                ),
            )

        acquired = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        retired = self.service.get(first.job_id)
        self.assertEqual(str(newer_target.resolve()), acquired.target_dir)
        self.assertEqual(newer_job_id, acquired.reused_from_job_id)
        self.assertEqual("delete_on_release", retired.cleanup_policy.value)
        self.assertEqual("pending", retired.cleanup_status.value)

    def test_incompatible_build_configuration_gets_a_distinct_pool(self) -> None:
        first = self.service.acquire(
            "session-a",
            CargoLaneKind.TEST,
            compatibility=self.compatibility(build_config="profile=test;features=default"),
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b",
            CargoLaneKind.TEST,
            compatibility=self.compatibility(build_config="profile=dev;features=default"),
        )

        self.assertNotEqual(first.reuse_key, second.reuse_key)
        self.assertNotEqual(first.target_dir, second.target_dir)
        self.assertEqual("retained", self.service.get(first.job_id).cleanup_policy.value)

    def test_windows_and_wsl_never_share_a_pool(self) -> None:
        windows = self.service.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(),
        )
        self.service.release(windows.job_id, session_id="session-a")

        wsl = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(
                platform="wsl",
                toolchain="stable-x86_64-unknown-linux-gnu",
                target_architecture="x86_64-unknown-linux-gnu",
            ),
        )

        self.assertNotEqual(windows.reuse_key, wsl.reuse_key)
        self.assertNotEqual(windows.target_dir, wsl.target_dir)

    def test_compatible_pool_is_busy_instead_of_allocating_a_fallback(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-b", CargoLaneKind.TEST, compatibility=compatibility
            )

        self.assertEqual("cargo_reuse_pool_busy", rejected.exception.code)
        compatible_jobs = [
            job for job in self.service.list() if job.reuse_key == first.reuse_key
        ]
        self.assertEqual([first.target_dir], [job.target_dir for job in compatible_jobs])

    def test_incomplete_compatibility_document_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=dev",
                ),
            )

        self.assertEqual("invalid_cargo_compatibility", rejected.exception.code)

    def test_missing_compatibility_is_ephemeral_by_default(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        self.assertIsNone(job.reuse_key)
        self.assertEqual("delete_on_release", job.cleanup_policy.value)
        self.assertEqual("pending", job.cleanup_status.value)

    def test_ephemeral_and_compatibility_are_mutually_exclusive(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                ephemeral=True,
                compatibility=self.compatibility(),
            )

        self.assertEqual("cargo_compatibility_conflict", rejected.exception.code)

    def test_ephemeral_lane_is_marked_for_release_cleanup(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        released = self.service.release(job.job_id, session_id="session-a")

        self.assertEqual("delete_on_release", released.cleanup_policy.value)
        self.assertEqual("pending", released.cleanup_status.value)
        self.assertIsNone(released.reuse_key)

    def test_foreign_session_cannot_mutate_job(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.start(
                job.job_id,
                session_id="session-b",
                pid=4242,
                command=["cargo", "check"],
            )
        self.assertEqual("cargo_job_owner_mismatch", rejected.exception.code)

    def test_dead_prestart_owner_is_reconciled_after_leased_timeout(self) -> None:
        job = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, owner_pid=9999
        )

        orphaned = self.service.reconcile_orphans(
            now=datetime.now(UTC) + timedelta(minutes=10),
            leased_timeout_seconds=300,
        )

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, self.service.get(job.job_id).status)

    def test_running_finish_and_release_preserve_job_audit(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.WORKSPACE)
        running = self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        self.service.process_tree_pids = lambda _root_pid: ()
        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        released = self.service.release(job.job_id, session_id="session-a")

        self.assertEqual(CargoJobStatus.RUNNING, running.status)
        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual(CargoJobStatus.RELEASED, released.status)
        self.assertEqual(0, released.exit_code)
        self.assertEqual(("cargo", "test"), released.command)

    def test_finish_rejects_live_process_and_keeps_compatible_target_owned(self) -> None:
        compatibility = self.compatibility()
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finish(job.job_id, session_id="session-a", exit_code=130)

        self.assertEqual("cargo_process_tree_alive", rejected.exception.code)
        self.assertEqual(CargoJobStatus.RUNNING, self.service.get(job.job_id).status)
        with self.assertRaises(CoordinatorError) as occupied:
            self.service.acquire(
                "session-b", CargoLaneKind.TEST, compatibility=compatibility
            )
        self.assertEqual("cargo_reuse_pool_busy", occupied.exception.code)

    def test_finish_rejects_live_child_when_registered_parent_has_exited(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.process_tree_pids = lambda root_pid: (3300, 35876) if root_pid == 9999 else ()
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finish(job.job_id, session_id="session-a", exit_code=130)

        self.assertEqual("cargo_process_tree_alive", rejected.exception.code)
        self.assertEqual(CargoJobStatus.RUNNING, self.service.get(job.job_id).status)
        self.assertEqual([3300, 35876], rejected.exception.details["livePids"])

    def test_release_rejects_late_live_descendant_and_blocks_reuse(self) -> None:
        compatibility = self.compatibility()
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        live_pids: tuple[int, ...] = ()
        self.service.process_tree_pids = lambda _root_pid: live_pids
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )
        self.service.finish(job.job_id, session_id="session-a", exit_code=130)
        live_pids = (3300, 35876)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.release(job.job_id, session_id="session-a")

        self.assertEqual("cargo_process_tree_alive", rejected.exception.code)
        self.assertEqual(CargoJobStatus.FAILED, self.service.get(job.job_id).status)
        self.assertEqual((3300, 35876), self.service.get(job.job_id).live_process_pids)
        with self.assertRaises(CoordinatorError) as occupied:
            self.service.acquire(
                "session-b", CargoLaneKind.TEST, compatibility=compatibility
            )
        self.assertEqual("cargo_process_tree_alive", occupied.exception.code)

    def test_orphan_reconcile_keeps_live_descendant_of_exited_registered_parent(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.process_tree_pids = lambda root_pid: (3300,) if root_pid == 9999 else ()
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )

        self.assertEqual((), self.service.reconcile_orphans())
        observed = self.service.get(job.job_id)
        self.assertEqual(CargoJobStatus.RUNNING, observed.status)
        self.assertEqual((3300,), observed.live_process_pids)

    def test_owner_finish_recovers_a_race_with_orphan_reconciliation(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )
        self.assertEqual((job.job_id,), tuple(item.job_id for item in self.service.reconcile_orphans()))

        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)

        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual(0, finished.exit_code)

    def test_dry_run_allocates_without_creating_target(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.GPU, dry_run=True)

        self.assertFalse(Path(job.target_dir).exists())
        self.assertTrue(job.dry_run)

    def test_reconcile_marks_dead_running_process_as_orphaned(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )

        orphaned = self.service.reconcile_orphans()

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, self.service.get(job.job_id).status)

    def test_reconcile_keeps_live_running_process(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )

        self.assertEqual((), self.service.reconcile_orphans())
        self.assertEqual(CargoJobStatus.RUNNING, self.service.get(job.job_id).status)

    def test_reconcile_reports_a_stale_live_job_without_freezing_other_lanes(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        now = datetime.now(UTC)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET last_heartbeat_at=? WHERE job_id=?",
                ((now - timedelta(seconds=61)).isoformat(), job.job_id),
            )

        self.assertEqual(
            (),
            self.service.reconcile_orphans(
                now=now,
                running_health_timeout_seconds=60,
            ),
        )
        self.assertEqual(CargoJobStatus.RUNNING, self.service.get(job.job_id).status)
        with self.database.connect() as connection:
            events = connection.execute(
                "SELECT event_type FROM events WHERE event_type='cargo.health_timeout'"
            ).fetchall()
        self.assertEqual(1, len(events))

        self.service.reconcile_orphans(now=now, running_health_timeout_seconds=60)
        with self.database.connect() as connection:
            events = connection.execute(
                "SELECT event_type FROM events WHERE event_type='cargo.health_timeout'"
            ).fetchall()
        self.assertEqual(1, len(events))

    def test_supervisor_finish_ignores_its_own_root_after_children_exit(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.process_tree_pids = lambda root_pid: (4242,) if root_pid == 4242 else ()
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["powershell", "validate-matrix.ps1"],
            root_is_supervisor=True,
        )

        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)

        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual((), finished.live_process_pids)

    def test_supervisor_finish_ignores_a_non_cargo_control_descendant(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.process_tree_pids = (
            lambda root_pid: (4242, 3300) if root_pid == 4242 else ()
        )
        self.service.supervisor_cargo_pids = lambda _root_pid: ()
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["powershell", "validate-matrix.ps1"],
            root_is_supervisor=True,
        )

        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)

        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual((), finished.live_process_pids)

    def test_supervisor_finish_rejects_a_live_descendant(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.process_tree_pids = (
            lambda root_pid: (4242, 3300) if root_pid == 4242 else ()
        )
        self.service.supervisor_cargo_pids = lambda root_pid: (3300,) if root_pid == 4242 else ()
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["powershell", "validate-matrix.ps1"],
            root_is_supervisor=True,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finish(job.job_id, session_id="session-a", exit_code=0)

        self.assertEqual("cargo_process_tree_alive", rejected.exception.code)
        self.assertEqual([3300], rejected.exception.details["livePids"])

    def test_legacy_released_job_never_reclaims_a_reused_pid(self) -> None:
        compatibility = self.compatibility()
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        self.service.process_tree_pids = lambda _root_pid: ()
        self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        released = self.service.release(job.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET root_process_creation_time=NULL WHERE job_id=?",
                (released.job_id,),
            )
        self.service.process_tree_pids = lambda root_pid: (4242,) if root_pid == 4242 else ()

        reused = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertEqual(released.target_dir, reused.target_dir)

    def test_terminal_job_with_recorded_exit_ignores_unreadable_reused_pid(self) -> None:
        compatibility = self.compatibility()
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.process_creation_times[4242] = "cargo-root-v1"
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        self.service.process_tree_pids = lambda _root_pid: ()
        self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        released = self.service.release(job.job_id, session_id="session-a")
        self.service.process_creation_time = lambda _pid: None
        self.service.process_tree_pids = lambda root_pid: (4242,) if root_pid == 4242 else ()

        reused = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertIsNotNone(released.process_tree_exited_at)
        self.assertEqual(released.target_dir, reused.target_dir)

    def test_pid_reuse_orphans_the_original_job_and_allows_target_reuse(self) -> None:
        compatibility = self.compatibility()
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.process_creation_times[4242] = "cargo-root-v1"
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        self.process_creation_times[4242] = "renderdoc-mcp-v2"

        orphaned = self.service.reconcile_orphans()

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        observed = self.service.get(job.job_id)
        self.assertEqual(CargoJobStatus.ORPHANED, observed.status)
        self.assertEqual((), observed.live_process_pids)
        self.service.release(job.job_id, session_id="session-a")
        reused = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )
        self.assertEqual(job.target_dir, reused.target_dir)


if __name__ == "__main__":
    unittest.main()
