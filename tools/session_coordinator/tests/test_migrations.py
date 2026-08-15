from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate


class MigrationTests(unittest.TestCase):
    def test_schema_64_persists_process_bound_fixture_leases(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            version = migrate(database)

            self.assertEqual(64, LATEST_SCHEMA_VERSION)
            self.assertEqual(64, version)
            with database.connect() as connection:
                columns = {
                    row["name"]
                    for row in connection.execute(
                        "PRAGMA table_info(artifact_fixture_leases)"
                    )
                }
            self.assertEqual(
                {
                    "lease_id",
                    "target_key",
                    "target_dir",
                    "prefix",
                    "owner_pid",
                    "owner_process_creation_time",
                    "status",
                    "created_at",
                    "released_at",
                },
                columns,
            )


if __name__ == "__main__":
    unittest.main()
