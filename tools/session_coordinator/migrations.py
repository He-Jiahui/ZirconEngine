from __future__ import annotations

import json
from collections.abc import Callable
from sqlite3 import Connection

from .database import Database
from .benchmark_validation_schema import migrate_benchmark_validation_schema
from .event_payloads import (
    MAX_CONTROL_EVENT_PAYLOAD_BYTES,
    encode_oversized_event_payload,
)
from .models import CoordinatorError
from .supervision.migration import migrate_supervision_schema


LATEST_SCHEMA_VERSION = 64


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


def _migration_17(connection: Connection) -> None:
    """Add versioned milestone topology, gate, review, and notification evidence.

    Schema 16 was already released for the M3 closed action enum.  Keeping this
    as a new migration avoids silently giving two databases with version 16
    different layouts.
    """
    connection.executescript(
        """
        ALTER TABLE workflow_runs ADD COLUMN current_topology_version_id TEXT;

        CREATE TABLE workflow_topology_versions (
            topology_version_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            version_number INTEGER NOT NULL CHECK (version_number > 0),
            plan_path TEXT NOT NULL,
            plan_id TEXT NOT NULL,
            schema_version INTEGER NOT NULL CHECK (schema_version > 0),
            source_kind TEXT NOT NULL CHECK (source_kind IN ('zircon-workflow', 'headings')),
            content_hash TEXT NOT NULL,
            topology_hash TEXT NOT NULL,
            topology_json TEXT NOT NULL,
            supersedes_id TEXT REFERENCES workflow_topology_versions(topology_version_id),
            created_at TEXT NOT NULL,
            UNIQUE(run_id, version_number),
            UNIQUE(run_id, content_hash),
            UNIQUE(run_id, topology_version_id)
        );
        CREATE INDEX workflow_topology_versions_run_created
            ON workflow_topology_versions(run_id, created_at);
        CREATE TRIGGER workflow_topology_versions_no_update
        BEFORE UPDATE ON workflow_topology_versions
        BEGIN
            SELECT RAISE(ABORT, 'workflow topology versions are immutable');
        END;
        CREATE TRIGGER workflow_topology_versions_no_delete
        BEFORE DELETE ON workflow_topology_versions
        BEGIN
            SELECT RAISE(ABORT, 'workflow topology versions are immutable');
        END;

        CREATE TABLE workflow_gate_evidence (
            evidence_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            topology_version_id TEXT NOT NULL
                REFERENCES workflow_topology_versions(topology_version_id) ON DELETE RESTRICT,
            node_id TEXT,
            attempt_id TEXT,
            gate_kind TEXT NOT NULL CHECK (gate_kind IN (
                'dependencies', 'slices', 'validation', 'review',
                'failure_audit', 'plan_output', 'commit_manifest'
            )),
            decision TEXT NOT NULL CHECK (decision IN ('accepted', 'rejected', 'stale')),
            decision_code TEXT NOT NULL,
            input_fingerprint TEXT NOT NULL,
            evidence_hash TEXT NOT NULL,
            blocking_node_ids_json TEXT NOT NULL DEFAULT '[]',
            applicable_failure_ids_json TEXT NOT NULL DEFAULT '[]',
            required_evidence_json TEXT NOT NULL DEFAULT '[]',
            payload_json TEXT NOT NULL DEFAULT '{}',
            source_revision TEXT,
            actor TEXT NOT NULL,
            action_id TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id, attempt_id)
                REFERENCES workflow_attempts(run_id, node_id, attempt_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT
        );
        CREATE INDEX workflow_gate_evidence_run_gate_created
            ON workflow_gate_evidence(run_id, gate_kind, created_at DESC);
        CREATE TRIGGER workflow_gate_evidence_no_update
        BEFORE UPDATE ON workflow_gate_evidence
        BEGIN
            SELECT RAISE(ABORT, 'workflow gate evidence is immutable');
        END;
        CREATE TRIGGER workflow_gate_evidence_no_delete
        BEFORE DELETE ON workflow_gate_evidence
        BEGIN
            SELECT RAISE(ABORT, 'workflow gate evidence is immutable');
        END;

        CREATE TABLE workflow_review_evidence (
            review_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE CASCADE,
            topology_version_id TEXT NOT NULL
                REFERENCES workflow_topology_versions(topology_version_id) ON DELETE RESTRICT,
            node_id TEXT,
            attempt_id TEXT,
            reviewer TEXT NOT NULL,
            executor TEXT NOT NULL,
            verdict TEXT NOT NULL CHECK (verdict IN ('accepted', 'rejected')),
            critical_count INTEGER NOT NULL CHECK (critical_count >= 0),
            important_count INTEGER NOT NULL CHECK (important_count >= 0),
            evidence_hash TEXT NOT NULL,
            input_fingerprint TEXT NOT NULL,
            summary TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id, attempt_id)
                REFERENCES workflow_attempts(run_id, node_id, attempt_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT,
            CHECK(reviewer <> executor)
        );
        CREATE INDEX workflow_review_evidence_run_created
            ON workflow_review_evidence(run_id, created_at DESC);
        CREATE TRIGGER workflow_review_evidence_no_update
        BEFORE UPDATE ON workflow_review_evidence
        BEGIN
            SELECT RAISE(ABORT, 'workflow review evidence is immutable');
        END;
        CREATE TRIGGER workflow_review_evidence_no_delete
        BEFORE DELETE ON workflow_review_evidence
        BEGIN
            SELECT RAISE(ABORT, 'workflow review evidence is immutable');
        END;

        CREATE TABLE notification_attempts (
            notification_attempt_id TEXT PRIMARY KEY,
            run_id TEXT REFERENCES workflow_runs(run_id) ON DELETE SET NULL,
            topology_version_id TEXT
                REFERENCES workflow_topology_versions(topology_version_id) ON DELETE SET NULL,
            node_id TEXT,
            action_id TEXT,
            commit_sha TEXT NOT NULL,
            channel TEXT NOT NULL CHECK (channel IN ('wecom')),
            status TEXT NOT NULL CHECK (status IN ('reserved', 'succeeded', 'failed', 'unknown')),
            message_hash TEXT NOT NULL,
            attempted_at TEXT NOT NULL,
            completed_at TEXT,
            exit_code INTEGER,
            provider_errcode TEXT,
            sanitized_error TEXT,
            UNIQUE(commit_sha, channel),
            CHECK(topology_version_id IS NULL OR run_id IS NOT NULL),
            CHECK(node_id IS NULL OR run_id IS NOT NULL),
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT
        );
        CREATE INDEX notification_attempts_run_time
            ON notification_attempts(run_id, attempted_at DESC);
        CREATE TRIGGER notification_attempts_terminal_update_only
        BEFORE UPDATE ON notification_attempts
        WHEN OLD.status <> 'reserved'
          OR NEW.status NOT IN ('succeeded', 'failed', 'unknown')
          OR NEW.notification_attempt_id <> OLD.notification_attempt_id
          OR NEW.run_id IS NOT OLD.run_id
          OR NEW.topology_version_id IS NOT OLD.topology_version_id
          OR NEW.node_id IS NOT OLD.node_id
          OR NEW.action_id IS NOT OLD.action_id
          OR NEW.commit_sha <> OLD.commit_sha
          OR NEW.channel <> OLD.channel
          OR NEW.message_hash <> OLD.message_hash
          OR NEW.attempted_at <> OLD.attempted_at
        BEGIN
            SELECT RAISE(ABORT, 'notification reservation may transition once to a terminal result');
        END;
        CREATE TRIGGER notification_attempts_no_delete
        BEFORE DELETE ON notification_attempts
        BEGIN
            SELECT RAISE(ABORT, 'notification attempts are immutable');
        END;

        CREATE TABLE workflow_commit_intents (
            intent_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE RESTRICT,
            topology_version_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE RESTRICT,
            action_id TEXT,
            actor TEXT NOT NULL,
            gate_fingerprint TEXT NOT NULL,
            paths_json TEXT NOT NULL,
            message TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'prepared', 'committed', 'reconciled', 'failed'
            )),
            commit_sha TEXT,
            error_text TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_commit_intents_status_created
            ON workflow_commit_intents(status, created_at);

        CREATE TABLE workflow_validation_bindings (
            validation_run_id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL REFERENCES validation_copies(job_id) ON DELETE RESTRICT,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE RESTRICT,
            topology_version_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE RESTRICT,
            template TEXT NOT NULL CHECK (template IN ('coordinator-actions', 'web-check')),
            input_fingerprint TEXT NOT NULL,
            action_id TEXT,
            actor TEXT NOT NULL,
            created_at TEXT NOT NULL,
            imported_at TEXT,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_validation_bindings_pending
            ON workflow_validation_bindings(imported_at, created_at);
        """
    )


def _migration_18(connection: Connection) -> None:
    connection.executescript(
        """
        CREATE TABLE workflow_milestone_manifests (
            manifest_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES workflow_runs(run_id) ON DELETE RESTRICT,
            topology_version_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE RESTRICT,
            paths_json TEXT NOT NULL,
            manifest_hash TEXT NOT NULL,
            actor TEXT NOT NULL,
            action_id TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id, topology_version_id)
                REFERENCES workflow_topology_versions(run_id, topology_version_id)
                ON DELETE RESTRICT,
            FOREIGN KEY(run_id, node_id)
                REFERENCES workflow_nodes(run_id, node_id) ON DELETE RESTRICT
        );
        CREATE INDEX workflow_milestone_manifests_node_created
            ON workflow_milestone_manifests(run_id, node_id, created_at DESC, manifest_id DESC);
        CREATE TRIGGER workflow_milestone_manifests_no_update
        BEFORE UPDATE ON workflow_milestone_manifests
        BEGIN
            SELECT RAISE(ABORT, 'workflow milestone manifests are immutable');
        END;
        CREATE TRIGGER workflow_milestone_manifests_no_delete
        BEFORE DELETE ON workflow_milestone_manifests
        BEGIN
            SELECT RAISE(ABORT, 'workflow milestone manifests are immutable');
        END;

        CREATE TRIGGER workflow_runs_topology_same_run_insert
        BEFORE INSERT ON workflow_runs
        WHEN NEW.current_topology_version_id IS NOT NULL
         AND NOT EXISTS (
             SELECT 1 FROM workflow_topology_versions version
             WHERE version.topology_version_id=NEW.current_topology_version_id
               AND version.run_id=NEW.run_id
         )
        BEGIN
            SELECT RAISE(ABORT, 'active topology version must belong to workflow run');
        END;
        CREATE TRIGGER workflow_runs_topology_same_run_update
        BEFORE UPDATE OF current_topology_version_id ON workflow_runs
        WHEN NEW.current_topology_version_id IS NOT NULL
         AND NOT EXISTS (
             SELECT 1 FROM workflow_topology_versions version
             WHERE version.topology_version_id=NEW.current_topology_version_id
               AND version.run_id=NEW.run_id
         )
        BEGIN
            SELECT RAISE(ABORT, 'active topology version must belong to workflow run');
        END;
        CREATE TRIGGER workflow_topology_supersedes_same_run_insert
        BEFORE INSERT ON workflow_topology_versions
        WHEN NEW.supersedes_id IS NOT NULL
         AND NOT EXISTS (
             SELECT 1 FROM workflow_topology_versions previous
             WHERE previous.topology_version_id=NEW.supersedes_id
               AND previous.run_id=NEW.run_id
         )
        BEGIN
            SELECT RAISE(ABORT, 'superseded topology version must belong to workflow run');
        END;
        """
    )


def _migration_19(connection: Connection) -> None:
    """Upgrade already-released v17/v18 databases without rewriting history."""
    connection.executescript(
        """
        ALTER TABLE workflow_validation_bindings ADD COLUMN source_manifest_hash TEXT;
        ALTER TABLE workflow_validation_bindings ADD COLUMN paths_json TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE workflow_validation_bindings ADD COLUMN terminal_status TEXT NOT NULL
            DEFAULT 'pending' CHECK (terminal_status IN ('pending', 'accepted', 'rejected'));
        ALTER TABLE workflow_validation_bindings ADD COLUMN terminal_code TEXT;

        CREATE UNIQUE INDEX workflow_milestone_manifest_single_binding
            ON workflow_milestone_manifests(run_id, topology_version_id, node_id);

        CREATE TRIGGER workflow_review_registered_sessions_insert
        BEFORE INSERT ON workflow_review_evidence
        WHEN NOT EXISTS (SELECT 1 FROM sessions WHERE session_id=NEW.reviewer)
          OR NOT EXISTS (SELECT 1 FROM sessions WHERE session_id=NEW.executor)
        BEGIN
            SELECT RAISE(ABORT, 'reviewer and executor must be registered Sessions');
        END;
        """
    )


def _migration_21(connection: Connection) -> None:
    """Track reusable Cargo cache identity and deterministic cleanup state."""
    connection.executescript(
        """
        ALTER TABLE cargo_jobs ADD COLUMN reuse_key TEXT;
        ALTER TABLE cargo_jobs ADD COLUMN compatibility_json TEXT;
        ALTER TABLE cargo_jobs ADD COLUMN compatibility_key TEXT;
        ALTER TABLE cargo_jobs ADD COLUMN reuse_profile TEXT;
        ALTER TABLE cargo_jobs ADD COLUMN cleanup_policy TEXT NOT NULL DEFAULT 'retained'
            CHECK (cleanup_policy IN ('retained', 'delete_on_release'));
        ALTER TABLE cargo_jobs ADD COLUMN cleanup_status TEXT NOT NULL DEFAULT 'retained'
            CHECK (cleanup_status IN ('retained', 'pending', 'deleted', 'failed'));
        ALTER TABLE cargo_jobs ADD COLUMN reused_from_job_id TEXT REFERENCES cargo_jobs(job_id);
        ALTER TABLE cargo_jobs ADD COLUMN cleanup_error TEXT;

        CREATE INDEX cargo_jobs_reuse_lookup
            ON cargo_jobs(reuse_key, status, released_at);
        CREATE UNIQUE INDEX cargo_jobs_active_reuse_key
            ON cargo_jobs(reuse_key)
            WHERE reuse_key IS NOT NULL AND status IN ('leased', 'running');
        CREATE INDEX cargo_jobs_compatibility_cleanup
            ON cargo_jobs(lane_kind, compatibility_key, status, released_at);
        CREATE INDEX cargo_jobs_cleanup_pending
            ON cargo_jobs(cleanup_status, released_at);
        """
    )


def _migration_22(connection: Connection) -> None:
    """Repair databases whose historical v21 marker predates Cargo pool metadata."""
    columns = {
        str(row[1]) for row in connection.execute("PRAGMA table_info(cargo_jobs)")
    }
    additions = {
        "reuse_key": "TEXT",
        "compatibility_json": "TEXT",
        "compatibility_key": "TEXT",
        "reuse_profile": "TEXT",
        "cleanup_policy": (
            "TEXT NOT NULL DEFAULT 'retained' "
            "CHECK (cleanup_policy IN ('retained', 'delete_on_release'))"
        ),
        "cleanup_status": (
            "TEXT NOT NULL DEFAULT 'retained' "
            "CHECK (cleanup_status IN ('retained', 'pending', 'deleted', 'failed'))"
        ),
        "reused_from_job_id": "TEXT REFERENCES cargo_jobs(job_id)",
        "cleanup_error": "TEXT",
    }
    for column, declaration in additions.items():
        if column not in columns:
            connection.execute(
                f"ALTER TABLE cargo_jobs ADD COLUMN {column} {declaration}"
            )
    connection.executescript(
        """
        CREATE INDEX IF NOT EXISTS cargo_jobs_reuse_lookup
            ON cargo_jobs(reuse_key, status, released_at);
        CREATE UNIQUE INDEX IF NOT EXISTS cargo_jobs_active_reuse_key
            ON cargo_jobs(reuse_key)
            WHERE reuse_key IS NOT NULL AND status IN ('leased', 'running');
        CREATE INDEX IF NOT EXISTS cargo_jobs_compatibility_cleanup
            ON cargo_jobs(lane_kind, compatibility_key, status, released_at);
        CREATE INDEX IF NOT EXISTS cargo_jobs_cleanup_pending
            ON cargo_jobs(cleanup_status, released_at);
        """
    )


def _migration_23(connection: Connection) -> None:
    """Fail historical Cargo rows without a complete reuse identity to ephemeral."""
    connection.execute(
        """
        UPDATE cargo_jobs
        SET cleanup_policy='delete_on_release', cleanup_status='pending', cleanup_error=NULL
        WHERE cleanup_policy='retained' AND cleanup_status='retained'
          AND (
              reuse_key IS NULL OR compatibility_json IS NULL
              OR compatibility_key IS NULL OR reuse_profile IS NULL
          )
        """
    )


def _migration_24(connection: Connection) -> None:
    """Compact legacy event payloads that exceed the control-plane contract."""
    oversized = connection.execute(
        """
        SELECT event_id, LENGTH(CAST(payload_json AS BLOB))
        FROM events
        WHERE LENGTH(CAST(payload_json AS BLOB)) > ?
        """,
        (MAX_CONTROL_EVENT_PAYLOAD_BYTES,),
    ).fetchall()
    connection.executemany(
        "UPDATE events SET payload_json=? WHERE event_id=?",
        (
            (
                encode_oversized_event_payload(
                    int(original_bytes),
                    reason="legacy_event_payload_compacted",
                ),
                int(event_id),
            )
            for event_id, original_bytes in oversized
        ),
    )


def _migration_25(connection: Connection) -> None:
    """Mark the one-time physical compaction that follows bounded event migration."""
    connection.execute("SELECT 1")


def _migration_26(connection: Connection) -> None:
    """Repair historical lifecycle conflicts, then enforce one active drain."""
    conflicts = connection.execute(
        """
        SELECT repository_key
        FROM service_lifecycle_intents
        WHERE kind IN ('service.stop', 'service.restart', 'service.force_stop')
          AND status IN ('accepted', 'draining')
        GROUP BY repository_key
        HAVING COUNT(*) > 1
        ORDER BY repository_key
        """
    ).fetchall()
    for conflict in conflicts:
        repository_key = str(conflict[0])
        rows = connection.execute(
            """
            SELECT intent_id, action_id
            FROM service_lifecycle_intents
            WHERE repository_key=?
              AND kind IN ('service.stop', 'service.restart', 'service.force_stop')
              AND status IN ('accepted', 'draining')
            ORDER BY intent_id
            """,
            (repository_key,),
        ).fetchall()
        intent_ids = [str(row[0]) for row in rows]
        action_ids = sorted(str(row[1]) for row in rows if row[1] is not None)
        result = json.dumps(
            {
                "errorCode": "migration.lifecycle_conflict",
                "intentIds": intent_ids,
                "repositoryKey": repository_key,
                "resolution": "all_conflicting_lifecycles_failed",
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        connection.execute(
            """
            UPDATE service_lifecycle_intents
            SET status='failed', error_code='migration.lifecycle_conflict',
                result_json=?, updated_at=datetime('now'), completed_at=datetime('now')
            WHERE repository_key=?
              AND kind IN ('service.stop', 'service.restart', 'service.force_stop')
              AND status IN ('accepted', 'draining')
            """,
            (result, repository_key),
        )
        if action_ids:
            placeholders = ",".join("?" for _ in action_ids)
            connection.execute(
                f"""
                UPDATE action_requests
                SET status='failed', error_code='migration.lifecycle_conflict',
                    reason='Schema 26 safely terminated conflicting lifecycle actions',
                    result_json=?, completed_at=datetime('now')
                WHERE action_id IN ({placeholders})
                  AND status IN ('previewed', 'executing')
                """,
                (result, *action_ids),
            )
        connection.execute(
            "INSERT INTO events(event_type, payload_json, created_at) "
            "VALUES ('schema.lifecycle_conflict_repaired', ?, datetime('now'))",
            (
                json.dumps(
                    {
                        "actionIds": action_ids,
                        "errorCode": "migration.lifecycle_conflict",
                        "intentIds": intent_ids,
                        "repositoryKey": repository_key,
                        "resolution": "all_conflicting_lifecycles_failed",
                    },
                    sort_keys=True,
                    separators=(",", ":"),
                ),
            ),
        )
    connection.execute(
        """
        CREATE UNIQUE INDEX service_lifecycle_one_active_reversible
        ON service_lifecycle_intents(repository_key)
        WHERE kind IN ('service.stop', 'service.restart', 'service.force_stop')
          AND status IN ('accepted', 'draining')
        """
    )


def _migration_27(connection: Connection) -> None:
    """Add privacy-bounded Codex source projections and reconciliation audit."""
    connection.executescript(
        """
        CREATE TABLE codex_sessions (
            thread_id TEXT PRIMARY KEY,
            rollout_path TEXT NOT NULL,
            source_location TEXT NOT NULL CHECK (
                source_location IN ('active', 'archived', 'missing')
            ),
            state TEXT NOT NULL CHECK (
                state IN ('active', 'idle', 'archived', 'unavailable')
            ),
            cwd TEXT NOT NULL,
            originator TEXT,
            cli_version TEXT,
            thread_source TEXT,
            last_event TEXT NOT NULL CHECK (last_event IN (
                'session_meta', 'task_started', 'task_completed', 'turn_aborted',
                'session_start', 'user_prompt_submit', 'stop',
                'subagent_start', 'subagent_stop', 'unknown'
            )),
            last_turn_id TEXT,
            bound_session_id TEXT REFERENCES sessions(session_id) ON DELETE SET NULL,
            diagnostic_code TEXT,
            first_seen_at TEXT NOT NULL,
            last_activity_at TEXT NOT NULL,
            last_synced_at TEXT NOT NULL,
            source_mtime_ns INTEGER NOT NULL CHECK (source_mtime_ns >= 0),
            source_size INTEGER NOT NULL CHECK (source_size >= 0),
            missing_scan_count INTEGER NOT NULL DEFAULT 0 CHECK (missing_scan_count >= 0),
            CHECK (
                (source_location='missing' AND state='unavailable') OR
                (source_location='archived' AND state='archived') OR
                (source_location='active' AND state IN ('active', 'idle'))
            )
        );

        CREATE INDEX codex_sessions_state_activity
            ON codex_sessions(state, last_activity_at DESC, thread_id);
        CREATE INDEX codex_sessions_bound_session
            ON codex_sessions(bound_session_id)
            WHERE bound_session_id IS NOT NULL;

        CREATE TABLE codex_sync_runs (
            run_id TEXT PRIMARY KEY,
            trigger_kind TEXT NOT NULL CHECK (
                trigger_kind IN ('startup', 'periodic', 'hook', 'controlled')
            ),
            status TEXT NOT NULL CHECK (
                status IN ('running', 'succeeded', 'partial', 'failed')
            ),
            scanned_count INTEGER NOT NULL CHECK (scanned_count >= 0),
            changed_count INTEGER NOT NULL CHECK (changed_count >= 0),
            diagnostic_count INTEGER NOT NULL CHECK (diagnostic_count >= 0),
            unavailable_count INTEGER NOT NULL CHECK (unavailable_count >= 0),
            duration_ms INTEGER NOT NULL CHECK (duration_ms >= 0),
            source_revision TEXT NOT NULL,
            error_code TEXT,
            created_at TEXT NOT NULL,
            completed_at TEXT
        );

        CREATE INDEX codex_sync_runs_created
            ON codex_sync_runs(created_at DESC, run_id);
        """
    )


def _migration_28(connection: Connection) -> None:
    """Extend the closed action audit enum without breaking dependent history."""
    action_kinds = """
        'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
        'lease.release_own', 'patch.process_own', 'validation.start',
        'validation.cancel', 'failure.refresh', 'topology.refresh',
        'service.drain_preview', 'service.drain', 'service.resume',
        'service.stop', 'service.restart', 'service.force_stop',
        'milestone.commit', 'session.complete', 'maintenance.cleanup',
        'codex.sessions.reconcile'
    """
    connection.executescript(
        f"""
        DROP TRIGGER action_requests_kind_insert;
        DROP TRIGGER action_requests_kind_update;
        DROP TRIGGER action_approvals_no_update;
        DROP TRIGGER action_approvals_no_delete;
        DROP TRIGGER service_supervision_events_no_update;
        DROP TRIGGER service_supervision_events_no_delete;
        DROP INDEX action_requests_actor_created;
        DROP INDEX action_requests_status_expiry;
        DROP INDEX action_approvals_action;
        DROP INDEX service_supervision_events_repository_created;
        DROP INDEX service_lifecycle_intents_repository_status;
        DROP INDEX service_lifecycle_one_active_reversible;

        ALTER TABLE action_approvals RENAME TO action_approvals_v27;
        ALTER TABLE service_supervision_events RENAME TO service_supervision_events_v27;
        ALTER TABLE service_lifecycle_intents RENAME TO service_lifecycle_intents_v27;
        ALTER TABLE action_requests RENAME TO action_requests_v27;

        CREATE TABLE action_requests (
            action_id TEXT PRIMARY KEY,
            action_kind TEXT NOT NULL CHECK (action_kind IN ({action_kinds})),
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
        CREATE TABLE service_supervision_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            repository_key TEXT NOT NULL,
            sequence INTEGER NOT NULL CHECK (sequence > 0),
            from_state TEXT CHECK (from_state IS NULL OR from_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            to_state TEXT NOT NULL CHECK (to_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            daemon_instance_id TEXT,
            process_id INTEGER CHECK (process_id IS NULL OR process_id > 0),
            process_creation_time TEXT,
            reason_code TEXT NOT NULL,
            actor TEXT,
            action_id TEXT REFERENCES action_requests(action_id),
            payload_json TEXT NOT NULL DEFAULT '{{}}',
            created_at TEXT NOT NULL,
            UNIQUE(repository_key, sequence)
        );
        CREATE TABLE service_lifecycle_intents (
            intent_id TEXT PRIMARY KEY,
            repository_key TEXT NOT NULL,
            action_id TEXT UNIQUE REFERENCES action_requests(action_id),
            kind TEXT NOT NULL CHECK (kind IN (
                'service.drain', 'service.resume', 'service.stop',
                'service.restart', 'service.force_stop'
            )),
            status TEXT NOT NULL CHECK (status IN (
                'accepted', 'draining', 'ready', 'stopping', 'awaiting_restart',
                'succeeded', 'failed', 'cancelled'
            )),
            requested_by TEXT NOT NULL,
            source_daemon_instance_id TEXT NOT NULL,
            successor_daemon_instance_id TEXT,
            deadline_at TEXT,
            error_code TEXT,
            result_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );

        INSERT INTO action_requests SELECT * FROM action_requests_v27;
        INSERT INTO action_approvals SELECT * FROM action_approvals_v27;
        INSERT INTO service_supervision_events SELECT * FROM service_supervision_events_v27;
        INSERT INTO service_lifecycle_intents SELECT * FROM service_lifecycle_intents_v27;
        DROP TABLE action_approvals_v27;
        DROP TABLE service_supervision_events_v27;
        DROP TABLE service_lifecycle_intents_v27;
        DROP TABLE action_requests_v27;

        CREATE INDEX action_requests_actor_created ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry ON action_requests(status, expires_at);
        CREATE INDEX action_approvals_action ON action_approvals(action_id);
        CREATE INDEX service_supervision_events_repository_created
            ON service_supervision_events(repository_key, created_at);
        CREATE INDEX service_lifecycle_intents_repository_status
            ON service_lifecycle_intents(repository_key, status, updated_at);
        CREATE UNIQUE INDEX service_lifecycle_one_active_reversible
            ON service_lifecycle_intents(repository_key)
            WHERE kind IN ('service.stop', 'service.restart', 'service.force_stop')
              AND status IN ('accepted', 'draining');

        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_approvals_no_update BEFORE UPDATE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER action_approvals_no_delete BEFORE DELETE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_update
        BEFORE UPDATE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_delete
        BEFORE DELETE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        """
    )


def _migration_29(connection: Connection) -> None:
    """Persist an in-progress copy marker without rewriting validation history."""
    connection.execute(
        "ALTER TABLE validation_copies ADD COLUMN materialization_started_at TEXT"
    )


def _migration_30(connection: Connection) -> None:
    """Persist every Cargo process-tree exit observation before pool reuse."""
    connection.executescript(
        """
        ALTER TABLE cargo_jobs ADD COLUMN process_tree_observed_at TEXT;
        ALTER TABLE cargo_jobs ADD COLUMN process_tree_live_pids_json TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE cargo_jobs ADD COLUMN process_tree_exited_at TEXT;
        """
    )


def _migration_31(connection: Connection) -> None:
    """Persist the Cargo root creation identity for PID-reuse-safe observation."""
    connection.execute("ALTER TABLE cargo_jobs ADD COLUMN root_process_creation_time TEXT")


def _migration_32(connection: Connection) -> None:
    """Distinguish actual Cargo roots from wrapper processes that supervise Cargo."""
    connection.execute(
        "ALTER TABLE cargo_jobs ADD COLUMN root_process_kind TEXT NOT NULL DEFAULT 'cargo'"
    )


def _migration_33(connection: Connection) -> None:
    """Add the audited immutable-evidence reconciliation action kind."""
    action_kinds = """
        'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
        'lease.release_own', 'patch.process_own', 'validation.start',
        'validation.cancel', 'failure.refresh', 'topology.refresh',
        'service.drain_preview', 'service.drain', 'service.resume',
        'service.stop', 'service.restart', 'service.force_stop',
        'milestone.commit', 'milestone.reconcile_accepted', 'session.complete',
        'maintenance.cleanup', 'codex.sessions.reconcile'
    """
    connection.executescript(
        f"""
        DROP TRIGGER action_requests_kind_insert;
        DROP TRIGGER action_requests_kind_update;
        DROP TRIGGER action_approvals_no_update;
        DROP TRIGGER action_approvals_no_delete;
        DROP TRIGGER service_supervision_events_no_update;
        DROP TRIGGER service_supervision_events_no_delete;
        DROP INDEX action_requests_actor_created;
        DROP INDEX action_requests_status_expiry;
        DROP INDEX action_approvals_action;
        DROP INDEX service_supervision_events_repository_created;
        DROP INDEX service_lifecycle_intents_repository_status;
        DROP INDEX service_lifecycle_one_active_reversible;

        ALTER TABLE action_approvals RENAME TO action_approvals_v32;
        ALTER TABLE service_supervision_events RENAME TO service_supervision_events_v32;
        ALTER TABLE service_lifecycle_intents RENAME TO service_lifecycle_intents_v32;
        ALTER TABLE action_requests RENAME TO action_requests_v32;

        CREATE TABLE action_requests (
            action_id TEXT PRIMARY KEY,
            action_kind TEXT NOT NULL CHECK (action_kind IN ({action_kinds})),
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
        CREATE TABLE service_supervision_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            repository_key TEXT NOT NULL,
            sequence INTEGER NOT NULL CHECK (sequence > 0),
            from_state TEXT CHECK (from_state IS NULL OR from_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            to_state TEXT NOT NULL CHECK (to_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            daemon_instance_id TEXT,
            process_id INTEGER CHECK (process_id IS NULL OR process_id > 0),
            process_creation_time TEXT,
            reason_code TEXT NOT NULL,
            actor TEXT,
            action_id TEXT REFERENCES action_requests(action_id),
            payload_json TEXT NOT NULL DEFAULT '{{}}',
            created_at TEXT NOT NULL,
            UNIQUE(repository_key, sequence)
        );
        CREATE TABLE service_lifecycle_intents (
            intent_id TEXT PRIMARY KEY,
            repository_key TEXT NOT NULL,
            action_id TEXT UNIQUE REFERENCES action_requests(action_id),
            kind TEXT NOT NULL CHECK (kind IN (
                'service.drain', 'service.resume', 'service.stop',
                'service.restart', 'service.force_stop'
            )),
            status TEXT NOT NULL CHECK (status IN (
                'accepted', 'draining', 'ready', 'stopping', 'awaiting_restart',
                'succeeded', 'failed', 'cancelled'
            )),
            requested_by TEXT NOT NULL,
            source_daemon_instance_id TEXT NOT NULL,
            successor_daemon_instance_id TEXT,
            deadline_at TEXT,
            error_code TEXT,
            result_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );

        INSERT INTO action_requests SELECT * FROM action_requests_v32;
        INSERT INTO action_approvals SELECT * FROM action_approvals_v32;
        INSERT INTO service_supervision_events SELECT * FROM service_supervision_events_v32;
        INSERT INTO service_lifecycle_intents SELECT * FROM service_lifecycle_intents_v32;
        DROP TABLE action_approvals_v32;
        DROP TABLE service_supervision_events_v32;
        DROP TABLE service_lifecycle_intents_v32;
        DROP TABLE action_requests_v32;

        CREATE INDEX action_requests_actor_created ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry ON action_requests(status, expires_at);
        CREATE INDEX action_approvals_action ON action_approvals(action_id);
        CREATE INDEX service_supervision_events_repository_created
            ON service_supervision_events(repository_key, created_at);
        CREATE INDEX service_lifecycle_intents_repository_status
            ON service_lifecycle_intents(repository_key, status, updated_at);
        CREATE UNIQUE INDEX service_lifecycle_one_active_reversible
            ON service_lifecycle_intents(repository_key)
            WHERE kind IN ('service.stop', 'service.restart', 'service.force_stop')
              AND status IN ('accepted', 'draining');

        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_approvals_no_update BEFORE UPDATE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER action_approvals_no_delete BEFORE DELETE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_update
        BEFORE UPDATE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_delete
        BEFORE DELETE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        """
    )


def _migration_34(connection: Connection) -> None:
    """Persist the optional source workflow node for Failure gate scoping."""
    connection.executescript(
        """
        ALTER TABLE failure_nodes ADD COLUMN origin_workflow_node TEXT;
        CREATE INDEX failure_nodes_origin_workflow_status
            ON failure_nodes(
                origin_plan, origin_workflow_node, status, priority, created_at
            );
        """
    )


def _migration_35(connection: Connection) -> None:
    """Keep bounded, sanitized external Codex evidence separate from rollouts."""
    connection.executescript(
        """
        CREATE TABLE codex_evidence_sources (
            source_id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            rollout_name TEXT NOT NULL,
            source_mtime_ns INTEGER NOT NULL,
            source_size INTEGER NOT NULL,
            indexed_at TEXT NOT NULL
        );

        CREATE TABLE codex_evidence_records (
            evidence_id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id TEXT NOT NULL REFERENCES codex_evidence_sources(source_id),
            thread_id TEXT NOT NULL,
            rollout_name TEXT NOT NULL,
            event_key_hash TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind IN (
                'validation', 'commit', 'failure', 'cleanup', 'task'
            )),
            outcome TEXT NOT NULL CHECK (outcome IN (
                'succeeded', 'failed', 'aborted', 'unknown'
            )),
            exit_code INTEGER,
            event_at TEXT NOT NULL,
            recorded_at TEXT NOT NULL,
            UNIQUE(source_id, event_key_hash)
        );

        CREATE INDEX codex_evidence_records_recent
            ON codex_evidence_records(event_at DESC, evidence_id DESC);
        CREATE INDEX codex_evidence_records_thread_recent
            ON codex_evidence_records(thread_id, event_at DESC, evidence_id DESC);
        """
    )


def _migration_36(connection: Connection) -> None:
    """Persist structured AI effort without retaining Codex conversation content."""
    connection.executescript(
        """
        CREATE TABLE ai_effort_baselines (
            baseline_id TEXT PRIMARY KEY,
            payload_json TEXT NOT NULL,
            recorded_at TEXT NOT NULL
        );

        CREATE TABLE ai_effort_milestones (
            ledger_id TEXT PRIMARY KEY,
            plan_id TEXT NOT NULL CHECK (length(trim(plan_id)) > 0),
            active_ai_hours REAL NOT NULL CHECK (active_ai_hours >= 0),
            outcome TEXT NOT NULL CHECK (outcome IN ('accepted', 'failed', 'superseded')),
            blocked_by_json TEXT NOT NULL,
            cost_class TEXT NOT NULL CHECK (
                cost_class IN ('delivery_design', 'repair_validation')
            ),
            source_session_id TEXT,
            recorded_at TEXT NOT NULL
        );

        CREATE INDEX ai_effort_milestones_plan_recorded
            ON ai_effort_milestones(plan_id, recorded_at, ledger_id);
        CREATE INDEX ai_effort_milestones_outcome_recorded
            ON ai_effort_milestones(outcome, recorded_at, ledger_id);

        CREATE TABLE ai_effort_forecast_scenarios (
            scenario_id TEXT PRIMARY KEY,
            effective_parallelism_min REAL NOT NULL CHECK (effective_parallelism_min > 0),
            effective_parallelism_max REAL NOT NULL CHECK (
                effective_parallelism_max >= effective_parallelism_min
            ),
            calendar_weeks_min REAL NOT NULL CHECK (calendar_weeks_min >= 0),
            calendar_weeks_max REAL NOT NULL CHECK (
                calendar_weeks_max >= calendar_weeks_min
            ),
            recorded_at TEXT NOT NULL
        );
        """
    )


def _migration_37(connection: Connection) -> None:
    """Persist CPU-lane reservations and coordinator-owned run evidence."""
    connection.executescript(
        """
        CREATE TABLE cargo_lane_reservations (
            reservation_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            lane_scope TEXT NOT NULL CHECK (lane_scope IN ('cpu')),
            compatibility_key TEXT NOT NULL,
            command_fingerprint TEXT NOT NULL,
            job_id TEXT REFERENCES cargo_jobs(job_id),
            status TEXT NOT NULL CHECK (status IN (
                'pending', 'leased', 'running', 'finished', 'released', 'expired'
            )),
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT
        );

        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_cpu
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope='cpu' AND status IN ('pending', 'leased', 'running');
        CREATE INDEX cargo_lane_reservations_session_status
            ON cargo_lane_reservations(session_id, status, created_at);
        CREATE INDEX cargo_lane_reservations_job
            ON cargo_lane_reservations(job_id);

        CREATE TABLE cargo_job_runs (
            run_id TEXT PRIMARY KEY,
            job_id TEXT NOT NULL UNIQUE REFERENCES cargo_jobs(job_id),
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            command_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'running', 'completed', 'finish_blocked', 'launch_failed'
            )),
            exit_code INTEGER,
            stdout_path TEXT NOT NULL,
            stderr_path TEXT NOT NULL,
            stdout_tail TEXT NOT NULL DEFAULT '',
            stderr_tail TEXT NOT NULL DEFAULT '',
            error_code TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT
        );
        CREATE INDEX cargo_job_runs_session_started
            ON cargo_job_runs(session_id, started_at DESC);
        """
    )


def _migration_38(connection: Connection) -> None:
    """Keep a completed CPU reservation until its owner explicitly hands it off."""
    connection.executescript(
        """
        DROP INDEX cargo_lane_reservations_one_active_cpu;
        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_cpu
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope='cpu' AND status IN ('pending', 'leased', 'running', 'finished');
        """
    )


def _migration_39(connection: Connection) -> None:
    """Persist the small allowlisted environment used by a managed Cargo run."""
    connection.execute(
        "ALTER TABLE cargo_job_runs ADD COLUMN environment_json TEXT NOT NULL DEFAULT '{}'"
    )


def _migration_40(connection: Connection) -> None:
    """Track resumable, privacy-safe progress for large Codex rollout sources."""
    connection.executescript(
        """
        ALTER TABLE codex_evidence_sources
            ADD COLUMN scan_offset INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE codex_evidence_sources
            ADD COLUMN prefix_hash TEXT NOT NULL DEFAULT '';
        ALTER TABLE codex_evidence_sources
            ADD COLUMN pending_calls_json TEXT NOT NULL DEFAULT '{}';
        ALTER TABLE codex_evidence_sources
            ADD COLUMN scan_complete INTEGER NOT NULL DEFAULT 0
                CHECK (scan_complete IN (0, 1));
        ALTER TABLE codex_evidence_sources
            ADD COLUMN scan_revision INTEGER NOT NULL DEFAULT 1;
        CREATE INDEX codex_evidence_sources_incomplete
            ON codex_evidence_sources(scan_complete, indexed_at, source_id);
        """
    )


def _migration_41(connection: Connection) -> None:
    """Persist the canonical compatibility payload for new CPU reservations."""
    connection.execute(
        "ALTER TABLE cargo_lane_reservations ADD COLUMN compatibility_json TEXT"
    )


def _migration_42(connection: Connection) -> None:
    """Persist exact GPU reservations, including their coordinator-approved target."""
    connection.executescript(
        """
        DROP INDEX cargo_lane_reservations_one_active_cpu;
        DROP INDEX cargo_lane_reservations_session_status;
        DROP INDEX cargo_lane_reservations_job;
        ALTER TABLE cargo_lane_reservations RENAME TO cargo_lane_reservations_legacy;

        CREATE TABLE cargo_lane_reservations (
            reservation_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            lane_scope TEXT NOT NULL CHECK (lane_scope IN ('cpu', 'gpu')),
            compatibility_key TEXT NOT NULL,
            compatibility_json TEXT,
            target_dir TEXT,
            command_fingerprint TEXT NOT NULL,
            job_id TEXT REFERENCES cargo_jobs(job_id),
            status TEXT NOT NULL CHECK (status IN (
                'pending', 'leased', 'running', 'finished', 'released', 'expired'
            )),
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT
        );

        INSERT INTO cargo_lane_reservations(
            reservation_id, session_id, lane_scope, compatibility_key,
            compatibility_json, command_fingerprint, job_id, status,
            created_at, expires_at, started_at, completed_at
        )
        SELECT reservation_id, session_id, lane_scope, compatibility_key,
               compatibility_json, command_fingerprint, job_id, status,
               created_at, expires_at, started_at, completed_at
        FROM cargo_lane_reservations_legacy;

        DROP TABLE cargo_lane_reservations_legacy;
        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_lane
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope IN ('cpu', 'gpu')
              AND status IN ('pending', 'leased', 'running', 'finished');
        CREATE INDEX cargo_lane_reservations_session_status
            ON cargo_lane_reservations(session_id, status, created_at);
        CREATE INDEX cargo_lane_reservations_job
            ON cargo_lane_reservations(job_id);
        """
    )


def _migration_43(connection: Connection) -> None:
    """Keep one durable FIFO successor behind an active lane reservation."""
    connection.executescript(
        """
        DROP INDEX cargo_lane_reservations_one_active_lane;
        CREATE UNIQUE INDEX cargo_lane_reservations_one_pending_lane
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope IN ('cpu', 'gpu') AND status='pending';
        """
    )


def _migration_44(connection: Connection) -> None:
    """Add the admission-preserving coordinator rollover to closed action enums."""
    action_kinds = """
        'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
        'lease.release_own', 'patch.process_own', 'validation.start',
        'validation.cancel', 'failure.refresh', 'topology.refresh',
        'service.drain_preview', 'service.drain', 'service.resume',
        'service.rollover', 'service.stop', 'service.restart', 'service.force_stop',
        'milestone.commit', 'milestone.reconcile_accepted', 'session.complete',
        'maintenance.cleanup', 'codex.sessions.reconcile'
    """
    connection.executescript(
        f"""
        DROP TRIGGER action_requests_kind_insert;
        DROP TRIGGER action_requests_kind_update;
        DROP TRIGGER action_approvals_no_update;
        DROP TRIGGER action_approvals_no_delete;
        DROP TRIGGER service_supervision_events_no_update;
        DROP TRIGGER service_supervision_events_no_delete;
        DROP INDEX action_requests_actor_created;
        DROP INDEX action_requests_status_expiry;
        DROP INDEX action_approvals_action;
        DROP INDEX service_supervision_events_repository_created;
        DROP INDEX service_lifecycle_intents_repository_status;
        DROP INDEX service_lifecycle_one_active_reversible;

        ALTER TABLE action_approvals RENAME TO action_approvals_v43;
        ALTER TABLE service_supervision_events RENAME TO service_supervision_events_v43;
        ALTER TABLE service_lifecycle_intents RENAME TO service_lifecycle_intents_v43;
        ALTER TABLE action_requests RENAME TO action_requests_v43;

        CREATE TABLE action_requests (
            action_id TEXT PRIMARY KEY,
            action_kind TEXT NOT NULL CHECK (action_kind IN ({action_kinds})),
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
        CREATE TABLE service_supervision_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            repository_key TEXT NOT NULL,
            sequence INTEGER NOT NULL CHECK (sequence > 0),
            from_state TEXT CHECK (from_state IS NULL OR from_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            to_state TEXT NOT NULL CHECK (to_state IN (
                'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
                'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
            )),
            daemon_instance_id TEXT,
            process_id INTEGER CHECK (process_id IS NULL OR process_id > 0),
            process_creation_time TEXT,
            reason_code TEXT NOT NULL,
            actor TEXT,
            action_id TEXT REFERENCES action_requests(action_id),
            payload_json TEXT NOT NULL DEFAULT '{{}}',
            created_at TEXT NOT NULL,
            UNIQUE(repository_key, sequence)
        );
        CREATE TABLE service_lifecycle_intents (
            intent_id TEXT PRIMARY KEY,
            repository_key TEXT NOT NULL,
            action_id TEXT UNIQUE REFERENCES action_requests(action_id),
            kind TEXT NOT NULL CHECK (kind IN (
                'service.drain', 'service.resume', 'service.rollover', 'service.stop',
                'service.restart', 'service.force_stop'
            )),
            status TEXT NOT NULL CHECK (status IN (
                'accepted', 'draining', 'ready', 'stopping', 'awaiting_restart',
                'succeeded', 'failed', 'cancelled'
            )),
            requested_by TEXT NOT NULL,
            source_daemon_instance_id TEXT NOT NULL,
            successor_daemon_instance_id TEXT,
            deadline_at TEXT,
            error_code TEXT,
            result_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );

        INSERT INTO action_requests SELECT * FROM action_requests_v43;
        INSERT INTO action_approvals SELECT * FROM action_approvals_v43;
        INSERT INTO service_supervision_events SELECT * FROM service_supervision_events_v43;
        INSERT INTO service_lifecycle_intents SELECT * FROM service_lifecycle_intents_v43;
        DROP TABLE action_approvals_v43;
        DROP TABLE service_supervision_events_v43;
        DROP TABLE service_lifecycle_intents_v43;
        DROP TABLE action_requests_v43;

        CREATE INDEX action_requests_actor_created ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry ON action_requests(status, expires_at);
        CREATE INDEX action_approvals_action ON action_approvals(action_id);
        CREATE INDEX service_supervision_events_repository_created
            ON service_supervision_events(repository_key, created_at);
        CREATE INDEX service_lifecycle_intents_repository_status
            ON service_lifecycle_intents(repository_key, status, updated_at);
        CREATE UNIQUE INDEX service_lifecycle_one_active_reversible
            ON service_lifecycle_intents(repository_key)
            WHERE kind IN ('service.stop', 'service.restart', 'service.force_stop', 'service.rollover')
              AND status IN ('accepted', 'draining', 'awaiting_restart');

        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({action_kinds})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_approvals_no_update BEFORE UPDATE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER action_approvals_no_delete BEFORE DELETE ON action_approvals
        BEGIN SELECT RAISE(ABORT, 'action approvals are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_update
        BEFORE UPDATE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        CREATE TRIGGER service_supervision_events_no_delete
        BEFORE DELETE ON service_supervision_events
        BEGIN SELECT RAISE(ABORT, 'service supervision events are immutable'); END;
        """
    )


def _migration_45(connection: Connection) -> None:
    """Persist failure scope metadata used by source-slice gate selection."""
    connection.executescript(
        """
        ALTER TABLE failure_nodes
            ADD COLUMN plan_link_mode TEXT NOT NULL DEFAULT '';
        ALTER TABLE failure_nodes
            ADD COLUMN related_code_json TEXT NOT NULL DEFAULT '[]';
        """
    )


def _migration_46(connection: Connection) -> None:
    """Allow an exact CPU successor queue while preserving GPU exclusivity."""
    connection.executescript(
        """
        DROP INDEX cargo_lane_reservations_one_pending_lane;
        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_lane
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope IN ('cpu', 'gpu')
              AND status IN ('leased', 'running', 'finished');
        CREATE UNIQUE INDEX cargo_lane_reservations_one_pending_gpu
            ON cargo_lane_reservations(lane_scope)
            WHERE lane_scope='gpu' AND status='pending';
        CREATE UNIQUE INDEX cargo_lane_reservations_one_pending_session_lane
            ON cargo_lane_reservations(lane_scope, session_id)
            WHERE status='pending';
        CREATE INDEX cargo_lane_reservations_pending_fifo
            ON cargo_lane_reservations(lane_scope, status, created_at, reservation_id);
        """
    )


def _migration_47(connection: Connection) -> None:
    """Retain index recovery bytes only while a finalize request is recoverable."""
    connection.execute(
        """
        UPDATE finalize_requests
        SET index_snapshot = NULL
        WHERE status IN ('committed', 'failed')
          AND index_snapshot IS NOT NULL
        """
    )


def _migration_48(connection: Connection) -> None:
    """Add one isolated CPU burst slot without relaxing the warm pool."""
    connection.executescript(
        """
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN execution_mode TEXT NOT NULL DEFAULT 'warm'
            CHECK (execution_mode IN ('warm', 'burst'));
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN burst_eligible INTEGER NOT NULL DEFAULT 0
            CHECK (burst_eligible IN (0, 1));
        DROP INDEX cargo_lane_reservations_one_active_lane;
        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_warm
            ON cargo_lane_reservations(lane_scope, execution_mode)
            WHERE lane_scope IN ('cpu', 'gpu') AND execution_mode='warm'
              AND status IN ('leased', 'running', 'finished');
        CREATE UNIQUE INDEX cargo_lane_reservations_one_active_burst
            ON cargo_lane_reservations(lane_scope, execution_mode)
            WHERE lane_scope='cpu' AND execution_mode='burst'
              AND status IN ('leased', 'running', 'finished');
        CREATE INDEX cargo_lane_reservations_cpu_warm_fifo
            ON cargo_lane_reservations(lane_scope, execution_mode, status, created_at, reservation_id)
            WHERE lane_scope='cpu' AND execution_mode='warm';
        """
    )


def _migration_49(connection: Connection) -> None:
    """Allow an audited open-failure reservation to outrank blocked FIFO work."""
    connection.executescript(
        """
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN priority_rank INTEGER NOT NULL DEFAULT 1000
            CHECK (priority_rank >= 0);
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN failure_lifecycle_key TEXT;
        CREATE INDEX cargo_lane_reservations_cpu_priority_fifo
            ON cargo_lane_reservations(
                lane_scope, execution_mode, status, priority_rank, created_at, reservation_id
            )
            WHERE lane_scope='cpu' AND execution_mode='warm';
        """
    )


def _migration_50(connection: Connection) -> None:
    """Persist command admission and proof-bound Cargo start acknowledgements."""
    connection.executescript(
        """
        CREATE TABLE command_requests (
            request_id TEXT PRIMARY KEY,
            command TEXT NOT NULL,
            arguments_hash TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN ('accepted', 'completed', 'failed')),
            received_at TEXT NOT NULL,
            accepted_at TEXT NOT NULL,
            completed_at TEXT,
            response_json TEXT,
            error_json TEXT,
            payload_compacted_at TEXT,
            retention_class TEXT NOT NULL DEFAULT 'durable'
                CHECK (retention_class IN ('durable', 'ephemeral'))
        );
        CREATE INDEX command_requests_status_created
            ON command_requests(status, received_at, request_id);
        CREATE INDEX command_requests_compaction_candidates
            ON command_requests(retention_class, payload_compacted_at, status, completed_at, request_id)
            WHERE retention_class='durable'
              AND payload_compacted_at IS NULL;
        CREATE INDEX command_requests_ephemeral_retention
            ON command_requests(retention_class, status, completed_at, request_id);

        CREATE TABLE cargo_start_requests (
            request_id TEXT PRIMARY KEY,
            reservation_id TEXT NOT NULL UNIQUE,
            job_id TEXT NOT NULL UNIQUE,
            session_id TEXT NOT NULL,
            command_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (
                status IN ('start_pending', 'started', 'launch_failed')
            ),
            acknowledged_at TEXT NOT NULL,
            deadline_at TEXT NOT NULL,
            run_id TEXT,
            error_code TEXT,
            error_message TEXT,
            completed_at TEXT
        );
        CREATE INDEX cargo_start_requests_pending_deadline
            ON cargo_start_requests(status, deadline_at, request_id);

        CREATE TRIGGER cargo_jobs_preserve_valid_start_pending
        BEFORE UPDATE OF status ON cargo_jobs
        WHEN OLD.status='leased'
          AND NEW.status='orphaned'
          AND EXISTS (
              SELECT 1 FROM cargo_start_requests
              WHERE job_id=OLD.job_id
                AND status='start_pending'
                AND deadline_at>NEW.last_heartbeat_at
          )
        BEGIN
            SELECT RAISE(IGNORE);
        END;

        ALTER TABLE cargo_lane_reservations
            ADD COLUMN dependency_lifecycle_key TEXT;
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN dependency_fixed_sha256 TEXT;
        ALTER TABLE cargo_lane_reservations
            ADD COLUMN source_copy_job_id TEXT REFERENCES validation_copies(job_id);
        ALTER TABLE cargo_jobs
            ADD COLUMN source_copy_job_id TEXT REFERENCES validation_copies(job_id);
        ALTER TABLE cargo_jobs
            ADD COLUMN source_copy_manifest_hash TEXT;
        CREATE INDEX cargo_lane_reservations_dependency
            ON cargo_lane_reservations(dependency_lifecycle_key, status, reservation_id)
            WHERE dependency_lifecycle_key IS NOT NULL;
        CREATE INDEX cargo_jobs_source_copy
            ON cargo_jobs(source_copy_job_id, status, job_id)
            WHERE source_copy_job_id IS NOT NULL;

        ALTER TABLE validation_copies
            ADD COLUMN external_sources_json TEXT NOT NULL DEFAULT '[]';
        ALTER TABLE validation_copies
            ADD COLUMN input_manifest_hash TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN error_code TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN error_stage TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN error_path TEXT;

        CREATE TABLE workflow_failure_deferrals (
            deferral_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            plan_path TEXT NOT NULL,
            topology_hash TEXT NOT NULL,
            source_milestone_key TEXT NOT NULL,
            target_milestone_key TEXT NOT NULL,
            failure_lifecycle_key TEXT NOT NULL,
            actor TEXT NOT NULL,
            action_id TEXT,
            created_at TEXT NOT NULL,
            UNIQUE(
                session_id, plan_path, topology_hash, source_milestone_key,
                failure_lifecycle_key
            )
        );
        CREATE INDEX workflow_failure_deferrals_context
            ON workflow_failure_deferrals(
                session_id, plan_path, topology_hash, source_milestone_key,
                failure_lifecycle_key
            );

        CREATE TABLE workflow_validation_template_bindings (
            validation_run_id TEXT PRIMARY KEY
                REFERENCES workflow_validation_bindings(validation_run_id) ON DELETE CASCADE,
            template TEXT NOT NULL
        );
        """
    )


def _migration_51(connection: Connection) -> None:
    """Persist durable, restartable Cargo validation-copy materialization phases."""
    connection.executescript(
        """
        ALTER TABLE validation_copies
            ADD COLUMN materialization_kind TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN materialization_request_json TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN materialization_phase TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN materialization_worker_id TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN materialization_attempt INTEGER NOT NULL DEFAULT 0;
        CREATE INDEX validation_copies_cargo_materialization_recovery
            ON validation_copies(status, materialization_kind, materialization_phase, job_id)
            WHERE status='planned' AND materialization_kind='cargo';
        """
    )


def _migration_52(connection: Connection) -> None:
    """Persist bounded, indexed state-convergence preview and apply audits."""
    connection.executescript(
        """
        CREATE TABLE governance_previews (
            fingerprint TEXT PRIMARY KEY,
            operation TEXT NOT NULL,
            candidate_count INTEGER NOT NULL,
            candidates_json TEXT NOT NULL,
            actor TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX governance_previews_created
            ON governance_previews(created_at, fingerprint);

        CREATE TABLE governance_applies (
            apply_id INTEGER PRIMARY KEY AUTOINCREMENT,
            fingerprint TEXT NOT NULL REFERENCES governance_previews(fingerprint),
            actor TEXT NOT NULL,
            candidate_count INTEGER NOT NULL,
            applied_count INTEGER NOT NULL,
            skipped_count INTEGER NOT NULL,
            conflict_count INTEGER NOT NULL,
            result_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX governance_applies_fingerprint_created
            ON governance_applies(fingerprint, created_at, apply_id);
        """
    )


def _migration_53(connection: Connection) -> None:
    """Store immutable validation submissions outside business Session state."""
    connection.executescript(
        """
        CREATE TABLE validation_tickets (
            ticket_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            plan_path TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'queued', 'materializing', 'running', 'passed', 'failed', 'snapshot_stale'
            )),
            dedupe_key TEXT NOT NULL,
            source_manifest_hash TEXT NOT NULL,
            source_manifest_json TEXT NOT NULL,
            command_json TEXT NOT NULL,
            toolchain_json TEXT NOT NULL,
            coverage_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX validation_tickets_reusable
            ON validation_tickets(dedupe_key, status, created_at, ticket_id);
        CREATE INDEX validation_tickets_session_status
            ON validation_tickets(session_id, status, created_at);

        CREATE TABLE validation_ticket_requests (
            request_id TEXT PRIMARY KEY,
            ticket_id TEXT NOT NULL REFERENCES validation_tickets(ticket_id),
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            created_at TEXT NOT NULL
        );
        CREATE INDEX validation_ticket_requests_ticket
            ON validation_ticket_requests(ticket_id, created_at);

        CREATE TABLE validation_ticket_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            ticket_id TEXT NOT NULL REFERENCES validation_tickets(ticket_id),
            event_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX validation_ticket_events_ticket
            ON validation_ticket_events(ticket_id, event_id);
        """
    )


def _migration_54(connection: Connection) -> None:
    """Persist exact Git blob candidates after their compile gate passes."""
    connection.executescript(
        """
        CREATE TABLE integration_candidates (
            candidate_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            plan_path TEXT NOT NULL,
            request_id TEXT NOT NULL UNIQUE,
            base_head TEXT NOT NULL,
            compile_ticket_id TEXT NOT NULL REFERENCES validation_tickets(ticket_id),
            status TEXT NOT NULL CHECK (status IN (
                'integration_ready', 'integrated_validation_pending', 'accepted', 'delayed_merge'
            )),
            commit_sha TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX integration_candidates_session_status
            ON integration_candidates(session_id, status, created_at);
        CREATE INDEX integration_candidates_compile_ticket
            ON integration_candidates(compile_ticket_id, status, created_at);

        CREATE TABLE integration_candidate_paths (
            candidate_id TEXT NOT NULL REFERENCES integration_candidates(candidate_id) ON DELETE CASCADE,
            path TEXT NOT NULL,
            blob_oid TEXT NOT NULL,
            PRIMARY KEY(candidate_id, path)
        );

        CREATE TABLE integration_candidate_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            candidate_id TEXT NOT NULL REFERENCES integration_candidates(candidate_id),
            event_type TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX integration_candidate_events_candidate
            ON integration_candidate_events(candidate_id, event_id);
        """
    )


def _migration_55(connection: Connection) -> None:
    """Retain the live lease proof that authorized each sealed candidate."""
    connection.executescript(
        """
        ALTER TABLE integration_candidates
            ADD COLUMN lease_evidence_json TEXT NOT NULL DEFAULT '[]';
        """
    )


def _migration_56(connection: Connection) -> None:
    """Add bounded retention metadata for large baseline and copy manifests."""
    connection.executescript(
        """
        ALTER TABLE baseline_epochs
            ADD COLUMN manifest_sha256 TEXT;
        ALTER TABLE baseline_epochs
            ADD COLUMN manifest_entry_count INTEGER;
        ALTER TABLE baseline_epochs
            ADD COLUMN manifest_byte_count INTEGER;
        ALTER TABLE baseline_epochs
            ADD COLUMN manifest_archive_path TEXT;
        ALTER TABLE baseline_epochs
            ADD COLUMN manifest_archived_at TEXT;
        CREATE INDEX baseline_epochs_manifest_retention
            ON baseline_epochs(manifest_archive_path, created_at, epoch_id);

        ALTER TABLE validation_copies
            ADD COLUMN manifest_sha256 TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN manifest_entry_count INTEGER;
        ALTER TABLE validation_copies
            ADD COLUMN manifest_byte_count INTEGER;
        ALTER TABLE validation_copies
            ADD COLUMN manifest_archive_path TEXT;
        ALTER TABLE validation_copies
            ADD COLUMN manifest_archived_at TEXT;
        CREATE INDEX validation_copies_manifest_retention
            ON validation_copies(manifest_archive_path, status, removed_at, created_at, job_id);

        CREATE TABLE manifest_retention_previews (
            fingerprint TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            candidate_count INTEGER NOT NULL,
            candidates_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX manifest_retention_previews_created
            ON manifest_retention_previews(created_at, fingerprint);

        CREATE TABLE manifest_retention_batches (
            batch_id TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            status TEXT NOT NULL CHECK (
                status IN ('applying', 'retired', 'compact_pending', 'compacted', 'failed')
            ),
            candidate_count INTEGER NOT NULL,
            archive_path TEXT,
            backup_path TEXT,
            created_at TEXT NOT NULL,
            retired_at TEXT,
            compacted_at TEXT,
            error_text TEXT
        );
        CREATE INDEX manifest_retention_batches_status_created
            ON manifest_retention_batches(status, created_at, batch_id);
        """
    )


def _migration_57(connection: Connection) -> None:
    """Persist numbered-Plan WIP ownership on each Session."""
    connection.executescript(
        """
        ALTER TABLE sessions
            ADD COLUMN plan_family_key TEXT;
        ALTER TABLE sessions
            ADD COLUMN session_role TEXT NOT NULL DEFAULT 'primary'
                CHECK (session_role IN ('primary', 'reviewer'));
        ALTER TABLE sessions
            ADD COLUMN parent_session_id TEXT REFERENCES sessions(session_id);
        CREATE INDEX sessions_plan_wip_primary
            ON sessions(plan_family_key, session_role, status, last_heartbeat_at, session_id);
        CREATE INDEX sessions_plan_wip_reviewer_parent
            ON sessions(parent_session_id, status, session_id)
            WHERE parent_session_id IS NOT NULL;
        """
    )


def _migration_58(connection: Connection) -> None:
    """Audit explicit transfers of abandoned exact-path ownership."""
    connection.executescript(
        """
        CREATE TABLE ownership_transfer_previews (
            fingerprint TEXT PRIMARY KEY,
            target_session_id TEXT NOT NULL REFERENCES sessions(session_id),
            baseline_epoch INTEGER NOT NULL REFERENCES baseline_epochs(epoch_id),
            candidates_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            applied_at TEXT
        );
        CREATE INDEX ownership_transfer_previews_target_created
            ON ownership_transfer_previews(target_session_id, created_at, fingerprint);

        CREATE TABLE ownership_transfers (
            fingerprint TEXT NOT NULL REFERENCES ownership_transfer_previews(fingerprint),
            path_key TEXT NOT NULL,
            display_path TEXT NOT NULL,
            target_session_id TEXT NOT NULL REFERENCES sessions(session_id),
            source_session_id TEXT REFERENCES sessions(session_id),
            baseline_epoch INTEGER NOT NULL REFERENCES baseline_epochs(epoch_id),
            content_hash TEXT NOT NULL,
            actor TEXT NOT NULL,
            transferred_at TEXT NOT NULL,
            PRIMARY KEY(fingerprint, path_key)
        );
        CREATE INDEX ownership_transfers_target_created
            ON ownership_transfers(target_session_id, transferred_at, path_key);
        """
    )


def _migration_59(connection: Connection) -> None:
    """Bind native benchmark validation to a durable, one-shot copy grant."""
    migrate_benchmark_validation_schema(connection)


def _migration_60(connection: Connection) -> None:
    """Distinguish Cargo cleanup from handle-bound unmanaged artifact deletion."""
    connection.executescript(
        """
        ALTER TABLE cleanup_reservations
            ADD COLUMN reservation_kind TEXT NOT NULL DEFAULT 'cargo'
                CHECK (reservation_kind IN ('cargo', 'artifact'));
        ALTER TABLE cleanup_reservations
            ADD COLUMN filesystem_identity TEXT;
        CREATE INDEX cleanup_reservations_kind_reserved
            ON cleanup_reservations(reservation_kind, reserved_at, target_key);
        """
    )


def _migration_61(connection: Connection) -> None:
    """Bind a produced viewer binary to one managed validation-copy run."""
    connection.executescript(
        """
        CREATE TABLE managed_artifact_receipts (
            receipt_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(session_id),
            job_id TEXT NOT NULL REFERENCES validation_copies(job_id),
            validation_ticket_id TEXT NOT NULL REFERENCES validation_tickets(ticket_id),
            artifact_kind TEXT NOT NULL CHECK (artifact_kind IN ('shader-pbr-viewer')),
            status TEXT NOT NULL CHECK (status IN ('pending', 'passed', 'rejected')),
            requested_input_manifest_hash TEXT NOT NULL,
            source_manifest_hash TEXT NOT NULL,
            run_id TEXT REFERENCES validation_copy_runs(run_id),
            target_relative_path TEXT,
            artifact_path TEXT,
            sha256 TEXT,
            byte_length INTEGER,
            command_json TEXT,
            command_sha256 TEXT,
            error_code TEXT,
            requested_at TEXT NOT NULL,
            completed_at TEXT,
            UNIQUE(job_id, validation_ticket_id, artifact_kind)
        );
        CREATE INDEX managed_artifact_receipts_job_status
            ON managed_artifact_receipts(job_id, status, requested_at, receipt_id);
        CREATE INDEX managed_artifact_receipts_ticket_status
            ON managed_artifact_receipts(
                validation_ticket_id, status, requested_at, receipt_id
            );
        """
    )


def _migration_62(connection: Connection) -> None:
    """Preserve structured validation-copy materialization diagnostics."""
    connection.execute(
        """
        ALTER TABLE validation_copies
            ADD COLUMN error_details_json TEXT NOT NULL DEFAULT '{}'
        """
    )


def _migration_63(connection: Connection) -> None:
    """Preserve structured failure dependency diagnostics."""
    connection.execute(
        """
        ALTER TABLE failure_diagnostics
            ADD COLUMN details_json TEXT NOT NULL DEFAULT '{}'
        """
    )


def _migration_64(connection: Connection) -> None:
    """Register process-bound test fixtures with artifact governance."""
    connection.executescript(
        """
        CREATE TABLE artifact_fixture_leases (
            lease_id TEXT PRIMARY KEY
                CHECK (length(lease_id) = 32),
            target_key TEXT NOT NULL UNIQUE,
            target_dir TEXT NOT NULL UNIQUE,
            prefix TEXT NOT NULL,
            owner_pid INTEGER NOT NULL CHECK (owner_pid > 0),
            owner_process_creation_time TEXT NOT NULL
                CHECK (owner_process_creation_time <> ''),
            status TEXT NOT NULL
                CHECK (status IN ('active', 'released', 'recovered')),
            created_at TEXT NOT NULL,
            released_at TEXT
        );
        CREATE INDEX idx_artifact_fixture_leases_active
            ON artifact_fixture_leases(status, created_at, lease_id);
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
    17: _migration_17,
    18: _migration_18,
    19: _migration_19,
    20: migrate_supervision_schema,
    21: _migration_21,
    22: _migration_22,
    23: _migration_23,
    24: _migration_24,
    25: _migration_25,
    26: _migration_26,
    27: _migration_27,
    28: _migration_28,
    29: _migration_29,
    30: _migration_30,
    31: _migration_31,
    32: _migration_32,
    33: _migration_33,
    34: _migration_34,
    35: _migration_35,
    36: _migration_36,
    37: _migration_37,
    38: _migration_38,
    39: _migration_39,
    40: _migration_40,
    41: _migration_41,
    42: _migration_42,
    43: _migration_43,
    44: _migration_44,
    45: _migration_45,
    46: _migration_46,
    47: _migration_47,
    48: _migration_48,
    49: _migration_49,
    50: _migration_50,
    51: _migration_51,
    52: _migration_52,
    53: _migration_53,
    54: _migration_54,
    55: _migration_55,
    56: _migration_56,
    57: _migration_57,
    58: _migration_58,
    59: _migration_59,
    60: _migration_60,
    61: _migration_61,
    62: _migration_62,
    63: _migration_63,
    64: _migration_64,
}


def migrate(database: Database) -> int:
    existing_database = False
    apply_compaction_marker = False
    apply_terminal_snapshot_compaction = False
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
        existing_database = bool(applied)
        apply_compaction_marker = 25 not in applied
        apply_terminal_snapshot_compaction = 47 not in applied
        for version in range(1, 25):
            if version in applied:
                continue
            MIGRATIONS[version](connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (?, datetime('now'))",
                (version,),
            )
    if apply_compaction_marker:
        if existing_database:
            with database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
                connection.execute("VACUUM")
        with database.transaction() as connection:
            _migration_25(connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (25, datetime('now'))"
            )
    with database.transaction() as connection:
        applied_after_compaction = {
            int(row[0])
            for row in connection.execute(
                "SELECT version FROM schema_version WHERE version>=26"
            )
        }
        for version in range(26, 47):
            if version in applied_after_compaction:
                continue
            MIGRATIONS[version](connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (?, datetime('now'))",
                (version,),
            )
    if apply_terminal_snapshot_compaction:
        # The recovery BLOB is not evidence after a terminal transition. Apply the
        # data cleanup before its schema marker, so a failed physical compaction is
        # retried on the next safe startup rather than silently leaving disk debt.
        with database.transaction() as connection:
            _migration_47(connection)
        if existing_database:
            with database.connect() as connection:
                connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
                connection.execute("VACUUM")
        with database.transaction() as connection:
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (47, datetime('now'))"
            )
    with database.transaction() as connection:
        applied_after_terminal_compaction = {
            int(row[0])
            for row in connection.execute(
                "SELECT version FROM schema_version WHERE version>=48"
            )
        }
        for version in range(48, LATEST_SCHEMA_VERSION + 1):
            if version in applied_after_terminal_compaction:
                continue
            MIGRATIONS[version](connection)
            connection.execute(
                "INSERT INTO schema_version(version, applied_at) VALUES (?, datetime('now'))",
                (version,),
            )
    return LATEST_SCHEMA_VERSION
