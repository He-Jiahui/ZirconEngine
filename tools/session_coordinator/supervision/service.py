from __future__ import annotations

import json
import os
import threading
import uuid
from collections.abc import Callable
from datetime import datetime, timezone
from sqlite3 import Connection

from ..database import Database
from ..models import CoordinatorError, SupervisionState, utc_text
from .models import (
    BlockerKind,
    LifecycleKind,
    LifecycleStatus,
    SupervisionBlocker,
    SupervisionSnapshot,
)


_ALLOWED_TRANSITIONS: dict[SupervisionState, frozenset[SupervisionState]] = {
    SupervisionState.STARTING: frozenset(
        {
            SupervisionState.HEALTHY,
            SupervisionState.DEGRADED,
            SupervisionState.RECOVERING,
            SupervisionState.READ_ONLY,
            SupervisionState.IDENTITY_MISMATCH,
            SupervisionState.FATAL_INTEGRITY_ERROR,
        }
    ),
    SupervisionState.HEALTHY: frozenset(
        {
            SupervisionState.DEGRADED,
            SupervisionState.DRAINING,
            SupervisionState.STOPPING,
            SupervisionState.OFFLINE,
            SupervisionState.RECOVERING,
            SupervisionState.READ_ONLY,
            SupervisionState.IDENTITY_MISMATCH,
            SupervisionState.FATAL_INTEGRITY_ERROR,
        }
    ),
    SupervisionState.DEGRADED: frozenset(
        {
            SupervisionState.HEALTHY,
            SupervisionState.DRAINING,
            SupervisionState.STOPPING,
            SupervisionState.OFFLINE,
            SupervisionState.RECOVERING,
            SupervisionState.READ_ONLY,
            SupervisionState.IDENTITY_MISMATCH,
            SupervisionState.FATAL_INTEGRITY_ERROR,
        }
    ),
    SupervisionState.DRAINING: frozenset(
        {
            SupervisionState.HEALTHY,
            SupervisionState.DEGRADED,
            SupervisionState.STOPPING,
            SupervisionState.OFFLINE,
        }
    ),
    SupervisionState.STOPPING: frozenset(
        {SupervisionState.OFFLINE, SupervisionState.RECOVERING}
    ),
    SupervisionState.OFFLINE: frozenset(
        {SupervisionState.STARTING, SupervisionState.RECOVERING}
    ),
    SupervisionState.RECOVERING: frozenset(
        {
            SupervisionState.STARTING,
            SupervisionState.HEALTHY,
            SupervisionState.DEGRADED,
            SupervisionState.OFFLINE,
            SupervisionState.IDENTITY_MISMATCH,
            SupervisionState.FATAL_INTEGRITY_ERROR,
        }
    ),
    SupervisionState.READ_ONLY: frozenset(
        {
            SupervisionState.HEALTHY,
            SupervisionState.DRAINING,
            SupervisionState.STOPPING,
            SupervisionState.OFFLINE,
            SupervisionState.FATAL_INTEGRITY_ERROR,
        }
    ),
    SupervisionState.IDENTITY_MISMATCH: frozenset({SupervisionState.OFFLINE}),
    SupervisionState.FATAL_INTEGRITY_ERROR: frozenset({SupervisionState.OFFLINE}),
}

_DRAIN_ALLOWED_OPERATIONS = frozenset(
    {
        "service.drain_preview",
        "service.drain",
        "service.resume",
        "service.stop",
        "service.restart",
        "service.force_stop",
        "lease.release",
        "lease.heartbeat",
        "cargo.heartbeat",
        "cargo.finish",
        "cargo.release",
        "supervision.recovery_record",
    }
)


class SupervisionService:
    """The sole persistence and mutation gate for coordinator supervision state."""

    def __init__(
        self,
        database: Database,
        *,
        repository_key: str,
        daemon_instance_id: str,
        process_creation_time: str,
        maintenance_active: Callable[[], bool] | None = None,
    ):
        self.database = database
        self.repository_key = repository_key
        self.daemon_instance_id = daemon_instance_id
        self.process_id = os.getpid()
        self.process_creation_time = process_creation_time
        self._maintenance_active = maintenance_active or (lambda: False)
        self._transition_lock = threading.RLock()

    def initialize(
        self,
        *,
        start_reason: str = "startup.begin",
        automatic_start: bool = False,
    ) -> SupervisionSnapshot:
        with self._transition_lock, self.database.transaction() as connection:
            row = self._state_row(connection)
            now = utc_text()
            if row is not None and automatic_start and bool(row["explicit_stop"]):
                raise CoordinatorError(
                    "explicit_stop_persisted",
                    "Automatic startup is suppressed until an explicit start is requested",
                )
            if row is None:
                connection.execute(
                    """
                    INSERT INTO service_recovery_state(
                        repository_key, state, daemon_instance_id, process_id,
                        process_creation_time, updated_at, last_reason_code
                    ) VALUES (?, 'starting', ?, ?, ?, ?, ?)
                    """,
                    (
                        self.repository_key,
                        self.daemon_instance_id,
                        self.process_id,
                        self.process_creation_time,
                        now,
                        start_reason,
                    ),
                )
                self._append_event(
                    connection,
                    None,
                    SupervisionState.STARTING,
                    start_reason,
                    actor="daemon",
                )
            else:
                previous = SupervisionState(row["state"])
                connection.execute(
                    """
                    UPDATE service_recovery_state
                    SET state='starting', daemon_instance_id=?, process_id=?,
                        process_creation_time=?, updated_at=?, last_reason_code=?
                    WHERE repository_key=?
                    """,
                    (
                        self.daemon_instance_id,
                        self.process_id,
                        self.process_creation_time,
                        now,
                        start_reason,
                        self.repository_key,
                    ),
                )
                self._append_event(
                    connection,
                    previous,
                    SupervisionState.STARTING,
                    start_reason,
                    actor="daemon",
                )
        return self.snapshot()

    def mark_healthy(self, *, reason_code: str = "startup.ready") -> SupervisionSnapshot:
        return self.transition(
            SupervisionState.HEALTHY,
            reason_code=reason_code,
            actor="daemon",
            updates={"explicit_stop": 0, "healthy_since": utc_text()},
        )

    def transition(
        self,
        state: SupervisionState,
        *,
        reason_code: str,
        actor: str,
        action_id: str | None = None,
        updates: dict[str, object] | None = None,
    ) -> SupervisionSnapshot:
        with self._transition_lock, self.database.transaction() as connection:
            self._transition(
                connection,
                state,
                reason_code=reason_code,
                actor=actor,
                action_id=action_id,
                updates=updates,
            )
        return self.snapshot(exclude_action_id=action_id)

    def _transition(
        self,
        connection: Connection,
        state: SupervisionState,
        *,
        reason_code: str,
        actor: str,
        action_id: str | None = None,
        updates: dict[str, object] | None = None,
    ) -> None:
        row = self._state_row(connection)
        if row is None:
            raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
        previous = SupervisionState(row["state"])
        if previous is not state and state not in _ALLOWED_TRANSITIONS[previous]:
            raise CoordinatorError(
                "supervision_transition_invalid",
                f"Cannot transition supervision from {previous.value} to {state.value}",
            )
        allowed_updates = {
            "explicit_stop",
            "maintenance_hold",
            "failure_count",
            "failure_window_started_at",
            "next_retry_at",
            "circuit_open_until",
            "healthy_since",
        }
        values = dict(updates or {})
        if set(values) - allowed_updates:
            raise ValueError("Unsupported supervision state update")
        assignments = ["state=?", "updated_at=?", "last_reason_code=?"]
        parameters: list[object] = [state.value, utc_text(), reason_code]
        for key in sorted(values):
            assignments.append(f"{key}=?")
            parameters.append(values[key])
        parameters.append(self.repository_key)
        connection.execute(
            f"UPDATE service_recovery_state SET {', '.join(assignments)} WHERE repository_key=?",
            tuple(parameters),
        )
        if previous is not state or row["last_reason_code"] != reason_code:
            self._append_event(
                connection,
                previous,
                state,
                reason_code,
                actor=actor,
                action_id=action_id,
            )

    def require_mutation_allowed(self, operation: str) -> None:
        with self.database.connect() as connection:
            row = self._state_row(connection)
        if row is None:
            raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
        state = SupervisionState(row["state"])
        if state in {SupervisionState.HEALTHY, SupervisionState.DEGRADED}:
            return
        if state is SupervisionState.DRAINING and operation in _DRAIN_ALLOWED_OPERATIONS:
            return
        raise CoordinatorError(
            "service_not_accepting_mutations",
            f"Coordinator supervision state is {state.value}",
            details={"state": state.value, "operation": operation},
        )

    def snapshot(
        self,
        connection: Connection | None = None,
        *,
        exclude_action_id: str | None = None,
    ) -> SupervisionSnapshot:
        if connection is not None:
            return self._snapshot(connection, exclude_action_id=exclude_action_id)
        with self.database.connect() as opened:
            return self._snapshot(opened, exclude_action_id=exclude_action_id)

    def record_recovery(
        self,
        *,
        failure_count: int,
        failure_window_started_at: int | None,
        next_retry_at: int | None,
        circuit_open_until: int | None,
        healthy_since: int | None,
    ) -> SupervisionSnapshot:
        """Persist the tray's bounded restart policy without changing daemon state."""
        if type(failure_count) is not int or not 0 <= failure_count <= 5:
            raise CoordinatorError(
                "recovery_state_invalid", "Recovery failure count must be between zero and five"
            )
        timestamps = {
            "failure_window_started_at": self._recovery_epoch_text(
                failure_window_started_at, "failureWindowStartedAt"
            ),
            "next_retry_at": self._recovery_epoch_text(next_retry_at, "nextRetryAt"),
            "circuit_open_until": self._recovery_epoch_text(
                circuit_open_until, "circuitOpenUntil"
            ),
            "healthy_since": self._recovery_epoch_text(healthy_since, "healthySince"),
        }
        if failure_count == 0 and any(
            timestamps[key] is not None
            for key in (
                "failure_window_started_at",
                "next_retry_at",
                "circuit_open_until",
            )
        ):
            raise CoordinatorError(
                "recovery_state_invalid",
                "A cleared recovery policy cannot retain failure or retry deadlines",
            )
        if failure_count > 0 and timestamps["failure_window_started_at"] is None:
            raise CoordinatorError(
                "recovery_state_invalid", "Recovery failures require a failure window"
            )
        if failure_count < 5 and timestamps["circuit_open_until"] is not None:
            raise CoordinatorError(
                "recovery_state_invalid", "Only five failures may open the recovery circuit"
            )
        if failure_count == 5 and (
            timestamps["circuit_open_until"] is None
            or timestamps["next_retry_at"] is not None
        ):
            raise CoordinatorError(
                "recovery_state_invalid",
                "An open recovery circuit requires a reset deadline and no retry deadline",
            )

        reason_code = (
            "tray.recovery_clear"
            if failure_count == 0
            else "tray.recovery_circuit_open"
            if failure_count == 5
            else "tray.recovery_backoff"
        )
        with self._transition_lock, self.database.transaction() as connection:
            row = self._state_row(connection)
            if row is None:
                raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
            updates: dict[str, object] = {"failure_count": failure_count, **timestamps}
            if all(row[key] == value for key, value in updates.items()):
                return self._snapshot(connection, exclude_action_id=None)
            now = utc_text()
            connection.execute(
                """
                UPDATE service_recovery_state
                SET failure_count=?, failure_window_started_at=?, next_retry_at=?,
                    circuit_open_until=?, healthy_since=?, updated_at=?, last_reason_code=?
                WHERE repository_key=?
                """,
                (
                    failure_count,
                    timestamps["failure_window_started_at"],
                    timestamps["next_retry_at"],
                    timestamps["circuit_open_until"],
                    timestamps["healthy_since"],
                    now,
                    reason_code,
                    self.repository_key,
                ),
            )
            state = SupervisionState(row["state"])
            self._append_event(
                connection,
                state,
                state,
                reason_code,
                actor="zircon-session-tray",
            )
        return self.snapshot()

    @staticmethod
    def _recovery_epoch_text(value: int | None, field: str) -> str | None:
        if value is None:
            return None
        if type(value) is not int or not 0 <= value <= 253_402_300_799:
            raise CoordinatorError(
                "recovery_state_invalid", f"{field} must be a valid Unix timestamp"
            )
        return datetime.fromtimestamp(value, tz=timezone.utc).isoformat()

    def _snapshot(
        self, connection: Connection, *, exclude_action_id: str | None
    ) -> SupervisionSnapshot:
        row = self._state_row(connection)
        if row is None:
            raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
        return SupervisionSnapshot(
            repository_key=row["repository_key"],
            state=SupervisionState(row["state"]),
            daemon_instance_id=row["daemon_instance_id"],
            process_id=row["process_id"],
            process_creation_time=row["process_creation_time"],
            explicit_stop=bool(row["explicit_stop"]),
            maintenance_hold=bool(row["maintenance_hold"]),
            failure_count=int(row["failure_count"]),
            next_retry_at=row["next_retry_at"],
            circuit_open_until=row["circuit_open_until"],
            healthy_since=row["healthy_since"],
            last_reason_code=row["last_reason_code"],
            updated_at=row["updated_at"],
            blockers=self.blockers(connection, exclude_action_id=exclude_action_id),
        )

    def blockers(
        self, connection: Connection, *, exclude_action_id: str | None = None
    ) -> tuple[SupervisionBlocker, ...]:
        blockers: list[SupervisionBlocker] = []
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.GIT_FINALIZE,
                row["request_id"],
                row["status"],
                session_id=row["session_id"],
            )
            for row in connection.execute(
                "SELECT request_id, session_id, status FROM finalize_requests "
                "WHERE status='finalizing' ORDER BY request_id"
            )
        )
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.CARGO,
                row["job_id"],
                row["status"],
                session_id=row["session_id"],
            )
            for row in connection.execute(
                "SELECT job_id, session_id, status FROM cargo_jobs "
                "WHERE status IN ('leased', 'running') ORDER BY job_id"
            )
        )
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.PATCH,
                str(row["patch_id"]),
                row["status"],
                session_id=row["session_id"],
            )
            for row in connection.execute(
                "SELECT patch_id, session_id, status FROM patches "
                "WHERE status='applying' ORDER BY patch_id"
            )
        )
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.VALIDATION,
                row["job_id"],
                row["status"],
                session_id=row["session_id"],
            )
            for row in connection.execute(
                "SELECT job_id, session_id, status FROM validation_copies "
                "WHERE status IN ('running', 'cleanup_pending') ORDER BY job_id"
            )
        )
        query = (
            "SELECT action_id, actor, status FROM action_requests "
            "WHERE status='executing'"
        )
        parameters: tuple[object, ...] = ()
        if exclude_action_id:
            query += " AND action_id<>?"
            parameters = (exclude_action_id,)
        query += " ORDER BY action_id"
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.CONTROLLED_ACTION,
                row["action_id"],
                row["status"],
                session_id=row["actor"],
            )
            for row in connection.execute(query, parameters)
        )
        if self._maintenance_active():
            blockers.append(
                SupervisionBlocker(BlockerKind.MAINTENANCE, "maintenance", "running")
            )
        blockers.extend(
            SupervisionBlocker(
                BlockerKind.LEASE,
                row["display_path"],
                "active",
                blocking=False,
                session_id=row["session_id"],
            )
            for row in connection.execute(
                "SELECT display_path, session_id FROM leases ORDER BY path_key"
            )
        )
        return tuple(blockers)

    def create_intent(
        self,
        kind: LifecycleKind,
        *,
        action_id: str,
        actor: str,
        deadline_at: str | None,
    ) -> str:
        intent_id = uuid.uuid4().hex
        now = utc_text()
        with self.database.transaction() as connection:
            if kind in {
                LifecycleKind.STOP,
                LifecycleKind.RESTART,
                LifecycleKind.FORCE_STOP,
            }:
                active = connection.execute(
                    """
                    SELECT intent_id, action_id, kind
                    FROM service_lifecycle_intents
                    WHERE repository_key=?
                      AND kind IN ('service.stop', 'service.restart', 'service.force_stop')
                      AND status IN ('accepted', 'draining')
                    LIMIT 1
                    """,
                    (self.repository_key,),
                ).fetchone()
                if active is not None:
                    raise CoordinatorError(
                        "lifecycle_already_active",
                        "A reversible service lifecycle is already draining",
                        details={
                            "actionId": active["action_id"],
                            "kind": active["kind"],
                        },
                    )
            connection.execute(
                """
                INSERT INTO service_lifecycle_intents(
                    intent_id, repository_key, action_id, kind, status,
                    requested_by, source_daemon_instance_id, deadline_at,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, 'accepted', ?, ?, ?, ?, ?)
                """,
                (
                    intent_id,
                    self.repository_key,
                    action_id,
                    kind.value,
                    actor,
                    self.daemon_instance_id,
                    deadline_at,
                    now,
                    now,
                ),
            )
        return intent_id

    def cancel_lifecycle(
        self, action_id: str, *, actor: str, reason: str
    ) -> dict[str, object]:
        """Atomically cancel a reversible drain, its action, and supervision state."""
        with self._transition_lock, self.database.transaction() as connection:
            row = connection.execute(
                """
                SELECT intent_id, status FROM service_lifecycle_intents
                WHERE repository_key=? AND action_id=?
                """,
                (self.repository_key, action_id),
            ).fetchone()
            if row is None or row["status"] not in {
                LifecycleStatus.ACCEPTED.value,
                LifecycleStatus.DRAINING.value,
            }:
                status = row["status"] if row is not None else "missing"
                raise CoordinatorError(
                    "action_not_cancellable", f"Lifecycle intent is {status}"
                )
            action = connection.execute(
                "SELECT status, action_kind FROM action_requests WHERE action_id=?",
                (action_id,),
            ).fetchone()
            if action is None or action["status"] != "executing":
                status = action["status"] if action is not None else "missing"
                raise CoordinatorError("action_not_cancellable", f"Action is {status}")
            now = utc_text()
            result = {"intentId": row["intent_id"], "state": "healthy"}
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='cancelled', error_code='lifecycle_cancelled',
                    result_json=?, updated_at=?, completed_at=?
                WHERE intent_id=?
                """,
                (json.dumps(result, sort_keys=True), now, now, row["intent_id"]),
            )
            connection.execute(
                """
                UPDATE action_requests
                SET status='cancelled', reason=?, result_json=?, completed_at=?
                WHERE action_id=? AND status='executing'
                """,
                (reason, json.dumps(result, sort_keys=True), now, action_id),
            )
            self._transition(
                connection,
                SupervisionState.HEALTHY,
                reason_code="lifecycle.cancelled",
                actor=actor,
                action_id=action_id,
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "action.cancelled",
                    json.dumps(
                        {"actionId": action_id, "kind": action["action_kind"]},
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        return result

    def fail_lifecycle(
        self, action_id: str, *, actor: str, error_code: str
    ) -> dict[str, object] | None:
        """Atomically terminate an active lifecycle/action pair and release draining."""
        with self._transition_lock, self.database.transaction() as connection:
            row = connection.execute(
                """
                SELECT intent.intent_id, intent.status AS intent_status,
                       action.status AS action_status
                FROM service_lifecycle_intents AS intent
                LEFT JOIN action_requests AS action ON action.action_id=intent.action_id
                WHERE intent.repository_key=? AND intent.action_id=?
                """,
                (self.repository_key, action_id),
            ).fetchone()
            if row is None or row["intent_status"] not in {
                LifecycleStatus.ACCEPTED.value,
                LifecycleStatus.DRAINING.value,
            }:
                return None
            now = utc_text()
            result = {
                "intentId": row["intent_id"],
                "state": "healthy",
                "errorCode": error_code,
                "reconciled": True,
            }
            encoded = json.dumps(result, sort_keys=True)
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='failed', error_code=?, result_json=?,
                    updated_at=?, completed_at=?
                WHERE intent_id=? AND status IN ('accepted', 'draining')
                """,
                (error_code, encoded, now, now, row["intent_id"]),
            )
            connection.execute(
                """
                UPDATE action_requests
                SET status='failed', error_code=?, result_json=?, completed_at=?
                WHERE action_id=? AND status IN ('previewed', 'executing')
                """,
                (error_code, encoded, now, action_id),
            )
            state = SupervisionState(self._state_row(connection)["state"])
            if state is SupervisionState.DRAINING:
                self._transition(
                    connection,
                    SupervisionState.HEALTHY,
                    reason_code="lifecycle.failure_reconciled",
                    actor=actor,
                    action_id=action_id,
                )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "lifecycle.failure_reconciled",
                    json.dumps(
                        {
                            "actionId": action_id,
                            "errorCode": error_code,
                            "intentId": row["intent_id"],
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        return result

    def commit_lifecycle_offline(
        self,
        intent_id: str,
        action_id: str,
        *,
        actor: str,
        kind: LifecycleKind,
    ) -> dict[str, object]:
        """Atomically cross the irreversible stopping/offline lifecycle boundary."""
        with self._transition_lock, self.database.transaction() as connection:
            intent = connection.execute(
                """
                SELECT status FROM service_lifecycle_intents
                WHERE repository_key=? AND intent_id=? AND action_id=? AND kind=?
                """,
                (self.repository_key, intent_id, action_id, kind.value),
            ).fetchone()
            action = connection.execute(
                "SELECT status FROM action_requests WHERE action_id=?",
                (action_id,),
            ).fetchone()
            if (
                intent is None
                or intent["status"] not in {
                    LifecycleStatus.ACCEPTED.value,
                    LifecycleStatus.DRAINING.value,
                }
                or action is None
                or action["status"] != "executing"
            ):
                raise CoordinatorError(
                    "lifecycle_commit_invalid",
                    "Lifecycle cannot cross offline without active intent and action proof",
                )
            self._transition(
                connection,
                SupervisionState.STOPPING,
                reason_code=f"lifecycle.{kind.name.lower()}.ready",
                actor=actor,
                action_id=action_id,
            )
            now = utc_text()
            result = {
                "intentId": intent_id,
                "state": "offline",
                "awaitingForceStopAcknowledgement": kind is LifecycleKind.FORCE_STOP,
            }
            if kind is LifecycleKind.RESTART:
                connection.execute(
                    """
                    UPDATE service_lifecycle_intents
                    SET status='awaiting_restart', updated_at=?
                    WHERE intent_id=? AND status IN ('accepted', 'draining')
                    """,
                    (now, intent_id),
                )
            else:
                encoded = json.dumps(result, sort_keys=True)
                connection.execute(
                    """
                    UPDATE service_lifecycle_intents
                    SET status='succeeded', error_code=NULL, result_json=?,
                        updated_at=?, completed_at=?
                    WHERE intent_id=? AND status IN ('accepted', 'draining')
                    """,
                    (encoded, now, now, intent_id),
                )
                connection.execute(
                    """
                    UPDATE action_requests
                    SET status='succeeded', error_code=NULL, result_json=?, completed_at=?
                    WHERE action_id=? AND status='executing'
                    """,
                    (encoded, now, action_id),
                )
            self._transition(
                connection,
                SupervisionState.OFFLINE,
                reason_code=f"lifecycle.{kind.name.lower()}.offline",
                actor=actor,
                action_id=action_id,
                updates={"explicit_stop": 0 if kind is LifecycleKind.RESTART else 1},
            )
        return result

    def record_lifecycle_shutdown_retry(
        self,
        action_id: str,
        *,
        kind: LifecycleKind,
        attempt: int,
        error_code: str,
    ) -> None:
        """Audit a sanitized post-commit shutdown retry without rewriting terminal proof."""
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "lifecycle.shutdown_retry",
                    json.dumps(
                        {
                            "actionId": action_id,
                            "attempt": attempt,
                            "errorCode": error_code,
                            "kind": kind.value,
                        },
                        sort_keys=True,
                    ),
                    utc_text(),
                ),
            )

    def update_intent(
        self,
        intent_id: str,
        status: LifecycleStatus,
        *,
        error_code: str | None = None,
        result: dict[str, object] | None = None,
        completed: bool = False,
        successor_instance_id: str | None = None,
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status=?, error_code=?, result_json=?, updated_at=?,
                    completed_at=?, successor_daemon_instance_id=COALESCE(?, successor_daemon_instance_id)
                WHERE intent_id=?
                """,
                (
                    status.value,
                    error_code,
                    json.dumps(result, sort_keys=True) if result is not None else None,
                    utc_text(),
                    utc_text() if completed else None,
                    successor_instance_id,
                    intent_id,
                ),
            )

    def _state_row(self, connection: Connection):
        return connection.execute(
            "SELECT * FROM service_recovery_state WHERE repository_key=?",
            (self.repository_key,),
        ).fetchone()

    def _append_event(
        self,
        connection: Connection,
        previous: SupervisionState | None,
        state: SupervisionState,
        reason_code: str,
        *,
        actor: str,
        action_id: str | None = None,
    ) -> None:
        sequence = int(
            connection.execute(
                "SELECT COALESCE(MAX(sequence), 0) + 1 FROM service_supervision_events "
                "WHERE repository_key=?",
                (self.repository_key,),
            ).fetchone()[0]
        )
        connection.execute(
            """
            INSERT INTO service_supervision_events(
                repository_key, sequence, from_state, to_state,
                daemon_instance_id, process_id, process_creation_time,
                reason_code, actor, action_id, payload_json, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '{}', ?)
            """,
            (
                self.repository_key,
                sequence,
                previous.value if previous else None,
                state.value,
                self.daemon_instance_id,
                self.process_id,
                self.process_creation_time,
                reason_code,
                actor,
                action_id,
                utc_text(),
            ),
        )
