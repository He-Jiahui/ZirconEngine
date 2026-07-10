from __future__ import annotations

import sqlite3
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate
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


if __name__ == "__main__":
    unittest.main()
