from __future__ import annotations

from sqlite3 import Connection


_ACTION_KINDS = """
    'session.heartbeat', 'session.activate', 'lease.claim_own_scope',
    'lease.release_own', 'patch.process_own', 'validation.start',
    'validation.cancel', 'failure.refresh', 'topology.refresh',
    'service.drain_preview', 'service.drain', 'service.resume',
    'service.stop', 'service.restart', 'service.force_stop',
    'milestone.commit', 'session.complete', 'maintenance.cleanup'
"""

_SUPERVISION_STATES = """
    'starting', 'healthy', 'degraded', 'draining', 'stopping', 'offline',
    'recovering', 'read_only', 'identity_mismatch', 'fatal_integrity_error'
"""

_LIFECYCLE_KINDS = """
    'service.drain', 'service.resume', 'service.stop',
    'service.restart', 'service.force_stop'
"""

_LIFECYCLE_STATUSES = """
    'accepted', 'draining', 'ready', 'stopping', 'awaiting_restart',
    'succeeded', 'failed', 'cancelled'
"""


def migrate_supervision_schema(connection: Connection) -> None:
    """Install schema 20 without losing the released v19 action audit trail."""
    connection.executescript(
        f"""
        DROP TRIGGER IF EXISTS action_requests_kind_insert;
        DROP TRIGGER IF EXISTS action_requests_kind_update;
        DROP TRIGGER IF EXISTS action_approvals_no_update;
        DROP TRIGGER IF EXISTS action_approvals_no_delete;

        ALTER TABLE action_approvals RENAME TO action_approvals_v19;
        ALTER TABLE action_requests RENAME TO action_requests_v19;

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

        INSERT INTO action_requests SELECT * FROM action_requests_v19;
        INSERT INTO action_approvals SELECT * FROM action_approvals_v19;
        DROP TABLE action_approvals_v19;
        DROP TABLE action_requests_v19;

        CREATE INDEX action_requests_actor_created
            ON action_requests(actor, created_at);
        CREATE INDEX action_requests_status_expiry
            ON action_requests(status, expires_at);
        CREATE INDEX action_approvals_action ON action_approvals(action_id);

        CREATE TRIGGER action_requests_kind_insert
        BEFORE INSERT ON action_requests
        WHEN NEW.action_kind NOT IN ({_ACTION_KINDS})
        BEGIN
            SELECT RAISE(ABORT, 'invalid controlled action kind');
        END;
        CREATE TRIGGER action_requests_kind_update
        BEFORE UPDATE OF action_kind ON action_requests
        WHEN NEW.action_kind NOT IN ({_ACTION_KINDS})
        BEGIN
            SELECT RAISE(ABORT, 'invalid controlled action kind');
        END;
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

        CREATE TABLE service_supervision_events (
            event_id INTEGER PRIMARY KEY AUTOINCREMENT,
            repository_key TEXT NOT NULL,
            sequence INTEGER NOT NULL CHECK (sequence > 0),
            from_state TEXT CHECK (
                from_state IS NULL OR from_state IN ({_SUPERVISION_STATES})
            ),
            to_state TEXT NOT NULL CHECK (to_state IN ({_SUPERVISION_STATES})),
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
        CREATE INDEX service_supervision_events_repository_created
            ON service_supervision_events(repository_key, created_at);
        CREATE TRIGGER service_supervision_events_no_update
        BEFORE UPDATE ON service_supervision_events
        BEGIN
            SELECT RAISE(ABORT, 'service supervision events are immutable');
        END;
        CREATE TRIGGER service_supervision_events_no_delete
        BEFORE DELETE ON service_supervision_events
        BEGIN
            SELECT RAISE(ABORT, 'service supervision events are immutable');
        END;

        CREATE TABLE service_recovery_state (
            repository_key TEXT PRIMARY KEY,
            state TEXT NOT NULL CHECK (state IN ({_SUPERVISION_STATES})),
            daemon_instance_id TEXT,
            process_id INTEGER CHECK (process_id IS NULL OR process_id > 0),
            process_creation_time TEXT,
            explicit_stop INTEGER NOT NULL DEFAULT 0 CHECK (explicit_stop IN (0, 1)),
            maintenance_hold INTEGER NOT NULL DEFAULT 0 CHECK (maintenance_hold IN (0, 1)),
            failure_count INTEGER NOT NULL DEFAULT 0 CHECK (failure_count >= 0),
            failure_window_started_at TEXT,
            next_retry_at TEXT,
            circuit_open_until TEXT,
            healthy_since TEXT,
            last_reason_code TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE service_lifecycle_intents (
            intent_id TEXT PRIMARY KEY,
            repository_key TEXT NOT NULL,
            action_id TEXT UNIQUE REFERENCES action_requests(action_id),
            kind TEXT NOT NULL CHECK (kind IN ({_LIFECYCLE_KINDS})),
            status TEXT NOT NULL CHECK (status IN ({_LIFECYCLE_STATUSES})),
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
        CREATE INDEX service_lifecycle_intents_repository_status
            ON service_lifecycle_intents(repository_key, status, updated_at);
        """
    )
