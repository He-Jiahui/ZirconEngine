from __future__ import annotations

import json
import sqlite3
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, MIGRATIONS, migrate


class SupervisionSchemaTests(unittest.TestCase):
    def _frozen_v19_database(self, root: str) -> Database:
        database = Database(Path(root) / "coordinator.sqlite3")
        with database.transaction() as connection:
            connection.execute(
                "CREATE TABLE schema_version(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
            )
            for version in range(1, 20):
                MIGRATIONS[version](connection)
                connection.execute(
                    "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                    (version,),
                )
            connection.execute(
                """
                INSERT INTO action_requests(
                    action_id, action_kind, risk, required_role, actor,
                    daemon_instance_id, parameters_json, impact_json,
                    warnings_json, state_fingerprint,
                    confirmation_phrase_hash, status, created_at, expires_at
                ) VALUES (
                    'action-v19', 'session.heartbeat', 'green', 'operator',
                    'schema-test', 'daemon-v19', '{}', '[]', '[]',
                    'fingerprint-v19', 'phrase-v19', 'succeeded', 'now', 'later'
                )
                """
            )
            connection.execute(
                """
                INSERT INTO action_approvals(
                    approval_id, action_id, actor, role, reason,
                    state_fingerprint, created_at
                ) VALUES (
                    'approval-v19', 'action-v19', 'schema-test', 'operator',
                    'preserve history', 'fingerprint-v19', 'now'
                )
                """
            )
        return database

    def _insert_action(self, connection: sqlite3.Connection, action_id: str, kind: str) -> None:
        connection.execute(
            """
            INSERT INTO action_requests(
                action_id, action_kind, risk, required_role, actor,
                daemon_instance_id, parameters_json, impact_json,
                warnings_json, state_fingerprint,
                confirmation_phrase_hash, status, created_at, expires_at
            ) VALUES (?, ?, 'red', 'maintainer', 'schema-test', 'daemon-v20',
                      '{}', '[]', '[]', 'fingerprint-v20', 'phrase-v20',
                      'previewed', 'now', 'later')
            """,
            (action_id, kind),
        )

    def test_schema_20_preserves_v19_action_history_and_constraints(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = self._frozen_v19_database(directory)

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                request = connection.execute(
                    "SELECT action_kind, status FROM action_requests WHERE action_id='action-v19'"
                ).fetchone()
                approval = connection.execute(
                    "SELECT reason FROM action_approvals WHERE approval_id='approval-v19'"
                ).fetchone()
                foreign_key_failures = list(connection.execute("PRAGMA foreign_key_check"))
            self.assertEqual(("session.heartbeat", "succeeded"), tuple(request))
            self.assertEqual("preserve history", approval[0])
            self.assertEqual([], foreign_key_failures)

            with self.assertRaisesRegex(sqlite3.IntegrityError, "immutable"):
                with database.transaction() as connection:
                    connection.execute(
                        "UPDATE action_approvals SET reason='changed' WHERE approval_id='approval-v19'"
                    )
            with self.assertRaisesRegex(sqlite3.IntegrityError, "immutable"):
                with database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM action_approvals WHERE approval_id='approval-v19'"
                    )

    def test_schema_20_closes_lifecycle_action_kind_enum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.transaction() as connection:
                for index, kind in enumerate(
                    (
                        "service.drain",
                        "service.resume",
                        "service.stop",
                        "service.restart",
                        "service.force_stop",
                    )
                ):
                    self._insert_action(connection, f"lifecycle-{index}", kind)

            with self.assertRaisesRegex(sqlite3.IntegrityError, "action kind|CHECK"):
                with database.transaction() as connection:
                    self._insert_action(connection, "shell", "shell.run")

    def test_schema_20_enforces_supervision_state_and_immutable_events(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO service_recovery_state(
                        repository_key, state, updated_at
                    ) VALUES ('repo', 'healthy', 'now')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO service_supervision_events(
                        repository_key, sequence, from_state, to_state,
                        reason_code, created_at
                    ) VALUES ('repo', 1, 'starting', 'healthy', 'startup.ready', 'now')
                    """
                )

            with self.assertRaisesRegex(sqlite3.IntegrityError, "immutable"):
                with database.transaction() as connection:
                    connection.execute(
                        "UPDATE service_supervision_events SET reason_code='rewritten' WHERE repository_key='repo'"
                    )
            with self.assertRaisesRegex(sqlite3.IntegrityError, "immutable"):
                with database.transaction() as connection:
                    connection.execute(
                        "DELETE FROM service_supervision_events WHERE repository_key='repo'"
                    )
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO service_recovery_state(repository_key, state, updated_at) "
                        "VALUES ('bad-repo', 'busy', 'now')"
                    )

    def test_schema_20_enforces_durable_lifecycle_intent_enum(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.transaction() as connection:
                self._insert_action(connection, "restart-action", "service.restart")
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status,
                        requested_by, source_daemon_instance_id,
                        created_at, updated_at
                    ) VALUES (
                        'intent', 'repo', 'restart-action', 'service.restart',
                        'accepted', 'schema-test', 'daemon-v20', 'now', 'now'
                    )
                    """
                )

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO service_lifecycle_intents(
                            intent_id, repository_key, kind, status,
                            requested_by, source_daemon_instance_id,
                            created_at, updated_at
                        ) VALUES (
                            'bad-intent', 'repo', 'service.restart', 'busy',
                            'schema-test', 'daemon-v20', 'now', 'now'
                        )
                        """
                    )

    def test_schema_26_allows_only_one_active_reversible_lifecycle_per_repository(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, kind, status,
                        requested_by, source_daemon_instance_id, created_at, updated_at
                    ) VALUES ('stop-a', 'repo', 'service.stop', 'draining',
                              'test', 'daemon', 'now', 'now')
                    """
                )

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO service_lifecycle_intents(
                            intent_id, repository_key, kind, status,
                            requested_by, source_daemon_instance_id, created_at, updated_at
                        ) VALUES ('restart-b', 'repo', 'service.restart', 'accepted',
                                  'test', 'daemon', 'now', 'now')
                        """
                    )

    def test_schema_26_fails_historical_conflicting_lifecycles_and_audits_repair(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version(version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 26):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                for action_id, kind, intent_id, status in (
                    ("action-stop", "service.stop", "intent-stop", "draining"),
                    ("action-restart", "service.restart", "intent-restart", "accepted"),
                ):
                    self._insert_action(connection, action_id, kind)
                    connection.execute(
                        "UPDATE action_requests SET status='executing' WHERE action_id=?",
                        (action_id,),
                    )
                    connection.execute(
                        """
                        INSERT INTO service_lifecycle_intents(
                            intent_id, repository_key, action_id, kind, status,
                            requested_by, source_daemon_instance_id, created_at, updated_at
                        ) VALUES (?, 'repo', ?, ?, ?, 'schema-test', 'daemon-v25', 'now', 'now')
                        """,
                        (intent_id, action_id, kind, status),
                    )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                intents = list(
                    connection.execute(
                        "SELECT intent_id, status, error_code, completed_at "
                        "FROM service_lifecycle_intents ORDER BY intent_id"
                    )
                )
                actions = list(
                    connection.execute(
                        "SELECT action_id, status, error_code, completed_at "
                        "FROM action_requests ORDER BY action_id"
                    )
                )
                event = connection.execute(
                    "SELECT payload_json FROM events "
                    "WHERE event_type='schema.lifecycle_conflict_repaired'"
                ).fetchone()
                index_names = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA index_list(service_lifecycle_intents)"
                    )
                }

            self.assertEqual(
                [
                    ("intent-restart", "failed", "migration.lifecycle_conflict", True),
                    ("intent-stop", "failed", "migration.lifecycle_conflict", True),
                ],
                [
                    (row[0], row[1], row[2], row[3] is not None)
                    for row in intents
                ],
            )
            self.assertEqual(
                [
                    ("action-restart", "failed", "migration.lifecycle_conflict", True),
                    ("action-stop", "failed", "migration.lifecycle_conflict", True),
                ],
                [
                    (row[0], row[1], row[2], row[3] is not None)
                    for row in actions
                ],
            )
            payload = json.loads(event[0])
            self.assertEqual("repo", payload["repositoryKey"])
            self.assertEqual(
                ["intent-restart", "intent-stop"], payload["intentIds"]
            )
            self.assertIn("service_lifecycle_one_active_reversible", index_names)


if __name__ == "__main__":
    unittest.main()
