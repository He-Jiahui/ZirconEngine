from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import LATEST_SCHEMA_VERSION, migrate


class MigrationTests(unittest.TestCase):
    def test_schema_69_indexes_only_active_validation_copy_blockers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 68
            ):
                self.assertEqual(68, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, created_at, updated_at, last_heartbeat_at
                    ) VALUES ('session-a', 'active', 'now', 'now', 'now')
                    """
                )
                for job_id, status in (
                    ("copy-cleanup", "cleanup_pending"),
                    ("copy-removed", "removed"),
                    ("copy-running", "running"),
                ):
                    connection.execute(
                        """
                        INSERT INTO validation_copies(
                            job_id, session_id, job_root, source_root, target_root,
                            head_commit, manifest_json, status, created_at
                        ) VALUES (?, 'session-a', ?, 'source', 'target', 'head', '{}', ?, 'now')
                        """,
                        (job_id, job_id, status),
                    )

            self.assertGreaterEqual(LATEST_SCHEMA_VERSION, 69)
            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            blocker_query = """
                SELECT job_id, session_id, status FROM validation_copies
                WHERE status IN ('running', 'cleanup_pending') ORDER BY job_id
            """
            with database.connect() as connection:
                rows = [
                    tuple(row) for row in connection.execute(blocker_query).fetchall()
                ]
                plan = " ".join(
                    str(row["detail"])
                    for row in connection.execute(
                        f"EXPLAIN QUERY PLAN {blocker_query}"
                    ).fetchall()
                )
                index_sql = connection.execute(
                    """
                    SELECT sql FROM sqlite_master
                    WHERE type='index' AND name='validation_copies_active_blockers'
                    """
                ).fetchone()

            self.assertEqual(
                [
                    ("copy-cleanup", "session-a", "cleanup_pending"),
                    ("copy-running", "session-a", "running"),
                ],
                rows,
            )
            self.assertIn("validation_copies_active_blockers", plan)
            self.assertIn("COVERING INDEX", plan)
            self.assertIsNotNone(index_sql)
            self.assertIn(
                "WHERE status IN ('running', 'cleanup_pending')", index_sql["sql"]
            )

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
