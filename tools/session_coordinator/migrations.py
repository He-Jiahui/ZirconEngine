from __future__ import annotations

from collections.abc import Callable
from sqlite3 import Connection

from .database import Database


LATEST_SCHEMA_VERSION = 13


def _migration_1(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE sessions (
            session_id TEXT PRIMARY KEY,
            display_name TEXT,
            plan_path TEXT,
            status TEXT NOT NULL CHECK (status IN (
                'registered', 'active', 'waiting_lease', 'resolving_failure',
                'waiting_validation', 'finalizing', 'completed', 'stale',
                'archived', 'cancelled'
            )),
            status_reason TEXT,
            base_head TEXT NOT NULL DEFAULT '',
            baseline_epoch INTEGER,
            write_scope_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_heartbeat_at TEXT NOT NULL,
            completed_at TEXT,
            archived_at TEXT
        );

        CREATE TABLE events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT REFERENCES sessions(session_id),
            event_type TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );

        CREATE INDEX events_session_created
            ON events(session_id, created_at);

        CREATE TABLE runtime_locks (
            lock_name TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            acquired_at TEXT NOT NULL,
            expires_at TEXT
        );
        """
    )


def _migration_2(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE baseline_epochs (
            epoch_id INTEGER PRIMARY KEY AUTOINCREMENT,
            head_commit TEXT NOT NULL,
            index_tree TEXT NOT NULL,
            health TEXT NOT NULL CHECK (health IN ('healthy', 'degraded')),
            manifest_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            degraded_at TEXT,
            degraded_reason TEXT
        );

        CREATE TABLE objects (
            object_hash TEXT PRIMARY KEY,
            byte_count INTEGER NOT NULL,
            compressed_byte_count INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE snapshots (
            snapshot_id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT REFERENCES sessions(session_id),
            baseline_epoch INTEGER REFERENCES baseline_epochs(epoch_id),
            manifest_json TEXT NOT NULL,
            purpose TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE attributions (
            path_key TEXT PRIMARY KEY,
            display_path TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            baseline_epoch INTEGER REFERENCES baseline_epochs(epoch_id),
            content_hash TEXT,
            attributed_at TEXT NOT NULL
        );

        CREATE TABLE leases (
            path_key TEXT PRIMARY KEY,
            display_path TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            base_hash TEXT,
            acquired_at TEXT NOT NULL,
            last_heartbeat_at TEXT NOT NULL,
            expires_at TEXT NOT NULL
        );

        CREATE INDEX leases_session ON leases(session_id);

        CREATE TABLE patches (
            patch_id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            patch_object_hash TEXT NOT NULL REFERENCES objects(object_hash),
            targets_json TEXT NOT NULL,
            base_hashes_json TEXT NOT NULL,
            base_objects_json TEXT NOT NULL,
            current_objects_json TEXT,
            status TEXT NOT NULL CHECK (status IN (
                'queued', 'applying', 'applied', 'needs_rebase', 'failed', 'cancelled'
            )),
            error_text TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            applied_at TEXT
        );

        CREATE INDEX patches_status_created ON patches(status, created_at, patch_id);

        CREATE TABLE watcher_state (
            path_key TEXT PRIMARY KEY,
            display_path TEXT NOT NULL,
            content_hash TEXT,
            observed_at TEXT NOT NULL
        );
        """
    )


def _migration_3(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE failure_nodes (
            node_id INTEGER PRIMARY KEY AUTOINCREMENT,
            lifecycle_key TEXT NOT NULL,
            artifact_path TEXT NOT NULL UNIQUE,
            kind TEXT NOT NULL CHECK (kind IN ('failure', 'fixed')),
            status TEXT NOT NULL CHECK (status IN ('open', 'fixed')),
            created_at TEXT NOT NULL,
            resolved_at TEXT,
            summary_slug TEXT NOT NULL,
            origin_plan TEXT NOT NULL,
            fixing_plan TEXT NOT NULL,
            origin_child_dir TEXT NOT NULL,
            fixing_child_dir TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 100,
            imported_at TEXT NOT NULL
        );

        CREATE INDEX failure_nodes_lifecycle ON failure_nodes(lifecycle_key);
        CREATE INDEX failure_nodes_fixer_status
            ON failure_nodes(fixing_plan, status, priority, created_at);

        CREATE TABLE failure_diagnostics (
            diagnostic_id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL,
            message TEXT NOT NULL,
            paths_json TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL
        );
        """
    )


def _migration_4(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE cargo_jobs (
            job_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            lane_kind TEXT NOT NULL CHECK (lane_kind IN ('check', 'test', 'workspace', 'gpu')),
            target_dir TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'leased', 'running', 'succeeded', 'failed', 'released', 'orphaned'
            )),
            dry_run INTEGER NOT NULL DEFAULT 0,
            pid INTEGER,
            command_json TEXT NOT NULL DEFAULT '[]',
            exit_code INTEGER,
            created_at TEXT NOT NULL,
            last_heartbeat_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT,
            released_at TEXT
        );

        CREATE UNIQUE INDEX cargo_jobs_active_target
            ON cargo_jobs(target_dir)
            WHERE status IN ('leased', 'running');
        CREATE INDEX cargo_jobs_status_heartbeat
            ON cargo_jobs(status, last_heartbeat_at);
        """
    )


def _migration_5(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE cargo_jobs ADD COLUMN target_key TEXT NOT NULL DEFAULT '';
        UPDATE cargo_jobs
        SET target_key = lower(replace(target_dir, '/', '\\'));
        DROP INDEX cargo_jobs_active_target;
        CREATE UNIQUE INDEX cargo_jobs_active_target
            ON cargo_jobs(target_key)
            WHERE status IN ('leased', 'running');
        """
    )


def _migration_6(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE cleanup_reservations (
            target_key TEXT PRIMARY KEY,
            target_dir TEXT NOT NULL,
            reserved_at TEXT NOT NULL
        );
        """
    )


def _migration_7(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE cleanup_plans (
            plan_id TEXT PRIMARY KEY,
            generated_at TEXT NOT NULL,
            older_than_hours INTEGER NOT NULL CHECK (older_than_hours > 0),
            candidates_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('planned', 'applying', 'applied', 'failed')),
            applied_at TEXT
        );
        """
    )


def _migration_8(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE git_mutex (
            lock_name TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            acquired_at TEXT NOT NULL
        );

        CREATE TABLE finalize_requests (
            request_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            message TEXT NOT NULL,
            paths_json TEXT NOT NULL,
            categories_json TEXT NOT NULL,
            untracked_json TEXT NOT NULL,
            validation_json TEXT NOT NULL DEFAULT '[]',
            maintenance INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL CHECK (status IN (
                'previewed', 'finalizing', 'committed', 'failed'
            )),
            commit_sha TEXT,
            error_text TEXT,
            created_at TEXT NOT NULL,
            completed_at TEXT
        );
        CREATE INDEX finalize_requests_session_created
            ON finalize_requests(session_id, created_at);

        CREATE TABLE validation_copies (
            job_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            job_root TEXT NOT NULL,
            source_root TEXT NOT NULL,
            manifest_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('planned', 'materialized', 'removed', 'failed')),
            created_at TEXT NOT NULL,
            removed_at TEXT
        );
        """
    )


def _migration_9(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE validation_copies
            ADD COLUMN target_root TEXT NOT NULL DEFAULT '';

        CREATE TABLE validation_copy_runs (
            run_id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL REFERENCES validation_copies(job_id),
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            command_json TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            stdout_text TEXT NOT NULL,
            stderr_text TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT NOT NULL
        );
        CREATE INDEX validation_copy_runs_job_started
            ON validation_copy_runs(job_id, started_at);
        """
    )


def _migration_10(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE finalize_requests ADD COLUMN start_head TEXT;
        ALTER TABLE finalize_requests ADD COLUMN index_existed INTEGER;
        ALTER TABLE finalize_requests ADD COLUMN index_snapshot BLOB;

        DROP INDEX validation_copy_runs_job_started;
        ALTER TABLE validation_copy_runs RENAME TO validation_copy_runs_v9;
        ALTER TABLE validation_copies RENAME TO validation_copies_v9;

        CREATE TABLE validation_copies (
            job_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            job_root TEXT NOT NULL,
            source_root TEXT NOT NULL,
            target_root TEXT NOT NULL,
            head_commit TEXT NOT NULL,
            manifest_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'planned', 'materialized', 'running', 'cleanup_pending', 'removed', 'failed'
            )),
            created_at TEXT NOT NULL,
            removed_at TEXT
        );
        INSERT INTO validation_copies(
            job_id, session_id, job_root, source_root, target_root, head_commit,
            manifest_json, status, created_at, removed_at
        )
        SELECT job_id, session_id, job_root, source_root, target_root, '',
               manifest_json, status, created_at, removed_at
        FROM validation_copies_v9;

        CREATE TABLE validation_copy_runs (
            run_id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL REFERENCES validation_copies(job_id),
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            command_json TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            stdout_text TEXT NOT NULL,
            stderr_text TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT NOT NULL
        );
        INSERT INTO validation_copy_runs
        SELECT * FROM validation_copy_runs_v9;
        CREATE INDEX validation_copy_runs_job_started
            ON validation_copy_runs(job_id, started_at);

        DROP TABLE validation_copy_runs_v9;
        DROP TABLE validation_copies_v9;
        """
    )


def _migration_11(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE finalize_requests ADD COLUMN ref_updated_sha TEXT;
        ALTER TABLE validation_copies ADD COLUMN run_pid INTEGER;
        """
    )


def _migration_12(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE legacy_note_imports (
            note_path TEXT PRIMARY KEY,
            content_hash TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            source_status TEXT,
            mapped_status TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL,
            archived_path TEXT
        );
        CREATE INDEX legacy_note_imports_session
            ON legacy_note_imports(session_id);

        CREATE TABLE legacy_archive_runs (
            run_id TEXT PRIMARY KEY,
            candidates_json TEXT NOT NULL,
            manifest_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('planned', 'applied', 'failed')),
            created_at TEXT NOT NULL,
            applied_at TEXT
        );

        CREATE TABLE object_gc_plans (
            plan_id TEXT PRIMARY KEY,
            snapshot_ids_json TEXT NOT NULL,
            object_hashes_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('planned', 'applying', 'applied', 'failed')),
            created_at TEXT NOT NULL,
            applied_at TEXT,
            error_text TEXT
        );

        CREATE TABLE maintenance_ticks (
            tick_id TEXT PRIMARY KEY,
            stale_sessions_json TEXT NOT NULL DEFAULT '[]',
            orphaned_cargo_json TEXT NOT NULL DEFAULT '[]',
            retention_plan_id TEXT,
            cleanup_plan_id TEXT,
            status TEXT NOT NULL CHECK (status IN ('succeeded', 'failed')),
            created_at TEXT NOT NULL,
            error_text TEXT
        );
        """
    )


def _migration_13(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE maintenance_ticks
            ADD COLUMN archived_sessions_json TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE maintenance_ticks
            ADD COLUMN legacy_archive_run_id TEXT;
        """
    )


MIGRATIONS: dict[int, Callable[[Connection], None]] = {
    1: _migration_1,
    2: _migration_2,
    3: _migration_3,
    4: _migration_4,
    5: _migration_5,
    6: _migration_6,
    7: _migration_7,
    8: _migration_8,
    9: _migration_9,
    10: _migration_10,
    11: _migration_11,
    12: _migration_12,
    13: _migration_13,
}


def migrate(database: Database) -> int:
    with database.transaction() as connection:
        connection.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
        )
        applied = {
            int(row[0]) for row in connection.execute("SELECT version FROM schema_version")
        }
        for version in range(1, LATEST_SCHEMA_VERSION + 1):
            if version in applied:
                continue
            MIGRATIONS[version](connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (?, datetime('now'))",
                (version,),
            )
    return LATEST_SCHEMA_VERSION
