from __future__ import annotations

import json
import tempfile
import threading
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.cleanup import CleanupService
from tools.session_coordinator.cleanup_deletion import begin_target_deletion
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
        self.target_root = root / "drive/cargo-targets"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.alive_pids: set[int] = {4242}
        self.jobs = CargoJobService(
            self.database,
            TargetPathPolicy([self.target_root]),
            repo_root=self.repo,
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

    def acquire_reusable(
        self,
        lane_kind: CargoLaneKind,
        *,
        build_config: str = "profile=test;features=default",
        requested_target: Path | None = None,
    ):
        return self.jobs.acquire(
            "session-a",
            lane_kind,
            requested_target=requested_target,
            compatibility=CargoCompatibility(
                platform="windows",
                toolchain="stable-x86_64-pc-windows-msvc",
                target_architecture="x86_64-pc-windows-msvc",
                workspace="Cargo.toml",
                build_config=build_config,
            ),
        )

    def record_validation_copy(
        self,
        job_root: Path,
        *,
        status: str = "materialized",
        job_id: str = "validation-copy",
    ) -> str:
        job_root.mkdir(parents=True, exist_ok=True)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       external_sources_json
                   ) VALUES (?, 'session-a', ?, ?, ?, 'head', '[]', ?, ?, '[]')""",
                (
                    job_id,
                    str(job_root),
                    str(job_root / "source"),
                    str(job_root / "workspace"),
                    status,
                    utc_text(),
                ),
            )
        return job_id

    def cleanup_events(self, event_type: str) -> list[dict[str, object]]:
        with self.database.connect() as connection:
            rows = connection.execute(
                "SELECT payload_json FROM events WHERE event_type=? ORDER BY event_id",
                (event_type,),
            ).fetchall()
        return [json.loads(row["payload_json"]) for row in rows]

    def assert_deletion_evidence(
        self,
        event_type: str,
        *,
        trigger: str,
        target_dir: str,
        owner_job_id: str,
        result: str = "deleted",
    ) -> None:
        payload = self.cleanup_events(event_type)[-1]
        self.assertEqual(trigger, payload["trigger"])
        self.assertEqual(target_dir, payload["target_dir"])
        self.assertEqual(owner_job_id, payload["owner_job_id"])
        self.assertEqual(result, payload["result"])
        self.assertEqual(target_dir.replace("/", "\\").casefold(), payload["target_key"])
        self.assertRegex(str(payload["deletion_id"]), r"^[0-9a-f]{32}$")
        self.assertEqual(owner_job_id, payload["before"]["owner_job_id"])
        self.assertIsInstance(payload["before"]["target_exists"], bool)
        self.assertIn("job_status", payload["before"])
        self.assertIn("process_alive", payload["before"])
        self.assertIsInstance(payload["executor"]["process_id"], int)
        self.assertTrue(payload["executor"]["thread_name"])

    def test_active_pid_and_lease_are_never_cleanup_candidates(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.TEST)
        (Path(job.target_dir) / "artifact").write_text("live", encoding="utf-8")
        self.jobs.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )

        plan = self.cleanup.plan(now=datetime.now(UTC) + timedelta(days=2), older_than_hours=1)

        self.assertEqual([], list(plan.candidates))
        self.assertTrue(any(item.code == "active_process" for item in plan.denied))
        self.assertTrue(Path(job.target_dir).exists())

    def test_released_stale_lane_is_deleted_only_after_plan(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        (Path(job.target_dir) / "artifact").write_text("stale", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        plan = self.cleanup.plan(now=cleanup_time, older_than_hours=1)

        applied = self.cleanup.apply(plan, now=cleanup_time)

        self.assertEqual((job.target_dir,), plan.candidates)
        self.assertEqual((job.target_dir,), applied.deleted)
        self.assertFalse(Path(job.target_dir).exists())

    def test_ephemeral_lane_is_deleted_immediately_after_release(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        (Path(job.target_dir) / "artifact").write_text("temporary", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")

        result = self.cleanup.cleanup_job_now(job.job_id)

        self.assertEqual((job.target_dir,), result.deleted)
        self.assertFalse(Path(job.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)
        self.assert_deletion_evidence(
            "cleanup.target_deletion_completed",
            trigger="prompt_cleanup",
            target_dir=job.target_dir,
            owner_job_id=job.job_id,
        )

    def test_immediate_cleanup_refuses_parent_of_materialized_validation_copy(self) -> None:
        job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "copy-parent",
        )
        marker = Path(job.target_dir) / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        copy_root = Path(job.target_dir) / "validation-copy"
        copy_id = self.record_validation_copy(copy_root)

        result = self.cleanup.cleanup_job_now(job.job_id)

        self.assertEqual((), result.deleted)
        self.assertEqual(
            ("validation_copy_overlap",), tuple(item.code for item in result.denied)
        )
        self.assertIn(copy_id, result.denied[0].message)
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        with self.database.connect() as connection:
            self.assertEqual(
                0,
                connection.execute(
                    "SELECT COUNT(*) FROM cleanup_reservations WHERE reservation_kind='cargo'"
                ).fetchone()[0],
            )
            self.assertEqual(
                "materialized",
                connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id=?", (copy_id,)
                ).fetchone()[0],
            )
        denial = self.cleanup_events("cleanup.validation_copy_overlap_denied")[-1]
        self.assertEqual("validation_copy_overlap", denial["code"])
        self.assertEqual("prompt_cleanup", denial["trigger"])
        self.assertEqual(job.target_dir, denial["target_dir"])
        self.assertEqual(copy_id, denial["validation_copy"]["job_id"])
        self.assertEqual(str(copy_root), denial["validation_copy"]["path"])

    def test_immediate_cleanup_refuses_child_of_materialized_validation_copy(self) -> None:
        copy_parent = self.target_root / "copy-owner"
        copy_parent.mkdir()
        job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=copy_parent / "cargo-child",
        )
        marker = Path(job.target_dir) / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        self.record_validation_copy(copy_parent)

        result = self.cleanup.cleanup_job_now(job.job_id)

        self.assertEqual((), result.deleted)
        self.assertEqual(
            ("validation_copy_overlap",), tuple(item.code for item in result.denied)
        )
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))

    def test_removed_and_nonoverlapping_validation_copies_do_not_block_cleanup(self) -> None:
        removed_job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "removed-copy-target",
        )
        self.jobs.release(removed_job.job_id, session_id="session-a")
        self.record_validation_copy(
            Path(removed_job.target_dir),
            status="removed",
            job_id="removed-validation-copy",
        )

        removed_result = self.cleanup.cleanup_job_now(removed_job.job_id)

        self.assertEqual((removed_job.target_dir,), removed_result.deleted)

        unrelated_job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.TEST,
            requested_target=self.target_root / "unrelated-cargo-target",
        )
        self.jobs.release(unrelated_job.job_id, session_id="session-a")
        self.record_validation_copy(
            self.target_root / "independent-validation-copy",
            job_id="independent-validation-copy",
        )

        unrelated_result = self.cleanup.cleanup_job_now(unrelated_job.job_id)

        self.assertEqual((unrelated_job.target_dir,), unrelated_result.deleted)

    def test_superseded_ephemeral_record_never_deletes_a_retained_pool(self) -> None:
        ephemeral = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "adopted-pool",
        )
        marker = Path(ephemeral.target_dir) / "artifact"
        marker.write_text("reusable", encoding="utf-8")
        self.jobs.release(ephemeral.job_id, session_id="session-a")
        reusable = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=Path(ephemeral.target_dir),
        )
        self.jobs.release(reusable.job_id, session_id="session-a")

        result = self.cleanup.cleanup_job_now(ephemeral.job_id)

        self.assertEqual((), result.deleted)
        self.assertEqual((), result.denied)
        self.assertTrue(marker.is_file())
        self.assertEqual("deleted", self.jobs.get(ephemeral.job_id).cleanup_status.value)
        self.assertEqual("retained", self.jobs.get(reusable.job_id).cleanup_status.value)

    def test_superseded_ephemeral_parent_never_deletes_retained_child_pool(self) -> None:
        ephemeral = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "adopted-parent",
        )
        self.jobs.release(ephemeral.job_id, session_id="session-a")
        reusable = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=Path(ephemeral.target_dir) / "retained-child",
        )
        marker = Path(reusable.target_dir) / "artifact"
        marker.write_text("reusable", encoding="utf-8")
        self.jobs.release(reusable.job_id, session_id="session-a")

        result = self.cleanup.cleanup_job_now(ephemeral.job_id)

        self.assertEqual((), result.deleted)
        self.assertEqual((), result.denied)
        self.assertTrue(marker.is_file())
        self.assertEqual("deleted", self.jobs.get(ephemeral.job_id).cleanup_status.value)
        self.assertEqual("retained", self.jobs.get(reusable.job_id).cleanup_status.value)

    def test_superseded_ephemeral_child_never_deletes_retained_parent_pool(self) -> None:
        reusable = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "retained-parent",
        )
        self.jobs.release(reusable.job_id, session_id="session-a")
        ephemeral = self.jobs.acquire(
            "session-a",
            CargoLaneKind.TEST,
            requested_target=Path(reusable.target_dir) / "ephemeral-child",
        )
        marker = Path(ephemeral.target_dir) / "artifact"
        marker.write_text("temporary", encoding="utf-8")
        self.jobs.release(ephemeral.job_id, session_id="session-a")

        result = self.cleanup.cleanup_job_now(ephemeral.job_id)

        self.assertEqual((), result.deleted)
        self.assertEqual((), result.denied)
        self.assertTrue(marker.is_file())
        self.assertEqual("deleted", self.jobs.get(ephemeral.job_id).cleanup_status.value)
        self.assertEqual("retained", self.jobs.get(reusable.job_id).cleanup_status.value)

    def test_failed_ephemeral_cleanup_is_retryable(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        original = __import__("shutil").rmtree
        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=OSError("locked"),
        ):
            failed = self.cleanup.cleanup_job_now(job.job_id)
        self.assertEqual("failed", self.jobs.get(job.job_id).cleanup_status.value)
        self.assertTrue(failed.denied)
        self.assert_deletion_evidence(
            "cleanup.target_deletion_completed",
            trigger="prompt_cleanup",
            target_dir=job.target_dir,
            owner_job_id=job.job_id,
            result="failed",
        )

        with patch("tools.session_coordinator.cleanup.shutil.rmtree", side_effect=original):
            retried = self.cleanup.retry_pending_jobs()

        self.assertEqual((job.job_id,), retried)
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)

    def test_prompt_cleanup_keeps_reservation_on_unexpected_delete_error(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")

        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=RuntimeError("unexpected prompt failure"),
        ):
            with self.assertRaisesRegex(RuntimeError, "unexpected prompt failure"):
                self.cleanup.cleanup_job_now(job.job_id)

        self.assert_unknown_delete_failure(job.target_dir, job.job_id)

    def test_missing_ephemeral_target_is_observed_without_claiming_a_deletion(self) -> None:
        job = self.jobs.acquire("session-a", CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        __import__("shutil").rmtree(job.target_dir)

        result = self.cleanup.cleanup_job_now(job.job_id)

        self.assertEqual((), result.deleted)
        self.assert_deletion_evidence(
            "cleanup.target_deletion_completed",
            trigger="prompt_cleanup",
            target_dir=job.target_dir,
            owner_job_id=job.job_id,
            result="already_missing",
        )
        payload = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertFalse(payload["before"]["target_exists"])

    def test_async_cleanup_drains_release_requested_during_running_pass(self) -> None:
        first_pass_entered = threading.Event()
        allow_first_pass_to_finish = threading.Event()
        second_pass_finished = threading.Event()
        retry_calls = 0

        def retry_pending_jobs(*, include_failed: bool = True) -> tuple[str, ...]:
            nonlocal retry_calls
            self.assertFalse(include_failed)
            retry_calls += 1
            if retry_calls == 1:
                first_pass_entered.set()
                self.assertTrue(allow_first_pass_to_finish.wait(timeout=2))
            elif retry_calls == 2:
                second_pass_finished.set()
            return ()

        with (
            patch.object(self.cleanup, "retry_pending_jobs", side_effect=retry_pending_jobs),
            patch.object(self.cleanup, "evict_idle_pools_under_pressure"),
        ):
            self.assertTrue(self.cleanup.schedule_pending_cleanup())
            self.assertTrue(first_pass_entered.wait(timeout=2))
            self.assertFalse(self.cleanup.schedule_pending_cleanup())
            allow_first_pass_to_finish.set()
            self.assertTrue(second_pass_finished.wait(timeout=2))

        self.assertGreaterEqual(retry_calls, 2)

    def test_apply_revalidates_process_liveness(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
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
        job = self.acquire_reusable(CargoLaneKind.CHECK)
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

    def test_pressure_eviction_removes_oldest_idle_pool_until_reserve_is_restored(self) -> None:
        oldest = self.acquire_reusable(
            CargoLaneKind.CHECK,
            build_config="profile=dev;features=oldest",
        )
        newest = self.acquire_reusable(
            CargoLaneKind.CHECK,
            build_config="profile=dev;features=newest",
        )
        self.jobs.release(oldest.job_id, session_id="session-a")
        self.jobs.release(newest.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET released_at=? WHERE job_id=?",
                ((datetime.now(UTC) - timedelta(days=2)).isoformat(), oldest.job_id),
            )
        free_values = iter((40 * 1024**3, 60 * 1024**3))
        self.cleanup.free_space = lambda _path: next(free_values)

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertEqual((oldest.target_dir,), result.deleted)
        self.assertFalse(Path(oldest.target_dir).exists())
        self.assertTrue(Path(newest.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(oldest.job_id).cleanup_status.value)
        self.assert_deletion_evidence(
            "cleanup.target_deletion_completed",
            trigger="pressure_eviction",
            target_dir=oldest.target_dir,
            owner_job_id=oldest.job_id,
        )

    def test_pressure_eviction_never_deletes_an_active_pool(self) -> None:
        active = self.acquire_reusable(CargoLaneKind.TEST)
        self.cleanup.free_space = lambda _path: 40 * 1024**3

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertEqual((), result.deleted)
        self.assertTrue(Path(active.target_dir).exists())
        self.assertTrue(any(item.code == "active_lease" for item in result.denied))

    def test_pressure_cleanup_keeps_reservation_on_unexpected_delete_error(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        self.cleanup.free_space = lambda _path: 40 * 1024**3

        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=RuntimeError("unexpected pressure failure"),
        ):
            with self.assertRaisesRegex(RuntimeError, "unexpected pressure failure"):
                self.cleanup.evict_idle_pools_under_pressure()

        self.assert_unknown_delete_failure(job.target_dir, job.job_id)

    def test_failed_reusable_cleanup_blocks_reuse_until_retry_succeeds(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        self.cleanup.free_space = lambda _path: 40 * 1024**3
        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=OSError("locked reusable pool"),
        ):
            failed = self.cleanup.evict_idle_pools_under_pressure()
        self.assertTrue(failed.denied)
        self.assertEqual("failed", self.jobs.get(job.job_id).cleanup_status.value)

        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.acquire(
                "session-a",
                CargoLaneKind.TEST,
                requested_target=Path(job.target_dir) / "child",
            )
        self.assertEqual("cargo_lane_cleanup_failed", rejected.exception.code)

        retried = self.cleanup.retry_pending_jobs()

        self.assertEqual((job.job_id,), retried)
        self.assertFalse(Path(job.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)
        replacement = self.jobs.acquire(
            "session-a",
            CargoLaneKind.TEST,
            requested_target=Path(job.target_dir) / "child",
        )
        self.assertEqual("leased", replacement.status.value)
        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual("retry_failed_cleanup", completed["trigger"])
        self.assertEqual("deleted", completed["result"])

    def test_failed_cleanup_retry_accepts_succeeded_unreleased_pool(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET status='succeeded', released_at=NULL WHERE job_id=?",
                (job.job_id,),
            )
        self.cleanup.free_space = lambda _path: 40 * 1024**3
        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=OSError("locked succeeded pool"),
        ):
            failed = self.cleanup.evict_idle_pools_under_pressure()
        self.assertTrue(failed.denied)
        self.assertEqual("failed", self.jobs.get(job.job_id).cleanup_status.value)

        retried = self.cleanup.retry_pending_jobs()

        self.assertEqual((job.job_id,), retried)
        self.assertFalse(Path(job.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)

    def test_successful_retry_settles_mixed_exact_target_history_once(self) -> None:
        pending = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "mixed-history",
        )
        self.jobs.release(pending.job_id, session_id="session-a")
        failed = self.acquire_reusable(
            CargoLaneKind.TEST,
            requested_target=Path(pending.target_dir),
        )
        self.jobs.release(failed.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE cargo_jobs SET cleanup_status='failed', cleanup_error='unknown'
                WHERE job_id=?
                """,
                (failed.job_id,),
            )

        retried = self.cleanup.retry_pending_jobs()

        self.assertEqual((pending.job_id,), retried)
        self.assertFalse(Path(pending.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(pending.job_id).cleanup_status.value)
        self.assertEqual("deleted", self.jobs.get(failed.job_id).cleanup_status.value)
        replacement = self.jobs.acquire(
            "session-a",
            CargoLaneKind.TEST,
            requested_target=Path(pending.target_dir) / "child",
        )
        self.assertEqual("leased", replacement.status.value)

    def test_pressure_eviction_revalidates_acquire_to_start_window(self) -> None:
        idle = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "acquire-window",
        )
        self.jobs.release(idle.job_id, session_id="session-a")
        free_space_calls = 0
        acquired = None

        def free_space(_root: Path) -> int:
            nonlocal free_space_calls, acquired
            free_space_calls += 1
            if free_space_calls == 1:
                acquired = self.acquire_reusable(
                    CargoLaneKind.TEST,
                    build_config="profile=test;features=concurrent-acquire",
                    requested_target=Path(idle.target_dir) / "child",
                )
            return 40 * 1024**3

        self.cleanup.free_space = free_space

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertIsNotNone(acquired)
        self.assertEqual((), result.deleted)
        self.assertTrue(Path(idle.target_dir).exists())
        self.assertTrue(any(item.code == "active_lease" for item in result.denied))

    def test_pressure_eviction_never_deletes_running_overlapping_pool(self) -> None:
        idle = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "running-window",
        )
        self.jobs.release(idle.job_id, session_id="session-a")
        running = self.acquire_reusable(
            CargoLaneKind.TEST,
            build_config="profile=test;features=concurrent-running",
            requested_target=Path(idle.target_dir) / "child",
        )
        self.jobs.start(
            running.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test"],
        )
        self.cleanup.free_space = lambda _path: 40 * 1024**3

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertEqual((), result.deleted)
        self.assertTrue(Path(idle.target_dir).exists())
        self.assertTrue(any(item.code == "active_process" for item in result.denied))

    def test_pressure_eviction_never_deletes_pool_with_live_recorded_process(self) -> None:
        released = self.acquire_reusable(CargoLaneKind.TEST)
        self.jobs.release(released.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET pid=? WHERE job_id=?",
                (4242, released.job_id),
            )
        self.cleanup.free_space = lambda _path: 40 * 1024**3

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertEqual((), result.deleted)
        self.assertTrue(Path(released.target_dir).exists())
        self.assertTrue(any(item.code == "active_process" for item in result.denied))

    def test_pressure_eviction_refuses_materialized_validation_copy_overlap(self) -> None:
        released = self.acquire_reusable(
            CargoLaneKind.TEST,
            requested_target=self.target_root / "pressure-copy-parent",
        )
        marker = Path(released.target_dir) / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        self.jobs.release(released.job_id, session_id="session-a")
        self.record_validation_copy(Path(released.target_dir))

        result = self.cleanup.evict_idle_pools_under_pressure()

        self.assertEqual((), result.deleted)
        self.assertTrue(
            any(item.code == "validation_copy_overlap" for item in result.denied)
        )
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        denial = self.cleanup_events("cleanup.validation_copy_overlap_denied")[-1]
        self.assertEqual("pressure_eviction", denial["trigger"])

    def test_orphaned_ephemeral_pool_is_retried_and_deleted(self) -> None:
        job = self.jobs.acquire(
            "session-a", CargoLaneKind.CHECK, owner_pid=9999
        )
        orphaned = self.jobs.reconcile_orphans(
            now=datetime.now(UTC) + timedelta(minutes=10),
            leased_timeout_seconds=1,
        )

        deleted = self.cleanup.retry_pending_jobs()

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual((job.job_id,), deleted)
        self.assertFalse(Path(job.target_dir).exists())

    def test_non_positive_retention_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            self.cleanup.plan(older_than_hours=0)

    def test_cleanup_plan_never_expands_to_newer_candidates(self) -> None:
        first = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(first.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        reviewed = self.cleanup.plan(now=cleanup_time, older_than_hours=1)

        second = self.acquire_reusable(
            CargoLaneKind.CHECK,
            build_config="profile=dev;features=default",
            requested_target=self.target_root / "newer-independent-lane",
        )
        self.jobs.release(second.job_id, session_id="session-a")
        applied = self.cleanup.apply(reviewed, now=cleanup_time)

        self.assertEqual((first.target_dir,), applied.deleted)
        self.assertTrue(Path(second.target_dir).exists())

    def test_cleanup_apply_rechecks_validation_copy_created_after_plan(self) -> None:
        job = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "planned-copy-parent",
        )
        marker = Path(job.target_dir) / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        plan = self.cleanup.plan(now=cleanup_time, older_than_hours=1)
        self.assertEqual((job.target_dir,), plan.candidates)
        self.record_validation_copy(Path(job.target_dir))

        applied = self.cleanup.apply(plan, now=cleanup_time)

        self.assertEqual((), applied.deleted)
        self.assertTrue(
            any(item.code == "validation_copy_overlap" for item in applied.denied)
        )
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        with self.database.connect() as connection:
            self.assertEqual(
                0,
                connection.execute(
                    "SELECT COUNT(*) FROM cleanup_reservations WHERE reservation_kind='cargo'"
                ).fetchone()[0],
            )
        denial = self.cleanup_events("cleanup.validation_copy_overlap_denied")[-1]
        self.assertEqual("explicit_plan", denial["trigger"])

    def test_cleanup_reservation_blocks_overlapping_reacquire_without_writer_lock(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, datetime('now'))",
                (job.target_dir.replace("/", "\\").casefold(), job.target_dir),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                requested_target=Path(job.target_dir) / "concurrent-child",
            )
        self.assertEqual("cargo_lane_cleanup_reserved", rejected.exception.code)

    def test_service_restart_recovers_abandoned_cleanup_reservations(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, datetime('now'))",
                ("abandoned", str(self.target_root / "abandoned")),
            )

        self.assertEqual(1, self.cleanup.recover_reservations())
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations"
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    def test_service_restart_preserves_artifact_cleanup_reservation(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at, reservation_kind,
                       filesystem_identity
                   ) VALUES (?, ?, datetime('now'), 'artifact', 'identity')""",
                ("artifact-target", str(self.target_root / "artifact-target")),
            )

        self.assertEqual(0, self.cleanup.recover_reservations())

        with self.database.connect() as connection:
            remaining = connection.execute(
                """SELECT COUNT(*) FROM cleanup_reservations
                   WHERE reservation_kind='artifact'"""
            ).fetchone()[0]
        self.assertEqual(1, remaining)

    def test_service_restart_finishes_interrupted_deletion_evidence(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        deletion_id = "a" * 32
        payload = {
            "deletion_id": deletion_id,
            "trigger": "pressure_eviction",
            "target_key": job.target_dir.replace("/", "\\").casefold(),
            "target_dir": job.target_dir,
            "owner_job_id": job.job_id,
            "owner_session_id": "session-a",
            "before": {
                "owner_job_id": job.job_id,
                "job_status": "released",
                "cleanup_status": "retained",
                "pid": None,
                "process_alive": False,
                "overlapping_jobs": [],
            },
            "executor": {"process_id": 10, "thread_name": "cleanup-worker"},
            "result": "reserved",
            "error": None,
        }
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, ?)",
                (payload["target_key"], job.target_dir, utc_text()),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "cleanup.target_deletion_started",
                    json.dumps(payload, sort_keys=True),
                    utc_text(),
                ),
            )
        self.assertEqual(1, self.cleanup.recover_reservations())

        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual(deletion_id, completed["deletion_id"])
        self.assertEqual("deleted_after_restart", completed["result"])
        self.assertTrue(completed["recovered"])
        self.assertFalse(Path(job.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)

    def test_service_restart_blocks_interrupted_deletion_owned_by_validation_copy(self) -> None:
        job = self.acquire_reusable(
            CargoLaneKind.CHECK,
            requested_target=self.target_root / "restart-copy-parent",
        )
        marker = Path(job.target_dir) / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        self.jobs.release(job.job_id, session_id="session-a")
        copy_id = self.record_validation_copy(Path(job.target_dir))
        key = job.target_dir.replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            rows = connection.execute(
                "SELECT * FROM cargo_jobs WHERE target_key=?", (key,)
            ).fetchall()
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, ?)",
                (key, job.target_dir, utc_text()),
            )
            begin_target_deletion(
                connection,
                trigger="pressure_eviction",
                target_key=key,
                target_dir=job.target_dir,
                owner=rows[-1],
                overlapping_jobs=rows,
                process_alive=lambda _pid: False,
            )

        self.assertEqual(1, self.cleanup.recover_reservations())

        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        self.assertEqual("failed", self.jobs.get(job.job_id).cleanup_status.value)
        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual("blocked_by_validation_copy_after_restart", completed["result"])
        self.assertIn(copy_id, completed["error"])
        denial = self.cleanup_events("cleanup.validation_copy_overlap_denied")[-1]
        self.assertEqual("restart_recovery", denial["trigger"])
        self.assertEqual(copy_id, denial["validation_copy"]["job_id"])
        with self.database.connect() as connection:
            self.assertEqual(
                0,
                connection.execute(
                    "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
                ).fetchone()[0],
            )
            self.assertEqual(
                "materialized",
                connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id=?", (copy_id,)
                ).fetchone()[0],
            )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status='removed', removed_at=? WHERE job_id=?",
                (utc_text(), copy_id),
            )

        retried = self.cleanup.cleanup_job_now(job.job_id)

        self.assertEqual((job.target_dir,), retried.deleted)
        self.assertFalse(Path(job.target_dir).exists())

    def test_service_restart_keeps_reservation_when_interrupted_deletion_fails(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        deletion_id = "b" * 32
        payload = {
            "deletion_id": deletion_id,
            "trigger": "pressure_eviction",
            "target_key": job.target_dir.replace("/", "\\").casefold(),
            "target_dir": job.target_dir,
            "owner_job_id": job.job_id,
            "owner_session_id": "session-a",
            "before": {
                "owner_job_id": job.job_id,
                "target_exists": True,
                "job_status": "released",
                "cleanup_status": "retained",
                "pid": None,
                "process_alive": False,
                "overlapping_jobs": [],
            },
            "executor": {"process_id": 10, "thread_name": "cleanup-worker"},
            "result": "reserved",
            "error": None,
        }
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at) VALUES (?, ?, ?)",
                (payload["target_key"], job.target_dir, utc_text()),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "cleanup.target_deletion_started",
                    json.dumps(payload, sort_keys=True),
                    utc_text(),
                ),
            )

        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=OSError("locked during recovery"),
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.cleanup.recover_reservations()

        self.assertEqual("cleanup_recovery_failed", rejected.exception.code)
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?",
                (payload["target_key"],),
            ).fetchone()[0]
        self.assertEqual(1, remaining)
        self.assertEqual("failed", self.jobs.get(job.job_id).cleanup_status.value)
        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual("failed_after_restart", completed["result"])

        self.assertEqual(1, self.cleanup.recover_reservations())
        self.assertFalse(Path(job.target_dir).exists())
        self.assertEqual("deleted", self.jobs.get(job.job_id).cleanup_status.value)
        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual(deletion_id, completed["deletion_id"])
        self.assertEqual("deleted_after_restart", completed["result"])

    def test_persisted_plan_still_refuses_untracked_managed_directory(self) -> None:
        untracked = self.target_root / "untracked"
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
        parent = self.acquire_reusable(CargoLaneKind.CHECK)
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

    def test_cleanup_deletes_outside_writer_transaction_and_blocks_child_reacquire(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        reviewed = self.cleanup.plan(now=cleanup_time, older_than_hours=1)
        original_rmtree = __import__("shutil").rmtree

        def delete_while_competing(path: Path) -> None:
            with self.assertRaises(CoordinatorError) as rejected:
                self.jobs.acquire(
                    "session-a",
                    CargoLaneKind.CHECK,
                    requested_target=path / "concurrent-child",
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
        self.assert_deletion_evidence(
            "cleanup.target_deletion_completed",
            trigger="explicit_plan",
            target_dir=job.target_dir,
            owner_job_id=job.job_id,
        )

    def test_explicit_cleanup_keeps_reservation_on_unexpected_delete_error(self) -> None:
        job = self.acquire_reusable(CargoLaneKind.CHECK)
        self.jobs.release(job.job_id, session_id="session-a")
        cleanup_time = datetime.now(UTC) + timedelta(days=2)
        plan = self.cleanup.plan(now=cleanup_time, older_than_hours=1)

        with patch(
            "tools.session_coordinator.cleanup.shutil.rmtree",
            side_effect=RuntimeError("unexpected explicit failure"),
        ):
            with self.assertRaisesRegex(RuntimeError, "unexpected explicit failure"):
                self.cleanup.apply(plan, now=cleanup_time)

        self.assert_unknown_delete_failure(job.target_dir, job.job_id)
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM cleanup_plans WHERE plan_id=?",
                (plan.plan_id,),
            ).fetchone()[0]
        self.assertEqual("applying", status)

    def assert_unknown_delete_failure(self, target_dir: str, job_id: str) -> None:
        self.assertTrue(Path(target_dir).exists())
        self.assertEqual("failed", self.jobs.get(job_id).cleanup_status.value)
        completed = self.cleanup_events("cleanup.target_deletion_completed")[-1]
        self.assertEqual("failed", completed["result"])
        self.assertIn("unexpected", completed["error"])
        with self.database.connect() as connection:
            reservations = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_dir=?",
                (target_dir,),
            ).fetchone()[0]
        self.assertEqual(1, reservations)


if __name__ == "__main__":
    unittest.main()
