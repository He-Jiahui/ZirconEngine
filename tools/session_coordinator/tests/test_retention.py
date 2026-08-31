from __future__ import annotations

import tempfile
import threading
import time
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest import mock

from tools.session_coordinator.cleanup import RetentionService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.helpers import init_repo


NOW = datetime(2026, 7, 11, 5, 0, tzinfo=UTC)


class RetentionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.objects = ObjectStore(self.database, config.object_root)
        self.snapshots = SnapshotService(self.database, self.repo, self.objects)
        self.retention = RetentionService(self.database, self.objects)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _snapshot(self, session_id: str, content: str) -> tuple[int, str]:
        self.sessions.register(session_id=session_id)
        self.sessions.set_status(session_id, SessionStatus.ACTIVE)
        target = self.repo / f"{session_id}.txt"
        target.write_text(content, encoding="utf-8")
        record = self.snapshots.create(
            session_id=session_id,
            paths=(target.name,),
            baseline_epoch=None,
            purpose="retention fixture",
        )
        return record.snapshot_id, next(iter(record.manifest.values()))  # type: ignore[return-value]

    def test_plan_keeps_active_and_patch_referenced_objects(self) -> None:
        active_snapshot, active_hash = self._snapshot("active", "active")
        expired_snapshot, expired_hash = self._snapshot("expired", "expired")
        self.sessions.set_status("expired", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'expired'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at = ? WHERE snapshot_id = ?",
                ((NOW - timedelta(days=20)).isoformat(), expired_snapshot),
            )
            connection.execute(
                """
                INSERT INTO patches(
                    session_id, patch_object_hash, targets_json, base_hashes_json,
                    base_objects_json, status, created_at, updated_at
                ) VALUES ('active', ?, '[]', '{}', '{}', 'queued', ?, ?)
                """,
                (active_hash, NOW.isoformat(), NOW.isoformat()),
            )

        plan = self.retention.plan(now=NOW)

        self.assertIn(expired_snapshot, plan.snapshot_ids)
        self.assertIn(expired_hash, plan.object_hashes)
        self.assertNotIn(active_snapshot, plan.snapshot_ids)
        self.assertNotIn(active_hash, plan.object_hashes)

    def test_apply_requires_persisted_plan_and_deletes_only_candidates(self) -> None:
        active_snapshot, active_hash = self._snapshot("active", "active")
        expired_snapshot, expired_hash = self._snapshot("expired", "expired")
        self.sessions.set_status("expired", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'expired'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at = ? WHERE snapshot_id = ?",
                ((NOW - timedelta(days=20)).isoformat(), expired_snapshot),
            )
        plan = self.retention.plan(now=NOW)

        result = self.retention.apply(plan)

        self.assertEqual((expired_snapshot,), result.deleted_snapshot_ids)
        self.assertEqual((expired_hash,), result.deleted_object_hashes)
        self.assertEqual(b"active", self.objects.get(active_hash))
        self.assertEqual(active_snapshot, self.snapshots.get(active_snapshot).snapshot_id)
        with self.assertRaises(Exception):
            self.objects.get(expired_hash)

    def test_archived_snapshot_remains_previewable_inside_thirty_days(self) -> None:
        snapshot_id, _ = self._snapshot("archived", "before")
        self.sessions.set_status("archived", SessionStatus.COMPLETED)
        self.sessions.set_status("archived", SessionStatus.ARCHIVED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET archived_at = ? WHERE session_id = 'archived'",
                ((NOW - timedelta(days=10)).isoformat(),),
            )
        (self.repo / "archived.txt").write_text("after", encoding="utf-8")

        plan = self.retention.plan(now=NOW)
        preview = self.snapshots.restore_preview(snapshot_id)

        self.assertNotIn(snapshot_id, plan.snapshot_ids)
        self.assertTrue(preview[0].would_change)

    def test_active_validation_ticket_retains_old_completed_session_snapshot(self) -> None:
        snapshot_id, object_hash = self._snapshot("completed", "queued source")
        ticket_id = "ticket-retained"
        self.sessions.set_status("completed", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at=? WHERE session_id='completed'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at=?, purpose=? WHERE snapshot_id=?",
                (
                    (NOW - timedelta(days=20)).isoformat(),
                    f"validation-ticket-source:{ticket_id}",
                    snapshot_id,
                ),
            )
            connection.execute(
                """
                INSERT INTO validation_tickets(
                    ticket_id, session_id, plan_path, status, dedupe_key,
                    source_manifest_hash, source_manifest_json, command_json,
                    toolchain_json, coverage_json, created_at, updated_at
                ) VALUES (?, 'completed', 'docs/plans/tooling/01.md', 'queued',
                          'dedupe', ?, ?, '["cargo","check"]', '{}', '{}', ?, ?)
                """,
                (
                    ticket_id,
                    "a" * 64,
                    '{"completed.txt":"' + object_hash + '"}',
                    NOW.isoformat(),
                    NOW.isoformat(),
                ),
            )

        active_plan = self.retention.plan(now=NOW)
        self.assertNotIn(snapshot_id, active_plan.snapshot_ids)
        self.assertNotIn(object_hash, active_plan.object_hashes)

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_tickets SET status='failed' WHERE ticket_id=?",
                (ticket_id,),
            )
        terminal_plan = self.retention.plan(now=NOW)
        self.assertIn(snapshot_id, terminal_plan.snapshot_ids)
        self.assertIn(object_hash, terminal_plan.object_hashes)

    def test_failed_gc_plan_can_be_replanned_after_quarantine_recovery(self) -> None:
        snapshot_id, object_hash = self._snapshot("expired", "retryable")
        self.sessions.set_status("expired", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'expired'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at = ? WHERE snapshot_id = ?",
                ((NOW - timedelta(days=20)).isoformat(), snapshot_id),
            )
        plan = self.retention.plan(now=NOW)
        object_path = self.objects.path_for_hash(object_hash)
        original = object_path.read_bytes()
        object_path.unlink()
        with self.assertRaises(Exception):
            self.retention.apply(plan, now=NOW)
        object_path.parent.mkdir(parents=True, exist_ok=True)
        object_path.write_bytes(original)

        retried = self.retention.plan(now=NOW)
        result = self.retention.apply(retried, now=NOW)

        self.assertEqual(plan.plan_id, retried.plan_id)
        self.assertEqual((snapshot_id,), result.deleted_snapshot_ids)

    def test_new_snapshot_for_old_completed_session_gets_its_own_retention_window(self) -> None:
        self.sessions.register(session_id="completed")
        self.sessions.set_status("completed", SessionStatus.ACTIVE)
        self.sessions.set_status("completed", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'completed'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
        target = self.repo / "completed.txt"
        target.write_text("recent snapshot", encoding="utf-8")
        snapshot = self.snapshots.create(
            session_id="completed",
            paths=(target.name,),
            baseline_epoch=None,
            purpose="recent terminal snapshot",
        )

        plan = self.retention.plan(now=NOW)

        self.assertNotIn(snapshot.snapshot_id, plan.snapshot_ids)

    def test_gc_serializes_new_snapshot_reference_and_leaves_it_restorable(self) -> None:
        expired_snapshot, object_hash = self._snapshot("expired", "shared")
        self.sessions.set_status("expired", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'expired'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at = ? WHERE snapshot_id = ?",
                ((NOW - timedelta(days=20)).isoformat(), expired_snapshot),
            )
        self.sessions.register(session_id="active")
        self.sessions.set_status("active", SessionStatus.ACTIVE)
        active_path = self.repo / "active.txt"
        active_path.write_text("shared", encoding="utf-8")
        plan = self.retention.plan(now=NOW)
        entered_move = threading.Event()
        allow_move = threading.Event()
        snapshot_done = threading.Event()
        errors: list[BaseException] = []
        real_replace = __import__("os").replace

        def controlled_replace(source, destination):
            if "gc-trash" in str(destination):
                entered_move.set()
                if not allow_move.wait(timeout=5):
                    raise TimeoutError("test did not release GC move")
            return real_replace(source, destination)

        def apply_gc() -> None:
            try:
                self.retention.apply(plan, now=NOW)
            except BaseException as error:
                errors.append(error)

        def create_snapshot() -> None:
            try:
                self.snapshots.create(
                    session_id="active",
                    paths=(active_path.name,),
                    baseline_epoch=None,
                    purpose="concurrent live reference",
                )
            except BaseException as error:
                errors.append(error)
            finally:
                snapshot_done.set()

        with mock.patch("tools.session_coordinator.cleanup.os.replace", controlled_replace):
            collector = threading.Thread(target=apply_gc)
            collector.start()
            self.assertTrue(entered_move.wait(timeout=5))
            creator = threading.Thread(target=create_snapshot)
            creator.start()
            time.sleep(0.1)
            self.assertFalse(snapshot_done.is_set())
            allow_move.set()
            collector.join(timeout=5)
            creator.join(timeout=5)

        self.assertEqual([], errors)
        self.assertEqual(b"shared", self.objects.get(object_hash))

    def test_startup_recovers_precommit_gc_quarantine(self) -> None:
        snapshot_id, object_hash = self._snapshot("expired", "recover")
        self.sessions.set_status("expired", SessionStatus.COMPLETED)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET completed_at = ? WHERE session_id = 'expired'",
                ((NOW - timedelta(days=20)).isoformat(),),
            )
            connection.execute(
                "UPDATE snapshots SET created_at = ? WHERE snapshot_id = ?",
                ((NOW - timedelta(days=20)).isoformat(), snapshot_id),
            )
        plan = self.retention.plan(now=NOW)
        source = self.objects.path_for_hash(object_hash)
        quarantine = self.objects.root.parent / "gc-trash" / plan.plan_id / object_hash
        quarantine.parent.mkdir(parents=True)
        source.replace(quarantine)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE object_gc_plans SET status = 'applying' WHERE plan_id = ?",
                (plan.plan_id,),
            )

        recovered = self.retention.recover_interrupted()

        self.assertEqual((plan.plan_id,), recovered)
        self.assertEqual(b"recover", self.objects.get(object_hash))
        self.assertEqual(snapshot_id, self.snapshots.get(snapshot_id).snapshot_id)


if __name__ == "__main__":
    unittest.main()
