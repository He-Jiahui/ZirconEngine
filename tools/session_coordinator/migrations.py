from __future__ import annotations

from collections.abc import Callable
from sqlite3 import Connection

from .database import Database


LATEST_SCHEMA_VERSION = 2


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


MIGRATIONS: dict[int, Callable[[Connection], None]] = {
    1: _migration_1,
    2: _migration_2,
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
