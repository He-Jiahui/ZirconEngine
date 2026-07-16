from __future__ import annotations

import json
import threading
import time
from collections.abc import Callable
from datetime import timedelta

from ..models import CoordinatorError, SupervisionState, parse_utc, utc_now, utc_text
from .models import LifecycleKind, LifecycleStatus
from .service import SupervisionService

SHUTDOWN_RETRY_SECONDS = (0.0, 0.1, 0.5, 1.0, 2.0, 5.0, 15.0, 30.0)


class LifecycleService:
    """Executes durable lifecycle intents without letting the tray mutate storage."""

    def __init__(
        self,
        supervision: SupervisionService,
        *,
        shutdown: Callable[[LifecycleKind], None] | None = None,
        # A blocker scan touches the Cargo/process inventory.  The production
        # coordinator can retain a large historical audit DB, so polling at
        # 10Hz starves ordinary HTTP/SQLite work during a drain.
        poll_seconds: float = 1.0,
        allow_global_shutdown: bool = False,
    ):
        self.supervision = supervision
        self._shutdown = shutdown
        self.poll_seconds = poll_seconds
        self.allow_global_shutdown = allow_global_shutdown
        self._workers: dict[str, threading.Thread] = {}
        self._drain_timers: dict[str, threading.Timer] = {}
        self._force_stop_timers: dict[str, threading.Timer] = {}
        self._force_stop_handoffs: set[str] = set()
        self._lock = threading.Lock()

    def set_shutdown(self, callback: Callable[[LifecycleKind], None]) -> None:
        self._shutdown = callback

    def close(self) -> None:
        """Cancel local bounded-drain timers before their backing store closes."""
        with self._lock:
            timers = tuple(self._drain_timers.values())
            self._drain_timers.clear()
        for timer in timers:
            timer.cancel()

    def request(
        self,
        kind: LifecycleKind,
        *,
        action_id: str,
        actor: str,
        timeout_seconds: float,
        release_maintenance_hold: bool = False,
        maintenance_hold_action_id: str | None = None,
    ) -> dict[str, object]:
        if timeout_seconds <= 0 or timeout_seconds > 300:
            raise CoordinatorError(
                "lifecycle_timeout_invalid", "Lifecycle timeout must be within 1-300 seconds"
            )
        if (
            kind
            in {
                LifecycleKind.STOP,
                LifecycleKind.RESTART,
                LifecycleKind.FORCE_STOP,
            }
            and not self.allow_global_shutdown
        ):
            raise CoordinatorError(
                "lifecycle_global_shutdown_disabled",
                "Global stop, restart, and force-stop are disabled while task admission is open",
            )
        if kind is LifecycleKind.RESUME:
            self._require_resume_not_restarting()
        deadline = utc_now() + timedelta(seconds=timeout_seconds)
        intent_id = self.supervision.create_intent(
            kind,
            action_id=action_id,
            actor=actor,
            deadline_at=utc_text(deadline),
        )
        try:
            return self._activate_intent(
                kind,
                intent_id=intent_id,
                action_id=action_id,
                actor=actor,
                timeout_seconds=timeout_seconds,
                release_maintenance_hold=release_maintenance_hold,
                maintenance_hold_action_id=maintenance_hold_action_id,
            )
        except BaseException:
            with self._lock:
                self._workers.pop(intent_id, None)
            self.supervision.fail_lifecycle(
                action_id,
                actor=actor,
                error_code="lifecycle_request_failed",
            )
            raise

    def _require_resume_not_restarting(self) -> None:
        """A resume cannot reopen mutation admission while a shutdown drain is active."""
        with self.supervision.database.connect() as connection:
            active = connection.execute(
                """
                SELECT action_id
                FROM service_lifecycle_intents
                WHERE repository_key=?
                  AND kind IN ('service.stop', 'service.restart', 'service.force_stop')
                  AND status IN ('accepted', 'draining')
                ORDER BY created_at
                LIMIT 1
                """,
                (self.supervision.repository_key,),
            ).fetchone()
        if active is not None:
            raise CoordinatorError(
                "lifecycle_restart_draining",
                "Cannot resume while a controlled service shutdown is draining",
                details={"actionId": active["action_id"]},
            )

    def _activate_intent(
        self,
        kind: LifecycleKind,
        *,
        intent_id: str,
        action_id: str,
        actor: str,
        timeout_seconds: float,
        release_maintenance_hold: bool,
        maintenance_hold_action_id: str | None,
    ) -> dict[str, object]:
        if kind is LifecycleKind.RESUME:
            drained_intent_id: str | None = None
            drained_action_id: str | None = None
            with self._lock:
                with self.supervision.database.connect() as connection:
                    active = connection.execute(
                        """
                        SELECT intent.action_id
                        FROM service_lifecycle_intents AS intent
                        WHERE intent.repository_key=?
                          AND intent.kind IN ('service.stop', 'service.restart', 'service.force_stop')
                          AND intent.status IN ('accepted', 'draining')
                        ORDER BY intent.created_at
                        LIMIT 1
                        """,
                        (self.supervision.repository_key,),
                    ).fetchone()
                if active is not None:
                    raise CoordinatorError(
                        "lifecycle_restart_draining",
                        "Cannot resume while a controlled service shutdown is draining",
                        details={"actionId": active["action_id"]},
                    )
                with self.supervision.database.connect() as connection:
                    state = connection.execute(
                        "SELECT maintenance_hold FROM service_recovery_state WHERE repository_key=?",
                        (self.supervision.repository_key,),
                    ).fetchone()
                if state is None:
                    raise CoordinatorError("supervision_uninitialized", "Supervision state is missing")
                if bool(state["maintenance_hold"]) and not release_maintenance_hold:
                    raise CoordinatorError(
                        "maintenance_hold_active",
                        "Coordinator maintenance hold requires an explicit controlled release",
                    )
                if bool(state["maintenance_hold"]):
                    self._require_matching_maintenance_hold(maintenance_hold_action_id)
                with self.supervision.database.connect() as connection:
                    latest_drain = connection.execute(
                        """
                        SELECT intent_id, action_id
                        FROM service_lifecycle_intents
                        WHERE repository_key=? AND kind='service.drain'
                          AND status IN ('accepted', 'draining', 'ready')
                        ORDER BY created_at DESC, intent_id DESC
                        LIMIT 1
                        """,
                        (self.supervision.repository_key,),
                    ).fetchone()
                if latest_drain is not None:
                    drained_intent_id = latest_drain["intent_id"]
                    drained_action_id = latest_drain["action_id"]
                self.supervision.transition(
                    SupervisionState.HEALTHY,
                    reason_code="lifecycle.resume",
                    actor=actor,
                    action_id=action_id,
                    updates=(
                        {"maintenance_hold": 0, "explicit_stop": 0}
                        if release_maintenance_hold
                        else None
                    ),
                )
                self.supervision.update_intent(
                    intent_id,
                    LifecycleStatus.SUCCEEDED,
                    result={"state": "healthy"},
                    completed=True,
                )
            if drained_intent_id is not None and drained_action_id is not None:
                with self._lock:
                    timer = self._drain_timers.pop(drained_intent_id, None)
                if timer is not None:
                    timer.cancel()
                self.supervision.complete_drain(
                    drained_action_id,
                    actor=actor,
                    timed_out=False,
                )
            return {"intentId": intent_id, "state": "healthy", "deferred": False}
        if kind is LifecycleKind.DRAIN:
            # A drain is now an auditable blocker observation, not a global
            # admission gate.  Long-running work must time out/reconcile at
            # the job level without freezing unrelated Sessions.
            blockers = self.supervision.snapshot(exclude_action_id=action_id).blockers
            self.supervision.update_intent(
                intent_id,
                LifecycleStatus.SUCCEEDED,
                result={
                    "admissionOpen": True,
                    "blockers": [item.to_dict() for item in blockers],
                },
                completed=True,
            )
            return {
                "intentId": intent_id,
                "state": self.supervision.snapshot().state.value,
                "deferred": False,
                "ready": True,
                "admissionOpen": True,
                "blockers": [item.to_dict() for item in blockers],
            }
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code=f"lifecycle.{kind.name.lower()}.accepted",
            actor=actor,
            action_id=action_id,
            updates={"maintenance_hold": 1}
            if kind in {
                LifecycleKind.RESTART,
                LifecycleKind.FORCE_STOP,
            }
            else None,
        )
        self.supervision.update_intent(intent_id, LifecycleStatus.DRAINING)
        worker = threading.Thread(
            target=self._complete_stop,
            args=(intent_id, action_id, actor, kind, time.monotonic() + timeout_seconds),
            name=f"zircon-lifecycle-{kind.name.lower()}-{intent_id[:8]}",
            daemon=True,
        )
        with self._lock:
            self._workers[intent_id] = worker
        worker.start()
        return {"intentId": intent_id, "state": "draining", "deferred": True}

    def _schedule_drain_deadline(
        self,
        intent_id: str,
        action_id: str,
        *,
        deadline,
    ) -> None:
        """Schedule the bounded drain completion and replace any recovered timer."""
        delay = max(0.0, (deadline - utc_now()).total_seconds())
        timer = threading.Timer(delay, self._complete_drain_deadline, args=(intent_id, action_id))
        timer.daemon = True
        with self._lock:
            previous = self._drain_timers.pop(intent_id, None)
            self._drain_timers[intent_id] = timer
        if previous is not None:
            previous.cancel()
        timer.start()

    def _complete_drain_deadline(self, intent_id: str, action_id: str) -> None:
        try:
            self.supervision.complete_drain(action_id, actor="daemon")
        finally:
            with self._lock:
                self._drain_timers.pop(intent_id, None)

    def _require_matching_maintenance_hold(self, action_id: str | None) -> None:
        """Only the explicit drain that established the current hold may release it."""
        if not action_id:
            raise CoordinatorError(
                "maintenance_hold_release_id_required",
                "Releasing a maintenance hold requires its controlled drain action ID",
            )
        with self.supervision.database.connect() as connection:
            source = connection.execute(
                """
                SELECT action_id
                FROM action_requests
                WHERE action_kind='service.drain' AND status='succeeded'
                ORDER BY completed_at DESC, action_id DESC
                LIMIT 1
                """
            ).fetchone()
        if source is None or source["action_id"] != action_id:
            raise CoordinatorError(
                "maintenance_hold_release_mismatch",
                "Maintenance hold may only be released by its current drain action",
                details={"maintenanceHoldActionId": source["action_id"] if source else None},
            )

    def recover_restart_intents(self) -> int:
        with self.supervision.database.connect() as connection:
            orphaned = connection.execute(
                """
                SELECT action_id
                FROM service_lifecycle_intents
                WHERE repository_key=?
                  AND kind IN ('service.stop', 'service.restart', 'service.force_stop')
                  AND status IN ('accepted', 'draining')
                  AND source_daemon_instance_id<>?
                ORDER BY created_at
                """,
                (
                    self.supervision.repository_key,
                    self.supervision.daemon_instance_id,
                ),
            ).fetchall()
        reconciled = 0
        for row in orphaned:
            if self.supervision.fail_lifecycle(
                row["action_id"],
                actor="daemon",
                error_code="lifecycle_orphan_recovered",
            ) is not None:
                reconciled += 1
        with self.supervision.database.connect() as connection:
            drains = connection.execute(
                """
                SELECT intent_id, action_id, deadline_at
                FROM service_lifecycle_intents
                WHERE repository_key=? AND kind='service.drain'
                  AND status IN ('accepted', 'draining', 'ready')
                ORDER BY created_at
                """,
                (self.supervision.repository_key,),
            ).fetchall()
        for row in drains:
            deadline = parse_utc(row["deadline_at"])
            if deadline <= utc_now():
                if self.supervision.complete_drain(row["action_id"], actor="daemon") is not None:
                    reconciled += 1
            else:
                self._schedule_drain_deadline(
                    row["intent_id"], row["action_id"], deadline=deadline
                )
        with self.supervision.database.transaction() as connection:
            rows = connection.execute(
                """
                SELECT intent_id, action_id FROM service_lifecycle_intents
                WHERE repository_key=? AND kind='service.restart'
                  AND status='awaiting_restart'
                ORDER BY created_at
                """,
                (self.supervision.repository_key,),
            ).fetchall()
            for row in rows:
                result = {
                    "intentId": row["intent_id"],
                    "state": "healthy",
                    "successorInstanceId": self.supervision.daemon_instance_id,
                }
                connection.execute(
                    """
                    UPDATE service_lifecycle_intents
                    SET status='succeeded', successor_daemon_instance_id=?,
                        result_json=?, updated_at=?, completed_at=?
                    WHERE intent_id=?
                    """,
                    (
                        self.supervision.daemon_instance_id,
                        json.dumps(result, sort_keys=True),
                        utc_text(),
                        utc_text(),
                        row["intent_id"],
                    ),
                )
                connection.execute(
                    """
                    UPDATE action_requests
                    SET status='succeeded', result_json=?, completed_at=?
                    WHERE action_id=? AND status='executing'
                    """,
                    (json.dumps(result, sort_keys=True), utc_text(), row["action_id"]),
                )
        return reconciled + len(rows)

    def cancel(self, action_id: str, *, actor: str, reason: str) -> dict[str, object]:
        """Cancel a confirmed stop/restart only while its drain is still reversible."""
        with self._lock:
            return self.supervision.cancel_lifecycle(
                action_id,
                actor=actor,
                reason=reason,
            )

    def acknowledge_force_stop(self, action_id: str) -> dict[str, object]:
        """Acknowledge durable terminal/offline proof before the HTTP transport closes."""
        with self._lock:
            if action_id in self._force_stop_handoffs:
                return {
                    "actionId": action_id,
                    "acknowledged": True,
                    "alreadyAcknowledged": True,
                }
            with self.supervision.database.connect() as connection:
                row = connection.execute(
                    """
                    SELECT intent.status AS intent_status, intent.kind,
                           action.status AS action_status
                    FROM service_lifecycle_intents AS intent
                    JOIN action_requests AS action ON action.action_id=intent.action_id
                    WHERE intent.repository_key=? AND intent.action_id=?
                    """,
                    (self.supervision.repository_key, action_id),
                ).fetchone()
            snapshot = self.supervision.snapshot()
            if (
                row is None
                or row["kind"] != LifecycleKind.FORCE_STOP.value
                or row["intent_status"] != LifecycleStatus.SUCCEEDED.value
                or row["action_status"] != "succeeded"
                or snapshot.state is not SupervisionState.OFFLINE
            ):
                raise CoordinatorError(
                    "force_stop_ack_invalid",
                    "Force-stop acknowledgement requires durable succeeded and offline state",
                )
            fallback = self._force_stop_timers.pop(action_id, None)
            timer = threading.Timer(
                0.2,
                self._shutdown_after_commit,
                args=(LifecycleKind.FORCE_STOP, action_id),
            )
            timer.daemon = True
            try:
                timer.start()
            except Exception as error:
                if fallback is not None:
                    self._force_stop_timers[action_id] = fallback
                raise CoordinatorError(
                    "force_stop_ack_schedule_failed",
                    "Force-stop acknowledgement could not schedule graceful shutdown",
                ) from error
            self._force_stop_handoffs.add(action_id)
            if fallback is not None:
                fallback.cancel()
        return {
            "actionId": action_id,
            "acknowledged": True,
            "alreadyAcknowledged": False,
        }

    def _complete_stop(
        self,
        intent_id: str,
        action_id: str,
        actor: str,
        kind: LifecycleKind,
        deadline: float,
    ) -> None:
        try:
            while time.monotonic() < deadline:
                if self._intent_cancelled(intent_id):
                    return
                blockers = self.supervision.snapshot(exclude_action_id=action_id).blockers
                if not any(item.blocking for item in blockers):
                    break
                time.sleep(self.poll_seconds)
            else:
                self.supervision.fail_lifecycle(
                    action_id, actor="daemon", error_code="lifecycle_drain_timeout"
                )
                return
            if self._shutdown is None:
                self.supervision.fail_lifecycle(
                    action_id,
                    actor="daemon",
                    error_code="lifecycle_shutdown_unavailable",
                )
                return
            with self._lock:
                if self._intent_cancelled(intent_id):
                    return
                self.supervision.commit_lifecycle_offline(
                    intent_id,
                    action_id=action_id,
                    actor=actor,
                    kind=kind,
                )
            if kind is LifecycleKind.FORCE_STOP:
                timer = threading.Timer(30.0, self._force_stop_fallback, args=(action_id,))
                timer.daemon = True
                with self._lock:
                    self._force_stop_timers[action_id] = timer
                try:
                    timer.start()
                except Exception:
                    with self._lock:
                        self._force_stop_timers.pop(action_id, None)
                        self._force_stop_handoffs.add(action_id)
                    self._shutdown_after_commit(kind, action_id)
            else:
                self._shutdown_after_commit(kind, action_id)
        except Exception:
            self.supervision.fail_lifecycle(
                action_id, actor="daemon", error_code="lifecycle_worker_failed"
            )
        finally:
            with self._lock:
                self._workers.pop(intent_id, None)

    def _intent_cancelled(self, intent_id: str) -> bool:
        with self.supervision.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM service_lifecycle_intents WHERE intent_id=?",
                (intent_id,),
            ).fetchone()
        return row is not None and row["status"] == LifecycleStatus.CANCELLED.value

    def _force_stop_fallback(self, action_id: str) -> None:
        with self._lock:
            self._force_stop_timers.pop(action_id, None)
            self._force_stop_handoffs.add(action_id)
        self._shutdown_after_commit(LifecycleKind.FORCE_STOP, action_id)

    def _shutdown_after_commit(self, kind: LifecycleKind, action_id: str) -> None:
        """Retry with bounded backoff without ever rewriting committed terminal proof."""
        attempt = 0
        while self._shutdown is not None:
            delay = SHUTDOWN_RETRY_SECONDS[
                min(attempt, len(SHUTDOWN_RETRY_SECONDS) - 1)
            ]
            if delay:
                time.sleep(delay)
            attempt += 1
            try:
                self._shutdown(kind)
                return
            except Exception:
                try:
                    self.supervision.record_lifecycle_shutdown_retry(
                        action_id,
                        kind=kind,
                        attempt=attempt,
                        error_code="lifecycle_shutdown_failed",
                    )
                except Exception:
                    # Shutdown retry is the final safety path; audit I/O must not stop it.
                    pass
