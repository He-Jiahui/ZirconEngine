from __future__ import annotations

from sqlite3 import Connection


_ACTION_KINDS = """
    'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
    'lease.release_own', 'patch.process_own', 'validation.start',
    'validation.cancel', 'validation.benchmark_grant.issue', 'failure.refresh',
    'topology.refresh', 'service.drain_preview', 'service.drain',
    'service.resume', 'service.rollover', 'service.stop', 'service.restart',
    'service.force_stop', 'milestone.commit', 'milestone.reconcile_accepted',
    'session.complete', 'maintenance.cleanup', 'codex.sessions.reconcile'
"""


def migrate_benchmark_validation_schema(connection: Connection) -> None:
    """Persist one-shot benchmark grants and their two immutable manifest identities."""
    _extend_closed_action_kind(connection)
    connection.executescript(
        """
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN copy_input_manifest_hash TEXT;
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN benchmark_name TEXT;
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN cargo_profile TEXT;
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN benchmark_grant_id TEXT;
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN root_pid INTEGER CHECK (root_pid IS NULL OR root_pid > 0);
        ALTER TABLE workflow_validation_bindings
            ADD COLUMN root_process_creation_time TEXT;

        CREATE TABLE benchmark_validation_grants (
            fifo_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
            grant_id TEXT NOT NULL UNIQUE,
            job_id TEXT NOT NULL UNIQUE
                REFERENCES validation_copies(job_id) ON DELETE RESTRICT,
            source_session_id TEXT NOT NULL
                REFERENCES sessions(session_id) ON DELETE RESTRICT,
            target_session_id TEXT NOT NULL
                REFERENCES sessions(session_id) ON DELETE RESTRICT,
            run_id TEXT NOT NULL
                REFERENCES workflow_runs(run_id) ON DELETE RESTRICT,
            milestone_id TEXT NOT NULL,
            input_manifest_hash TEXT NOT NULL CHECK (length(input_manifest_hash) = 64),
            scoped_manifest_hash TEXT NOT NULL CHECK (length(scoped_manifest_hash) = 64),
            benchmark_name TEXT NOT NULL,
            cargo_profile TEXT NOT NULL CHECK (cargo_profile IN ('release', 'profiling')),
            command_json TEXT NOT NULL,
            status TEXT NOT NULL CHECK (status IN (
                'issued', 'launching', 'consumed', 'denied'
            )),
            issued_at TEXT NOT NULL,
            acquired_at TEXT,
            consumed_at TEXT,
            denied_at TEXT,
            validation_run_id TEXT UNIQUE,
            root_pid INTEGER CHECK (root_pid IS NULL OR root_pid > 0),
            root_process_creation_time TEXT,
            job_isolated INTEGER NOT NULL DEFAULT 0 CHECK (job_isolated IN (0, 1)),
            error_code TEXT
        );
        CREATE INDEX benchmark_validation_grants_target_fifo
            ON benchmark_validation_grants(
                target_session_id, status, fifo_sequence
            );
        CREATE INDEX benchmark_validation_grants_source_copy
            ON benchmark_validation_grants(source_session_id, job_id);
        CREATE UNIQUE INDEX workflow_validation_bindings_benchmark_grant
            ON workflow_validation_bindings(benchmark_grant_id)
            WHERE benchmark_grant_id IS NOT NULL;
        """
    )


def _extend_closed_action_kind(connection: Connection) -> None:
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

        ALTER TABLE action_approvals RENAME TO action_approvals_v58;
        ALTER TABLE service_supervision_events RENAME TO service_supervision_events_v58;
        ALTER TABLE service_lifecycle_intents RENAME TO service_lifecycle_intents_v58;
        ALTER TABLE action_requests RENAME TO action_requests_v58;

        CREATE TABLE action_requests (
            action_id TEXT PRIMARY KEY,
            action_kind TEXT NOT NULL CHECK (action_kind IN ({_ACTION_KINDS})),
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

        INSERT INTO action_requests SELECT * FROM action_requests_v58;
        INSERT INTO action_approvals SELECT * FROM action_approvals_v58;
        INSERT INTO service_supervision_events SELECT * FROM service_supervision_events_v58;
        INSERT INTO service_lifecycle_intents SELECT * FROM service_lifecycle_intents_v58;
        DROP TABLE action_approvals_v58;
        DROP TABLE service_supervision_events_v58;
        DROP TABLE service_lifecycle_intents_v58;
        DROP TABLE action_requests_v58;

        CREATE INDEX action_requests_actor_created ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry ON action_requests(status, expires_at);
        CREATE INDEX action_approvals_action ON action_approvals(action_id);
        CREATE INDEX service_supervision_events_repository_created
            ON service_supervision_events(repository_key, created_at);
        CREATE INDEX service_lifecycle_intents_repository_status
            ON service_lifecycle_intents(repository_key, status, updated_at);
        CREATE UNIQUE INDEX service_lifecycle_one_active_reversible
            ON service_lifecycle_intents(repository_key)
            WHERE kind IN (
                'service.stop', 'service.restart', 'service.force_stop', 'service.rollover'
            )
              AND status IN ('accepted', 'draining', 'awaiting_restart');

        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({_ACTION_KINDS})
        BEGIN SELECT RAISE(ABORT, 'invalid controlled action kind'); END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({_ACTION_KINDS})
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
