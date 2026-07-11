from __future__ import annotations

import json
import threading
import time
from collections.abc import Callable
from datetime import timedelta

from ..models import CoordinatorError, SupervisionState, utc_now, utc_text
from .models import LifecycleKind, LifecycleStatus
from .service import SupervisionService


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
        if kind is LifecycleKind.RESUME:
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
        return len(rows)

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
                blockers = self.supervision.snapshot(exclude_action_id=action_id).blockers
                if not any(item.blocking for item in blockers):
                    break
                time.sleep(self.poll_seconds)
            else:
                self._fail_intent(intent_id, action_id, "lifecycle_drain_timeout")
                self.supervision.transition(
                    SupervisionState.HEALTHY,
                    reason_code="lifecycle.drain_timeout",
                    actor="daemon",
                    action_id=action_id,
                )
                return
            if self._shutdown is None:
                self._fail_intent(intent_id, action_id, "lifecycle_shutdown_unavailable")
                self.supervision.transition(
                    SupervisionState.HEALTHY,
                    reason_code="lifecycle.shutdown_unavailable",
                    actor="daemon",
                    action_id=action_id,
                )
                return
            self.supervision.transition(
                SupervisionState.STOPPING,
                reason_code=f"lifecycle.{kind.name.lower()}.ready",
                actor=actor,
                action_id=action_id,
            )
            if kind is LifecycleKind.RESTART:
                self.supervision.update_intent(intent_id, LifecycleStatus.AWAITING_RESTART)
            else:
                result = {"intentId": intent_id, "state": "offline"}
                self.supervision.update_intent(
                    intent_id, LifecycleStatus.SUCCEEDED, result=result, completed=True
                )
                self._finish_action(action_id, result)
            self.supervision.transition(
                SupervisionState.OFFLINE,
                reason_code=f"lifecycle.{kind.name.lower()}.offline",
                actor="daemon",
                action_id=action_id,
                updates={"explicit_stop": 0 if kind is LifecycleKind.RESTART else 1},
            )
            self._shutdown(kind)
        except Exception:
            self._fail_intent(intent_id, action_id, "lifecycle_worker_failed")
        finally:
            with self._lock:
                self._workers.pop(intent_id, None)

    def _finish_action(self, action_id: str, result: dict[str, object]) -> None:
        with self.supervision.database.transaction() as connection:
            connection.execute(
                """
                UPDATE action_requests
                SET status='succeeded', result_json=?, completed_at=?
                WHERE action_id=? AND status='executing'
                """,
                (json.dumps(result, sort_keys=True), utc_text(), action_id),
            )

    def _fail_intent(self, intent_id: str, action_id: str, code: str) -> None:
        self.supervision.update_intent(
            intent_id, LifecycleStatus.FAILED, error_code=code, completed=True
        )
        with self.supervision.database.transaction() as connection:
            connection.execute(
                """
                UPDATE action_requests
                SET status='failed', error_code=?, completed_at=?
                WHERE action_id=? AND status='executing'
                """,
                (code, utc_text(), action_id),
            )
