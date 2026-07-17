from __future__ import annotations

import tempfile
import unittest
import os
from pathlib import Path

from tools.session_coordinator.codex_sync.discovery import CodexSessionDiscovery
from tools.session_coordinator.codex_sync.models import CodexSyncTrigger
from tools.session_coordinator.codex_sync.store import CodexSessionStore
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate
from tools.session_coordinator.tests.codex_rollout_fixture import write_rollout


class CodexStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "ZirconEngine"
        self.repo.mkdir()
        self.codex_home = self.root / "codex"
        self.database = Database(self.root / "coordinator.sqlite3")
        migrate(self.database)
        self.store = CodexSessionStore(
            self.database, clock=lambda: "2026-07-13T00:00:00+00:00"
        )

    def _discover(self):
        return CodexSessionDiscovery(self.codex_home, self.repo).discover()

    def test_schema_v27_and_reconcile_are_idempotent(self) -> None:
        secret = "database-secret-marker"
        write_rollout(
            self.codex_home,
            thread_id="thread-one",
            cwd=self.repo,
            lifecycle=("task_started",),
            secret_marker=secret,
        )

        clocks = iter(("2026-07-13T00:00:00+00:00", "2026-07-13T00:01:00+00:00"))
        store = CodexSessionStore(self.database, clock=lambda: next(clocks))
        first = store.reconcile(self._discover(), trigger=CodexSyncTrigger.STARTUP)
        with self.database.connect() as connection:
            first_projection = tuple(
                connection.execute(
                    "SELECT * FROM codex_sessions WHERE thread_id='thread-one'"
                ).fetchone()
            )
        second = store.reconcile(self._discover(), trigger=CodexSyncTrigger.PERIODIC)

        with self.database.connect() as connection:
            version = connection.execute("SELECT MAX(version) FROM schema_version").fetchone()[0]
            rows = connection.execute("SELECT * FROM codex_sessions").fetchall()
            session_event_count = connection.execute(
                "SELECT COUNT(*) FROM events WHERE event_type LIKE 'codex.session.%'"
            ).fetchone()[0]
            payloads = "\n".join(
                row[0] for row in connection.execute("SELECT payload_json FROM events")
            )
        self.assertEqual(LATEST_SCHEMA_VERSION, version)
        self.assertEqual(1, len(rows))
        self.assertEqual(1, first.changed_count)
        self.assertEqual(0, second.changed_count)
        self.assertEqual(first_projection, tuple(rows[0]))
        self.assertEqual(1, session_event_count)
        self.assertNotIn(secret, payloads)
        self.assertNotIn(secret, "|".join(str(value) for value in rows[0]))

    def test_exact_thread_id_binding_only(self) -> None:
        write_rollout(self.codex_home, thread_id="exact-thread", cwd=self.repo)
        write_rollout(self.codex_home, thread_id="similar-thread", cwd=self.repo)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, display_name, status, created_at, updated_at,
                    last_heartbeat_at
                ) VALUES ('exact-thread', 'similar-thread', 'registered', 'now', 'now', 'now')
                """
            )

        self.store.reconcile(self._discover(), trigger=CodexSyncTrigger.STARTUP)

        with self.database.connect() as connection:
            bindings = {
                row["thread_id"]: row["bound_session_id"]
                for row in connection.execute(
                    "SELECT thread_id, bound_session_id FROM codex_sessions"
                )
            }
        self.assertEqual("exact-thread", bindings["exact-thread"])
        self.assertIsNone(bindings["similar-thread"])

    def test_missing_requires_two_complete_membership_scans(self) -> None:
        rollout = write_rollout(self.codex_home, thread_id="vanishing-thread", cwd=self.repo)
        self.store.reconcile(self._discover(), trigger=CodexSyncTrigger.STARTUP)
        rollout.unlink()

        first_missing = self.store.reconcile(
            self._discover(), trigger=CodexSyncTrigger.PERIODIC
        )
        with self.database.connect() as connection:
            after_first = connection.execute(
                "SELECT state, missing_scan_count FROM codex_sessions"
            ).fetchone()
        second_missing = self.store.reconcile(
            self._discover(), trigger=CodexSyncTrigger.PERIODIC
        )
        with self.database.connect() as connection:
            after_second = connection.execute(
                "SELECT source_location, state, missing_scan_count FROM codex_sessions"
            ).fetchone()

        self.assertEqual(("idle", 1), tuple(after_first))
        self.assertEqual(("missing", "unavailable", 2), tuple(after_second))
        self.assertEqual(0, first_missing.unavailable_count)
        self.assertEqual(1, second_missing.unavailable_count)

        third_missing = self.store.reconcile(
            self._discover(), trigger=CodexSyncTrigger.PERIODIC
        )
        with self.database.connect() as connection:
            after_third = connection.execute(
                "SELECT source_location, state, missing_scan_count FROM codex_sessions"
            ).fetchone()
        self.assertEqual(("missing", "unavailable", 2), tuple(after_third))
        self.assertEqual(0, third_missing.changed_count)

    def test_metadata_only_source_revision_refresh_is_quiet(self) -> None:
        rollout = write_rollout(
            self.codex_home,
            thread_id="quiet-thread",
            cwd=self.repo,
            lifecycle=("task_started",),
        )
        clocks = iter(("2026-07-13T00:00:00+00:00", "2026-07-13T00:01:00+00:00"))
        store = CodexSessionStore(self.database, clock=lambda: next(clocks))
        first = store.reconcile(self._discover(), trigger=CodexSyncTrigger.STARTUP)
        stat = rollout.stat()
        os.utime(
            rollout,
            ns=(stat.st_atime_ns, stat.st_mtime_ns + 1_000_000_000),
        )

        second = store.reconcile(self._discover(), trigger=CodexSyncTrigger.PERIODIC)

        with self.database.connect() as connection:
            event_types = [
                row[0]
                for row in connection.execute("SELECT event_type FROM events ORDER BY event_id")
            ]
            row = connection.execute(
                "SELECT source_mtime_ns, last_synced_at FROM codex_sessions WHERE thread_id='quiet-thread'"
            ).fetchone()
        self.assertEqual(1, first.changed_count)
        self.assertEqual(0, second.changed_count)
        self.assertEqual(["codex.session.discovered", "codex.sync.completed"], event_types)
        self.assertEqual(stat.st_mtime_ns + 1_000_000_000, row[0])
        self.assertEqual("2026-07-13T00:01:00+00:00", row[1])

    def test_visible_codex_lifecycle_change_remains_a_timeline_event(self) -> None:
        write_rollout(
            self.codex_home,
            thread_id="visible-thread",
            cwd=self.repo,
            lifecycle=("task_started",),
        )
        clocks = iter(("2026-07-13T00:00:00+00:00", "2026-07-13T00:01:00+00:00"))
        store = CodexSessionStore(self.database, clock=lambda: next(clocks))
        store.reconcile(self._discover(), trigger=CodexSyncTrigger.STARTUP)
        write_rollout(
            self.codex_home,
            thread_id="visible-thread",
            cwd=self.repo,
            lifecycle=("task_completed",),
        )

        changed = store.reconcile(self._discover(), trigger=CodexSyncTrigger.PERIODIC)

        with self.database.connect() as connection:
            event_types = [
                row[0]
                for row in connection.execute("SELECT event_type FROM events ORDER BY event_id")
            ]
        self.assertEqual(1, changed.changed_count)
        self.assertEqual(
            [
                "codex.session.discovered",
                "codex.sync.completed",
                "codex.session.state_changed",
                "codex.sync.completed",
            ],
            event_types,
        )


if __name__ == "__main__":
    unittest.main()
