from __future__ import annotations

import unittest
from datetime import UTC, datetime
from tempfile import TemporaryDirectory
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.governance import (
    GovernanceApplyResult,
    GovernanceCandidate,
    GovernancePreview,
    StateConvergenceService,
)
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


class GovernancePreviewTests(unittest.TestCase):
    def test_preview_fingerprint_is_stable_for_candidate_and_condition_order(self) -> None:
        first = GovernancePreview.create(
            "converge",
            (
                GovernanceCandidate(
                    kind="cargo_run",
                    identity="run-b",
                    action="complete",
                    reason="terminal_parent",
                    expected={"status": "running", "jobStatus": "orphaned"},
                ),
                GovernanceCandidate(
                    kind="session",
                    identity="session-a",
                    action="mark_stale",
                    reason="heartbeat_expired",
                    expected={"status": "active", "heartbeat": "2026-07-30T00:00:00+00:00"},
                ),
            ),
        )
        second = GovernancePreview.create(
            "converge",
            (
                GovernanceCandidate(
                    kind="session",
                    identity="session-a",
                    action="mark_stale",
                    reason="heartbeat_expired",
                    expected={"heartbeat": "2026-07-30T00:00:00+00:00", "status": "active"},
                ),
                GovernanceCandidate(
                    kind="cargo_run",
                    identity="run-b",
                    action="complete",
                    reason="terminal_parent",
                    expected={"jobStatus": "orphaned", "status": "running"},
                ),
            ),
        )

        self.assertEqual(first.fingerprint, second.fingerprint)
        self.assertEqual(first.candidates, second.candidates)

    def test_preview_rejects_duplicate_resource_identity(self) -> None:
        candidate = GovernanceCandidate(
            kind="session",
            identity="session-a",
            action="mark_stale",
            reason="heartbeat_expired",
            expected={"status": "active"},
        )

        with self.assertRaises(ValueError):
            GovernancePreview.create("converge", (candidate, candidate))

    def test_preview_rejects_stale_apply_fingerprint(self) -> None:
        preview = GovernancePreview.create(
            "converge",
            (
                GovernanceCandidate(
                    kind="reservation",
                    identity="reservation-a",
                    action="expire",
                    reason="owner_not_executable",
                    expected={"status": "pending", "jobId": None},
                ),
            ),
        )

        with self.assertRaises(CoordinatorError) as rejected:
            preview.require_fingerprint("different")

        self.assertEqual("governance_preview_stale", rejected.exception.code)


class StateConvergencePreviewTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.database = Database(Path(self.temporary_directory.name) / "coordinator.sqlite3")
        migrate(self.database)

    def test_preview_marks_only_an_unprotected_expired_session_stale(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, status, created_at, updated_at, last_heartbeat_at
                ) VALUES ('eligible', 'active', '2026-07-30T00:00:00+00:00',
                          '2026-07-30T00:00:00+00:00', '2026-07-30T00:00:00+00:00')
                """
            )
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, status, created_at, updated_at, last_heartbeat_at
                ) VALUES ('leased', 'active', '2026-07-30T00:00:00+00:00',
                          '2026-07-30T00:00:00+00:00', '2026-07-30T00:00:00+00:00')
                """
            )
            connection.execute(
                """
                INSERT INTO leases(
                    path_key, display_path, session_id, acquired_at, last_heartbeat_at, expires_at
                ) VALUES ('leased-path', 'leased-path', 'leased', '2026-07-30T00:00:00+00:00',
                          '2026-07-30T00:00:00+00:00', '2026-07-31T01:00:00+00:00')
                """
            )

        preview = StateConvergenceService(self.database).preview(
            now=datetime(2026, 7, 31, tzinfo=UTC), stale_after_seconds=300
        )

        self.assertEqual("converge", preview.operation)
        self.assertEqual(1, len(preview.candidates))
        candidate = preview.candidates[0]
        self.assertEqual(("session", "eligible", "mark_stale"), (candidate.kind, candidate.identity, candidate.action))
        self.assertEqual("active", candidate.expected["status"])


class StateConvergenceApplyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        self.root = Path(self.temporary_directory.name)
        self.database = Database(self.root / "state" / "coordinator.sqlite3")
        migrate(self.database)
        self.notes = self.root / ".codex" / "sessions"
        self.notes.mkdir(parents=True)

    def _insert_session(
        self, session_id: str, status: str, *, updated_at: str = "2026-07-30T00:00:00+00:00"
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, status, created_at, updated_at, last_heartbeat_at
                ) VALUES (?, ?, '2026-07-30T00:00:00+00:00', ?, '2026-07-30T00:00:00+00:00')
                """,
                (session_id, status, updated_at),
            )

    def _write_note(self, session_id: str, *, name: str = "session.md") -> Path:
        note = self.notes / name
        note.write_text(
            "---\n"
            f"session: {session_id}\n"
            "status: active\n"
            "updated_at: 2026-07-30T00:00:00+00:00\n"
            "---\n\n"
            "# Coordination Warning\n",
            encoding="utf-8",
        )
        return note

    def test_apply_uses_cas_and_audits_session_and_note_retirement(self) -> None:
        self._insert_session("stale-candidate", "active")
        self._insert_session("archive-candidate", "stale")
        self._insert_session("completed-owner", "completed")
        self._write_note("completed-owner")
        service = StateConvergenceService(self.database, self.root)
        now = datetime(2026, 7, 31, tzinfo=UTC)

        preview = service.preview(
            now=now, stale_after_seconds=300, archive_after_seconds=300
        )
        result = service.apply(preview, fingerprint=preview.fingerprint, actor="test", now=now)

        self.assertIsInstance(result, GovernanceApplyResult)
        self.assertEqual((), result.conflicts)
        self.assertEqual("stale", self._status("stale-candidate"))
        self.assertEqual("archived", self._status("archive-candidate"))
        archived_note = self.notes / "archive" / "session.md"
        self.assertTrue(archived_note.is_file())
        self.assertIn("status: archived", archived_note.read_text(encoding="utf-8"))
        with self.database.connect() as connection:
            audit = connection.execute(
                "SELECT payload_json FROM events WHERE event_type='governance.converge_applied'"
            ).fetchone()
        self.assertIsNotNone(audit)
        self.assertIn(preview.fingerprint, str(audit["payload_json"]))
        with self.database.connect() as connection:
            apply_audit = connection.execute(
                """
                SELECT candidate_count, applied_count, skipped_count, conflict_count
                FROM governance_applies WHERE fingerprint=?
                """,
                (preview.fingerprint,),
            ).fetchone()
        self.assertEqual((3, 3, 0, 0), tuple(apply_audit))

    def test_apply_reports_a_heartbeat_race_without_transitioning_the_session(self) -> None:
        self._insert_session("racing-session", "active")
        service = StateConvergenceService(self.database, self.root)
        now = datetime(2026, 7, 31, tzinfo=UTC)
        preview = service.preview(now=now, stale_after_seconds=300)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at='2026-07-31T00:00:00+00:00' WHERE session_id='racing-session'"
            )

        result = service.apply(preview, fingerprint=preview.fingerprint, actor="test", now=now)

        self.assertEqual("active", self._status("racing-session"))
        self.assertEqual(("session:racing-session",), result.conflicts)

    def test_recorded_preview_can_be_loaded_without_trusting_client_candidates(self) -> None:
        self._insert_session("stale-candidate", "active")
        service = StateConvergenceService(self.database, self.root)
        now = datetime(2026, 7, 31, tzinfo=UTC)
        preview = service.preview(now=now, stale_after_seconds=300)

        service.record_preview(preview, actor="test", now=now)
        restored = service.load_preview(preview.fingerprint)

        self.assertEqual(preview, restored)
        with self.database.connect() as connection:
            persisted = connection.execute(
                """
                SELECT operation, candidate_count, actor FROM governance_previews
                WHERE fingerprint=?
                """,
                (preview.fingerprint,),
            ).fetchone()
        self.assertEqual(("converge", 1, "test"), tuple(persisted))

    def test_apply_preserves_a_note_changed_after_preview(self) -> None:
        self._insert_session("completed-owner", "completed")
        note = self._write_note("completed-owner")
        service = StateConvergenceService(self.database, self.root)
        now = datetime(2026, 7, 31, tzinfo=UTC)
        preview = service.preview(now=now)
        note.write_text(note.read_text(encoding="utf-8") + "\nchanged after preview\n", encoding="utf-8")

        result = service.apply(preview, fingerprint=preview.fingerprint, actor="test", now=now)

        self.assertEqual(("session_note:.codex/sessions/session.md",), result.conflicts)
        self.assertTrue(note.is_file())
        self.assertFalse((self.notes / "archive" / "session.md").exists())

    def _status(self, session_id: str) -> str:
        with self.database.connect() as connection:
            return str(
                connection.execute("SELECT status FROM sessions WHERE session_id=?", (session_id,)).fetchone()["status"]
            )


if __name__ == "__main__":
    unittest.main()
