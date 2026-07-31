from __future__ import annotations

import importlib
import json
import os
import tempfile
import threading
import time
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest import mock

from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoJobStatus,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.cargo_runner import CargoJobRunner
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class ReservedStartTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive" / "cargo-targets"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.cargo_jobs = CargoJobService(
            self.database,
            TargetPathPolicy([self.target_root]),
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda _pid: False,
            process_tree_pids=lambda _pid: (),
            supervisor_cargo_pids=lambda pid: (pid,),
        )
        self.runner = CargoJobRunner(
            self.database,
            self.cargo_jobs,
            repo_root=self.repo,
            log_root=root / "run-logs",
        )
        self.scheduled: list[object] = []
        try:
            module = importlib.import_module("tools.session_coordinator.reserved_starts")
        except ModuleNotFoundError:
            module = None
        service_class = getattr(module, "ReservedCargoStartService", None)
        self.assertIsNotNone(
            service_class,
            "ReservedCargoStartService must durably own proof-bound launch admission",
        )
        self.proof_pairs: list[tuple[str, str, str]] = []

        def proof_guard(_connection, reservation_id: str, session_id: str, job_id: str) -> None:
            self.proof_pairs.append((reservation_id, session_id, job_id))

        self.starts = service_class(
            self.database,
            self.cargo_jobs,
            self.runner,
            proof_guard=proof_guard,
            scheduler=self.scheduled.append,
            start_deadline_seconds=900,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    @staticmethod
    def compatibility() -> CargoCompatibility:
        return CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="Cargo.toml",
            build_config="profile=test;features=default;rustflags=;incremental=0;debug=0",
        )

    def _reserved_job(self, command: tuple[str, ...]):
        reservation = self.cargo_jobs.reserve_cpu(
            "session-a", compatibility=self.compatibility(), command=command
        )
        job = self.cargo_jobs.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )
        return reservation, job

    def test_disconnect_ack_protects_start_pending_and_duplicate_is_exactly_once(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "import time; time.sleep(0.2)")
        reservation, job = self._reserved_job(command)
        request_id = "d" * 32

        first = self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )
        duplicate = self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )

        self.assertEqual(first, duplicate)
        self.assertEqual("start_pending", first["status"])
        self.assertEqual(1, len(self.scheduled))
        self.assertEqual(
            [(reservation["reservationId"], "session-a", job.job_id)], self.proof_pairs
        )

        orphaned = self.cargo_jobs.reconcile_orphans(
            now=datetime.now(UTC) + timedelta(minutes=6), leased_timeout_seconds=300
        )
        self.assertEqual((), orphaned)
        self.assertEqual(CargoJobStatus.LEASED, self.cargo_jobs.get(job.job_id).status)

        self.scheduled.pop()()
        started = self.starts.status(request_id)
        with self.database.connect() as connection:
            persisted_job = connection.execute(
                "SELECT pid, command_json, started_at FROM cargo_jobs WHERE job_id=?",
                (job.job_id,),
            ).fetchone()
            run_count = connection.execute(
                "SELECT COUNT(*) FROM cargo_job_runs WHERE job_id=?", (job.job_id,)
            ).fetchone()[0]
        self.assertEqual("started", started["status"])
        self.assertIsNotNone(persisted_job["pid"])
        self.assertEqual(list(command), json.loads(persisted_job["command_json"]))
        self.assertIsNotNone(persisted_job["started_at"])
        self.assertEqual(1, run_count)
        deadline = time.monotonic() + 5
        run_status = self.runner.status(job.job_id, session_id="session-a")
        while run_status["status"] == "running" and time.monotonic() < deadline:
            time.sleep(0.02)
            run_status = self.runner.status(job.job_id, session_id="session-a")
        self.assertEqual("completed", run_status["status"])

    def test_launch_failure_is_terminal_without_fabricating_cargo_exit(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "raise SystemExit(0)")
        reservation, job = self._reserved_job(command)
        request_id = "e" * 32
        self.runner.popen = lambda *_args, **_kwargs: (_ for _ in ()).throw(
            OSError("simulated spawn failure")
        )

        self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )
        self.scheduled.pop()()

        status = self.starts.status(request_id)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, pid, command_json, exit_code, started_at FROM cargo_jobs WHERE job_id=?",
                (job.job_id,),
            ).fetchone()
            run_count = connection.execute(
                "SELECT COUNT(*) FROM cargo_job_runs WHERE job_id=?", (job.job_id,)
            ).fetchone()[0]
        self.assertEqual("launch_failed", status["status"])
        self.assertEqual("cargo_launch_failed", status["errorCode"])
        self.assertEqual(("released", None, "[]", None, None), tuple(row))
        self.assertEqual(0, run_count)

    def test_post_registration_runner_failure_is_not_recovered_as_started(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "import time; time.sleep(5)")
        reservation, job = self._reserved_job(command)
        request_id = "7" * 32
        self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )

        with mock.patch(
            "tools.session_coordinator.cargo_runner.threading.Thread.start",
            side_effect=RuntimeError("simulated monitor thread failure"),
        ):
            self.scheduled.pop()()

        status = self.starts.status(request_id)
        with self.database.connect() as connection:
            persisted_job = connection.execute(
                "SELECT status, exit_code FROM cargo_jobs WHERE job_id=?", (job.job_id,)
            ).fetchone()
            persisted_run = connection.execute(
                "SELECT status, exit_code, error_code FROM cargo_job_runs WHERE job_id=?",
                (job.job_id,),
            ).fetchone()
        self.assertEqual("launch_failed", status["status"])
        self.assertEqual(("released", None), tuple(persisted_job))
        self.assertEqual(("launch_failed", None, "cargo_launch_failed"), tuple(persisted_run))

    def test_unproven_spawn_cleanup_keeps_live_job_owned(self) -> None:
        command = ("cargo", "test")
        reservation, job = self._reserved_job(command)
        request_id = "6" * 32
        self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )

        class UnkillableProcess:
            pid = os.getpid()

            @staticmethod
            def poll():
                return None

            @staticmethod
            def kill():
                raise OSError("simulated kill failure")

            @staticmethod
            def wait(*, timeout=None):
                raise TimeoutError(f"still alive after {timeout}")

        self.runner.popen = lambda *_args, **_kwargs: UnkillableProcess()
        with mock.patch(
            "tools.session_coordinator.cargo_runner.threading.Thread.start",
            side_effect=RuntimeError("simulated monitor thread failure"),
        ):
            self.scheduled.pop()()

        status = self.starts.status(request_id)
        with self.database.connect() as connection:
            persisted_job = connection.execute(
                "SELECT status FROM cargo_jobs WHERE job_id=?", (job.job_id,)
            ).fetchone()
            persisted_reservation = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
            persisted_run = connection.execute(
                "SELECT status, exit_code FROM cargo_job_runs WHERE job_id=?", (job.job_id,)
            ).fetchone()
        self.assertEqual("launch_failed", status["status"])
        self.assertEqual("cargo_launch_cleanup_unproven", status["errorCode"])
        self.assertEqual("running", persisted_job["status"])
        self.assertEqual("running", persisted_reservation["status"])
        self.assertEqual(("running", None), tuple(persisted_run))

    def test_dedicated_deadline_terminalizes_pending_launch(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "raise SystemExit(0)")
        reservation, job = self._reserved_job(command)
        request_id = "f" * 32
        ack = self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )

        expired = self.starts.reconcile_expired(
            now=datetime.fromisoformat(ack["deadlineAt"]) + timedelta(seconds=1)
        )

        self.assertEqual((request_id,), expired)
        self.assertEqual("launch_failed", self.starts.status(request_id)["status"])
        self.assertEqual(CargoJobStatus.RELEASED, self.cargo_jobs.get(job.job_id).status)

    def test_successor_immediately_terminalizes_unlaunched_pending_ack(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "raise SystemExit(0)")
        reservation, job = self._reserved_job(command)
        request_id = "5" * 32
        self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )
        successor_scheduled: list[object] = []
        successor = type(self.starts)(
            self.database,
            self.cargo_jobs,
            self.runner,
            proof_guard=lambda *_args: None,
            scheduler=successor_scheduled.append,
            start_deadline_seconds=900,
        )

        reconciled = successor.reconcile_interrupted()

        status = successor.status(request_id)
        with self.database.connect() as connection:
            run_count = connection.execute(
                "SELECT COUNT(*) FROM cargo_job_runs WHERE job_id=?", (job.job_id,)
            ).fetchone()[0]
        self.assertEqual((request_id,), reconciled)
        self.assertEqual("launch_failed", status["status"])
        self.assertEqual("cargo_launch_interrupted_before_spawn", status["errorCode"])
        self.assertEqual(CargoJobStatus.RELEASED, self.cargo_jobs.get(job.job_id).status)
        self.assertEqual(0, run_count)
        self.assertEqual([], successor_scheduled)

    def test_deadline_reconcile_cannot_terminalize_a_registered_launch(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "import time; time.sleep(0.2)")
        reservation, job = self._reserved_job(command)
        request_id = "a" * 32
        ack = self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )
        registered = threading.Event()
        release_launch = threading.Event()
        original_start = self.runner.start

        def blocked_after_registration(**kwargs):
            run = original_start(**kwargs)
            registered.set()
            self.assertTrue(release_launch.wait(timeout=5))
            return run

        self.runner.start = blocked_after_registration
        launch = threading.Thread(target=self.scheduled.pop())
        launch.start()
        self.assertTrue(registered.wait(timeout=5))
        expired: list[tuple[str, ...]] = []
        reconcile = threading.Thread(
            target=lambda: expired.append(
                self.starts.reconcile_expired(
                    now=datetime.fromisoformat(ack["deadlineAt"]) + timedelta(seconds=1)
                )
            )
        )
        reconcile.start()
        reconcile.join(timeout=0.1)
        release_launch.set()
        launch.join(timeout=5)
        reconcile.join(timeout=5)
        deadline = time.monotonic() + 5
        run_status = self.runner.status(job.job_id, session_id="session-a")
        while run_status["status"] == "running" and time.monotonic() < deadline:
            time.sleep(0.02)
            run_status = self.runner.status(job.job_id, session_id="session-a")
        while time.monotonic() < deadline:
            with self.runner._running_lock:
                if job.job_id not in self.runner._running:
                    break
            time.sleep(0.02)

        self.assertFalse(launch.is_alive())
        self.assertFalse(reconcile.is_alive())
        self.assertEqual([()], expired)
        self.assertEqual("started", self.starts.status(request_id)["status"])
        with self.database.connect() as connection:
            run_count = connection.execute(
                "SELECT COUNT(*) FROM cargo_job_runs WHERE job_id=?", (job.job_id,)
            ).fetchone()[0]
        self.assertEqual(1, run_count)

    def test_deadline_terminalizes_slow_preflight_before_spawn(self) -> None:
        command = (os.fspath(Path(os.sys.executable)), "-c", "raise SystemExit(0)")
        reservation, job = self._reserved_job(command)
        request_id = "b" * 32
        ack = self.starts.accept(
            request_id=request_id,
            reservation_id=reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
            command=command,
        )
        preflight_entered = threading.Event()
        release_preflight = threading.Event()
        runner_calls: list[str] = []

        def slow_preflight(*_args, **_kwargs):
            preflight_entered.set()
            self.assertTrue(release_preflight.wait(timeout=5))
            return {}

        def forbidden_start(**_kwargs):
            runner_calls.append("started")
            raise RuntimeError("runner must not start after the launch deadline")

        self.cargo_jobs.reserved_run_environment = slow_preflight
        self.runner.start = forbidden_start
        launch = threading.Thread(target=self.scheduled.pop())
        launch.start()
        self.assertTrue(preflight_entered.wait(timeout=5))
        expired: list[tuple[str, ...]] = []
        reconcile = threading.Thread(
            target=lambda: expired.append(
                self.starts.reconcile_expired(
                    now=datetime.fromisoformat(ack["deadlineAt"]) + timedelta(seconds=1)
                )
            )
        )
        reconcile.start()
        reconcile.join(timeout=0.2)
        reconciled_before_release = not reconcile.is_alive()
        release_preflight.set()
        launch.join(timeout=5)
        reconcile.join(timeout=5)

        self.assertTrue(reconciled_before_release)
        self.assertEqual([(request_id,)], expired)
        self.assertEqual([], runner_calls)
        self.assertEqual("launch_failed", self.starts.status(request_id)["status"])

    def test_ordinary_unacked_lease_keeps_existing_300_second_watchdog(self) -> None:
        job = self.cargo_jobs.acquire("session-a", CargoLaneKind.TEST)

        orphaned = self.cargo_jobs.reconcile_orphans(
            now=datetime.now(UTC) + timedelta(minutes=6), leased_timeout_seconds=300
        )

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, self.cargo_jobs.get(job.job_id).status)


if __name__ == "__main__":
    unittest.main()
