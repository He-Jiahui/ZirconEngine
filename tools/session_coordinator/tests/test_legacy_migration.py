from __future__ import annotations

import hashlib
import json
import os
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest import mock

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.legacy import LegacyMigrationService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


NOW = datetime(2026, 7, 11, 5, 0, tzinfo=UTC)


class LegacyMigrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.session_root = self.repo / ".codex/sessions"
        self.session_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.service = LegacyMigrationService(
            self.database,
            self.repo,
            self.sessions,
            process_alive=lambda pid: pid == 4242,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _note(
        self,
        name: str,
        *,
        status: str,
        age: timedelta,
        pid: int | None = None,
        plan: str = "docs/plans/runtime/01-runtime.md",
    ) -> Path:
        path = self.session_root / name
        pid_line = f"pid: {pid}\n" if pid is not None else ""
        path.write_text(
            "---\n"
            f"session: {path.stem}\n"
            f"status: {status}\n"
            f"{pid_line}"
            "related_plans:\n"
            f"  - {plan}\n"
            "---\n\n# Legacy note\n",
            encoding="utf-8",
        )
        timestamp = (NOW - age).timestamp()
        path.touch()
        import os

        os.utime(path, (timestamp, timestamp))
        return path

    def test_report_is_deterministic_and_never_mutates_sources(self) -> None:
        old = self._note("old-session.md", status="blocked", age=timedelta(hours=30))
        recent = self._note("recent-session.md", status="active", age=timedelta(minutes=5))
        before = {path.name: self._sha(path) for path in (old, recent)}

        first = self.service.report(now=NOW).to_dict()
        second = self.service.report(now=NOW).to_dict()

        self.assertEqual(first, second)
        self.assertEqual(before, {path.name: self._sha(path) for path in (old, recent)})
        by_id = {item["session_id"]: item for item in first["notes"]}
        self.assertEqual("stale", by_id["old-session"]["mapped_status"])
        self.assertEqual("blocked", by_id["old-session"]["status_reason"])
        self.assertTrue(by_id["old-session"]["archive_eligible"])
        self.assertEqual("active", by_id["recent-session"]["mapped_status"])

    def test_live_pid_keeps_old_note_active(self) -> None:
        self._note(
            "live-session.md", status="completed", age=timedelta(days=3), pid=4242
        )

        report = self.service.report(now=NOW)

        self.assertEqual(SessionStatus.ACTIVE, report.notes[0].mapped_status)
        self.assertFalse(report.notes[0].archive_eligible)
        self.assertIn("live_pid", report.notes[0].activity_reasons)

        self.service.import_notes(now=NOW)
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("live-session").status)

    def test_apply_imports_idempotently_without_moving_note(self) -> None:
        note = self._note(
            "import-session.md", status="mystery-state", age=timedelta(hours=2)
        )

        first = self.service.import_notes(now=NOW)
        second = self.service.import_notes(now=NOW)

        self.assertEqual(first.to_dict(), second.to_dict())
        imported = self.sessions.get("import-session")
        self.assertEqual(SessionStatus.STALE, imported.status)
        self.assertEqual("mystery-state", imported.status_reason)
        self.assertEqual("docs/plans/runtime/01-runtime.md", imported.plan_path)
        self.assertTrue(note.exists())
        with self.database.connect() as connection:
            count = connection.execute(
                "SELECT COUNT(*) FROM legacy_note_imports WHERE session_id = ?",
                ("import-session",),
            ).fetchone()[0]
        self.assertEqual(1, count)

    def test_reimport_reactivation_clears_obsolete_terminal_timestamps(self) -> None:
        note = self._note(
            "reactivate.md", status="completed", age=timedelta(hours=3)
        )
        self.service.import_notes(now=NOW)
        text = note.read_text(encoding="utf-8").replace(
            "status: completed", "status: working\npid: 4242"
        )
        note.write_text(text, encoding="utf-8")
        timestamp = (NOW - timedelta(hours=2)).timestamp()
        os.utime(note, (timestamp, timestamp))

        self.service.import_notes(now=NOW)

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, completed_at, archived_at FROM sessions WHERE session_id = 'reactivate'"
            ).fetchone()
        self.assertEqual("active", row["status"])
        self.assertIsNone(row["completed_at"])
        self.assertIsNone(row["archived_at"])

    def test_archive_moves_only_eligible_notes_and_preserves_hash_manifest(self) -> None:
        old = self._note("archive-me.md", status="stale", age=timedelta(days=2))
        recent = self._note("keep-me.md", status="active", age=timedelta(minutes=2))
        old_hash = self._sha(old)
        self.service.import_notes(now=NOW)

        preview = self.service.archive_notes(now=NOW, apply=False)
        self.assertTrue(old.exists())
        self.assertEqual((".codex/sessions/archive-me.md",), preview.candidates)

        applied = self.service.archive_notes(now=NOW, apply=True)

        archived = self.session_root / "archive/archive-me.md"
        self.assertFalse(old.exists())
        self.assertTrue(recent.exists())
        self.assertTrue(archived.exists())
        self.assertEqual(old_hash, self._sha(archived))
        entry = applied.manifest[0]
        self.assertEqual(old_hash, entry.before_hash)
        self.assertEqual(old_hash, entry.after_hash)
        self.assertEqual(SessionStatus.ARCHIVED, self.sessions.get("archive-me").status)

    def test_archive_rechecks_service_activity_before_moving(self) -> None:
        source = self._note("race.md", status="stale", age=timedelta(days=2))
        self.service.import_notes(now=NOW)
        original_destination = self.service._archive_destination

        def activate_then_choose(*args, **kwargs):
            self.sessions.heartbeat("race")
            return original_destination(*args, **kwargs)

        with mock.patch.object(
            self.service, "_archive_destination", side_effect=activate_then_choose
        ):
            with self.assertRaises(Exception):
                self.service.archive_notes(now=NOW, apply=True)

        self.assertTrue(source.exists())
        self.assertFalse((self.session_root / "archive/race.md").exists())

    def test_archive_rollback_never_consumes_preexisting_same_hash_destination(self) -> None:
        first = self._note("first.md", status="stale", age=timedelta(days=2))
        second = self._note("second.md", status="stale", age=timedelta(days=2))
        self.service.import_notes(now=NOW)
        archive_root = self.session_root / "archive"
        archive_root.mkdir()
        preexisting = archive_root / "first.md"
        preexisting.write_bytes(first.read_bytes())
        real_replace = os.replace

        def fail_second(source, destination):
            if Path(source).name == "second.md":
                raise OSError("injected second move failure")
            return real_replace(source, destination)

        with mock.patch("tools.session_coordinator.legacy.os.replace", fail_second):
            with self.assertRaises(OSError):
                self.service.archive_notes(now=NOW, apply=True)

        self.assertTrue(first.exists())
        self.assertTrue(second.exists())
        self.assertTrue(preexisting.exists())
        self.assertEqual(self._sha(first), self._sha(preexisting))

    def test_startup_recovers_durable_precommit_archive_intent(self) -> None:
        source = self._note("crash.md", status="stale", age=timedelta(days=2))
        before_hash = self._sha(source)
        destination = self.session_root / "archive/crash.md"
        destination.parent.mkdir()
        run_id = "legacy-archive-crash-fixture"
        entry = {
            "source_path": ".codex/sessions/crash.md",
            "destination_path": ".codex/sessions/archive/crash.md",
            "session_id": "crash",
            "before_hash": before_hash,
            "after_hash": before_hash,
        }
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO legacy_archive_runs(
                    run_id, candidates_json, manifest_json, status, created_at
                ) VALUES (?, ?, ?, 'planned', ?)
                """,
                (
                    run_id,
                    json.dumps([entry["source_path"]]),
                    json.dumps([entry]),
                    NOW.isoformat(),
                ),
            )
        os.replace(source, destination)

        recovered = self.service.recover_interrupted_archives()

        self.assertEqual((run_id,), recovered)
        self.assertTrue(source.exists())
        self.assertFalse(destination.exists())
        self.assertEqual(before_hash, self._sha(source))

    @staticmethod
    def _sha(path: Path) -> str:
        return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    unittest.main()
