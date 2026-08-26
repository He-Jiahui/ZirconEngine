from __future__ import annotations

import sqlite3
import tempfile
import unittest
from contextlib import contextmanager
from pathlib import Path
from unittest import mock

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

    def test_latest_schema_persists_benchmark_grants_and_dual_manifest_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.connect() as connection:
                grant_columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(benchmark_validation_grants)"
                    )
                }
                binding_columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(workflow_validation_bindings)"
                    )
                }

            self.assertTrue(
                {
                    "grant_id",
                    "job_id",
                    "target_session_id",
                    "input_manifest_hash",
                    "scoped_manifest_hash",
                    "benchmark_name",
                    "cargo_profile",
                    "fifo_sequence",
                    "status",
                    "root_pid",
                    "root_process_creation_time",
                    "job_isolated",
                    "validation_run_id",
                }
                <= grant_columns
            )
            self.assertTrue(
                {
                    "copy_input_manifest_hash",
                    "benchmark_name",
                    "cargo_profile",
                    "benchmark_grant_id",
                    "root_pid",
                    "root_process_creation_time",
                }
                <= binding_columns
            )

    def test_latest_schema_distinguishes_artifact_cleanup_reservations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(cleanup_reservations)"
                    )
                }

            self.assertIn("reservation_kind", columns)
            self.assertIn("filesystem_identity", columns)

    def test_schema_60_upgrades_existing_cleanup_reservations_as_cargo(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute("DELETE FROM schema_version WHERE version=60")
                connection.execute(
                    """CREATE TABLE cleanup_reservations_v59(
                           target_key TEXT PRIMARY KEY,
                           target_dir TEXT NOT NULL,
                           reserved_at TEXT NOT NULL
                       )"""
                )
                connection.execute(
                    """INSERT INTO cleanup_reservations_v59
                       SELECT target_key, target_dir, reserved_at
                       FROM cleanup_reservations"""
                )
                connection.execute("DROP TABLE cleanup_reservations")
                connection.execute(
                    "ALTER TABLE cleanup_reservations_v59 RENAME TO cleanup_reservations"
                )
                connection.execute(
                    """INSERT INTO cleanup_reservations
                       VALUES ('legacy', 'D:\\cargo-targets\\legacy', 'now')"""
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                row = connection.execute(
                    """SELECT reservation_kind, filesystem_identity
                       FROM cleanup_reservations WHERE target_key='legacy'"""
                ).fetchone()
            self.assertEqual(("cargo", None), tuple(row))

    def test_latest_schema_persists_optional_failure_workflow_node(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1] for row in connection.execute("PRAGMA table_info(failure_nodes)")
                }
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]

            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertIn("origin_workflow_node", columns)

    def test_latest_schema_persists_structured_failure_diagnostic_details(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(failure_diagnostics)"
                    )
                }

            self.assertIn("details_json", columns)

    def test_latest_schema_persists_delegated_failure_return_proofs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(failure_return_delegation_proofs)"
                    )
                }

            self.assertTrue(
                {
                    "proof_id",
                    "lifecycle_key",
                    "origin_session_id",
                    "fixing_session_id",
                    "origin_plan",
                    "fixing_plan",
                    "destination_path",
                    "content_hash",
                    "baseline_epoch",
                    "authorization_event_id",
                    "consumed_closeout_id",
                    "consumed_input_fingerprint",
                    "consumed_commit_sha",
                    "consumed_at",
                }
                <= columns
            )

    def test_latest_schema_persists_future_path_ownership_transfers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1]: row
                    for row in connection.execute(
                        "PRAGMA table_info(ownership_transfers)"
                    )
                }

            self.assertIn("path_state", columns)
            self.assertEqual(0, columns["content_hash"][3])
            self.assertEqual("'existing'", columns["path_state"][4])

    def test_schema_67_preserves_existing_ownership_transfer_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 66
            ):
                self.assertEqual(66, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, created_at, updated_at, last_heartbeat_at
                    ) VALUES ('target', 'active', 'now', 'now', 'now')
                    """
                )
                cursor = connection.execute(
                    """
                    INSERT INTO baseline_epochs(
                        head_commit, index_tree, health, manifest_json, created_at
                    ) VALUES ('head', 'tree', 'healthy', '{}', 'now')
                    """
                )
                baseline_epoch = int(cursor.lastrowid)
                connection.execute(
                    """
                    INSERT INTO ownership_transfer_previews(
                        fingerprint, target_session_id, baseline_epoch,
                        candidates_json, created_at, applied_at
                    ) VALUES ('preview', 'target', ?, '{}', 'now', 'now')
                    """,
                    (baseline_epoch,),
                )
                connection.execute(
                    """
                    INSERT INTO ownership_transfers(
                        fingerprint, path_key, display_path, target_session_id,
                        source_session_id, baseline_epoch, content_hash, actor, transferred_at
                    ) VALUES ('preview', 'owned.txt', 'owned.txt', 'target', NULL, ?,
                              'existing-hash', 'fixture', 'now')
                    """,
                    (baseline_epoch,),
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                row = connection.execute(
                    """
                    SELECT content_hash, path_state
                    FROM ownership_transfers WHERE fingerprint='preview'
                    """
                ).fetchone()
            self.assertEqual(("existing-hash", "existing"), tuple(row))

    def test_schema_68_backfills_immutable_failure_lifecycle_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 67
            ):
                self.assertEqual(67, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (
                        'origin|fixer|open',
                        'docs/plans/fixer/01/failure-2026-08-24-open.md',
                        'failure', 'open', '2026-08-24', NULL, 'open',
                        'docs/plans/origin/01-origin.md',
                        'docs/plans/fixer/01-fixer.md',
                        'docs/plans/origin/01', 'docs/plans/fixer/01', 0,
                        '2026-08-24T01:00:00+00:00'
                    )
                    """
                )
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (
                        'origin|fixer|closed',
                        'docs/plans/origin/01/fixed-2026-08-25-closed.md',
                        'fixed', 'fixed', '2026-08-24', '2026-08-25', 'closed',
                        'docs/plans/origin/01-origin.md',
                        'docs/plans/fixer/01-fixer.md',
                        'docs/plans/origin/01', 'docs/plans/fixer/01', 0,
                        '2026-08-25T01:00:00+00:00'
                    )
                    """
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                rows = connection.execute(
                    """
                    SELECT lifecycle_key, event_kind, artifact_path, created_at
                    FROM failure_lifecycle_events
                    ORDER BY lifecycle_key, event_id
                    """
                ).fetchall()
                with self.assertRaisesRegex(
                    sqlite3.IntegrityError, "failure lifecycle events are immutable"
                ):
                    connection.execute(
                        "UPDATE failure_lifecycle_events SET artifact_path='rewritten.md'"
                    )
                connection.rollback()
                with self.assertRaisesRegex(
                    sqlite3.IntegrityError, "failure lifecycle events are immutable"
                ):
                    connection.execute("DELETE FROM failure_lifecycle_events")

            self.assertEqual(
                [
                    (
                        "origin|fixer|closed",
                        "added",
                        "docs/plans/fixer/01/failure-2026-08-24-closed.md",
                        "2026-08-24",
                    ),
                    (
                        "origin|fixer|closed",
                        "fixed",
                        "docs/plans/origin/01/fixed-2026-08-25-closed.md",
                        "2026-08-25",
                    ),
                    (
                        "origin|fixer|open",
                        "added",
                        "docs/plans/fixer/01/failure-2026-08-24-open.md",
                        "2026-08-24",
                    ),
                ],
                [tuple(row) for row in rows],
            )

    def test_schema_63_preserves_existing_failure_diagnostics_with_empty_details(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 62
            ):
                self.assertEqual(62, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO failure_diagnostics(
                        code, message, paths_json, created_at
                    ) VALUES ('cycle', 'legacy cycle', '["legacy.md"]', 'now')
                    """
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                row = connection.execute(
                    """SELECT code, message, paths_json, details_json
                       FROM failure_diagnostics"""
                ).fetchone()
            self.assertEqual(
                ("cycle", "legacy cycle", '["legacy.md"]', "{}"), tuple(row)
            )

    def test_schema_47_clears_only_terminal_finalize_snapshots_and_compacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            terminal_snapshot = b"terminal-index" * (128 * 1024)
            live_snapshot = b"live-index" * (128 * 1024)
            with database.transaction() as connection:
                connection.execute("DELETE FROM schema_version WHERE version = 47")
                connection.execute(
                    "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                    "VALUES ('snapshot-owner', 'registered', 'now', 'now', 'now')"
                )
                for request_id, status, snapshot in (
                    ("terminal-committed", "committed", terminal_snapshot),
                    ("terminal-failed", "failed", terminal_snapshot),
                    ("live-finalizing", "finalizing", live_snapshot),
                ):
                    connection.execute(
                        """
                        INSERT INTO finalize_requests(
                            request_id, session_id, message, paths_json, categories_json,
                            untracked_json, maintenance, status, created_at, completed_at,
                            index_snapshot
                        ) VALUES (?, 'snapshot-owner', 'test snapshot retention', '[]', '{}',
                                  '[]', 0, ?, 'now', 'now', ?)
                        """,
                        (request_id, status, snapshot),
                    )
            with database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
            bytes_before = database.path.stat().st_size

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                snapshots = {
                    row["request_id"]: row["index_snapshot"]
                    for row in connection.execute(
                        "SELECT request_id, index_snapshot FROM finalize_requests "
                        "WHERE request_id LIKE 'terminal-%' OR request_id='live-finalizing'"
                    )
                }
            self.assertIsNone(snapshots["terminal-committed"])
            self.assertIsNone(snapshots["terminal-failed"])
            self.assertEqual(live_snapshot, snapshots["live-finalizing"])
            self.assertLess(database.path.stat().st_size, bytes_before)

    def test_latest_schema_preserves_warm_exclusivity_and_adds_one_cpu_burst_slot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.connect() as connection:
                columns = {
                    row[1]
                    for row in connection.execute("PRAGMA table_info(cargo_lane_reservations)")
                }
                indexes = {
                    row[1]
                    for row in connection.execute("PRAGMA index_list(cargo_lane_reservations)")
                }

            self.assertTrue({"execution_mode", "burst_eligible"} <= columns)
            self.assertTrue(
                {
                    "cargo_lane_reservations_one_active_warm",
                    "cargo_lane_reservations_one_active_burst",
                    "cargo_lane_reservations_cpu_warm_fifo",
                }
                <= indexes
            )

    def test_schema_51_persists_failure_barriers_deferrals_and_durable_cargo_copy_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)

            with database.connect() as connection:
                reservation_columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(cargo_lane_reservations)"
                    )
                }
                job_columns = {
                    row[1] for row in connection.execute("PRAGMA table_info(cargo_jobs)")
                }
                copy_columns = {
                    row[1]
                    for row in connection.execute("PRAGMA table_info(validation_copies)")
                }
                tables = {
                    row[0]
                    for row in connection.execute(
                        "SELECT name FROM sqlite_master WHERE type='table'"
                    )
                }

        self.assertTrue(
            {"dependency_lifecycle_key", "dependency_fixed_sha256", "source_copy_job_id"}
            <= reservation_columns
        )
        self.assertTrue({"source_copy_job_id", "source_copy_manifest_hash"} <= job_columns)
        self.assertTrue(
            {
                "external_sources_json",
                "input_manifest_hash",
                "error_code",
                "error_stage",
                "error_path",
                "error_details_json",
                "materialization_kind",
                "materialization_request_json",
                "materialization_phase",
                "materialization_worker_id",
                "materialization_attempt",
            }
            <= copy_columns
        )
        self.assertIn("workflow_failure_deferrals", tables)

    def test_schema_62_preserves_legacy_copy_failure_while_adding_details(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with mock.patch(
                "tools.session_coordinator.migrations.LATEST_SCHEMA_VERSION", 61
            ):
                self.assertEqual(61, migrate(database))
            with database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, base_head, write_scope_json,
                        created_at, updated_at, last_heartbeat_at
                    ) VALUES ('legacy', 'active', 'head', '[]', 'now', 'now', 'now')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO validation_copies(
                        job_id, session_id, job_root, source_root, target_root,
                        head_commit, manifest_json, status, created_at,
                        external_sources_json, error_code, error_stage, error_path
                    ) VALUES (
                        'legacy-copy', 'legacy', 'job', 'source', 'target',
                        'head', '[]', 'failed', 'now', '[]',
                        'validation_copy_compile_time_resource_missing',
                        'closure_planning', 'missing.rs'
                    )
                    """
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                row = connection.execute(
                    """
                    SELECT error_code, error_stage, error_path, error_details_json
                    FROM validation_copies WHERE job_id='legacy-copy'
                    """
                ).fetchone()
        self.assertEqual(
            (
                "validation_copy_compile_time_resource_missing",
                "closure_planning",
                "missing.rs",
                "{}",
            ),
            tuple(row),
        )

    def test_schema_41_preserves_evidence_progress_and_adds_reservation_payload(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")

            migrate(database)

            with database.connect() as connection:
                evidence_columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(codex_evidence_sources)"
                    )
                }
                reservation_columns = {
                    row[1]
                    for row in connection.execute(
                        "PRAGMA table_info(cargo_lane_reservations)"
                    )
                }
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]

            self.assertEqual(LATEST_SCHEMA_VERSION, version)
            self.assertIn("scan_offset", evidence_columns)
            self.assertIn("compatibility_json", reservation_columns)

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

    def test_schema_27_enforces_codex_projection_contracts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                    "VALUES ('bound-thread', 'registered', 'now', 'now', 'now')"
                )
                connection.execute(
                    """
                    INSERT INTO codex_sessions(
                        thread_id, rollout_path, source_location, state, cwd,
                        last_event, bound_session_id, first_seen_at,
                        last_activity_at, last_synced_at, source_mtime_ns,
                        source_size
                    ) VALUES (
                        'bound-thread', 'rollout.jsonl', 'active', 'idle',
                        'E:\\Git\\ZirconEngine', 'stop', 'bound-thread', 'now',
                        'now', 'now', 1, 1
                    )
                    """
                )
                indexes = {
                    row[1]
                    for row in connection.execute("PRAGMA index_list(codex_sessions)")
                }

            self.assertTrue(
                {"codex_sessions_state_activity", "codex_sessions_bound_session"}
                <= indexes
            )
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO codex_sessions(
                            thread_id, rollout_path, source_location, state, cwd,
                            last_event, first_seen_at, last_activity_at,
                            last_synced_at, source_mtime_ns, source_size
                        ) VALUES (
                            'invalid-state', 'rollout.jsonl', 'archived', 'active',
                            'E:\\Git\\ZirconEngine', 'unknown', 'now', 'now',
                            'now', 1, 1
                        )
                        """
                    )

            with database.transaction() as connection:
                connection.execute("DELETE FROM sessions WHERE session_id='bound-thread'")
            with database.connect() as connection:
                binding = connection.execute(
                    "SELECT bound_session_id FROM codex_sessions WHERE thread_id='bound-thread'"
                ).fetchone()[0]
            self.assertIsNone(binding)

    def test_schema_27_failure_rolls_back_to_valid_v26(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 27):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )

            original = MIGRATIONS[27]

            def fail_after_ddl(connection) -> None:
                connection.execute("CREATE TABLE injected_v27_partial(value TEXT)")
                raise sqlite3.OperationalError("simulated schema 27 failure")

            MIGRATIONS[27] = fail_after_ddl
            try:
                with self.assertRaisesRegex(sqlite3.OperationalError, "simulated schema 27"):
                    migrate(database)
            finally:
                MIGRATIONS[27] = original

            with database.connect() as connection:
                version = connection.execute(
                    "SELECT MAX(version) FROM schema_version"
                ).fetchone()[0]
                partial = connection.execute(
                    "SELECT COUNT(*) FROM sqlite_master "
                    "WHERE type='table' AND name='injected_v27_partial'"
                ).fetchone()[0]
            self.assertEqual(26, version)
            self.assertEqual(0, partial)

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

    def test_schema_28_preserves_action_and_supervision_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 28):
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
                        'history-action', 'service.stop', 'red', 'maintainer',
                        'tester', 'instance', '{}', '[]', '[]', 'fingerprint',
                        'phrase', 'succeeded', 'now', 'later'
                    )
                    """
                )
                connection.execute(
                    "INSERT INTO action_approvals VALUES "
                    "('approval', 'history-action', 'tester', 'maintainer', 'reason', 'fingerprint', 'now')"
                )
                connection.execute(
                    """
                    INSERT INTO service_supervision_events(
                        repository_key, sequence, to_state, reason_code,
                        action_id, created_at
                    ) VALUES ('repo', 1, 'offline', 'test', 'history-action', 'now')
                    """
                )
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status,
                        requested_by, source_daemon_instance_id, created_at,
                        updated_at
                    ) VALUES (
                        'intent', 'repo', 'history-action', 'service.stop',
                        'succeeded', 'tester', 'instance', 'now', 'now'
                    )
                    """
                )

            migrate(database)

            with database.connect() as connection:
                counts = tuple(
                    connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                    for table in (
                        "action_requests",
                        "action_approvals",
                        "service_supervision_events",
                        "service_lifecycle_intents",
                    )
                )
                foreign_targets = {
                    row[2]
                    for table in (
                        "action_approvals",
                        "service_supervision_events",
                        "service_lifecycle_intents",
                    )
                    for row in connection.execute(f"PRAGMA foreign_key_list({table})")
                }
                connection.execute(
                    """
                    INSERT INTO action_requests(
                        action_id, action_kind, risk, required_role, actor,
                        daemon_instance_id, parameters_json, impact_json,
                        warnings_json, state_fingerprint,
                        confirmation_phrase_hash, status, created_at, expires_at
                    ) VALUES (
                        'codex-action', 'codex.sessions.reconcile', 'yellow',
                        'maintainer', 'tester', 'instance', '{}', '[]', '[]',
                        'fingerprint', 'phrase', 'previewed', 'now', 'later'
                    )
                    """
                )
            self.assertEqual((1, 1, 1, 1), counts)
            self.assertEqual({"action_requests"}, foreign_targets)
            with self.assertRaises(sqlite3.IntegrityError):
                with database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO action_requests(
                            action_id, action_kind, risk, required_role, actor,
                            daemon_instance_id, parameters_json, impact_json,
                            warnings_json, state_fingerprint,
                            confirmation_phrase_hash, status, created_at, expires_at
                        ) VALUES (
                            'invalid-action', 'arbitrary.command', 'yellow',
                            'maintainer', 'tester', 'instance', '{}', '[]', '[]',
                            'fingerprint', 'phrase', 'previewed', 'now', 'later'
                        )
                        """
                    )

    def test_schema_32_adds_cargo_root_identity_without_rewriting_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 31):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                connection.execute(
                    "INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at) "
                    "VALUES ('session-a', 'registered', 'now', 'now', 'now')"
                )
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, target_key,
                        status, dry_run, pid, created_at, last_heartbeat_at,
                        process_tree_live_pids_json
                    ) VALUES (
                        'legacy-terminal', 'session-a', 'test', 'D:\\cargo-targets\\legacy',
                        'd:\\cargo-targets\\legacy', 'released', 0, 4242, 'now', 'now', '[4242]'
                    )
                    """
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                columns = {
                    row[1] for row in connection.execute("PRAGMA table_info(cargo_jobs)")
                }
                row = connection.execute(
                    "SELECT root_process_creation_time, root_process_kind, "
                    "process_tree_live_pids_json FROM cargo_jobs WHERE job_id='legacy-terminal'"
                ).fetchone()
            self.assertTrue(
                {"root_process_creation_time", "root_process_kind"} <= columns
            )
            self.assertEqual((None, "cargo", "[4242]"), tuple(row))

    def test_schema_34_preserves_v33_failure_rows_and_adds_scope_index(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            database = Database(Path(directory) / "coordinator.sqlite3")
            with database.transaction() as connection:
                connection.execute(
                    "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
                )
                for version in range(1, 34):
                    MIGRATIONS[version](connection)
                    connection.execute(
                        "INSERT INTO schema_version(version, applied_at) VALUES (?, 'now')",
                        (version,),
                    )
                connection.execute(
                    """
                    INSERT INTO failure_nodes(
                        lifecycle_key, artifact_path, kind, status, created_at,
                        resolved_at, summary_slug, origin_plan, fixing_plan,
                        origin_child_dir, fixing_child_dir, priority, imported_at
                    ) VALUES (
                        'origin|fixer|legacy',
                        'docs/plans/fixer/01/failure-legacy.md',
                        'failure', 'open', '2026-07-15', NULL, 'legacy',
                        'docs/plans/origin/01-origin.md',
                        'docs/plans/fixer/01-fixer.md',
                        'docs/plans/origin/01', 'docs/plans/fixer/01', 7, 'now'
                    )
                    """
                )

            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))
            self.assertEqual(LATEST_SCHEMA_VERSION, migrate(database))

            with database.connect() as connection:
                row = connection.execute(
                    """SELECT lifecycle_key, artifact_path, status, priority,
                              origin_workflow_node
                       FROM failure_nodes WHERE summary_slug='legacy'"""
                ).fetchone()
                index = connection.execute(
                    """SELECT name, sql FROM sqlite_master
                       WHERE type='index'
                         AND name='failure_nodes_origin_workflow_status'"""
                ).fetchone()
            self.assertEqual(
                (
                    "origin|fixer|legacy",
                    "docs/plans/fixer/01/failure-legacy.md",
                    "open",
                    7,
                    None,
                ),
                tuple(row),
            )
            self.assertEqual("failure_nodes_origin_workflow_status", index[0])
            self.assertIn("origin_workflow_node", index[1])


if __name__ == "__main__":
    unittest.main()
