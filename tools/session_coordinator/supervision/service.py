from __future__ import annotations

import hashlib
import json
import os
import threading
import uuid
from collections.abc import Callable, Iterable
from contextlib import AbstractContextManager, nullcontext
from datetime import datetime, timedelta, timezone
from sqlite3 import Connection

from ..cargo_reservations import (
    cpu_warm_fifo_predecessor,
    reconcile_cpu_fifo_eligibility,
)
from ..database import Database
from ..models import CoordinatorError, SupervisionState, parse_utc, utc_now, utc_text
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
            SupervisionState.DRAINING,
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
        "service.resume.release",
        "service.stop",
        "service.restart",
        "service.force_stop",
        "session.heartbeat",
        "lease.release",
        "lease.heartbeat",
        "cargo.heartbeat",
        "cargo.finish",
        "cargo.release",
        "cargo.renew_cpu_reservation",
        "cargo.release_cpu_reservation",
        "codex.sessions.reconcile",
        "milestone.reconcile_accepted",
        "supervision.recovery_record",
    }
)

# A successor needs a short quiet period so concurrent local monitors cannot
# turn one requested rollover into repeated service replacements.
ROLLOVER_STABILIZATION_SECONDS = 60

_MAINTENANCE_SESSION_OPERATIONS = frozenset(
    {
        "baseline.attribute",
        "lease.claim",
        "lease.claim_own_scope",
        "failure.return",
        "session.activate",
        "session.register",
        "session.set_status",
        "topology.refresh",
        "validation.start",
        "milestone.review",
        "milestone.commit",
        "maintenance.cleanup",
        # A maintenance-scoped dependency closure may finalize only its
        # explicitly leased manifest; generic finalize remains unavailable.
        "finalize.preview",
        "finalize.commit",
        # A held coordinator may consume and run only the already-audited CPU
        # reservation of an explicitly configured Session.  New reservation,
        # recovery, promotion, generic acquisition, and generic process start
        # remain outside this exception.
        "cargo.consume_cpu_reservation",
        "cargo.run_reserved",
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
        maintenance_session_id: str | None = None,
        maintenance_session_ids: Iterable[str] | None = None,
    ):
        self.database = database
        self.repository_key = repository_key
        self.daemon_instance_id = daemon_instance_id
        self.process_id = os.getpid()
        self.process_creation_time = process_creation_time
        self._maintenance_active = maintenance_active or (lambda: False)
        scoped_ids = set(maintenance_session_ids or ())
        if maintenance_session_id:
            scoped_ids.add(maintenance_session_id)
        self._maintenance_session_ids = frozenset(
            session_id.strip() for session_id in scoped_ids if session_id.strip()
        )
        self._transition_lock = threading.RLock()
        self._cargo_start_transition: Callable[[], AbstractContextManager[None]] = (
            nullcontext
        )

    def set_cargo_start_transition(
        self, transition: Callable[[], AbstractContextManager[None]]
    ) -> None:
        """Serialize rollover arming with the managed process-launch interval."""
        self._cargo_start_transition = transition

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

    def require_mutation_allowed(self, operation: str) -> str:
        """Return the durable state revision that authorized this mutation."""
        with self.database.connect() as connection:
            return self._require_mutation_allowed(connection, operation)

    def require_mutation_allowed_in_connection(
        self, connection: Connection, operation: str
    ) -> str:
        """Authorize a mutation inside the transaction that records its intent."""
        return self._require_mutation_allowed(connection, operation)

    def require_mutation_checkpoint(
        self, connection: Connection, operation: str, checkpoint: str
    ) -> None:
        """Reject work that passed an earlier gate before a hold was committed."""
        row = self._state_row(connection)
        if row is None:
            raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
        if str(row["updated_at"]) != checkpoint:
            raise CoordinatorError(
                "admission_checkpoint_stale",
                "Coordinator admission changed after this mutation was authorized",
                details={"operation": operation},
            )
        self._require_mutation_allowed(connection, operation)

    def require_cargo_start_allowed_in_connection(
        self, connection: Connection, operation: str
    ) -> None:
        """Fence new Cargo starts after this daemon commits its rollover handoff."""
        intent = connection.execute(
            """
            SELECT intent_id, action_id FROM service_lifecycle_intents
            WHERE repository_key=? AND source_daemon_instance_id=?
              AND kind='service.rollover' AND status='awaiting_restart'
            ORDER BY updated_at DESC, intent_id DESC
            LIMIT 1
            """,
            (self.repository_key, self.daemon_instance_id),
        ).fetchone()
        if intent is None:
            return
        raise CoordinatorError(
            "cargo_start_rollover_pending",
            "The current daemon has committed its rollover handoff; Cargo start is deferred to the successor",
            details={
                "operation": operation,
                "intentId": intent["intent_id"],
                "actionId": intent["action_id"],
                "daemonInstanceId": self.daemon_instance_id,
            },
        )

    def _require_mutation_allowed(self, connection: Connection, operation: str) -> str:
        operation_name, separator, session_id = operation.partition("@")
        row = self._state_row(connection)
        maintenance_session_ids = self._maintenance_session_ids_for_hold(connection, row=row)
        if row is None:
            raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
        scoped_maintenance_operation = (
            bool(separator)
            and session_id in maintenance_session_ids
            and operation_name in _MAINTENANCE_SESSION_OPERATIONS
        )
        state = SupervisionState(row["state"])
        if operation_name == "service.resume.release" and separator:
            raise CoordinatorError(
                "maintenance_release_scope_invalid",
                "Explicit maintenance release is authorized by its drain action, not a Session",
                details={"state": state.value, "operation": operation},
            )
        if (
            maintenance_session_ids
            and operation_name == "service.resume"
            and (bool(row["explicit_stop"]) or bool(row["maintenance_hold"]))
        ):
            raise CoordinatorError(
                "maintenance_scope_resume_blocked",
                "A scoped maintenance daemon requires an unscoped explicit resume",
                details={"state": state.value, "operation": operation},
            )
        if (
            bool(row["explicit_stop"])
            and operation_name not in _DRAIN_ALLOWED_OPERATIONS
            and not scoped_maintenance_operation
        ):
            raise CoordinatorError(
                "service_explicit_stop_active",
                "Coordinator is under an explicit stop and is not accepting new mutations",
                details={"state": state.value, "operation": operation},
            )
        if (
            bool(row["maintenance_hold"])
            and operation_name not in _DRAIN_ALLOWED_OPERATIONS
            and not scoped_maintenance_operation
        ):
            raise CoordinatorError(
                "maintenance_hold_active",
                "Coordinator maintenance hold is not accepting new mutations",
                details={"state": state.value, "operation": operation},
            )
        if state in {SupervisionState.HEALTHY, SupervisionState.DEGRADED}:
            return str(row["updated_at"])
        if state is SupervisionState.DRAINING and (
            operation_name in _DRAIN_ALLOWED_OPERATIONS or scoped_maintenance_operation
        ):
            return str(row["updated_at"])
        raise CoordinatorError(
            "service_not_accepting_mutations",
            f"Coordinator supervision state is {state.value}",
            details={"state": state.value, "operation": operation},
        )

    def establish_proof_bound_drain(
        self,
        intent_id: str,
        *,
        action_id: str,
        actor: str,
    ) -> dict[str, object]:
        """Bind the first scoped drain as the durable admission proof.

        Later drain actions are auditable coalescings only: they cannot swap
        the scope that is permitted to consume the existing proof reservation.
        """
        with self._transition_lock, self.database.transaction() as connection:
            state = self._state_row(connection)
            if state is None:
                raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
            proof = self._proof_bound_drain(connection)
            if bool(state["maintenance_hold"]) and proof is not None:
                result = {
                    "intentId": intent_id,
                    "state": SupervisionState(state["state"]).value,
                    "admissionOpen": False,
                    "proofBound": False,
                    "coalescedToActionId": proof["action_id"],
                }
            else:
                blockers = self._snapshot(connection, exclude_action_id=action_id).blockers
                self._transition(
                    connection,
                    SupervisionState.DRAINING,
                    reason_code="lifecycle.drain.proof_bound_hold",
                    actor=actor,
                    action_id=action_id,
                    updates={"maintenance_hold": 1},
                )
                result = {
                    "intentId": intent_id,
                    "state": SupervisionState.DRAINING.value,
                    "admissionOpen": False,
                    "proofBound": True,
                    "blockers": [item.to_dict() for item in blockers],
                }
            now = utc_text()
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='succeeded', error_code=NULL, result_json=?, updated_at=?, completed_at=?
                WHERE intent_id=? AND status IN ('accepted', 'draining', 'ready')
                """,
                (json.dumps(result, sort_keys=True), now, now, intent_id),
            )
        return result

    def proof_bound_drain(self, connection=None):
        """Return the immutable drain proof that currently owns the hold scope."""
        if connection is not None:
            return self._proof_bound_drain(connection)
        with self.database.connect() as opened:
            return self._proof_bound_drain(opened)

    def bootstrap_proof_bound_handoff(
        self,
        *,
        reservation_id: str,
        maintenance_session_ids: Iterable[str],
        actor: str,
        expected_daemon_instance_id: str | None = None,
        expected_process_id: int | None = None,
        expected_process_creation_time: str | None = None,
    ) -> dict[str, object]:
        """Persist a quiet-window proof before replacing a legacy coordinator.

        It binds the resulting hold to exactly one existing pending CPU
        reservation. Generic admission is rejected by the predecessor as soon
        as this transaction commits, and a successor reads the same proof from
        SQLite rather than from transient environment state.
        """
        normalized_reservation = reservation_id.strip()
        normalized_actor = actor.strip()
        session_ids = tuple(
            session_id.strip()
            for session_id in maintenance_session_ids
            if isinstance(session_id, str) and session_id.strip()
        )
        if not normalized_reservation:
            raise CoordinatorError(
                "bootstrap_reservation_invalid",
                "Bootstrap requires one existing CPU reservation ID",
            )
        if not normalized_actor:
            raise CoordinatorError(
                "bootstrap_actor_invalid", "Bootstrap requires an auditable actor"
            )
        if not session_ids or len(session_ids) > 16 or len(set(session_ids)) != len(session_ids):
            raise CoordinatorError(
                "bootstrap_maintenance_scope_invalid",
                "Bootstrap maintenance Sessions must be unique and non-empty",
            )

        with self._transition_lock, self.database.transaction() as connection:
            state = self._state_row(connection)
            if state is None:
                raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
            if (
                expected_daemon_instance_id is not None
                and state["daemon_instance_id"] != expected_daemon_instance_id
            ) or (
                expected_process_id is not None and int(state["process_id"]) != expected_process_id
            ) or (
                expected_process_creation_time is not None
                and state["process_creation_time"] != expected_process_creation_time
            ):
                raise CoordinatorError(
                    "bootstrap_predecessor_changed",
                    "Coordinator predecessor changed before the proof-bound handoff committed",
                )
            proof = self._proof_bound_drain(connection)
            if bool(state["maintenance_hold"]):
                if proof is None:
                    raise CoordinatorError(
                        "bootstrap_hold_proof_missing",
                        "A pre-existing maintenance hold has no durable proof",
                    )
                if (
                    self._proof_result(proof).get("bootstrapReservationId")
                    == normalized_reservation
                    and self._proof_session_ids(proof) == session_ids
                ):
                    return {
                        "actionId": proof["action_id"],
                        "admissionOpen": False,
                        "proofBound": True,
                        "reservationId": normalized_reservation,
                        "recovered": True,
                    }
                raise CoordinatorError(
                    "bootstrap_hold_active",
                    "A different proof-bound maintenance hold is already active",
                )

            active_jobs = connection.execute(
                "SELECT job_id FROM cargo_jobs WHERE status IN ('leased', 'running') ORDER BY job_id"
            ).fetchall()
            if active_jobs:
                raise CoordinatorError(
                    "bootstrap_cargo_active",
                    "Bootstrap requires no leased or running Cargo job",
                    details={"jobIds": [row["job_id"] for row in active_jobs]},
                )
            reconcile_cpu_fifo_eligibility(connection, now=utc_text())
            reservation = connection.execute(
                """
                SELECT reservation_id, session_id, lane_scope, status, job_id,
                       compatibility_key, compatibility_json, command_fingerprint,
                       created_at, priority_rank, execution_mode
                FROM cargo_lane_reservations
                WHERE reservation_id=?
                """,
                (normalized_reservation,),
            ).fetchone()
            if (
                reservation is None
                or reservation["lane_scope"] != "cpu"
                or reservation["execution_mode"] != "warm"
                or reservation["status"] != "pending"
                or reservation["job_id"] is not None
                or reservation["session_id"] not in session_ids
            ):
                raise CoordinatorError(
                    "bootstrap_reservation_ineligible",
                    "Bootstrap proof must bind one pending CPU reservation in its maintenance scope",
                )
            binding = self._reservation_proof_binding(connection, reservation)
            if binding["sourceManifestFingerprint"] is None:
                raise CoordinatorError(
                    "bootstrap_reservation_payload_invalid",
                    "Bootstrap proof requires an existing source-bound reservation",
                )
            if binding["fifoPredecessor"] is not None:
                raise CoordinatorError(
                    "bootstrap_reservation_not_fifo_head",
                    "Bootstrap proof may bind only the current FIFO CPU head",
                    details={"predecessor": binding["fifoPredecessor"]},
                )
            reservation_ledger = self._cpu_reservation_ledger(connection)
            writable = connection.execute(
                "SELECT session_id, status FROM sessions WHERE session_id IN ({})".format(
                    ",".join("?" for _ in session_ids)
                ),
                session_ids,
            ).fetchall()
            writable_statuses = {"active", "resolving_failure"}
            if (
                len(writable) != len(session_ids)
                or any(row["status"] not in writable_statuses for row in writable)
            ):
                raise CoordinatorError(
                    "bootstrap_maintenance_session_not_writable",
                    "Bootstrap maintenance Sessions must be active or resolving failure",
                )

            action_id = uuid.uuid4().hex
            intent_id = uuid.uuid4().hex
            now = utc_text()
            parameters = {
                "timeoutSeconds": 300,
                "maintenanceSessionIds": list(session_ids),
            }
            result = {
                "intentId": intent_id,
                "state": SupervisionState.DRAINING.value,
                "admissionOpen": False,
                "proofBound": True,
                "bootstrapReservationId": normalized_reservation,
                "bootstrapProtocol": "proof-bound-handoff-v1",
                "reservationBinding": binding,
                "cpuReservationLedger": reservation_ledger,
                "predecessor": {
                    "instanceId": state["daemon_instance_id"],
                    "pid": state["process_id"],
                    "processCreationTime": state["process_creation_time"],
                },
            }
            state_fingerprint = hashlib.sha256(
                json.dumps(
                    {
                        "repositoryKey": self.repository_key,
                        "reservationId": normalized_reservation,
                        "maintenanceSessionIds": list(session_ids),
                    },
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
            ).hexdigest()
            connection.execute(
                """
                INSERT INTO action_requests(
                    action_id, action_kind, risk, required_role, actor,
                    daemon_instance_id, parameters_json, impact_json, warnings_json,
                    state_fingerprint, confirmation_phrase_hash, status, result_json,
                    created_at, expires_at, confirmed_at, completed_at, reason
                ) VALUES (?, 'service.drain', 'red', 'maintainer', ?, ?, ?, '[]', '[]',
                          ?, ?, 'succeeded', ?, ?, ?, ?, ?, ?)
                """,
                (
                    action_id,
                    normalized_actor,
                    state["daemon_instance_id"],
                    json.dumps(parameters, sort_keys=True),
                    state_fingerprint,
                    hashlib.sha256(b"CONFIRM SERVICE.DRAIN").hexdigest(),
                    json.dumps(result, sort_keys=True),
                    now,
                    now,
                    now,
                    now,
                    "local proof-bound bootstrap handoff",
                ),
            )
            connection.execute(
                """
                INSERT INTO service_lifecycle_intents(
                    intent_id, repository_key, action_id, kind, status, requested_by,
                    source_daemon_instance_id, created_at, updated_at, completed_at, result_json
                ) VALUES (?, ?, ?, 'service.drain', 'succeeded', ?, ?, ?, ?, ?, ?)
                """,
                (
                    intent_id,
                    self.repository_key,
                    action_id,
                    normalized_actor,
                    state["daemon_instance_id"],
                    now,
                    now,
                    now,
                    json.dumps(result, sort_keys=True),
                ),
            )
            self._transition(
                connection,
                SupervisionState.DRAINING,
                reason_code="bootstrap.proof_bound_handoff",
                actor=normalized_actor,
                action_id=action_id,
                updates={"maintenance_hold": 1},
            )
            fence_event = connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "lifecycle.bootstrap_proof_bound_handoff",
                    json.dumps(
                        {
                            "actionId": action_id,
                            "intentId": intent_id,
                            "reservationId": normalized_reservation,
                            "maintenanceSessionIds": list(session_ids),
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
            result["bootstrapFenceEventId"] = int(fence_event.lastrowid)
            encoded_result = json.dumps(result, sort_keys=True)
            connection.execute(
                "UPDATE action_requests SET result_json=? WHERE action_id=?",
                (encoded_result, action_id),
            )
            connection.execute(
                "UPDATE service_lifecycle_intents SET result_json=? WHERE intent_id=?",
                (encoded_result, intent_id),
            )
        return {
            "actionId": action_id,
            "admissionOpen": False,
            "proofBound": True,
            "reservationId": normalized_reservation,
            "fenceEventId": result["bootstrapFenceEventId"],
            "recovered": False,
        }

    def require_proof_bound_reservation(
        self, reservation_id: str, *, session_id: str, job_id: str | None = None
    ) -> None:
        """Require the exact reservation selected by an active bootstrap proof."""
        with self.database.connect() as connection:
            self.require_proof_bound_reservation_in_connection(
                connection, reservation_id, session_id=session_id, job_id=job_id
            )

    def require_proof_bound_reservation_in_connection(
        self, connection: Connection, reservation_id: str, *, session_id: str, job_id: str | None = None
    ) -> None:
        """Run proof validation in the caller's reservation write transaction."""
        state = self._state_row(connection)
        if state is None or not bool(state["maintenance_hold"]):
            return
        reconcile_cpu_fifo_eligibility(connection, now=utc_text())
        proof = self._proof_bound_drain(connection)
        if proof is None:
            raise CoordinatorError(
                "maintenance_proof_missing",
                "Maintenance hold has no durable reservation proof",
            )
        result = self._proof_result(proof)
        binding = result.get("reservationBinding")
        if binding is None:
            return
        if (
            not isinstance(binding, dict)
            or binding.get("reservationId") != reservation_id
            or session_id not in self._proof_session_ids(proof)
        ):
            raise CoordinatorError(
                "maintenance_proof_reservation_mismatch",
                "Maintenance hold permits only its proof-bound reservation",
            )
        reservation = connection.execute(
            """
            SELECT reservation_id, session_id, lane_scope, status, job_id,
                   compatibility_key, compatibility_json, command_fingerprint,
                   created_at, priority_rank, execution_mode
            FROM cargo_lane_reservations WHERE reservation_id=?
            """,
            (reservation_id,),
        ).fetchone()
        if (
            reservation is None
            or reservation["session_id"] != session_id
        ):
            raise CoordinatorError(
                "maintenance_proof_reservation_mismatch",
                "Proof-bound reservation ownership or consumed job does not match",
            )
        current_binding = self._reservation_proof_binding(connection, reservation)
        immutable_fields = set(binding).difference({"status", "jobId", "fifoPredecessor"})
        if (
            binding.get("status") != "pending"
            or binding.get("jobId") is not None
            or any(binding.get(field) != current_binding.get(field) for field in immutable_fields)
            or (
                job_id is None
                and (
                    reservation["status"] != "pending"
                    or reservation["job_id"] is not None
                    or current_binding.get("fifoPredecessor") != binding.get("fifoPredecessor")
                )
            )
            or (
                job_id is not None
                and (reservation["status"] != "leased" or reservation["job_id"] != job_id)
            )
        ):
            raise CoordinatorError(
                "maintenance_proof_reservation_mismatch",
                "Proof-bound reservation ownership or consumed job does not match",
            )

    @staticmethod
    def _proof_result(proof) -> dict[str, object]:
        try:
            decoded = json.loads(proof["result_json"] or "{}")
        except (TypeError, ValueError, json.JSONDecodeError):
            return {}
        return decoded if isinstance(decoded, dict) else {}

    @staticmethod
    def _proof_session_ids(proof) -> tuple[str, ...]:
        try:
            parameters = json.loads(proof["parameters_json"])
        except (TypeError, ValueError, json.JSONDecodeError):
            return ()
        raw_ids = parameters.get("maintenanceSessionIds", [])
        if not isinstance(raw_ids, list):
            return ()
        return tuple(
            session_id.strip()
            for session_id in raw_ids
            if isinstance(session_id, str) and session_id.strip()
        )

    @classmethod
    def _reservation_proof_binding(cls, connection: Connection, reservation) -> dict[str, object]:
        """Return immutable payload and FIFO identity for a proof-bound reservation."""
        compatibility_json = reservation["compatibility_json"] or ""
        try:
            compatibility = json.loads(compatibility_json)
        except (TypeError, ValueError, json.JSONDecodeError):
            compatibility = None
        source_manifest = cls._source_manifest_from_compatibility(compatibility)
        predecessor = cpu_warm_fifo_predecessor(connection, reservation)
        return {
            "reservationId": reservation["reservation_id"],
            "sessionId": reservation["session_id"],
            "laneScope": reservation["lane_scope"],
            "status": reservation["status"],
            "jobId": reservation["job_id"],
            "compatibilityKey": reservation["compatibility_key"],
            "compatibilityPayloadFingerprint": hashlib.sha256(
                compatibility_json.encode("utf-8")
            ).hexdigest(),
            "commandFingerprint": reservation["command_fingerprint"],
            "sourceManifestFingerprint": cls._source_manifest_fingerprint(source_manifest),
            "createdAt": reservation["created_at"],
            "priorityRank": reservation["priority_rank"],
            "executionMode": reservation["execution_mode"],
            "fifoPredecessor": predecessor,
        }

    @classmethod
    def _cpu_reservation_ledger(cls, connection: Connection) -> list[dict[str, object]]:
        """Capture every live CPU reservation at the predecessor fence.

        Legacy schema49 reservation writes did not consistently emit a cargo
        event.  A durable row-level ledger closes that blind spot after the
        predecessor exits, including a stale promotion of an existing row.
        """
        rows = connection.execute(
            """
            SELECT rowid AS reservation_row_id, reservation_id, session_id, lane_scope,
                   status, job_id, compatibility_key, compatibility_json,
                   command_fingerprint, target_dir, execution_mode, burst_eligible,
                   created_at, priority_rank, failure_lifecycle_key
            FROM cargo_lane_reservations
            WHERE lane_scope='cpu' AND status IN ('pending', 'leased', 'running')
            ORDER BY reservation_row_id
            """
        ).fetchall()
        return [
            {
                "rowId": int(row["reservation_row_id"]),
                "reservationId": row["reservation_id"],
                "sessionId": row["session_id"],
                "laneScope": row["lane_scope"],
                "status": row["status"],
                "jobId": row["job_id"],
                "compatibilityKey": row["compatibility_key"],
                "compatibilityPayloadFingerprint": hashlib.sha256(
                    (row["compatibility_json"] or "").encode("utf-8")
                ).hexdigest(),
                "commandFingerprint": row["command_fingerprint"],
                "targetDir": row["target_dir"],
                "executionMode": row["execution_mode"],
                "burstEligible": bool(row["burst_eligible"]),
                "createdAt": row["created_at"],
                "priorityRank": row["priority_rank"],
                "failureLifecycleKey": row["failure_lifecycle_key"],
            }
            for row in rows
        ]

    @staticmethod
    def _source_manifest_from_compatibility(compatibility) -> dict[str, str]:
        if not isinstance(compatibility, dict):
            return {}
        raw_manifest = compatibility.get("source_manifest")
        if not isinstance(raw_manifest, dict):
            try:
                raw_manifest = json.loads(compatibility.get("build_config", "{}"))["source_manifest"]
            except (KeyError, TypeError, ValueError, json.JSONDecodeError):
                return {}
        if not isinstance(raw_manifest, dict):
            return {}
        return {
            str(path): str(digest).casefold()
            for path, digest in sorted(raw_manifest.items())
            if isinstance(path, str) and isinstance(digest, str)
        }

    @staticmethod
    def _source_manifest_fingerprint(source_manifest: dict[str, str]) -> str | None:
        if not source_manifest:
            return None
        payload = "\n".join(
            f"{path.casefold()}={digest.casefold()}"
            for path, digest in sorted(source_manifest.items())
        )
        return hashlib.sha256(payload.encode("utf-8")).hexdigest()

    def _maintenance_session_ids_for_hold(self, connection, *, row) -> frozenset[str]:
        if row is None or not bool(row["maintenance_hold"]):
            return self._maintenance_session_ids
        proof = self._proof_bound_drain(connection)
        if proof is None:
            return self._maintenance_session_ids
        return frozenset(self._proof_session_ids(proof))

    def _proof_bound_drain(self, connection):
        return connection.execute(
            """
            SELECT intent.intent_id, intent.action_id, intent.result_json, action.parameters_json
            FROM service_lifecycle_intents AS intent
            JOIN action_requests AS action ON action.action_id=intent.action_id
            WHERE intent.repository_key=?
              AND intent.kind='service.drain'
              AND intent.status='succeeded'
              AND json_extract(intent.result_json, '$.proofBound')=1
            ORDER BY intent.created_at DESC, intent.intent_id DESC
            LIMIT 1
            """,
            (self.repository_key,),
        ).fetchone()

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
                LifecycleKind.ROLLOVER,
            }:
                active = connection.execute(
                    """
                    SELECT intent_id, action_id, kind
                    FROM service_lifecycle_intents
                    WHERE repository_key=?
                      AND kind IN ('service.stop', 'service.restart', 'service.force_stop', 'service.rollover')
                      AND status IN ('accepted', 'draining', 'awaiting_restart')
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

    def recent_rollover_successor(self) -> dict[str, object] | None:
        """Return the healthy current successor while its rollover is stabilizing."""
        with self.database.connect() as connection:
            state = self._state_row(connection)
            row = connection.execute(
                """
                SELECT intent_id, action_id, successor_daemon_instance_id, completed_at
                FROM service_lifecycle_intents
                WHERE repository_key=? AND kind='service.rollover' AND status='succeeded'
                  AND successor_daemon_instance_id=? AND completed_at IS NOT NULL
                ORDER BY completed_at DESC, intent_id DESC
                LIMIT 1
                """,
                (self.repository_key, self.daemon_instance_id),
            ).fetchone()
        if (
            state is None
            or state["state"] != SupervisionState.HEALTHY.value
            or bool(state["maintenance_hold"])
            or row is None
            or utc_now() - parse_utc(row["completed_at"])
            > timedelta(seconds=ROLLOVER_STABILIZATION_SECONDS)
        ):
            return None
        return {
            "state": SupervisionState.HEALTHY.value,
            "admissionOpen": True,
            "coalesced": True,
            "coalescedIntentId": row["intent_id"],
            "coalescedActionId": row["action_id"],
            "successorInstanceId": row["successor_daemon_instance_id"],
        }

    def arm_rollover(self, intent_id: str, *, action_id: str, actor: str) -> dict[str, object]:
        """Persist a short reload handoff without closing general task admission.

        Live Cargo descendants defer the handoff while task admission stays open.
        Leased-but-unstarted jobs remain in SQLite so the successor preserves
        their exact reservation and FIFO state, but they have no accepted Cargo
        process identity and therefore cannot defer the handoff.
        """
        with (
            self._cargo_start_transition(),
            self._transition_lock,
            self.database.transaction() as connection,
        ):
            intent = connection.execute(
                """
                SELECT intent_id, status, kind, result_json
                FROM service_lifecycle_intents
                WHERE repository_key=? AND intent_id=? AND action_id=?
                """,
                (self.repository_key, intent_id, action_id),
            ).fetchone()
            action = connection.execute(
                "SELECT status FROM action_requests WHERE action_id=?", (action_id,)
            ).fetchone()
            if (
                intent is None
                or intent["kind"] != LifecycleKind.ROLLOVER.value
                or intent["status"] != LifecycleStatus.ACCEPTED.value
                or action is None
                or action["status"] != "executing"
            ):
                raise CoordinatorError(
                    "lifecycle_rollover_invalid",
                    "Rollover requires an accepted intent and executing controlled action",
                )
            live_jobs: list[dict[str, object]] = []
            for row in connection.execute(
                """
                SELECT job_id, session_id, status, process_tree_live_pids_json
                FROM cargo_jobs
                WHERE status IN ('running', 'orphaned')
                ORDER BY job_id
                """
            ):
                try:
                    pids = sorted(
                        {int(value) for value in json.loads(row["process_tree_live_pids_json"] or "[]")}
                    )
                except (TypeError, ValueError, json.JSONDecodeError):
                    pids = []
                if pids:
                    live_jobs.append(
                        {
                            "jobId": row["job_id"],
                            "sessionId": row["session_id"],
                            "status": row["status"],
                            "liveProcessPids": pids,
                        }
                    )
            pending_starts = [
                {
                    "requestId": row["request_id"],
                    "reservationId": row["reservation_id"],
                    "jobId": row["job_id"],
                    "sessionId": row["session_id"],
                    "acknowledgedAt": row["acknowledged_at"],
                    "deadlineAt": row["deadline_at"],
                }
                for row in connection.execute(
                    """
                    SELECT request_id, reservation_id, job_id, session_id,
                           acknowledged_at, deadline_at
                    FROM cargo_start_requests
                    WHERE status='start_pending'
                    ORDER BY acknowledged_at, request_id
                    """
                )
            ]
            if live_jobs or pending_starts:
                result = {
                    "intentId": intent_id,
                    "state": "healthy",
                    "admissionOpen": True,
                    "successorPending": False,
                    "waitingForCargo": True,
                    "liveCargo": live_jobs,
                    "pendingCargoStarts": pending_starts,
                }
                previous_result = json.loads(intent["result_json"] or "{}")
                if (
                    previous_result.get("waitingForCargo")
                    and previous_result.get("liveCargo") == live_jobs
                    and previous_result.get("pendingCargoStarts") == pending_starts
                ):
                    # 保持准入开放时，相同 Cargo 等待不能无限刷审计流。
                    return previous_result
                now = utc_text()
                connection.execute(
                    """
                    UPDATE service_lifecycle_intents
                    SET result_json=?, updated_at=?
                    WHERE intent_id=? AND status='accepted'
                    """,
                    (json.dumps(result, sort_keys=True), now, intent_id),
                )
                connection.execute(
                    "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                    (
                        "lifecycle.rollover_waiting_for_cargo",
                        json.dumps(
                            {
                                "actionId": action_id,
                                "intentId": intent_id,
                                "jobs": live_jobs,
                                "pendingStarts": pending_starts,
                                "requestedBy": actor,
                            },
                            sort_keys=True,
                        ),
                        now,
                    ),
                )
                return result
            now = utc_text()
            result = {
                "intentId": intent_id,
                "state": "healthy",
                "admissionOpen": True,
                "successorPending": True,
            }
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='awaiting_restart', result_json=?, updated_at=?
                WHERE intent_id=? AND status='accepted'
                """,
                (json.dumps(result, sort_keys=True), now, intent_id),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "lifecycle.rollover_armed",
                    json.dumps(
                        {
                            "actionId": action_id,
                            "intentId": intent_id,
                            "requestedBy": actor,
                        },
                        sort_keys=True,
                    ),
                    now,
                ),
            )
        return result

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
                LifecycleStatus.AWAITING_RESTART.value,
            }:
                return None
            recovery_state = self._state_row(connection)
            state = SupervisionState(recovery_state["state"])
            preserve_maintenance_hold = bool(recovery_state["maintenance_hold"])
            now = utc_text()
            result = {
                "intentId": row["intent_id"],
                "state": "draining" if preserve_maintenance_hold else "healthy",
                "errorCode": error_code,
                "reconciled": True,
            }
            encoded = json.dumps(result, sort_keys=True)
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='failed', error_code=?, result_json=?,
                    updated_at=?, completed_at=?
                WHERE intent_id=? AND status IN ('accepted', 'draining', 'awaiting_restart')
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
            if state is SupervisionState.DRAINING:
                self._transition(
                    connection,
                    (
                        SupervisionState.DRAINING
                        if preserve_maintenance_hold
                        else SupervisionState.HEALTHY
                    ),
                    reason_code=(
                        "lifecycle.failure_hold_preserved"
                        if preserve_maintenance_hold
                        else "lifecycle.failure_reconciled"
                    ),
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

    def complete_drain(
        self,
        action_id: str,
        *,
        actor: str,
        timed_out: bool = True,
    ) -> dict[str, object] | None:
        """Record a drain deadline without reopening a proof-bound admission hold.

        A controlled drain is now a durable proof-bound gate.  Deadline and
        restart recovery can finish its audit intent, but only an explicit
        resume after the scoped reservation releases may reopen mutations.
        """
        with self._transition_lock, self.database.transaction() as connection:
            intent = connection.execute(
                """
                SELECT intent_id, status
                FROM service_lifecycle_intents
                WHERE repository_key=? AND action_id=? AND kind='service.drain'
                """,
                (self.repository_key, action_id),
            ).fetchone()
            if intent is None or intent["status"] not in {
                LifecycleStatus.ACCEPTED.value,
                LifecycleStatus.DRAINING.value,
                LifecycleStatus.READY.value,
            }:
                return None
            newer_drain = connection.execute(
                """
                SELECT intent_id, action_id
                FROM service_lifecycle_intents
                WHERE repository_key=? AND kind='service.drain'
                  AND status IN ('accepted', 'draining', 'ready')
                ORDER BY created_at DESC, intent_id DESC
                LIMIT 1
                """,
                (self.repository_key,),
            ).fetchone()
            state_row = self._state_row(connection)
            if state_row is None:
                raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
            superseded = newer_drain is not None and newer_drain["intent_id"] != intent["intent_id"]
            result = {
                "intentId": intent["intent_id"],
                "state": SupervisionState(state_row["state"]).value,
                "timedOut": timed_out,
                "admissionOpen": False,
            }
            if superseded:
                result["supersededByActionId"] = newer_drain["action_id"]
            now = utc_text()
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET status='succeeded', error_code=NULL, result_json=?, updated_at=?, completed_at=?
                WHERE intent_id=? AND status IN ('accepted', 'draining', 'ready')
                """,
                (json.dumps(result, sort_keys=True), now, now, intent["intent_id"]),
            )
            connection.execute(
                "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, ?)",
                (
                    "lifecycle.drain_completed",
                    json.dumps(
                        {
                            "actionId": action_id,
                            "intentId": intent["intent_id"],
                            "reopened": False,
                            "superseded": superseded,
                            "timedOut": timed_out,
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
