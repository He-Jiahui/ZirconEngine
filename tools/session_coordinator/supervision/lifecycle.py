from __future__ import annotations

import json
import threading
import time
from collections.abc import Callable
from datetime import timedelta

from ..models import CoordinatorError, SupervisionState, utc_now, utc_text
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
        poll_seconds: float = 0.1,
    ):
        self.supervision = supervision
        self._shutdown = shutdown
        self.poll_seconds = poll_seconds
        self._workers: dict[str, threading.Thread] = {}
        self._force_stop_timers: dict[str, threading.Timer] = {}
        self._force_stop_handoffs: set[str] = set()
        self._lock = threading.Lock()

    def set_shutdown(self, callback: Callable[[LifecycleKind], None]) -> None:
        self._shutdown = callback

    def request(
        self,
        kind: LifecycleKind,
        *,
        action_id: str,
        actor: str,
        timeout_seconds: float,
    ) -> dict[str, object]:
        if timeout_seconds <= 0 or timeout_seconds > 300:
            raise CoordinatorError(
                "lifecycle_timeout_invalid", "Lifecycle timeout must be within 1-300 seconds"
            )
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

    def _activate_intent(
        self,
        kind: LifecycleKind,
        *,
        intent_id: str,
        action_id: str,
        actor: str,
        timeout_seconds: float,
    ) -> dict[str, object]:
        if kind is LifecycleKind.RESUME:
            with self._lock:
                with self.supervision.database.connect() as connection:
                    active = connection.execute(
                        """
                        SELECT intent.action_id, action.status AS action_status
                        FROM service_lifecycle_intents AS intent
                        LEFT JOIN action_requests AS action ON action.action_id=intent.action_id
                        WHERE intent.repository_key=?
                          AND intent.kind IN ('service.stop', 'service.restart', 'service.force_stop')
                          AND intent.status IN ('accepted', 'draining')
                        ORDER BY intent.created_at
                        LIMIT 1
                        """,
                        (self.supervision.repository_key,),
                    ).fetchone()
                if active is not None:
                    if active["action_status"] == "executing":
                        self.supervision.cancel_lifecycle(
                            active["action_id"],
                            actor=actor,
                            reason="service.resume cancelled reversible lifecycle drain",
                        )
                    else:
                        self.supervision.fail_lifecycle(
                            active["action_id"],
                            actor=actor,
                            error_code="lifecycle_orphan_reconciled",
                        )
                self.supervision.transition(
                    SupervisionState.HEALTHY,
                    reason_code="lifecycle.resume",
                    actor=actor,
                    action_id=action_id,
                )
                self.supervision.update_intent(
                    intent_id,
                    LifecycleStatus.SUCCEEDED,
                    result={"state": "healthy"},
                    completed=True,
                )
            return {"intentId": intent_id, "state": "healthy", "deferred": False}
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code=f"lifecycle.{kind.name.lower()}.accepted",
            actor=actor,
            action_id=action_id,
        )
        if kind is LifecycleKind.DRAIN:
            blockers = self.supervision.snapshot(exclude_action_id=action_id).blockers
            blocking = [item.to_dict() for item in blockers if item.blocking]
            status = LifecycleStatus.DRAINING if blocking else LifecycleStatus.READY
            self.supervision.update_intent(
                intent_id,
                status,
                result={"blockers": [item.to_dict() for item in blockers]},
            )
            return {
                "intentId": intent_id,
                "state": "draining",
                "deferred": False,
                "ready": not blocking,
                "blockers": [item.to_dict() for item in blockers],
            }
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
