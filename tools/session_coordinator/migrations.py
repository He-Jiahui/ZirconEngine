from __future__ import annotations

from collections.abc import Callable
from sqlite3 import Connection

from .database import Database
from .models import CoordinatorError


LATEST_SCHEMA_VERSION = 16


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


def _migration_14(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE workflow_runs (
            run_id TEXT PRIMARY KEY,
            session_id TEXT REFERENCES sessions(session_id),
            workflow_key TEXT NOT NULL,
            plan_path TEXT,
            topology_hash TEXT,
            state TEXT NOT NULL CHECK (state IN (
                'registered', 'active', 'waiting_dependency', 'waiting_lease',
                'resolving_failure', 'waiting_validation', 'waiting_review',
                'finalizing', 'succeeded', 'failed', 'cancelled', 'stale', 'archived'
            )),
            status_reason TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT,
            UNIQUE(session_id, workflow_key)
        );
        CREATE INDEX workflow_runs_state_updated
            ON workflow_runs(state, updated_at);

        CREATE TABLE workflow_nodes (
            node_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            node_key TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind IN (
                'goal', 'milestone', 'slice', 'validation', 'review',
                'commit', 'notification', 'closeout'
            )),
            title TEXT NOT NULL,
            stage TEXT NOT NULL,
            state TEXT NOT NULL CHECK (state IN (
                'pending', 'ready', 'running', 'waiting_external',
                'succeeded', 'failed', 'cancelled', 'skipped'
            )),
            owner_session_id TEXT REFERENCES sessions(session_id),
            status_reason TEXT,
            attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(run_id, node_key),
            UNIQUE(run_id, node_id)
        );
        CREATE INDEX workflow_nodes_run_stage
            ON workflow_nodes(run_id, stage, node_key);

        CREATE TABLE workflow_edges (
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            from_node_id TEXT NOT NULL,
            to_node_id TEXT NOT NULL,
            edge_kind TEXT NOT NULL CHECK (edge_kind IN ('depends_on', 'failure_dependency')),
            PRIMARY KEY(run_id, from_node_id, to_node_id, edge_kind),
            CHECK(from_node_id <> to_node_id),
            FOREIGN KEY(run_id, from_node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE CASCADE,
            FOREIGN KEY(run_id, to_node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE CASCADE
        );

        CREATE TABLE workflow_attempts (
            attempt_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            attempt_number INTEGER NOT NULL CHECK (attempt_number > 0),
            state TEXT NOT NULL CHECK (state IN (
                'pending', 'ready', 'running', 'waiting_external',
                'succeeded', 'failed', 'cancelled', 'skipped'
            )),
            accepted INTEGER NOT NULL DEFAULT 1 CHECK (accepted IN (0, 1)),
            evidence_json TEXT NOT NULL DEFAULT '{}',
            started_at TEXT NOT NULL,
            completed_at TEXT,
            UNIQUE(node_id, attempt_number),
            UNIQUE(run_id, node_id, attempt_id),
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_attempts_node_number
            ON workflow_attempts(node_id, attempt_number DESC);
        CREATE TRIGGER workflow_attempts_immutable_update
        BEFORE UPDATE ON workflow_attempts
        BEGIN
            SELECT RAISE(ABORT, 'workflow attempts are immutable');
        END;
        CREATE TRIGGER workflow_attempts_immutable_delete
        BEFORE DELETE ON workflow_attempts
        BEGIN
            SELECT RAISE(ABORT, 'workflow attempts are immutable');
        END;

        CREATE TABLE workflow_artifacts (
            artifact_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            node_id TEXT,
            attempt_id TEXT,
            artifact_kind TEXT NOT NULL CHECK (artifact_kind IN (
                'log', 'report', 'screenshot', 'manifest', 'plan_record',
                'failure_handoff', 'fixed_handoff', 'commit', 'other'
            )),
            display_name TEXT NOT NULL,
            storage_path TEXT,
            content_hash TEXT,
            byte_count INTEGER,
            metadata_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            CHECK(attempt_id IS NULL OR node_id IS NOT NULL),
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id, attempt_id)
                REFERENCES workflow_attempts(run_id, node_id, attempt_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_artifacts_run_created
            ON workflow_artifacts(run_id, created_at);

        CREATE TABLE workflow_diagnostics (
            diagnostic_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            node_id TEXT REFERENCES workflow_nodes(node_id) ON DELETE CASCADE,
            code TEXT NOT NULL,
            severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'error')),
            message TEXT NOT NULL,
            applicable INTEGER NOT NULL DEFAULT 1 CHECK (applicable IN (0, 1)),
            details_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_diagnostics_run_applicable
            ON workflow_diagnostics(run_id, applicable, resolved_at);

        CREATE TABLE web_control_sessions (
            session_token_hash TEXT PRIMARY KEY,
            session_id TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL CHECK (role IN (
                'observer', 'operator', 'committer', 'maintainer'
            )),
            actor TEXT NOT NULL,
            daemon_instance_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_seen_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            revoked_at TEXT
        );
        CREATE INDEX web_control_sessions_instance_expiry
            ON web_control_sessions(daemon_instance_id, expires_at);

        CREATE TABLE web_bootstrap_tickets (
            ticket_hash TEXT PRIMARY KEY,
            role TEXT NOT NULL CHECK (role IN (
                'observer', 'operator', 'committer', 'maintainer'
            )),
            actor TEXT NOT NULL,
            daemon_instance_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            consumed_at TEXT
        );
        CREATE INDEX web_bootstrap_tickets_instance_expiry
            ON web_bootstrap_tickets(daemon_instance_id, expires_at);
        """
    )


def _migration_15(connection: Connection) -> None:
    connection.executescript(
        """
        ALTER TABLE web_control_sessions ADD COLUMN bound_session_id TEXT;
        ALTER TABLE web_control_sessions ADD COLUMN csrf_token_hash TEXT;
        ALTER TABLE web_control_sessions ADD COLUMN elevated_until TEXT;

        CREATE TABLE web_elevation_grants (
            grant_hash TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN (
                'operator', 'committer', 'maintainer'
            )),
            bound_session_id TEXT,
            daemon_instance_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            consumed_at TEXT
        );
        CREATE INDEX web_elevation_grants_instance_expiry
            ON web_elevation_grants(daemon_instance_id, expires_at);

        CREATE TABLE action_requests (
            action_id TEXT PRIMARY KEY,
            action_kind TEXT NOT NULL CHECK (action_kind IN (
                'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
                'lease.release_own', 'patch.process_own', 'validation.start',
                'validation.cancel', 'failure.refresh', 'topology.refresh',
                'service.drain_preview', 'milestone.commit', 'session.complete',
                'service.restart', 'maintenance.cleanup'
            )),
            risk TEXT NOT NULL CHECK (risk IN ('green', 'yellow', 'red')),
            required_role TEXT NOT NULL CHECK (required_role IN (
                'observer', 'operator', 'committer', 'maintainer'
            )),
            actor TEXT NOT NULL,
            web_session_id TEXT,
            bound_session_id TEXT,
            daemon_instance_id TEXT NOT NULL,
            parameters_json TEXT NOT NULL,
            impact_json TEXT NOT NULL,
            warnings_json TEXT NOT NULL,
            state_fingerprint TEXT NOT NULL,
            confirmation_phrase_hash TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'previewed', 'executing', 'succeeded', 'failed', 'cancelled',
                'expired', 'state_changed', 'denied'
            )),
            reason TEXT,
            result_json TEXT,
            error_code TEXT,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            confirmed_at TEXT,
            completed_at TEXT
        );
        CREATE INDEX action_requests_actor_created
            ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry
            ON action_requests(status, expires_at);

        CREATE TABLE action_approvals (
            approval_id TEXT PRIMARY KEY,
            action_id TEXT NOT NULL REFERENCES action_requests(action_id),
            actor TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN (
                'observer', 'operator', 'committer', 'maintainer'
            )),
            reason TEXT NOT NULL,
            state_fingerprint TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX action_approvals_action ON action_approvals(action_id);
        CREATE TRIGGER action_approvals_no_update
        BEFORE UPDATE ON action_approvals
        BEGIN
            SELECT RAISE(ABORT, 'action approvals are immutable');
        END;
        CREATE TRIGGER action_approvals_no_delete
        BEFORE DELETE ON action_approvals
        BEGIN
            SELECT RAISE(ABORT, 'action approvals are immutable');
        END;
        """
    )


def _migration_16(connection: Connection) -> None:
    """Enforce the M3 action enum for databases that applied early schema 15."""
    allowed = """
        'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
        'lease.release_own', 'patch.process_own', 'validation.start',
        'validation.cancel', 'failure.refresh', 'topology.refresh',
        'service.drain_preview', 'milestone.commit', 'session.complete',
        'service.restart', 'maintenance.cleanup'
    """
    connection.executescript(
        f"""
        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({allowed})
        BEGIN
            SELECT RAISE(ABORT, 'invalid controlled action kind');
        END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({allowed})
        BEGIN
            SELECT RAISE(ABORT, 'invalid controlled action kind');
        END;
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
    14: _migration_14,
    15: _migration_15,
    16: _migration_16,
}


def migrate(database: Database) -> int:
    with database.transaction() as connection:
        connection.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)"
        )
        applied = {
            int(row[0]) for row in connection.execute("SELECT version FROM schema_version")
        }
        newer_versions = sorted(
            version for version in applied if version > LATEST_SCHEMA_VERSION
        )
        if newer_versions:
            raise CoordinatorError(
                "schema_version_newer",
                "Coordinator database was created by a newer service version",
                details={"versions": newer_versions, "supported": LATEST_SCHEMA_VERSION},
            )
        for version in range(1, LATEST_SCHEMA_VERSION + 1):
            if version in applied:
                continue
            MIGRATIONS[version](connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (?, datetime('now'))",
                (version,),
            )
    return LATEST_SCHEMA_VERSION
