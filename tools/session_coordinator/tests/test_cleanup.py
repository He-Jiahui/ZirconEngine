from __future__ import annotations

import json
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.cargo_jobs import CargoJobService, CargoLaneKind, TargetPathPolicy
from tools.session_coordinator.cleanup import CleanupService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.models import utc_text
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CleanupTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/targets/zircon-engine"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.alive_pids: set[int] = {4242}
        self.jobs = CargoJobService(
            self.database,
            TargetPathPolicy([self.target_root]),
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid in self.alive_pids,
        )
        self.cleanup = CleanupService(
            self.database,
            self.jobs,
            process_alive=lambda pid: pid in self.alive_pids,
            free_space=lambda _path: 40 * 1024**3,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_active_pid_and_lease_are_never_cleanup_candidates(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.TEST)
        (Path(job.target_dir) / "artifact").write_text("live", encoding="utf-8")
        self.jobs.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )

        plan = self.cleanup.plan(now=datetime.now(UTC) + timedelta(days=2), older_than_hours=1)

        self.assertEqual([], list(plan.candidates))
        self.assertTrue(any(item.code == "active_process" for item in plan.denied))
        self.assertTrue(Path(job.target_dir).exists())

    def test_released_stale_lane_is_deleted_only_after_plan(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        (Path(job.target_dir) / "artifact").write_text("stale", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        plan = self.cleanup.plan(now=cleanup_time, older_than_hours=1)

        applied = self.cleanup.apply(plan, now=cleanup_time)

        self.assertEqual((job.target_dir,), plan.candidates)
        self.assertEqual((job.target_dir,), applied.deleted)
        self.assertFalse(Path(job.target_dir).exists())

    def test_apply_revalidates_process_liveness(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        plan = self.cleanup.plan(now=datetime.now(UTC) + timedelta(days=2), older_than_hours=1)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET pid = ? WHERE job_id = ?", (4242, job.job_id)
            )

        applied = self.cleanup.apply(plan)

        self.assertEqual((), applied.deleted)
        self.assertTrue(any(item.code == "active_process" for item in applied.denied))
        self.assertTrue(Path(job.target_dir).exists())

    def test_apply_revalidates_retention_window(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        plan = self.cleanup.plan(now=datetime.now(UTC) + timedelta(days=2), older_than_hours=1)

        applied = self.cleanup.apply(plan)

        self.assertEqual((), applied.deleted)
        self.assertTrue(any(item.code == "retention_active" for item in applied.denied))
        self.assertTrue(Path(job.target_dir).exists())

    def test_target_policy_rejects_parent_and_escape_paths(self) -> None:
        with self.assertRaises(CoordinatorError):
            self.jobs.target_policy.validate(self.target_root.parent)
        with self.assertRaises(CoordinatorError):
            self.jobs.target_policy.validate(self.target_root / "lanes/../../escape")

    def test_plan_reports_low_disk_pressure_for_managed_roots(self) -> None:
        plan = self.cleanup.plan()

        self.assertEqual(((str(self.target_root.resolve()), 40 * 1024**3),), plan.free_bytes_by_root)
        self.assertEqual((str(self.target_root.resolve()),), plan.pressure_roots)

    def test_non_positive_retention_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            self.cleanup.plan(older_than_hours=0)

    def test_cleanup_plan_never_expands_to_newer_candidates(self) -> None:
        first = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(first.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        reviewed = self.cleanup.plan(now=cleanup_time, older_than_hours=1)

        second = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(second.job_id, session_id="session-a")
        applied = self.cleanup.apply(reviewed, now=cleanup_time)

        self.assertEqual((first.target_dir,), applied.deleted)
        self.assertTrue(Path(second.target_dir).exists())

    def test_cleanup_reservation_blocks_reacquire_without_holding_writer_lock(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, datetime('now'))",
                (job.target_dir.replace("/", "\\").casefold(), job.target_dir),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.acquire(
                "session-a", CargoLaneKind.CHECK, requested_target=job.target_dir
            )
        self.assertEqual("cargo_lane_cleanup_reserved", rejected.exception.code)

    def test_service_restart_recovers_abandoned_cleanup_reservations(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, datetime('now'))",
                ("abandoned", str(self.target_root / "lanes/abandoned")),
            )

        self.assertEqual(1, self.cleanup.recover_reservations())
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations"
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    def test_persisted_plan_still_refuses_untracked_managed_directory(self) -> None:
        untracked = self.target_root / "lanes/untracked"
        untracked.mkdir(parents=True)
        generated_at = datetime.now(UTC)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cleanup_plans(
                    plan_id, generated_at, older_than_hours, candidates_json, status
                ) VALUES (?, ?, 1, ?, 'planned')
                """,
                ("forged-plan", generated_at.isoformat(), json.dumps([str(untracked)])),
            )

        applied = self.cleanup.apply(
            self.cleanup.get_plan("forged-plan"), now=generated_at
        )

        self.assertEqual((), applied.deleted)
        self.assertTrue(any(item.code == "untracked_target" for item in applied.denied))
        self.assertTrue(untracked.exists())

    def test_cleanup_refuses_active_legacy_descendant(self) -> None:
        parent = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(parent.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        reviewed = self.cleanup.plan(now=cleanup_time, older_than_hours=1)
        child = str(Path(parent.target_dir) / "legacy-child")
        now = utc_text()
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, target_key,
                    status, dry_run, created_at, last_heartbeat_at
                ) VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?)
                """,
                (
                    "legacy-child-job",
                    "session-a",
                    "check",
                    child,
                    child.replace("/", "\\").casefold(),
                    "leased",
                    now,
                    now,
                ),
            )

        applied = self.cleanup.apply(reviewed, now=cleanup_time)

        self.assertEqual((), applied.deleted)
        self.assertTrue(any(item.code == "active_lease" for item in applied.denied))
        self.assertTrue(Path(parent.target_dir).exists())

    def test_cleanup_deletes_outside_writer_transaction_and_blocks_reacquire(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        reviewed = self.cleanup.plan(now=cleanup_time, older_than_hours=1)
        original_rmtree = __import__("shutil").rmtree

        def delete_while_competing(path: Path) -> None:
            with self.assertRaises(CoordinatorError) as rejected:
                self.jobs.acquire(
                    "session-a", CargoLaneKind.CHECK, requested_target=path
                )
            self.assertEqual("cargo_lane_cleanup_reserved", rejected.exception.code)
            with self.database.transaction() as connection:
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES ('test.concurrent_write', '{}', datetime('now'))"
                )
            original_rmtree(path)

        with patch("tools.session_coordinator.cleanup.shutil.rmtree", delete_while_competing):
            applied = self.cleanup.apply(reviewed, now=cleanup_time)

        self.assertEqual((job.target_dir,), applied.deleted)


if __name__ == "__main__":
    unittest.main()
