from __future__ import annotations

import gzip
import json
import unittest
from datetime import UTC, datetime
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.session_coordinator.database import Database
from tools.session_coordinator.manifest_retention import ManifestRetentionService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class ManifestRetentionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.root = Path(self.temporary_directory.name)
        self.database = Database(self.root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.service = ManifestRetentionService(self.database, self.root / "state")
        self.now = datetime(2026, 7, 31, tzinfo=UTC)

    def _session(self, session_id: str, status: str, baseline_epoch: int | None = None) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, status, baseline_epoch, created_at, updated_at, last_heartbeat_at
                ) VALUES (?, ?, ?, '2026-07-01T00:00:00+00:00',
                          '2026-07-01T00:00:00+00:00', '2026-07-01T00:00:00+00:00')
                """,
                (session_id, status, baseline_epoch),
            )

    def _baseline(self, manifest: object, created_at: str) -> int:
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """
                INSERT INTO baseline_epochs(head_commit, index_tree, health, manifest_json, created_at)
                VALUES ('head', 'tree', 'healthy', ?, ?)
                """,
                (json.dumps(manifest, sort_keys=True), created_at),
            )
        return int(cursor.lastrowid)

    def _validation_copy(
        self,
        job_id: str,
        session_id: str,
        manifest: object,
        status: str,
        created_at: str,
        removed_at: str | None = None,
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root, head_commit,
                    manifest_json, status, created_at, removed_at
                ) VALUES (?, ?, ?, ?, ?, 'head', ?, ?, ?, ?)
                """,
                (
                    job_id,
                    session_id,
                    str(self.root / "jobs" / job_id),
                    str(self.root / "jobs" / job_id / "source"),
                    str(self.root / "jobs" / job_id / "target"),
                    json.dumps(manifest, sort_keys=True),
                    status,
                    created_at,
                    removed_at,
                ),
            )

    def test_preview_retires_only_old_unreferenced_terminal_manifests(self) -> None:
        protected_epoch = self._baseline({"protected.rs": "a"}, "2026-07-01T00:00:00+00:00")
        expired_epoch = self._baseline({"expired.rs": "b"}, "2026-07-01T00:00:00+00:00")
        latest_epoch = self._baseline({"latest.rs": "c"}, "2026-07-01T00:00:00+00:00")
        self._session("active-owner", "active", protected_epoch)
        self._session("terminal-owner", "completed")
        self._validation_copy(
            "old-removed",
            "terminal-owner",
            ["old.rs"],
            "removed",
            "2026-07-01T00:00:00+00:00",
            "2026-07-02T00:00:00+00:00",
        )
        self._validation_copy(
            "old-running",
            "active-owner",
            ["running.rs"],
            "running",
            "2026-07-01T00:00:00+00:00",
        )
        self._validation_copy(
            "recent-removed",
            "terminal-owner",
            ["recent.rs"],
            "removed",
            "2026-07-30T00:00:00+00:00",
            "2026-07-30T00:00:00+00:00",
        )

        preview = self.service.preview(now=self.now)

        self.assertEqual(
            {("baseline_epochs", str(expired_epoch)), ("validation_copies", "old-removed")},
            {(candidate.table, candidate.identity) for candidate in preview.candidates},
        )
        self.assertNotIn(
            ("baseline_epochs", str(protected_epoch)),
            {(candidate.table, candidate.identity) for candidate in preview.candidates},
        )
        self.assertNotIn(
            ("baseline_epochs", str(latest_epoch)),
            {(candidate.table, candidate.identity) for candidate in preview.candidates},
        )

    def test_apply_archives_verified_manifests_before_retiring_their_database_payloads(self) -> None:
        expired_epoch = self._baseline({"expired.rs": "b"}, "2026-07-01T00:00:00+00:00")
        self._baseline({"latest.rs": "c"}, "2026-07-30T00:00:00+00:00")
        self._session("terminal-owner", "completed")
        self._validation_copy(
            "old-removed",
            "terminal-owner",
            ["old.rs"],
            "removed",
            "2026-07-01T00:00:00+00:00",
            "2026-07-02T00:00:00+00:00",
        )
        preview = self.service.preview(now=self.now)

        result = self.service.apply(
            preview, fingerprint=preview.fingerprint, actor="test", now=self.now
        )
        repeated = self.service.apply(
            preview, fingerprint=preview.fingerprint, actor="test", now=self.now
        )

        self.assertTrue(result.archive_path.is_file())
        self.assertTrue(result.backup_path.is_file())
        self.assertEqual(result, repeated)
        with gzip.open(result.archive_path, "rt", encoding="utf-8") as stream:
            entries = [json.loads(line) for line in stream]
        self.assertEqual(2, len(entries))
        self.assertEqual(
            {"baseline_epochs", "validation_copies"}, {entry["table"] for entry in entries}
        )
        with self.database.connect() as connection:
            baseline = connection.execute(
                """
                SELECT manifest_json, manifest_sha256, manifest_entry_count,
                       manifest_byte_count, manifest_archive_path, manifest_archived_at
                FROM baseline_epochs WHERE epoch_id=?
                """,
                (expired_epoch,),
            ).fetchone()
            copy = connection.execute(
                """
                SELECT manifest_json, manifest_archive_path, manifest_archived_at
                FROM validation_copies WHERE job_id='old-removed'
                """
            ).fetchone()
            batch = connection.execute(
                "SELECT status, candidate_count FROM manifest_retention_batches WHERE batch_id=?",
                (preview.fingerprint,),
            ).fetchone()
        self.assertEqual("{}", baseline["manifest_json"])
        self.assertEqual(1, baseline["manifest_entry_count"])
        self.assertGreater(int(baseline["manifest_byte_count"]), 0)
        self.assertEqual(str(result.archive_path.relative_to(self.root / "state")), baseline["manifest_archive_path"])
        self.assertIsNotNone(baseline["manifest_archived_at"])
        self.assertEqual("[]", copy["manifest_json"])
        self.assertIsNotNone(copy["manifest_archive_path"])
        self.assertEqual(("retired", 2), tuple(batch))

    def test_archive_verification_failure_leaves_database_manifests_intact(self) -> None:
        expired_epoch = self._baseline({"expired.rs": "b"}, "2026-07-01T00:00:00+00:00")
        self._baseline({"latest.rs": "c"}, "2026-07-30T00:00:00+00:00")
        preview = self.service.preview(now=self.now)
        original_verifier = self.service._verify_archive

        def reject_archive(*_arguments: object, **_kwargs: object) -> None:
            raise CoordinatorError("manifest_retention_archive_hash_mismatch", "test mismatch")

        self.service._verify_archive = reject_archive  # type: ignore[method-assign]
        self.addCleanup(setattr, self.service, "_verify_archive", original_verifier)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.apply(preview, fingerprint=preview.fingerprint, actor="test", now=self.now)

        self.assertEqual("manifest_retention_archive_hash_mismatch", rejected.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT manifest_json, manifest_archive_path FROM baseline_epochs WHERE epoch_id=?",
                (expired_epoch,),
            ).fetchone()
        self.assertEqual(json.dumps({"expired.rs": "b"}, sort_keys=True), row["manifest_json"])
        self.assertIsNone(row["manifest_archive_path"])

    def test_compact_requires_a_retired_batch_and_keeps_sqlite_healthy(self) -> None:
        expired_epoch = self._baseline({"expired.rs": "b"}, "2026-07-01T00:00:00+00:00")
        self._baseline({"latest.rs": "c"}, "2026-07-30T00:00:00+00:00")
        preview = self.service.preview(now=self.now)
        applied = self.service.apply(
            preview, fingerprint=preview.fingerprint, actor="test", now=self.now
        )
        queued = self.service.queue_compact(applied.batch_id, actor="test", now=self.now)

        compacted = self.service.compact(applied.batch_id, actor="test", now=self.now)

        self.assertEqual(applied.batch_id, compacted.batch_id)
        self.assertEqual("compact_pending", queued.status)
        self.assertEqual("ok", compacted.quick_check)
        self.assertGreaterEqual(compacted.size_before, compacted.size_after)
        with self.database.connect() as connection:
            archived = connection.execute(
                "SELECT manifest_archive_path FROM baseline_epochs WHERE epoch_id=?",
                (expired_epoch,),
            ).fetchone()
            batch = connection.execute(
                "SELECT status, compacted_at FROM manifest_retention_batches WHERE batch_id=?",
                (applied.batch_id,),
            ).fetchone()
        self.assertIsNotNone(archived["manifest_archive_path"])
        self.assertEqual("compacted", batch["status"])
        self.assertIsNotNone(batch["compacted_at"])


if __name__ == "__main__":
    unittest.main()
