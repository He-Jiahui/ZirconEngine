from __future__ import annotations

import os
import shutil
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.artifact_governance import ArtifactGovernanceService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class ArtifactProductStagingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.builds_root = root / "D" / "ZirconBuilds"
        self.builds_root.mkdir(parents=True)
        self.database = Database(root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.governance = ArtifactGovernanceService(
            self.database, roots=(self.builds_root,)
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_identity_preserving_publish_stays_managed_until_removed(self) -> None:
        final_path = self.builds_root / "editor-current"
        lease = self.governance.acquire_product_staging(
            "build-editor", final_path=final_path, owner_pid=os.getpid()
        )

        self.assertFalse(Path(lease.staging_path).exists())
        self.assertEqual((), self.governance.scan())
        staging_path = Path(lease.staging_path)
        staging_path.mkdir()
        (staging_path / "zircon_editor.exe").write_bytes(b"editor")
        self.assertEqual((), self.governance.scan())

        publishing = self.governance.begin_product_staging_publish(
            lease.lease_id, owner_pid=os.getpid()
        )
        staging_path.rename(final_path)
        published = self.governance.complete_product_staging_publish(
            lease.lease_id, owner_pid=os.getpid()
        )

        self.assertEqual("publishing", publishing.status)
        self.assertEqual("published", published.status)
        self.assertEqual(
            publishing.staging_filesystem_identity,
            published.published_filesystem_identity,
        )
        self.assertEqual((), self.governance.scan())

        shutil.rmtree(final_path)
        self.governance.recover_reservations()
        final_path.mkdir()
        candidates = self.governance.scan()
        self.assertEqual([str(final_path.resolve())], [item.path for item in candidates])
        final_path.rmdir()

        replacement = self.governance.acquire_product_staging(
            "build-editor", final_path=final_path, owner_pid=os.getpid()
        )
        self.assertNotEqual(lease.lease_id, replacement.lease_id)
        self.assertEqual(str(final_path.resolve()), replacement.final_path)

    def test_release_requires_same_owner_and_both_paths_absent(self) -> None:
        with self.assertRaises(CoordinatorError) as unsupported:
            self.governance.acquire_product_staging(
                "manual-bundle",
                final_path=self.builds_root / "manual-bundle",
                owner_pid=os.getpid(),
            )
        self.assertEqual(
            "artifact_product_staging_purpose_invalid", unsupported.exception.code
        )

        final_path = self.builds_root / "editor-failed"
        lease = self.governance.acquire_product_staging(
            "build-editor", final_path=final_path, owner_pid=os.getpid()
        )
        staging_path = Path(lease.staging_path)
        staging_path.mkdir()

        with self.assertRaises(CoordinatorError) as foreign:
            self.governance.release_product_staging(
                lease.lease_id, owner_pid=os.getpid() + 1
            )
        self.assertEqual("artifact_product_staging_owner_mismatch", foreign.exception.code)

        with self.assertRaises(CoordinatorError) as present:
            self.governance.release_product_staging(
                lease.lease_id, owner_pid=os.getpid()
            )
        self.assertEqual("artifact_product_staging_path_still_exists", present.exception.code)

        staging_path.rmdir()
        released = self.governance.release_product_staging(
            lease.lease_id, owner_pid=os.getpid()
        )
        self.assertEqual("released", released.status)

        staging_path.mkdir()
        candidates = self.governance.scan()
        self.assertEqual([str(staging_path.resolve())], [item.path for item in candidates])

    def test_recovery_publishes_only_the_captured_staging_identity(self) -> None:
        accepted_final = self.builds_root / "editor-recovered"
        accepted = self.governance.acquire_product_staging(
            "build-editor", final_path=accepted_final, owner_pid=os.getpid()
        )
        accepted_staging = Path(accepted.staging_path)
        accepted_staging.mkdir()
        (accepted_staging / "marker").write_text("accepted", encoding="utf-8")
        self.governance.begin_product_staging_publish(
            accepted.lease_id, owner_pid=os.getpid()
        )
        accepted_staging.rename(accepted_final)

        rejected_final = self.builds_root / "editor-rejected"
        rejected = self.governance.acquire_product_staging(
            "build-editor", final_path=rejected_final, owner_pid=os.getpid()
        )
        rejected_staging = Path(rejected.staging_path)
        rejected_staging.mkdir()
        (rejected_staging / "marker").write_text("rejected", encoding="utf-8")
        self.governance.begin_product_staging_publish(
            rejected.lease_id, owner_pid=os.getpid()
        )
        shutil.copytree(rejected_staging, rejected_final)
        shutil.rmtree(rejected_staging)

        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE artifact_product_staging_leases
                   SET owner_process_creation_time='stale-owner'
                   WHERE lease_id IN (?, ?)""",
                (accepted.lease_id, rejected.lease_id),
            )

        self.governance.recover_reservations()

        with self.database.connect() as connection:
            rows = {
                row["lease_id"]: row["status"]
                for row in connection.execute(
                    """SELECT lease_id, status
                       FROM artifact_product_staging_leases
                       WHERE lease_id IN (?, ?)""",
                    (accepted.lease_id, rejected.lease_id),
                )
            }
        self.assertEqual("published", rows[accepted.lease_id])
        self.assertEqual("recovered", rows[rejected.lease_id])
        self.assertEqual(
            [str(rejected_final.resolve())],
            [item.path for item in self.governance.scan()],
        )

    def test_filesystem_identity_probe_does_not_hold_sqlite_write_lock(self) -> None:
        final_path = self.builds_root / "editor-concurrent"
        lease = self.governance.acquire_product_staging(
            "build-editor", final_path=final_path, owner_pid=os.getpid()
        )
        Path(lease.staging_path).mkdir()
        probe_entered = threading.Event()
        release_probe = threading.Event()
        errors: list[BaseException] = []

        def blocking_identity(path: Path) -> str:
            probe_entered.set()
            if not release_probe.wait(timeout=20):
                raise TimeoutError("identity probe was not released")
            return "sealed-identity"

        def begin_publish() -> None:
            try:
                self.governance.begin_product_staging_publish(
                    lease.lease_id, owner_pid=os.getpid()
                )
            except BaseException as error:
                errors.append(error)

        with mock.patch(
            "tools.session_coordinator.artifact_product_staging.filesystem_identity",
            side_effect=blocking_identity,
        ):
            worker = threading.Thread(target=begin_publish)
            worker.start()
            self.assertTrue(probe_entered.wait(timeout=5))
            started = time.monotonic()
            try:
                with self.database.transaction() as connection:
                    connection.execute(
                        """INSERT INTO events(event_type, payload_json, created_at)
                           VALUES ('test.concurrent_write', '{}', 'now')"""
                    )
            finally:
                release_probe.set()
                worker.join(timeout=10)

        self.assertLess(time.monotonic() - started, 2.0)
        self.assertFalse(worker.is_alive())
        self.assertEqual([], errors)


if __name__ == "__main__":
    unittest.main()
