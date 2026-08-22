from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate


class MigrationTests(unittest.TestCase):
    def test_schema_65_persists_fixture_and_product_staging_leases(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            version = migrate(database)

            self.assertGreaterEqual(LATEST_SCHEMA_VERSION, 65)
            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            with database.connect() as connection:
                fixture_columns = {
                    row["name"]
                    for row in connection.execute(
                        "PRAGMA table_info(artifact_fixture_leases)"
                    )
                }
                staging_columns = {
                    row["name"]
                    for row in connection.execute(
                        "PRAGMA table_info(artifact_product_staging_leases)"
                    )
                }
                staging_indexes = {
                    row["name"]: row["sql"]
                    for row in connection.execute(
                        """SELECT name, sql FROM sqlite_master
                           WHERE type='index'
                             AND tbl_name='artifact_product_staging_leases'"""
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
                fixture_columns,
            )
            self.assertEqual(
                {
                    "lease_id",
                    "purpose",
                    "staging_target_key",
                    "staging_dir",
                    "final_target_key",
                    "final_dir",
                    "owner_pid",
                    "owner_process_creation_time",
                    "status",
                    "staging_filesystem_identity",
                    "published_filesystem_identity",
                    "created_at",
                    "publishing_at",
                    "published_at",
                    "released_at",
                },
                staging_columns,
            )
            self.assertIn(
                "WHERE status IN ('active', 'publishing')",
                staging_indexes["idx_artifact_product_staging_live_source"],
            )
            self.assertIn(
                "WHERE status IN ('active', 'publishing', 'published')",
                staging_indexes["idx_artifact_product_staging_live_final"],
            )


if __name__ == "__main__":
    unittest.main()
