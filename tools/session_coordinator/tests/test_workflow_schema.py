from __future__ import annotations

import sqlite3
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, MIGRATIONS, migrate
from tools.session_coordinator.models import CoordinatorError


class WorkflowSchemaTests(unittest.TestCase):
    def test_schema_14_creates_workflow_and_browser_auth_tables(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            version = migrate(database)

            with database.connect() as connection:
                tables = {
                    row[0]
                    for row in connection.execute(
                        "SELECT name FROM sqlite_master WHERE type = 'table'"
                    )
                }
            self.assertEqual(14, LATEST_SCHEMA_VERSION)
            self.assertEqual(14, version)
            self.assertTrue(
                {
                    "workflow_runs",
                    "workflow_nodes",
                    "workflow_edges",
                    "workflow_attempts",
                    "workflow_artifacts",
                    "workflow_diagnostics",
                    "web_control_sessions",
                    "web_bootstrap_tickets",
                }
                <= tables
            )

    def test_schema_13_upgrades_without_losing_existing_sessions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 14):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, created_at, updated_at, last_heartbeat_at
                    ) VALUES ('existing-session', 'registered', 'now', 'now', 'now')
                    """
                )

            self.assertEqual(14, migrate(database))

            with database.connect() as connection:
                self.assertIsNotNone(
                    connection.execute(
                        "SELECT 1 FROM sessions WHERE session_id = 'existing-session'"
                    ).fetchone()
                )
                self.assertIsNotNone(
                    connection.execute(
                        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'workflow_runs'"
                    ).fetchone()
                )

    def test_newer_schema_is_rejected_without_downgrade_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO schema_version(version, applied_at) VALUES (15, 'future')"
                )
                connection.execute("CREATE TABLE future_marker(value TEXT)")
                connection.execute("INSERT INTO future_marker VALUES ('preserved')")

            with self.assertRaises(CoordinatorError) as rejected:
                migrate(database)

            self.assertEqual("schema_version_newer", rejected.exception.code)
            with database.connect() as connection:
                self.assertEqual(
                    "preserved",
                    connection.execute("SELECT value FROM future_marker").fetchone()[0],
                )

    def test_workflow_state_checks_reject_free_form_values(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO workflow_runs(
                            run_id, workflow_key, state, created_at, updated_at
                        ) VALUES ('run-a', 'goal-a', 'almost_done', 'now', 'now')
                        """
                    )

    def test_duplicate_workflow_edges_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO workflow_runs(run_id, workflow_key, state, created_at, updated_at) "
                    "VALUES ('run-a', 'goal-a', 'registered', 'now', 'now')"
                )
                for node in ("one", "two"):
                    connection.execute(
                        """
                        INSERT INTO workflow_nodes(
                            node_id, run_id, node_key, kind, title, stage, state,
                            created_at, updated_at
                        ) VALUES (?, 'run-a', ?, 'slice', ?, 'implementation',
                                  'pending', 'now', 'now')
                        """,
                        (node, node, node),
                    )
                connection.execute(
                    "INSERT INTO workflow_edges(run_id, from_node_id, to_node_id, edge_kind) "
                    "VALUES ('run-a', 'one', 'two', 'depends_on')"
                )

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO workflow_edges(run_id, from_node_id, to_node_id, edge_kind) "
                        "VALUES ('run-a', 'one', 'two', 'depends_on')"
                    )

    def test_workflow_foreign_keys_reject_unknown_sessions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO workflow_runs(
                            run_id, session_id, workflow_key, state, created_at, updated_at
                        ) VALUES ('run-a', 'missing-session', 'goal-a', 'registered', 'now', 'now')
                        """
                    )

    def test_cross_run_edges_and_artifacts_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                for run_id in ("run-a", "run-b"):
                    connection.execute(
                        "INSERT INTO workflow_runs(run_id, workflow_key, state, created_at, updated_at) "
                        "VALUES (?, ?, 'registered', 'now', 'now')",
                        (run_id, run_id),
                    )
                    connection.execute(
                        """
                        INSERT INTO workflow_nodes(
                            node_id, run_id, node_key, kind, title, stage, state,
                            created_at, updated_at
                        ) VALUES (?, ?, 'goal', 'goal', 'Goal', 'goal', 'pending', 'now', 'now')
                        """,
                        (f"{run_id}:goal", run_id),
                    )

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO workflow_edges(
                            run_id, from_node_id, to_node_id, edge_kind
                        ) VALUES ('run-a', 'run-a:goal', 'run-b:goal', 'depends_on')
                        """
                    )
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO workflow_artifacts(
                            artifact_id, run_id, node_id, artifact_kind,
                            display_name, created_at
                        ) VALUES (
                            'artifact-a', 'run-a', 'run-b:goal', 'report',
                            'cross-run', 'now'
                        )
                        """
                    )

    def test_attempts_are_immutable_and_artifact_kinds_are_enumerated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO workflow_runs(run_id, workflow_key, state, created_at, updated_at) "
                    "VALUES ('run-a', 'run-a', 'registered', 'now', 'now')"
                )
                connection.execute(
                    """
                    INSERT INTO workflow_nodes(
                        node_id, run_id, node_key, kind, title, stage, state,
                        created_at, updated_at
                    ) VALUES (
                        'node-a', 'run-a', 'goal', 'goal', 'Goal', 'goal',
                        'pending', 'now', 'now'
                    )
                    """
                )
                connection.execute(
                    """
                    INSERT INTO workflow_attempts(
                        attempt_id, run_id, node_id, attempt_number, state,
                        evidence_json, started_at
                    ) VALUES ('attempt-a', 'run-a', 'node-a', 1, 'running', '{}', 'now')
                    """
                )

            for statement in (
                "UPDATE workflow_attempts SET state = 'failed' WHERE attempt_id = 'attempt-a'",
                "DELETE FROM workflow_attempts WHERE attempt_id = 'attempt-a'",
            ):
                with self.subTest(statement=statement), self.assertRaises(
                    sqlite3.IntegrityError
                ):
                    with database.transaction() as connection:
                        connection.execute(statement)
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO workflow_artifacts(
                            artifact_id, run_id, artifact_kind, display_name, created_at
                        ) VALUES ('artifact-a', 'run-a', 'anything', 'invalid', 'now')
                        """
                    )


if __name__ == "__main__":
    unittest.main()
