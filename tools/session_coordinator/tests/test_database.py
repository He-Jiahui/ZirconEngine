from __future__ import annotations

import sqlite3
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, MIGRATIONS, migrate
from tools.session_coordinator.tests.helpers import init_repo


class DatabaseTests(unittest.TestCase):
    def test_migration_enables_wal_and_is_idempotent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = init_repo(Path(directory) / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=Path(directory) / "state")
            database = Database(config.database_path)

            migrate(database)
            migrate(database)

            with database.connect() as connection:
                version = connection.execute("SELECT MAX(version) FROM schema_version").fetchone()[0]
                journal_mode = connection.execute("PRAGMA journal_mode").fetchone()[0]
                tables = {
                    row[0]
                    for row in connection.execute(
                        "SELECT name FROM sqlite_master WHERE type = 'table'"
                    )
                }

            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertEqual("wal", journal_mode.lower())
            self.assertTrue({"sessions", "events", "baseline_epochs", "leases", "patches"} <= tables)

    def test_transaction_rolls_back_on_error(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                        "VALUES ('duplicate', 'registered', 'now', 'now', 'now')"
                    )
                    connection.execute(
                        "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                        "VALUES ('duplicate', 'registered', 'now', 'now', 'now')"
                    )

            with database.connect() as connection:
                count = connection.execute(
                    "SELECT COUNT(*) FROM sessions WHERE session_id = 'duplicate'"
                ).fetchone()[0]
            self.assertEqual(0, count)

    def test_schema_23_enforces_cargo_reuse_and_cleanup_contracts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                    "VALUES ('session-a', 'registered', 'now', 'now', 'now')"
                )
                columns = {
                    row[1]
                    for row in connection.execute("PRAGMA table_info(cargo_jobs)")
                }
                indexes = {
                    row[1]
                    for row in connection.execute("PRAGMA index_list(cargo_jobs)")
                }

            self.assertTrue(
                {
                    "reuse_key",
                    "compatibility_json",
                    "cleanup_policy",
                    "cleanup_status",
                    "reused_from_job_id",
                    "cleanup_error",
                }
                <= columns
            )
            self.assertIn("cargo_jobs_active_reuse_key", indexes)

            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO cargo_jobs(
                            job_id, session_id, lane_kind, target_dir, target_key,
                            status, dry_run, created_at, last_heartbeat_at,
                            cleanup_policy, cleanup_status
                        ) VALUES ('invalid', 'session-a', 'check', 'D:\\targets\\invalid',
                                  'd:\\targets\\invalid', 'leased', 0, 'now', 'now',
                                  'never', 'retained')
                        """
                    )

            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, target_key,
                        status, dry_run, created_at, last_heartbeat_at,
                        reuse_key, compatibility_json
                    ) VALUES ('first', 'session-a', 'check', 'D:\\targets\\first',
                              'd:\\targets\\first', 'leased', 0, 'now', 'now',
                              'same-key', '{}')
                    """
                )
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO cargo_jobs(
                            job_id, session_id, lane_kind, target_dir, target_key,
                            status, dry_run, created_at, last_heartbeat_at,
                            reuse_key, compatibility_json
                        ) VALUES ('second', 'session-a', 'test', 'E:\\targets\\second',
                                  'e:\\targets\\second', 'leased', 0, 'now', 'now',
                                  'same-key', '{}')
                        """
                    )

    def test_schema_22_repairs_a_version_21_database_missing_cargo_pool_columns(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 21):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                # Production briefly used version 21 for another migration. Reproduce
                # that marker without the Cargo-pool columns so v22 must repair it.
                connection.execute(
                    "INSERT INTO schema_version(version, applied_at) VALUES (21, 'now')"
                )

            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1] for row in connection.execute("PRAGMA table_info(cargo_jobs)")
                }
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]

            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertTrue(
                {"reuse_key", "compatibility_json", "cleanup_policy", "cleanup_status"}
                <= columns
            )

    def test_schema_23_demotes_historical_unkeyed_targets_to_ephemeral_cleanup(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 23):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                connection.execute(
                    "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                    "VALUES ('session-a', 'registered', 'now', 'now', 'now')"
                )
                for job_id, status, reuse_key, compatibility_json in (
                    ("legacy-released", "released", None, None),
                    ("legacy-running", "running", None, None),
                    ("reusable-released", "released", "pool-key", "{}"),
                ):
                    connection.execute(
                        """
                        INSERT INTO cargo_jobs(
                            job_id, session_id, lane_kind, target_dir, target_key,
                            status, dry_run, created_at, last_heartbeat_at,
                            reuse_key, compatibility_json, compatibility_key, reuse_profile,
                            cleanup_policy, cleanup_status
                        ) VALUES (?, 'session-a', 'check', ?, ?, ?, 0, 'now', 'now',
                                  ?, ?, ?, ?, 'retained', 'retained')
                        """,
                        (
                            job_id,
                            f"D:\\cargo-targets\\{job_id}",
                            f"d:\\cargo-targets\\{job_id}",
                            status,
                            reuse_key,
                            compatibility_json,
                            reuse_key,
                            compatibility_json,
                        ),
                    )

            migrate(database)

            with database.connect() as connection:
                rows = {
                    row["job_id"]: (row["cleanup_policy"], row["cleanup_status"])
                    for row in connection.execute(
                        "SELECT job_id, cleanup_policy, cleanup_status FROM cargo_jobs"
                    )
                }
            self.assertEqual(("delete_on_release", "pending"), rows["legacy-released"])
            self.assertEqual(("delete_on_release", "pending"), rows["legacy-running"])
            self.assertEqual(("retained", "retained"), rows["reusable-released"])

    def test_schema_24_compacts_legacy_oversized_event_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 24):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                marker = "legacy-payload-marker"
                oversized = '{"value":"' + marker * 2048 + '"}'
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, 'now')",
                    ("legacy.oversized", oversized),
                )

            migrate(database)

            with database.connect() as connection:
                raw_payload = connection.execute(
                    "SELECT payload_json FROM events WHERE event_type='legacy.oversized'"
                ).fetchone()[0]
            self.assertLessEqual(len(raw_payload.encode("utf-8")), 16 * 1024)
            self.assertNotIn(marker, raw_payload)
            self.assertIn('"truncated": true', raw_payload)

    def test_schema_25_reclaims_free_pages_after_event_compaction(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 25):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                oversized = '{"value":"' + "x" * (4 * 1024 * 1024) + '"}'
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, 'now')",
                    ("legacy.oversized", oversized),
                )
                connection.execute(
                    "UPDATE events SET payload_json='{}' WHERE event_type='legacy.oversized'"
                )
            with database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
                free_pages_before = connection.execute(
                    "PRAGMA freelist_count"
                ).fetchone()[0]
            bytes_before = database.path.stat().st_size

            migrate(database)

            with database.connect() as connection:
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]
                free_pages_after = connection.execute(
                    "PRAGMA freelist_count"
                ).fetchone()[0]
            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertGreater(free_pages_before, 0)
            self.assertLess(free_pages_after, free_pages_before)
            self.assertLess(database.path.stat().st_size, bytes_before)

    def test_schema_25_marker_is_retried_after_vacuum_failure(self) -> None:
        class RejectVacuumConnection:
            def __init__(self, connection):
                self.connection = connection

            def __getattr__(self, name):
                return getattr(self.connection, name)

            def execute(self, statement, parameters=()):
                if statement.strip().upper() == "VACUUM":
                    raise sqlite3.OperationalError("simulated vacuum failure")
                return self.connection.execute(statement, parameters)

        class RejectVacuumDatabase(Database):
            @contextmanager
            def connect(self):
                with super().connect() as connection:
                    yield RejectVacuumConnection(connection)

        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "coordinator.sqlite3"
            database = Database(path)
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 25):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )

            with self.assertRaisesRegex(sqlite3.OperationalError, "simulated vacuum"):
                migrate(RejectVacuumDatabase(path))

            with database.connect() as connection:
                failed_versions = {
                    row[0] for row in connection.execute("SELECT version FROM schema_version")
                }
            self.assertNotIn(25, failed_versions)

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
            with database.connect() as connection:
                self.assertEqual(
                    LATEST_SCHEMA_VERSION,
                    connection.execute("SELECT MAX(version) FROM schema_version").fetchone()[0],
                )

if __name__ == "__main__":
    unittest.main()
