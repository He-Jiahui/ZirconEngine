from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path

from tools.session_coordinator.cargo_jobs import (
    CargoJobService,
    CargoJobStatus,
    CargoLaneKind,
    TargetPathPolicy,
    target_identity,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CargoJobTests(unittest.TestCase):
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
        SessionService(self.database, self.repo).register(session_id="session-b")
        self.policy = TargetPathPolicy([self.target_root])
        self.service = CargoJobService(
            self.database,
            self.policy,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_allocated_lane_is_unique_and_under_allowlisted_lanes_root(self) -> None:
        first = self.service.acquire("session-a", CargoLaneKind.CHECK)
        second = self.service.acquire("session-a", CargoLaneKind.CHECK)

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertTrue(Path(first.target_dir).is_relative_to(self.target_root / "lanes"))
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
        requested = self.target_root / "lanes/shared-check"
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        with self.assertRaises(CoordinatorError) as occupied:
            self.service.acquire(
                "session-a", CargoLaneKind.TEST, requested_target=requested
            )

        self.assertEqual(CargoJobStatus.LEASED, first.status)
        self.assertEqual("cargo_lane_occupied", occupied.exception.code)

    def test_target_identity_is_case_and_separator_insensitive(self) -> None:
        self.assertEqual(
            target_identity(r"E:\targets\zircon-engine\lanes\Check-A"),
            target_identity("e:/TARGETS/zircon-engine/lanes/check-a"),
        )

    def test_nested_lane_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                requested_target=self.target_root / "lanes/parent/child",
            )
        self.assertEqual("cargo_target_not_managed", rejected.exception.code)

    def test_no_configured_target_drive_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([])
        self.assertEqual("target_root_unavailable", rejected.exception.code)

    def test_symlink_lane_escape_is_rejected_when_supported(self) -> None:
        outside = self.target_root.parent / "outside"
        outside.mkdir()
        link = self.target_root / "lanes/escaped-link"
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

    def test_released_explicit_lane_can_be_reused(self) -> None:
        requested = self.target_root / "lanes/manual-check"
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertTrue(requested.is_dir())

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
        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        released = self.service.release(job.job_id, session_id="session-a")

        self.assertEqual(CargoJobStatus.RUNNING, running.status)
        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual(CargoJobStatus.RELEASED, released.status)
        self.assertEqual(0, released.exit_code)
        self.assertEqual(("cargo", "test"), released.command)

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


if __name__ == "__main__":
    unittest.main()
