from __future__ import annotations

import json
import tempfile
import threading
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path
from unittest import mock

from tools.session_coordinator.codex_sync.evidence import CodexEvidenceProjector
from tools.session_coordinator.codex_sync.history import CodexHistoricalEvidenceCollector
from tools.session_coordinator.codex_sync.models import (
    CodexDiscoveryResult,
    CodexReconcileResult,
)
from tools.session_coordinator.codex_sync.worker import CodexSyncWorker
from tools.session_coordinator import migrations
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CodexEvidenceProjectionTests(unittest.TestCase):
    def test_evidence_source_cursor_defaults_to_incomplete_without_deleting_records(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "state.sqlite3")
            with mock.patch.object(migrations, "LATEST_SCHEMA_VERSION", 39):
                migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO codex_evidence_sources(
                           source_id, thread_id, rollout_name, source_mtime_ns,
                           source_size, indexed_at
                       ) VALUES ('source-a', 'thread-a', 'rollout-a.jsonl', 1, 2, ? )""",
                    ("2026-07-15T08:00:00+00:00",),
                )
                connection.execute(
                    """INSERT INTO codex_evidence_records(
                           source_id, thread_id, rollout_name, event_key_hash, kind,
                           outcome, exit_code, event_at, recorded_at
                       ) VALUES (
                           'source-a', 'thread-a', 'rollout-a.jsonl', 'event-a',
                           'task', 'succeeded', NULL, ?, ?
                       )""",
                    ("2026-07-15T08:00:00+00:00", "2026-07-15T08:01:00+00:00"),
                )

            migrate(database)

            with database.connect() as connection:
                source = connection.execute(
                    """SELECT scan_offset, prefix_hash, pending_calls_json,
                              scan_complete, scan_revision
                       FROM codex_evidence_sources WHERE source_id='source-a'"""
                ).fetchone()
                record_count = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records WHERE source_id='source-a'"
                ).fetchone()[0]
            self.assertEqual((0, "", "{}", 0, 1), tuple(source))
            self.assertEqual(1, record_count)

    def test_streams_events_before_and_after_the_old_tail_window(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-stream.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            metadata = {
                "type": "session_meta",
                "timestamp": "2026-07-15T08:00:00Z",
                "payload": {"session_id": "thread-stream", "cwd": str(repo)},
            }
            first_task = {
                "type": "event_msg",
                "timestamp": "2026-07-15T08:00:01Z",
                "payload": {"type": "task_completed", "turn_id": "turn-first"},
            }
            filler = {
                "type": "event_msg",
                "timestamp": "2026-07-15T08:00:02Z",
                "payload": {"type": "ignored", "padding": "x" * 1024},
            }
            last_task = {
                "type": "event_msg",
                "timestamp": "2026-07-15T08:00:03Z",
                "payload": {"type": "task_completed", "turn_id": "turn-last"},
            }
            rollout.write_text(
                "\n".join(
                    json.dumps(value)
                    for value in (metadata, first_task, *((filler,) * 300), last_task)
                )
                + "\n",
                encoding="utf-8",
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            generated_at = datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc)

            for _ in range(20):
                collector.collect_month(generated_at, byte_budget=32 * 1024)
                with database.connect() as connection:
                    complete = connection.execute(
                        "SELECT scan_complete FROM codex_evidence_sources"
                    ).fetchone()[0]
                if complete:
                    break

            with database.connect() as connection:
                records = connection.execute(
                    """SELECT event_at, kind, outcome FROM codex_evidence_records
                       WHERE thread_id='thread-stream' ORDER BY event_at"""
                ).fetchall()
                source = connection.execute(
                    "SELECT scan_complete, scan_offset, source_size FROM codex_evidence_sources"
                ).fetchone()
            self.assertEqual(
                [
                    ("2026-07-15T08:00:01+00:00", "task", "succeeded"),
                    ("2026-07-15T08:00:03+00:00", "task", "succeeded"),
                ],
                [tuple(row) for row in records],
            )
            self.assertEqual((1, source[2], source[2]), tuple(source))

    def test_finalizes_an_exact_budget_eof_cursor_without_rescanning(self) -> None:
        """A budget boundary at EOF needs one zero-byte completion pass."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-eof.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            rollout.write_text(
                "\n".join(
                    (
                        json.dumps(
                            {
                                "type": "session_meta",
                                "timestamp": "2026-07-15T08:00:00Z",
                                "payload": {"session_id": "thread-eof", "cwd": str(repo)},
                            }
                        ),
                        json.dumps(
                            {
                                "type": "event_msg",
                                "timestamp": "2026-07-15T08:00:01Z",
                                "payload": {
                                    "type": "task_completed",
                                    "turn_id": "turn-eof",
                                },
                            }
                        ),
                    )
                )
                + "\n",
                encoding="utf-8",
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            generated_at = datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc)

            collector.collect_month(generated_at, byte_budget=rollout.stat().st_size)
            with database.connect() as connection:
                first = connection.execute(
                    "SELECT scan_offset, source_size, scan_complete FROM codex_evidence_sources"
                ).fetchone()
                records_before = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records"
                ).fetchone()[0]
            self.assertEqual((first[1], first[1], 0), tuple(first))
            self.assertEqual(1, records_before)

            collector.collect_month(generated_at, byte_budget=64 * 1024)
            collector.collect_month(generated_at, byte_budget=64 * 1024)
            with database.connect() as connection:
                final = connection.execute(
                    "SELECT scan_offset, source_size, scan_complete FROM codex_evidence_sources"
                ).fetchone()
                records_after = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records"
                ).fetchone()[0]
            self.assertEqual((final[1], final[1], 1), tuple(final))
            self.assertEqual(1, records_after)

    def test_large_source_cannot_consume_the_entire_incremental_budget(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollouts = root / "codex-home/sessions/2026/07/15"
            rollouts.mkdir(parents=True)

            def metadata(thread_id: str) -> str:
                return json.dumps(
                    {
                        "type": "session_meta",
                        "timestamp": "2026-07-15T08:00:00Z",
                        "payload": {"session_id": thread_id, "cwd": str(repo)},
                    }
                )

            large = rollouts / "rollout-2026-07-15T08-00-00-thread-a.jsonl"
            large.write_text(
                metadata("thread-a")
                + "\n"
                + (json.dumps({"type": "ignored", "padding": "x" * 1024}) + "\n") * 700,
                encoding="utf-8",
            )
            small = rollouts / "rollout-2026-07-15T08-00-00-thread-b.jsonl"
            small.write_text(
                metadata("thread-b")
                + "\n"
                + json.dumps(
                    {
                        "type": "event_msg",
                        "timestamp": "2026-07-15T08:00:01Z",
                        "payload": {"type": "task_completed", "turn_id": "turn-b"},
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )

            collector.collect_month(
                datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
                byte_budget=600 * 1024,
            )

            with database.connect() as connection:
                event = connection.execute(
                    "SELECT kind, outcome FROM codex_evidence_records WHERE thread_id='thread-b'"
                ).fetchone()
            self.assertEqual(("task", "succeeded"), tuple(event))

    def test_incremental_budget_rotates_to_unseen_sources_on_the_next_cycle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollouts = root / "codex-home/sessions/2026/07/15"
            rollouts.mkdir(parents=True)

            def metadata(thread_id: str) -> str:
                return json.dumps(
                    {
                        "type": "session_meta",
                        "timestamp": "2026-07-15T08:00:00Z",
                        "payload": {"session_id": thread_id, "cwd": str(repo)},
                    }
                )

            for index in range(16):
                (rollouts / f"rollout-2026-07-15T08-00-00-thread-{index:02}.jsonl").write_text(
                    metadata(f"thread-{index:02}")
                    + "\n"
                    + (json.dumps({"type": "ignored", "padding": "x" * 1024}) + "\n")
                    * 1_500,
                    encoding="utf-8",
                )
            delayed = rollouts / "rollout-2026-07-15T08-00-00-thread-99.jsonl"
            delayed.write_text(
                metadata("thread-99")
                + "\n"
                + json.dumps(
                    {
                        "type": "event_msg",
                        "timestamp": "2026-07-15T08:00:01Z",
                        "payload": {"type": "task_completed", "turn_id": "turn-99"},
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )

            collector.collect_month(
                datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
                byte_budget=8 * 1024 * 1024,
            )
            with database.connect() as connection:
                first_cycle = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records WHERE thread_id='thread-99'"
                ).fetchone()[0]
            self.assertEqual(0, first_cycle)

            collector.collect_month(
                datetime(2026, 7, 15, 8, 2, tzinfo=timezone.utc),
                byte_budget=8 * 1024 * 1024,
            )
            with database.connect() as connection:
                event = connection.execute(
                    "SELECT kind, outcome FROM codex_evidence_records WHERE thread_id='thread-99'"
                ).fetchone()
            self.assertEqual(("task", "succeeded"), tuple(event))

    def test_changed_completed_source_precedes_historical_backfill(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollouts = root / "codex-home/sessions/2026/07/15"
            rollouts.mkdir(parents=True)

            def metadata(thread_id: str) -> str:
                return json.dumps(
                    {
                        "type": "session_meta",
                        "timestamp": "2026-07-15T08:00:00Z",
                        "payload": {"session_id": thread_id, "cwd": str(repo)},
                    }
                )

            active = rollouts / "rollout-2026-07-15T08-00-00-thread-99.jsonl"
            active.write_text(metadata("thread-99") + "\n", encoding="utf-8")
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            collector.collect_month(
                datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
                byte_budget=8 * 1024 * 1024,
            )

            for index in range(16):
                (rollouts / f"rollout-2026-07-15T08-00-00-thread-{index:02}.jsonl").write_text(
                    metadata(f"thread-{index:02}")
                    + "\n"
                    + (json.dumps({"type": "ignored", "padding": "x" * 1024}) + "\n")
                    * 1_500,
                    encoding="utf-8",
                )
            with active.open("a", encoding="utf-8") as handle:
                handle.write(
                    json.dumps(
                        {
                            "type": "event_msg",
                            "timestamp": "2026-07-15T08:00:02Z",
                            "payload": {"type": "task_completed", "turn_id": "turn-active"},
                        }
                    )
                    + "\n"
                )

            collector.collect_month(
                datetime(2026, 7, 15, 8, 2, tzinfo=timezone.utc),
                byte_budget=8 * 1024 * 1024,
            )
            with database.connect() as connection:
                event = connection.execute(
                    "SELECT kind, outcome FROM codex_evidence_records WHERE thread_id='thread-99'"
                ).fetchone()
            self.assertEqual(("task", "succeeded"), tuple(event))

    def test_classifies_output_after_a_pending_call_crosses_scan_windows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-call.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            metadata = {
                "type": "session_meta",
                "timestamp": "2026-07-15T08:00:00Z",
                "payload": {"session_id": "thread-call", "cwd": str(repo)},
            }
            call = {
                "type": "response_item",
                "timestamp": "2026-07-15T08:00:01Z",
                "payload": {
                    "type": "custom_tool_call",
                    "call_id": "call-validation",
                    "name": "exec",
                    "input": "cargo test PRIVATE_COMMAND_TEXT",
                },
            }
            rollout.write_text(
                "\n".join(json.dumps(value) for value in (metadata, call)) + "\n",
                encoding="utf-8",
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            generated_at = datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc)

            collector.collect_month(generated_at, byte_budget=64 * 1024)
            with database.connect() as connection:
                pending_before = connection.execute(
                    "SELECT pending_calls_json FROM codex_evidence_sources"
                ).fetchone()[0]
            self.assertNotIn("PRIVATE_COMMAND_TEXT", pending_before)

            output = {
                "type": "response_item",
                "timestamp": "2026-07-15T08:00:02Z",
                "payload": {
                    "type": "custom_tool_call_output",
                    "call_id": "call-validation",
                    "output": "Exit code: 0 PRIVATE_OUTPUT_TEXT",
                },
            }
            with rollout.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(output) + "\n")
            collector.collect_month(generated_at, byte_budget=64 * 1024)

            with database.connect() as connection:
                record = connection.execute(
                    """SELECT kind, outcome, exit_code FROM codex_evidence_records
                       WHERE thread_id='thread-call'"""
                ).fetchone()
                pending_after = connection.execute(
                    "SELECT pending_calls_json FROM codex_evidence_sources"
                ).fetchone()[0]
            self.assertEqual(("validation", "succeeded", 0), tuple(record))
            self.assertEqual("{}", pending_after)

    def test_retries_a_partial_jsonl_line_after_it_is_completed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-partial.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            metadata = {
                "type": "session_meta",
                "timestamp": "2026-07-15T08:00:00Z",
                "payload": {"session_id": "thread-partial", "cwd": str(repo)},
            }
            task = {
                "type": "event_msg",
                "timestamp": "2026-07-15T08:00:01Z",
                "payload": {"type": "task_completed", "turn_id": "turn-partial"},
            }
            task_line = json.dumps(task)
            split_at = len(task_line) // 2
            rollout.write_text(
                json.dumps(metadata) + "\n" + task_line[:split_at], encoding="utf-8"
            )
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            generated_at = datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc)

            collector.collect_month(generated_at, byte_budget=64 * 1024)
            with database.connect() as connection:
                source_before = connection.execute(
                    "SELECT scan_offset, scan_complete FROM codex_evidence_sources"
                ).fetchone()
                record_count_before = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records"
                ).fetchone()[0]
            self.assertLess(source_before[0], rollout.stat().st_size)
            self.assertEqual(0, source_before[1])
            self.assertEqual(0, record_count_before)

            with rollout.open("a", encoding="utf-8") as handle:
                handle.write(task_line[split_at:] + "\n")
            collector.collect_month(generated_at, byte_budget=64 * 1024)
            with database.connect() as connection:
                source_after = connection.execute(
                    "SELECT scan_offset, source_size, scan_complete FROM codex_evidence_sources"
                ).fetchone()
                records = connection.execute(
                    "SELECT kind, outcome FROM codex_evidence_records"
                ).fetchall()
            self.assertEqual((source_after[1], source_after[1], 1), tuple(source_after))
            self.assertEqual([("task", "succeeded")], [tuple(row) for row in records])

    def test_source_replacement_uses_a_new_revision_without_deleting_old_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-revision.jsonl"
            )
            rollout.parent.mkdir(parents=True)

            def source(timestamp: str, turn_id: str, padding: str) -> str:
                return "\n".join(
                    (
                        json.dumps(
                            {
                                "type": "session_meta",
                                "timestamp": timestamp,
                                "payload": {
                                    "session_id": "thread-revision",
                                    "cwd": str(repo),
                                    "padding": padding,
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "type": "event_msg",
                                "timestamp": timestamp,
                                "payload": {"type": "task_completed", "turn_id": turn_id},
                            }
                        ),
                    )
                ) + "\n"

            rollout.write_text(source("2026-07-15T08:00:01Z", "turn-old", "old-padding"), encoding="utf-8")
            collector = CodexHistoricalEvidenceCollector(
                database, codex_home=root / "codex-home", repo_root=repo
            )
            generated_at = datetime(2026, 7, 15, 8, 2, tzinfo=timezone.utc)
            collector.collect_month(generated_at, byte_budget=64 * 1024)

            rollout.write_text(source("2026-07-15T08:00:02Z", "turn-new", "n"), encoding="utf-8")
            collector.collect_month(generated_at, byte_budget=64 * 1024)
            with database.connect() as connection:
                rows = connection.execute(
                    """SELECT event_at, event_key_hash FROM codex_evidence_records
                       WHERE thread_id='thread-revision' ORDER BY event_at"""
                ).fetchall()
                source_state = connection.execute(
                    "SELECT scan_revision, scan_complete FROM codex_evidence_sources"
                ).fetchone()
            self.assertEqual(
                ["2026-07-15T08:00:01+00:00", "2026-07-15T08:00:02+00:00"],
                [row["event_at"] for row in rows],
            )
            self.assertNotEqual(rows[0]["event_key_hash"], rows[1]["event_key_hash"])
            self.assertEqual((2, 1), tuple(source_state))

    def test_does_not_replace_existing_history_with_an_empty_collection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            history = (
                root
                / "codex-home/sessions/2026/07/zircon-engine-evidence-history-2026-07.md"
            )
            history.parent.mkdir(parents=True)
            history.write_text("# Existing verified evidence\n", encoding="utf-8")

            CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-empty", include_history=True)

            self.assertEqual("# Existing verified evidence\n", history.read_text(encoding="utf-8"))

    def test_projects_aggregate_history_backfill_progress_without_source_details(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.executemany(
                    """INSERT INTO codex_evidence_sources(
                           source_id, thread_id, rollout_name, source_mtime_ns,
                           source_size, indexed_at, scan_offset, prefix_hash,
                           pending_calls_json, scan_complete, scan_revision
                       ) VALUES (?, ?, ?, 1, ?, ?, ?, '', '{}', ?, 1)""",
                    (
                        (
                            "source-complete",
                            "thread-one",
                            "private-source-one.jsonl",
                            100,
                            "2026-07-15T08:00:00+00:00",
                            100,
                            1,
                        ),
                        (
                            "source-pending",
                            "thread-two",
                            "private-source-two.jsonl",
                            200,
                            "2026-07-15T08:00:00+00:00",
                            50,
                            0,
                        ),
                    ),
                )
                connection.executemany(
                    """INSERT INTO codex_evidence_records(
                           source_id, thread_id, rollout_name, event_key_hash, kind,
                           outcome, exit_code, event_at, recorded_at
                       ) VALUES (?, 'thread-one', 'private-source-one.jsonl', ?,
                                 'task', 'succeeded', NULL, ?, ?)""",
                    (
                        (
                            "source-complete",
                            "event-one",
                            "2026-07-15T07:59:00+00:00",
                            "2026-07-15T08:00:00+00:00",
                        ),
                        (
                            "source-complete",
                            "event-two",
                            "2026-07-15T07:59:01+00:00",
                            "2026-07-15T08:00:00+00:00",
                        ),
                        (
                            "source-pending",
                            "event-three",
                            "2026-07-15T07:59:02+00:00",
                            "2026-07-15T08:00:00+00:00",
                        ),
                    ),
                )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-progress")

            text = output.read_text(encoding="utf-8")
            self.assertIn("## 历史回填进度", text)
            self.assertIn("| 2 | 1 | 3 |", text)
            progress = text.split("## 历史回填进度", maxsplit=1)[1].split("##", maxsplit=1)[0]
            self.assertNotIn("private-source-one.jsonl", progress)
            self.assertNotIn("private-source-two.jsonl", progress)
            self.assertNotIn("scan_offset", progress)

    def test_projects_incremental_sanitized_history_into_the_month_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-history.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            rollout.write_text(
                "\n".join(
                    (
                        json.dumps(
                            {
                                "type": "session_meta",
                                "timestamp": "2026-07-15T08:00:00Z",
                                "payload": {
                                    "session_id": "thread-history",
                                    "cwd": str(repo),
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "type": "response_item",
                                "timestamp": "2026-07-15T08:00:01Z",
                                "payload": {
                                    "type": "custom_tool_call",
                                    "call_id": "call-validation",
                                    "name": "exec",
                                    "input": "cargo test TOP_SECRET_SHOULD_NOT_BE_STORED",
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "type": "response_item",
                                "timestamp": "2026-07-15T08:00:02Z",
                                "payload": {
                                    "type": "custom_tool_call_output",
                                    "call_id": "call-validation",
                                    "output": "Exit code: 0",
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "type": "event_msg",
                                "timestamp": "2026-07-15T08:00:03Z",
                                "payload": {"type": "task_completed", "turn_id": "turn-history"},
                            }
                        ),
                    )
                )
                + "\n",
                encoding="utf-8",
            )

            projector = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
            )
            output = projector.project(run_id="sync-history", include_history=True)
            history = root / "codex-home/sessions/2026/07/zircon-engine-evidence-history-2026-07.md"

            self.assertTrue(history.exists())
            self.assertIn("thread-history", history.read_text(encoding="utf-8"))
            live = output.read_text(encoding="utf-8")
            self.assertIn("## 最近外部会话证据", live)
            self.assertIn("thread-history", live)
            self.assertNotIn("TOP_SECRET_SHOULD_NOT_BE_STORED", live)
            self.assertNotIn(str(repo), live)
            with database.connect() as connection:
                records = connection.execute(
                    "SELECT kind, outcome, exit_code FROM codex_evidence_records "
                    "WHERE thread_id='thread-history' ORDER BY kind"
                ).fetchall()
            self.assertEqual(
                [("task", "succeeded", None), ("validation", "succeeded", 0)],
                [tuple(row) for row in records],
            )

            projector.project(run_id="sync-history-repeat", include_history=True)
            with database.connect() as connection:
                count = connection.execute(
                    "SELECT COUNT(*) FROM codex_evidence_records WHERE thread_id='thread-history'"
                ).fetchone()[0]
            self.assertEqual(2, count)

    def test_live_projection_advances_cursor_without_rewriting_month_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = (
                root
                / "codex-home/sessions/2026/07/15"
                / "rollout-2026-07-15T08-00-00-thread-live-cursor.jsonl"
            )
            rollout.parent.mkdir(parents=True)
            rollout.write_text(
                "\n".join(
                    (
                        json.dumps(
                            {
                                "type": "session_meta",
                                "timestamp": "2026-07-15T08:00:00Z",
                                "payload": {"session_id": "thread-live-cursor", "cwd": str(repo)},
                            }
                        ),
                        json.dumps(
                            {
                                "type": "event_msg",
                                "timestamp": "2026-07-15T08:00:01Z",
                                "payload": {
                                    "type": "task_completed",
                                    "turn_id": "turn-live-cursor",
                                },
                            }
                        ),
                    )
                )
                + "\n",
                encoding="utf-8",
            )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
            ).project(run_id="sync-live-cursor")

            self.assertIn("thread-live-cursor", output.read_text(encoding="utf-8"))
            history = root / "codex-home/sessions/2026/07/zircon-engine-evidence-history-2026-07.md"
            self.assertFalse(history.exists())

    def test_projects_sanitized_live_session_evidence_into_codex_month_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(
                session_id="session-a", plan_path="docs/plans/runtime/01-feature.md"
            )
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO codex_sessions(
                           thread_id, rollout_path, source_location, state, cwd,
                           last_event, first_seen_at, last_activity_at, last_synced_at,
                           source_mtime_ns, source_size, missing_scan_count
                       ) VALUES (
                           'thread-a', 'C:\\private\\secret\\rollout.jsonl',
                           'active', 'active', 'E:\\private\\workspace',
                           'task_completed', '2026-07-15T00:00:00+00:00',
                           '2026-07-15T07:59:00+00:00', '2026-07-15T07:59:01+00:00',
                           1, 2, 0
                       )"""
                )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-a")

            self.assertEqual(
                root / "codex-home/sessions/2026/07/zircon-engine-evidence-live-2026-07-15.md",
                output,
            )
            text = output.read_text(encoding="utf-8")
            self.assertIn("thread-a", text)
            self.assertIn("session-a", text)
            self.assertIn("docs/plans/runtime/01-feature.md", text)
            self.assertIn("rollout.jsonl", text)
            self.assertNotIn("C:\\private", text)
            self.assertNotIn("E:\\private", text)
            self.assertNotIn("secret", text)

    def test_uses_local_calendar_date_for_live_evidence_filename(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 18, 30, tzinfo=timezone.utc),
                local_timezone=timezone(timedelta(hours=8)),
            ).project(run_id="sync-local-date")

            self.assertEqual(
                root / "codex-home/sessions/2026/07/zircon-engine-evidence-live-2026-07-16.md",
                output,
            )

    def test_history_projection_redacts_webhook_like_tool_text(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            rollout = root / "codex-home/sessions/2026/07/15/rollout-2026-07-15T08-00-00-thread-webhook.jsonl"
            rollout.parent.mkdir(parents=True)
            webhook = "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=NOT_A_REAL_KEY"
            rollout.write_text(
                "\n".join(
                    (
                        json.dumps(
                            {
                                "type": "session_meta",
                                "timestamp": "2026-07-15T08:00:00Z",
                                "payload": {"session_id": "thread-webhook", "cwd": str(repo)},
                            }
                        ),
                        json.dumps(
                            {
                                "type": "response_item",
                                "timestamp": "2026-07-15T08:00:01Z",
                                "payload": {
                                    "type": "custom_tool_call",
                                    "call_id": "call-webhook",
                                    "name": "exec",
                                    "input": f"cargo test {webhook}",
                                },
                            }
                        ),
                        json.dumps(
                            {
                                "type": "response_item",
                                "timestamp": "2026-07-15T08:00:02Z",
                                "payload": {
                                    "type": "custom_tool_call_output",
                                    "call_id": "call-webhook",
                                    "output": f"Exit code: 0 {webhook}",
                                },
                            }
                        ),
                    )
                )
                + "\n",
                encoding="utf-8",
            )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 1, tzinfo=timezone.utc),
            ).project(run_id="sync-webhook", include_history=True)

            rendered = output.read_text(encoding="utf-8")
            self.assertIn("thread-webhook", rendered)
            self.assertNotIn(webhook, rendered)
            with database.connect() as connection:
                stored = connection.execute(
                    "SELECT event_key_hash, kind, outcome, exit_code FROM codex_evidence_records"
                ).fetchone()
            self.assertEqual(("validation", "succeeded", 0), tuple(stored)[1:])
            self.assertNotIn(webhook, " ".join(str(value) for value in stored))

    def test_projects_only_current_state_and_open_failure_chain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(
                session_id="session-current",
                plan_path="docs/plans/runtime/01-feature.md",
            )
            sessions.register(
                session_id="session-stale",
                plan_path="docs/plans/runtime/02-stale.md",
            )
            with database.transaction() as connection:
                connection.execute(
                    "UPDATE sessions SET status='active', last_heartbeat_at=? "
                    "WHERE session_id='session-current'",
                    ("2026-07-15T07:59:00+00:00",),
                )
                connection.execute(
                    "UPDATE sessions SET status='active', last_heartbeat_at=? "
                    "WHERE session_id='session-stale'",
                    ("2026-07-15T01:00:00+00:00",),
                )
                connection.executemany(
                    """INSERT INTO codex_sessions(
                           thread_id, rollout_path, source_location, state, cwd,
                           last_event, first_seen_at, last_activity_at, last_synced_at,
                           source_mtime_ns, source_size, missing_scan_count
                       ) VALUES (?, ?, 'active', 'idle', 'E:\\private\\workspace',
                                 'task_completed', ?, ?, ?, 1, 2, 0)""",
                    (
                        (
                            "thread-current",
                            "C:\\private\\current.jsonl",
                            "2026-07-15T07:00:00+00:00",
                            "2026-07-15T07:59:00+00:00",
                            "2026-07-15T07:59:00+00:00",
                        ),
                        (
                            "thread-stale",
                            "C:\\private\\stale.jsonl",
                            "2026-07-15T01:00:00+00:00",
                            "2026-07-15T01:00:00+00:00",
                            "2026-07-15T01:00:00+00:00",
                        ),
                    ),
                )
                connection.execute(
                    """INSERT INTO failure_nodes(
                           lifecycle_key, artifact_path, kind, status, created_at,
                           resolved_at, summary_slug, origin_plan, fixing_plan,
                           origin_child_dir, fixing_child_dir, priority, imported_at
                       ) VALUES (
                           'origin|fixer|current-session-contract',
                           'docs/plans/runtime/02/failure-2026-07-15-current-session-contract.md',
                           'failure', 'open', '2026-07-15', NULL,
                           'current-session-contract',
                           'docs/plans/runtime/01-origin.md',
                           'docs/plans/runtime/02-fixer.md',
                           'docs/plans/runtime/01', 'docs/plans/runtime/02', 10,
                           '2026-07-15T07:59:00+00:00'
                       )"""
                )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-current")

            text = output.read_text(encoding="utf-8")
            self.assertIn("thread-current", text)
            self.assertNotIn("thread-stale", text)
            self.assertIn("session-current", text)
            self.assertNotIn("session-stale", text)
            self.assertIn("## 开放 Failure", text)
            self.assertIn("current-session-contract", text)

    def test_projects_recent_terminal_cpu_reservation_without_private_payload(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(
                session_id="session-current",
                plan_path="docs/plans/runtime/01-feature.md",
            )
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO cargo_lane_reservations(
                           reservation_id, session_id, lane_scope, compatibility_key,
                           command_fingerprint, job_id, status, created_at, expires_at,
                           started_at, completed_at
                       ) VALUES (?, ?, 'cpu', ?, ?, NULL, 'expired', ?, ?, NULL, ?)""",
                    (
                        "reservation-expired",
                        "session-current",
                        "private-compatibility-key",
                        "private-command-fingerprint",
                        "2026-07-15T07:50:00+00:00",
                        "2026-07-15T07:55:00+00:00",
                        "2026-07-15T07:59:00+00:00",
                    ),
                )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-reservation")

            text = output.read_text(encoding="utf-8")
            self.assertIn("## 最近调度转换", text)
            self.assertIn("reservation-expired", text)
            self.assertIn("session-current", text)
            self.assertIn("cpu", text)
            self.assertIn("expired", text)
            self.assertIn("2026-07-15T07:59:00+00:00", text)
            self.assertNotIn("private-compatibility-key", text)
            self.assertNotIn("private-command-fingerprint", text)

    def test_projects_only_current_cargo_health_timeouts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(
                session_id="session-health",
                plan_path="docs/plans/runtime/01-feature.md",
            )
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO cargo_jobs(
                           job_id, session_id, lane_kind, target_dir, status, dry_run,
                           created_at, last_heartbeat_at, target_key
                       ) VALUES (?, ?, 'test', ?, 'running', 0, ?, ?, ?)""",
                    (
                        "job-still-stale",
                        "session-health",
                        str(root / "target-stale"),
                        "2026-07-15T07:00:00+00:00",
                        "2026-07-15T07:50:00+00:00",
                        "target-stale",
                    ),
                )
                connection.execute(
                    """INSERT INTO cargo_jobs(
                           job_id, session_id, lane_kind, target_dir, status, dry_run,
                           created_at, last_heartbeat_at, target_key
                       ) VALUES (?, ?, 'gpu', ?, 'running', 0, ?, ?, ?)""",
                    (
                        "job-heartbeat-recovered",
                        "session-health",
                        str(root / "target-recovered"),
                        "2026-07-15T07:00:00+00:00",
                        "2026-07-15T07:59:30+00:00",
                        "target-recovered",
                    ),
                )
                connection.executemany(
                    """INSERT INTO events(session_id, event_type, payload_json, created_at)
                       VALUES (?, 'cargo.health_timeout', ?, ?)""",
                    (
                        (
                            "session-health",
                            json.dumps(
                                {
                                    "jobId": "job-still-stale",
                                    "laneKind": "test",
                                    "livePids": [101, 102],
                                    "heartbeatAgeSeconds": 600,
                                    "timeoutSeconds": 300,
                                }
                            ),
                            "2026-07-15T07:55:00+00:00",
                        ),
                        (
                            "session-health",
                            json.dumps(
                                {
                                    "jobId": "job-heartbeat-recovered",
                                    "laneKind": "gpu",
                                    "livePids": [201],
                                    "heartbeatAgeSeconds": 600,
                                    "timeoutSeconds": 300,
                                }
                            ),
                            "2026-07-15T07:55:00+00:00",
                        ),
                    ),
                )

            output = CodexEvidenceProjector(
                database,
                codex_home=root / "codex-home",
                repo_root=repo,
                now=lambda: datetime(2026, 7, 15, 8, 0, tzinfo=timezone.utc),
            ).project(run_id="sync-health")

            text = output.read_text(encoding="utf-8")
            health = text.split("## 当前任务健康告警", maxsplit=1)[1].split(
                "##", maxsplit=1
            )[0]
            self.assertIn("job-still-stale", health)
            self.assertIn("session-health", health)
            self.assertIn("600s / 300s", health)
            self.assertIn("2", health)
            self.assertNotIn("job-heartbeat-recovered", health)

    def test_worker_projects_evidence_after_committing_a_codex_sync_run(self) -> None:
        completed = threading.Event()
        result = CodexReconcileResult(
            run_id="sync-a", scanned_count=1, changed_count=1,
            diagnostic_count=0, unavailable_count=0,
        )
        store = mock.Mock()
        store.reconcile.side_effect = lambda *_args, **_kwargs: (completed.set(), result)[1]
        spool = mock.Mock()
        spool.validated_pending.return_value = ()
        projector = mock.Mock()
        discovery = CodexDiscoveryResult(
            sessions=(), diagnostics=(), membership_complete=True,
            scanned_count=0, source_revision="revision",
        )
        worker = CodexSyncWorker(
            discover=lambda _full: discovery,
            store=store,
            spool=spool,
            writable=lambda: True,
            project=projector,
            membership_interval_seconds=60,
            full_interval_seconds=60,
        )

        worker.start()
        self.assertTrue(completed.wait(timeout=2))
        worker.stop()

        projector.assert_called_once_with(result, True)


if __name__ == "__main__":
    unittest.main()
