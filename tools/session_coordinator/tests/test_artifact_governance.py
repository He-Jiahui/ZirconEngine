from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.artifact_governance import ArtifactGovernanceService
from tools.session_coordinator.cargo_jobs import CargoJobService, CargoLaneKind, TargetPathPolicy
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class ArtifactGovernanceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.cargo_root = root / "D" / "cargo-targets"
        self.targets_root = root / "D" / "targets"
        self.builds_root = root / "E" / "ZirconBuilds"
        for item in (self.cargo_root, self.targets_root, self.builds_root):
            item.mkdir(parents=True)
        self.database = Database(root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.jobs = CargoJobService(
            self.database,
            TargetPathPolicy([self.cargo_root, self.targets_root, self.builds_root]),
            repo_root=self.repo,
        )
        self.governance = ArtifactGovernanceService(
            self.database,
            roots=(self.cargo_root, self.targets_root, self.builds_root),
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_scans_unregistered_leaf_beside_registered_cargo_target(self) -> None:
        job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.cargo_root / "verify" / "registered",
        )
        unmanaged = self.cargo_root / "verify" / "unregistered"
        unmanaged.mkdir(parents=True)

        candidates = self.governance.scan()

        self.assertEqual([str(unmanaged.resolve())], [item.path for item in candidates])
        self.assertNotIn(job.target_dir, [item.path for item in candidates])

    def test_cleanup_removes_unknown_build_directory_and_persists_audit_event(self) -> None:
        unmanaged = self.builds_root / "manual-renderdoc-output"
        unmanaged.mkdir()
        (unmanaged / "capture.rdc").write_text("stale", encoding="utf-8")

        result = self.governance.cleanup()

        self.assertEqual((str(unmanaged.resolve()),), result.deleted)
        self.assertFalse(unmanaged.exists())
        with self.database.connect() as connection:
            event = connection.execute(
                "SELECT event_type FROM events WHERE event_type='artifact.unmanaged_deleted'"
            ).fetchone()
        self.assertIsNotNone(event)

    def test_cleanup_processes_one_directory_and_records_start_before_delete(self) -> None:
        first = self.builds_root / "first"
        second = self.builds_root / "second"
        first.mkdir()
        second.mkdir()

        result = self.governance.cleanup()

        self.assertEqual(1, len(result.deleted))
        self.assertEqual(1, sum(item.exists() for item in (first, second)))
        with self.database.connect() as connection:
            event = connection.execute(
                "SELECT event_type FROM events WHERE event_type='artifact.unmanaged_delete_started'"
            ).fetchone()
        self.assertIsNotNone(event)

    def test_require_clean_rejects_unregistered_directory(self) -> None:
        unmanaged = self.targets_root / "manual-target"
        unmanaged.mkdir()

        with self.assertRaises(CoordinatorError) as rejected:
            self.governance.require_clean()

        self.assertEqual("unmanaged_artifacts_detected", rejected.exception.code)

    def test_does_not_preserve_directory_only_because_a_historical_job_was_deleted(self) -> None:
        self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.cargo_root / "pool" / "active",
        )
        retired = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.cargo_root / "pool" / "retired",
        )
        retired_path = Path(retired.target_dir)
        retired_path.mkdir(parents=True, exist_ok=True)
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE cargo_jobs
                   SET status='released', cleanup_status='deleted'
                   WHERE job_id=?""",
                (retired.job_id,),
            )

        candidates = self.governance.scan()

        self.assertEqual([str(retired_path.resolve())], [item.path for item in candidates])

    def test_cleanup_reservation_protects_only_its_ephemeral_descendant(self) -> None:
        reserved = self.cargo_root / "zircon-engine/ephemeral/test/job-a"
        manual_sibling = self.cargo_root / "zircon-engine/ephemeral/test/manual"
        reserved.mkdir(parents=True)
        manual_sibling.mkdir(parents=True)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at)
                   VALUES (?, ?, '2026-07-15T00:00:00+00:00')""",
                (
                    str(reserved.resolve()).replace("/", "\\").casefold(),
                    str(reserved.resolve()),
                ),
            )

        candidates = self.governance.scan()

        self.assertEqual([str(manual_sibling.resolve())], [item.path for item in candidates])

    def test_rejection_includes_the_managed_cargo_snapshot(self) -> None:
        job = self.jobs.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=self.cargo_root / "verify" / "registered",
        )
        (self.targets_root / "manual-target").mkdir()

        with self.assertRaises(CoordinatorError) as rejected:
            self.governance.require_clean()

        self.assertEqual("unmanaged_artifacts_detected", rejected.exception.code)
        self.assertEqual(
            [{"jobId": job.job_id, "status": "leased", "targetDir": job.target_dir}],
            rejected.exception.details["managedCargo"],
        )

    def test_second_ephemeral_acquire_can_cross_a_sibling_cleanup_window(self) -> None:
        reserved = self.cargo_root / "zircon-engine/ephemeral/test/job-a"
        requested = self.cargo_root / "zircon-engine/ephemeral/test/job-b"
        reserved.mkdir(parents=True)
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(target_key, target_dir, reserved_at)
                   VALUES (?, ?, '2026-07-15T00:00:00+00:00')""",
                (
                    str(reserved.resolve()).replace("/", "\\").casefold(),
                    str(reserved.resolve()),
                ),
            )

        self.governance.require_clean()
        job = self.jobs.acquire(
            "session-a", CargoLaneKind.TEST, requested_target=requested
        )

        self.assertEqual(str(requested.resolve()), job.target_dir)


if __name__ == "__main__":
    unittest.main()
