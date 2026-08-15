from __future__ import annotations

import os
import shutil
import stat
import subprocess
import tempfile
import threading
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.artifact_governance import ArtifactGovernanceService
from tools.session_coordinator.cargo_jobs import CargoJobService, CargoLaneKind, TargetPathPolicy
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.processes import process_creation_time
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

    def test_live_fixture_lease_is_managed_until_its_owner_removes_it(self) -> None:
        lease = self.governance.acquire_fixture("paths-contract", owner_pid=os.getpid())
        fixture = Path(lease.path)
        fixture.mkdir(parents=True)
        marker = fixture / "keep.txt"
        marker.write_text("active", encoding="utf-8")

        self.assertEqual((), self.governance.scan())
        result = self.governance.cleanup(max_candidates=10)

        self.assertEqual((), result.deleted)
        self.assertTrue(marker.is_file())

    def test_live_fixture_lease_survives_before_the_directory_is_created(self) -> None:
        lease = self.governance.acquire_fixture("creation-window", owner_pid=os.getpid())

        result = self.governance.cleanup(max_candidates=10)

        self.assertEqual((), result.deleted)
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM artifact_fixture_leases WHERE lease_id=?",
                (lease.lease_id,),
            ).fetchone()["status"]
        self.assertEqual("active", status)

    def test_fixture_release_requires_removal_and_does_not_exempt_recreation(self) -> None:
        lease = self.governance.acquire_fixture("release-contract", owner_pid=os.getpid())
        fixture = Path(lease.path)
        fixture.mkdir(parents=True)

        with self.assertRaises(CoordinatorError) as rejected:
            self.governance.release_fixture(lease.lease_id, owner_pid=os.getpid())

        self.assertEqual("artifact_fixture_still_exists", rejected.exception.code)
        shutil.rmtree(fixture)
        parent = fixture.parent
        if parent.is_dir() and not any(parent.iterdir()):
            parent.rmdir()
        released = self.governance.release_fixture(
            lease.lease_id, owner_pid=os.getpid()
        )
        self.assertEqual("released", released.status)

        fixture.mkdir(parents=True)
        candidates = self.governance.scan()

        self.assertTrue(
            any(fixture.is_relative_to(Path(item.path)) for item in candidates),
            candidates,
        )

    def test_fixture_release_rejects_a_foreign_owner(self) -> None:
        lease = self.governance.acquire_fixture("owner-contract", owner_pid=os.getpid())

        with self.assertRaises(CoordinatorError) as rejected:
            self.governance.release_fixture(lease.lease_id, owner_pid=os.getpid() + 1)

        self.assertEqual("artifact_fixture_owner_mismatch", rejected.exception.code)

    def test_cleanup_recovers_missing_parent_reservation_before_fixture_acquire(self) -> None:
        missing_parent = self.builds_root / f"mvp-test-fixtures-{os.getpid()}"
        key = str(missing_parent.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-15T00:00:00+00:00', 'artifact', 'old')""",
                (key, str(missing_parent.resolve())),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.governance.acquire_fixture("stale-reservation", owner_pid=os.getpid())
        self.assertEqual("artifact_fixture_cleanup_reserved", rejected.exception.code)

        result = self.governance.cleanup()
        lease = self.governance.acquire_fixture(
            "stale-reservation", owner_pid=os.getpid()
        )

        self.assertEqual((str(missing_parent.resolve()),), result.deleted)
        self.assertTrue(Path(lease.path).is_relative_to(missing_parent))
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    def test_dead_fixture_identity_is_unmanaged_and_recovered_after_cleanup(self) -> None:
        lease_id = "1" * 32
        fixture = (
            self.builds_root
            / f"mvp-test-fixtures-{os.getpid()}"
            / f"dead-contract-{lease_id}"
        )
        fixture.mkdir(parents=True)
        key = str(fixture.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO artifact_fixture_leases(
                       lease_id, target_key, target_dir, prefix, owner_pid,
                       owner_process_creation_time, status, created_at
                   ) VALUES (?, ?, ?, 'dead-contract', ?, ?, 'active', ?)""",
                (
                    lease_id,
                    key,
                    str(fixture.resolve()),
                    os.getpid(),
                    process_creation_time(os.getpid()) + "-reused",
                    "2026-08-16T00:00:00+00:00",
                ),
            )

        result = self.governance.cleanup(max_candidates=10)

        self.assertTrue(any(Path(path) == fixture.parent for path in result.deleted))
        self.assertFalse(fixture.exists())
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM artifact_fixture_leases WHERE lease_id=?",
                (lease_id,),
            ).fetchone()["status"]
        self.assertEqual("recovered", status)

    def test_dead_missing_fixture_lease_is_recovered_without_prefix_exemption(self) -> None:
        lease_id = "2" * 32
        fixture = (
            self.builds_root
            / f"mvp-test-fixtures-{os.getpid()}"
            / f"missing-contract-{lease_id}"
        )
        key = str(fixture.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO artifact_fixture_leases(
                       lease_id, target_key, target_dir, prefix, owner_pid,
                       owner_process_creation_time, status, created_at
                   ) VALUES (?, ?, ?, 'missing-contract', ?, ?, 'active', ?)""",
                (
                    lease_id,
                    key,
                    str(fixture.resolve()),
                    os.getpid(),
                    process_creation_time(os.getpid()) + "-reused",
                    "2026-08-16T00:00:00+00:00",
                ),
            )

        result = self.governance.cleanup(max_candidates=10)

        self.assertEqual((), result.deleted)
        self.assertEqual((), result.failed)
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT status, released_at FROM artifact_fixture_leases
                   WHERE lease_id=?""",
                (lease_id,),
            ).fetchone()
        self.assertEqual("recovered", row["status"])
        self.assertIsNotNone(row["released_at"])

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

    @unittest.skipUnless(os.name == "nt", "Windows readonly deletion semantics")
    def test_cleanup_removes_unknown_git_copy_with_readonly_object(self) -> None:
        unmanaged = self.builds_root / "unregistered-review-copy"
        git_object = unmanaged / ".git" / "objects" / "aa" / "readonly"
        git_object.parent.mkdir(parents=True)
        git_object.write_bytes(b"object")
        git_object.chmod(stat.S_IREAD)

        result = self.governance.cleanup()

        self.assertEqual((str(unmanaged.resolve()),), result.deleted)
        self.assertEqual((), result.failed)
        self.assertFalse(unmanaged.exists())
        with self.database.connect() as connection:
            events = [
                row["event_type"]
                for row in connection.execute(
                    """SELECT event_type FROM events
                       WHERE event_type LIKE 'artifact.unmanaged_delete%'
                       ORDER BY event_id"""
                )
            ]
        self.assertEqual(
            ["artifact.unmanaged_delete_started", "artifact.unmanaged_deleted"],
            events,
        )

    @unittest.skipUnless(os.name == "nt", "Windows junction semantics")
    def test_scan_and_cleanup_ignore_junction_to_outside_root(self) -> None:
        outside = self.builds_root.parent / "junction-outside"
        outside.mkdir()
        marker = outside / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        junction = self.builds_root / "unregistered-junction"
        subprocess.run(
            ["cmd.exe", "/c", "mklink", "/J", str(junction), str(outside)],
            check=True,
            capture_output=True,
            text=True,
        )

        try:
            self.assertTrue(junction.is_junction())
            self.assertEqual((), self.governance.scan())
            result = self.governance.cleanup()
        finally:
            if junction.exists():
                os.rmdir(junction)

        self.assertEqual((), result.deleted)
        self.assertEqual((), result.failed)
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))

    @unittest.skipUnless(os.name == "nt", "Windows junction semantics")
    def test_configured_root_junction_is_rejected_before_resolution(self) -> None:
        outside = self.builds_root.parent / "configured-root-outside"
        outside.mkdir()
        marker = outside / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        configured_root = self.builds_root.parent / "configured-root-junction"
        subprocess.run(
            ["cmd.exe", "/c", "mklink", "/J", str(configured_root), str(outside)],
            check=True,
            capture_output=True,
            text=True,
        )

        try:
            with self.assertRaises(CoordinatorError) as rejected:
                ArtifactGovernanceService(self.database, roots=(configured_root,))
        finally:
            if configured_root.is_junction():
                os.rmdir(configured_root)

        self.assertEqual("artifact_governance_root_reparse", rejected.exception.code)
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))

    def test_cleanup_reservation_blocks_exact_cargo_acquire_during_delete(self) -> None:
        unmanaged = self.builds_root / "unregistered-review-copy"
        unmanaged.mkdir()
        delete_entered = threading.Event()
        release_delete = threading.Event()
        cleanup_result = []

        def delayed_delete(path: Path, *, expected_identity: str) -> None:
            self.assertTrue(expected_identity)
            delete_entered.set()
            self.assertTrue(release_delete.wait(5))
            __import__("shutil").rmtree(path)

        worker = threading.Thread(
            target=lambda: cleanup_result.append(self.governance.cleanup())
        )
        with mock.patch(
            "tools.session_coordinator.artifact_governance._remove_candidate_tree",
            side_effect=delayed_delete,
        ):
            worker.start()
            self.assertTrue(delete_entered.wait(5))
            try:
                with self.assertRaises(CoordinatorError) as rejected:
                    self.jobs.acquire(
                        "session-a",
                        CargoLaneKind.TEST,
                        requested_target=unmanaged,
                    )
                self.assertEqual("cargo_lane_cleanup_reserved", rejected.exception.code)
            finally:
                release_delete.set()
                worker.join(5)

        self.assertFalse(worker.is_alive())
        self.assertEqual((str(unmanaged.resolve()),), cleanup_result[0].deleted)
        with self.database.connect() as connection:
            reservations = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_dir=?",
                (str(unmanaged.resolve()),),
            ).fetchone()[0]
        self.assertEqual(0, reservations)

    def test_concurrent_cleanup_calls_delete_candidate_once(self) -> None:
        unmanaged = self.builds_root / "unregistered-review-copy"
        unmanaged.mkdir()
        delete_entered = threading.Event()
        release_delete = threading.Event()
        results = []

        def delayed_delete(path: Path, *, expected_identity: str) -> None:
            delete_entered.set()
            self.assertTrue(release_delete.wait(5))
            __import__("shutil").rmtree(path)

        def cleanup() -> None:
            results.append(self.governance.cleanup())

        with mock.patch(
            "tools.session_coordinator.artifact_governance._remove_candidate_tree",
            side_effect=delayed_delete,
        ) as remove:
            first = threading.Thread(target=cleanup)
            second = threading.Thread(target=cleanup)
            first.start()
            self.assertTrue(delete_entered.wait(5))
            second.start()
            release_delete.set()
            first.join(5)
            second.join(5)

        self.assertFalse(first.is_alive())
        self.assertFalse(second.is_alive())
        self.assertEqual(1, remove.call_count)
        self.assertEqual(1, sum(bool(result.deleted) for result in results))

    def test_restart_recovers_artifact_reservation_with_same_identity(self) -> None:
        unmanaged = self.builds_root / "unregistered-review-copy"
        unmanaged.mkdir()
        identity = __import__(
            "tools.session_coordinator.windows_tree_delete",
            fromlist=["filesystem_identity"],
        ).filesystem_identity(unmanaged)
        key = str(unmanaged.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-13T00:00:00+00:00', 'artifact', ?)""",
                (key, str(unmanaged.resolve()), identity),
            )

        result = self.governance.recover_reservations()

        self.assertEqual((str(unmanaged.resolve()),), result.deleted)
        self.assertFalse(unmanaged.exists())
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    def test_restart_keeps_reservation_that_overlaps_legacy_validation_copy(self) -> None:
        unmanaged = self.builds_root / "legacy-validation-copy"
        unmanaged.mkdir()
        marker = unmanaged / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        identity = __import__(
            "tools.session_coordinator.windows_tree_delete",
            fromlist=["filesystem_identity"],
        ).filesystem_identity(unmanaged)
        key = str(unmanaged.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO validation_copies(
                       job_id, session_id, job_root, source_root, target_root,
                       head_commit, manifest_json, status, created_at,
                       external_sources_json
                   ) VALUES (
                       'legacy-validation', 'session-a', ?, ?, ?, 'head', '[]',
                       'planned', '2026-08-13T00:00:00+00:00', '[]'
                   )""",
                (str(unmanaged), str(unmanaged / "source"), str(unmanaged / "target")),
            )
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-13T00:00:00+00:00', 'artifact', ?)""",
                (key, str(unmanaged.resolve()), identity),
            )

        result = self.governance.recover_reservations()

        self.assertEqual((), result.deleted)
        self.assertEqual((str(unmanaged.resolve()),), tuple(item.path for item in result.failed))
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()[0]
        self.assertEqual(1, remaining)

    def test_restart_keeps_outside_artifact_reservation_without_deleting(self) -> None:
        outside = self.builds_root.parent / "outside-reservation"
        outside.mkdir()
        marker = outside / "keep.txt"
        marker.write_text("keep", encoding="utf-8")
        key = str(outside.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-13T00:00:00+00:00', 'artifact', 'forged')""",
                (key, str(outside.resolve())),
            )

        result = self.governance.recover_reservations()

        self.assertEqual((), result.deleted)
        self.assertEqual((str(outside.resolve()),), tuple(item.path for item in result.failed))
        self.assertEqual("keep", marker.read_text(encoding="utf-8"))
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()[0]
        self.assertEqual(1, remaining)

    def test_restart_releases_missing_artifact_reservation(self) -> None:
        missing = self.builds_root / "missing-reservation"
        key = str(missing.resolve()).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-13T00:00:00+00:00', 'artifact', 'old')""",
                (key, str(missing.resolve())),
            )

        result = self.governance.recover_reservations()

        self.assertEqual((str(missing.resolve()),), result.deleted)
        self.assertEqual((), result.failed)
        with self.database.connect() as connection:
            remaining = connection.execute(
                "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?", (key,)
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    def test_restart_recovers_all_missing_mvp_fixture_parent_reservations(self) -> None:
        missing = tuple(
            self.builds_root / f"mvp-test-fixtures-{pid}"
            for pid in (11376, 29760, 10976, 16996)
        )
        with self.database.transaction() as connection:
            for path in missing:
                key = str(path.resolve()).replace("/", "\\").casefold()
                connection.execute(
                    """INSERT INTO cleanup_reservations(
                           target_key, target_dir, reserved_at,
                           reservation_kind, filesystem_identity
                       ) VALUES (?, ?, '2026-08-15T00:00:00+00:00', 'artifact', 'old')""",
                    (key, str(path.resolve())),
                )

        result = self.governance.recover_reservations()

        self.assertEqual(
            tuple(sorted(str(path.resolve()) for path in missing)),
            tuple(sorted(result.deleted)),
        )
        self.assertEqual((), result.failed)
        with self.database.connect() as connection:
            remaining = connection.execute(
                """SELECT COUNT(*) FROM cleanup_reservations
                   WHERE reservation_kind='artifact'"""
            ).fetchone()[0]
        self.assertEqual(0, remaining)

    @unittest.skipUnless(os.name == "nt", "Windows junction semantics")
    def test_restart_keeps_dangling_junction_reservation(self) -> None:
        missing_target = self.builds_root / "missing-junction-target"
        missing_target.mkdir()
        junction = self.builds_root / "dangling-reservation"
        subprocess.run(
            ["cmd.exe", "/c", "mklink", "/J", str(junction), str(missing_target)],
            check=True,
            capture_output=True,
            text=True,
        )
        os.rmdir(missing_target)
        key = str(junction).replace("/", "\\").casefold()
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO cleanup_reservations(
                       target_key, target_dir, reserved_at,
                       reservation_kind, filesystem_identity
                   ) VALUES (?, ?, '2026-08-13T00:00:00+00:00', 'artifact', 'old')""",
                (key, str(junction)),
            )

        try:
            result = self.governance.recover_reservations()

            self.assertEqual((), result.deleted)
            self.assertEqual((str(junction),), tuple(item.path for item in result.failed))
            self.assertTrue(junction.is_junction())
            with self.database.connect() as connection:
                remaining = connection.execute(
                    "SELECT COUNT(*) FROM cleanup_reservations WHERE target_key=?",
                    (key,),
                ).fetchone()[0]
            self.assertEqual(1, remaining)
        finally:
            if junction.is_junction():
                os.rmdir(junction)

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
