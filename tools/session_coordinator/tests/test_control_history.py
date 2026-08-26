from __future__ import annotations

import json
import sqlite3
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.control_plane.history import ControlHistoryService
from tools.session_coordinator.database import Database
from tools.session_coordinator import migrations as migrations_module
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_tickets import ValidationTicketService


class ControlHistoryTests(unittest.TestCase):
    def test_validation_history_projects_ticket_timeline_without_raw_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            plan_path = "docs/plans/tooling/01-control-history.md"
            plan = repo / plan_path
            plan.parent.mkdir(parents=True)
            plan.write_text("# Control history\n", encoding="utf-8")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(
                session_id="history-owner",
                plan_path=plan_path,
            )
            tickets = ValidationTicketService(database)
            receipt = tickets.submit(
                session_id="history-owner",
                request_id="history-request",
                source_manifest={"README.md": "a" * 64},
                command=["cargo", "test", "-p", "zircon_runtime"],
                toolchain={"cargo": "1.94.1"},
                coverage={"kind": "focused"},
            )
            tickets.transition(
                receipt.ticket.ticket_id,
                "running",
                evidence={"phase": "run", "jobId": "job-a", "runId": "run-a"},
            )
            tickets.record_result(
                receipt.ticket.ticket_id,
                "failed",
                evidence={
                    "phase": "test",
                    "errorCode": "assertion_failed",
                    "error": "large raw diagnostic must not be projected",
                },
            )

            projection = ControlHistoryService(database).validation(limit=10)

        self.assertEqual(1, projection["statusCounts"]["failed"])
        self.assertFalse(projection["truncated"])
        ticket = projection["tickets"][0]
        self.assertEqual("failed", ticket["status"])
        self.assertEqual(
            ["queued", "running", "failed"],
            [event["toStatus"] for event in ticket["events"]],
        )
        self.assertEqual("assertion_failed", ticket["events"][-1]["errorCode"])
        self.assertNotIn("large raw diagnostic", json.dumps(projection))

    def test_failure_history_projects_added_and_fixed_lifecycle_events(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            database = Database(root / "state.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.executemany(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        (
                            "open-chain",
                            "docs/plans/runtime/01/failure-open.md",
                            "failure",
                            "open",
                            "2026-08-24T01:00:00+00:00",
                            None,
                            "open-failure",
                            "docs/plans/editor/01.md",
                            "docs/plans/runtime/01.md",
                            "docs/plans/editor/01",
                            "docs/plans/runtime/01",
                            10,
                            "2026-08-24T01:01:00+00:00",
                        ),
                        (
                            "fixed-chain",
                            "docs/plans/editor/01/fixed-2026-08-25-fixed.md",
                            "fixed",
                            "fixed",
                            "2026-08-24T02:00:00+00:00",
                            "2026-08-25T02:00:00+00:00",
                            "fixed-failure",
                            "docs/plans/editor/01.md",
                            "docs/plans/runtime/02.md",
                            "docs/plans/editor/01",
                            "docs/plans/runtime/02",
                            20,
                            "2026-08-25T02:01:00+00:00",
                        ),
                    ),
                )
                connection.executemany(
                    """
                    INSERT INTO failure_lifecycle_events(
                        lifecycle_key, event_kind, artifact_path, created_at, recorded_at
                    ) VALUES (?, ?, ?, ?, ?)
                    """,
                    (
                        (
                            "open-chain",
                            "added",
                            "docs/plans/runtime/01/failure-2026-08-24-open-failure.md",
                            "2026-08-24T01:00:00+00:00",
                            "2026-08-24T01:01:00+00:00",
                        ),
                        (
                            "fixed-chain",
                            "added",
                            "docs/plans/runtime/02/failure-2026-08-24-fixed-failure.md",
                            "2026-08-24T02:00:00+00:00",
                            "2026-08-24T02:01:00+00:00",
                        ),
                        (
                            "fixed-chain",
                            "fixed",
                            "docs/plans/editor/01/fixed-2026-08-25-fixed.md",
                            "2026-08-25T02:00:00+00:00",
                            "2026-08-25T02:01:00+00:00",
                        ),
                    ),
                )

            projection = ControlHistoryService(database).failures(limit=10)
            first_page = ControlHistoryService(database).failures(limit=1)

        self.assertEqual({"open": 1, "fixed": 1}, projection["statusCounts"])
        self.assertEqual(
            ["open-chain", "fixed-chain"],
            [item["lifecycleKey"] for item in projection["chains"]],
        )
        self.assertEqual(["open-chain"], [item["lifecycleKey"] for item in first_page["chains"]])
        self.assertTrue(first_page["truncated"])
        fixed = next(item for item in projection["chains"] if item["lifecycleKey"] == "fixed-chain")
        opened = next(item for item in projection["chains"] if item["lifecycleKey"] == "open-chain")
        self.assertEqual(["added", "fixed"], [event["kind"] for event in fixed["events"]])
        self.assertNotEqual(
            fixed["events"][0]["artifactPath"],
            fixed["events"][1]["artifactPath"],
        )
        self.assertEqual(["added"], [event["kind"] for event in opened["events"]])

    def test_schema_68_backfills_original_and_fixed_artifact_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "state.sqlite3")
            with mock.patch.object(migrations_module, "LATEST_SCHEMA_VERSION", 67):
                migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (?, ?, 'fixed', 'fixed', ?, ?, ?, ?, ?, ?, ?, 10, ?)
                    """,
                    (
                        "legacy-fixed",
                        "docs/plans/origin/01/fixed-2026-08-25-regression.md",
                        "2026-08-24",
                        "2026-08-25",
                        "regression",
                        "docs/plans/origin/01.md",
                        "docs/plans/fixing/02.md",
                        "docs/plans/origin/01",
                        "docs/plans/fixing/02",
                        "2026-08-25T01:00:00+00:00",
                    ),
                )

            migrate(database)
            projection = ControlHistoryService(database).failures(limit=10)
            with self.assertRaisesRegex(sqlite3.IntegrityError, "immutable"):
                with database.transaction() as connection:
                    connection.execute(
                        "UPDATE failure_lifecycle_events SET artifact_path='changed'"
                    )

        self.assertEqual(
            [
                "docs/plans/fixing/02/failure-2026-08-24-regression.md",
                "docs/plans/origin/01/fixed-2026-08-25-regression.md",
            ],
            [event["artifactPath"] for event in projection["chains"][0]["events"]],
        )


if __name__ == "__main__":
    unittest.main()
