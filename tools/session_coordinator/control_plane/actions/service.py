from __future__ import annotations

import hashlib
import json
import secrets
import threading
import uuid
from dataclasses import replace
from datetime import timedelta

from ...database import Database
from ...models import CoordinatorError, WebControlRole, parse_utc, utc_now, utc_text
from .catalog import ACTION_CATALOG, action_spec
from .fingerprint import ActionFingerprinter
from .models import (
    ActionContext,
    ActionKind,
    ActionRecord,
    ActionRisk,
    ActionStatus,
)
from .permissions import require_permission


ACTION_PREVIEW_SECONDS = 120


class ActionService:
    """Persists preview/confirm state and keeps execution inside the closed catalog."""

    def __init__(
        self,
        database: Database,
        fingerprinter: ActionFingerprinter,
        executor,
        *,
        daemon_instance_id: str,
        mutation_lock=None,
    ):
        self.database = database
        self.fingerprinter = fingerprinter
        self.executor = executor
        self.daemon_instance_id = daemon_instance_id
        self._confirmation_lock = mutation_lock or threading.RLock()

    def catalog(self) -> dict[str, object]:
        return {"actions": [spec.to_dict() for spec in ACTION_CATALOG.values()]}

    def preview(
        self,
        context: ActionContext,
        kind: str,
        payload: dict[str, object],
    ) -> ActionRecord:
        spec = action_spec(kind)
        parameters = spec.parse_parameters(payload)
        target_session_id = getattr(parameters, "session_id", None)
        try:
            self._require_instance(context)
            require_permission(context, spec, target_session_id)
        except CoordinatorError as error:
            self._record_denial(context, spec, parameters.to_payload(), error.code)
            raise
        fingerprint = self.fingerprinter.capture(
            spec, parameters, bound_session_id=context.bound_session_id
        )
        impact = self.fingerprinter.impact(
            spec, parameters, bound_session_id=context.bound_session_id
        )
        action_id = uuid.uuid4().hex
        phrase = None if spec.preview_only else self._confirmation_phrase(spec.kind)
        now = utc_now()
        expires_at = now + timedelta(seconds=ACTION_PREVIEW_SECONDS)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO action_requests(
                    action_id, action_kind, risk, required_role, actor,
                    web_session_id, bound_session_id, daemon_instance_id,
                    parameters_json, impact_json, warnings_json,
                    state_fingerprint, confirmation_phrase_hash, status,
                    created_at, expires_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'previewed', ?, ?)
                """,
                (
                    action_id,
                    spec.kind.value,
                    spec.risk.value,
                    spec.required_role.value,
                    context.actor,
                    context.web_session_id,
                    context.bound_session_id,
                    self.daemon_instance_id,
                    json.dumps(parameters.to_payload(), sort_keys=True),
                    json.dumps(impact, ensure_ascii=False),
                    json.dumps(spec.warnings, ensure_ascii=False),
                    fingerprint.digest,
                    self._digest(phrase or ""),
                    utc_text(now),
                    utc_text(expires_at),
                ),
            )
            self._event(
                connection,
                target_session_id,
                "action.previewed",
                {"actionId": action_id, "kind": spec.kind.value, "actor": context.actor},
            )
        return ActionRecord(
            action_id=action_id,
            kind=spec.kind,
            risk=spec.risk,
            required_role=spec.required_role,
            actor=context.actor,
            web_session_id=context.web_session_id,
            bound_session_id=context.bound_session_id,
            parameters=parameters.to_payload(),
            impact=impact,
            warnings=spec.warnings,
            state_fingerprint=fingerprint.digest,
            status=ActionStatus.PREVIEWED,
            created_at=utc_text(now),
            expires_at=utc_text(expires_at),
            confirmation_phrase=phrase,
        )

    def confirm(
        self,
        context: ActionContext,
        action_id: str,
        *,
        phrase: str,
        reason: str,
    ) -> ActionRecord:
        normalized_reason = reason.strip()
        if not normalized_reason or len(normalized_reason) > 500:
            raise CoordinatorError("action_reason_invalid", "Confirmation reason is required")
        state_changed = False
        expired = False
        with self._confirmation_lock:
            with self.database.transaction() as connection:
                row = self._request_row(connection, action_id)
                spec = action_spec(row["action_kind"])
                parameters = spec.parse_parameters(json.loads(row["parameters_json"]))
                context = self._context_for_request(context, row)
                self._require_request_identity(context, row)
                require_permission(context, spec, getattr(parameters, "session_id", None))
                if spec.preview_only:
                    raise CoordinatorError(
                        "action_preview_only", "This catalog action intentionally has no M3 executor"
                    )
                if row["status"] != ActionStatus.PREVIEWED.value:
                    raise CoordinatorError(
                        "action_not_confirmable", f"Action is {row['status']}"
                    )
                if parse_utc(row["expires_at"]) <= utc_now():
                    connection.execute(
                        "UPDATE action_requests SET status = 'expired', completed_at = ? WHERE action_id = ?",
                        (utc_text(), action_id),
                    )
                    expired = True
                elif not secrets.compare_digest(
                    row["confirmation_phrase_hash"], self._digest(phrase)
                ):
                    raise CoordinatorError(
                        "action_confirmation_mismatch", "Confirmation phrase does not match"
                    )
                else:
                    current = self.fingerprinter.capture(
                        spec,
                        parameters,
                        bound_session_id=context.bound_session_id,
                        connection=connection,
                    )
                    if not secrets.compare_digest(
                        current.digest, row["state_fingerprint"]
                    ):
                        connection.execute(
                            """UPDATE action_requests
                               SET status = 'state_changed', completed_at = ?, error_code = 'action_state_changed'
                               WHERE action_id = ?""",
                            (utc_text(), action_id),
                        )
                        self._event(
                            connection,
                            getattr(parameters, "session_id", None),
                            "action.state_changed",
                            {"actionId": action_id, "kind": spec.kind.value},
                        )
                        state_changed = True
                    else:
                        now = utc_text()
                        connection.execute(
                            """UPDATE action_requests
                               SET status = 'executing', reason = ?, confirmed_at = ?
                               WHERE action_id = ?""",
                            (normalized_reason, now, action_id),
                        )
                        connection.execute(
                            """INSERT INTO action_approvals(
                                   approval_id, action_id, actor, role, reason,
                                   state_fingerprint, created_at
                               ) VALUES (?, ?, ?, ?, ?, ?, ?)""",
                            (
                                uuid.uuid4().hex,
                                action_id,
                                context.actor,
                                context.role.value,
                                normalized_reason,
                                current.digest,
                                now,
                            ),
                        )
            if expired:
                raise CoordinatorError("action_expired", "Action preview has expired")
            if state_changed:
                raise CoordinatorError(
                    "action_state_changed",
                    "Coordinator state changed after preview; create a fresh preview",
                )
            return self._execute_confirmed(
                context,
                action_id,
                spec,
                parameters,
                resource_snapshot=current.payload.get("actionResources", {}),
            )

    def _execute_confirmed(
        self,
        context,
        action_id,
        spec,
        parameters,
        *,
        resource_snapshot,
    ) -> ActionRecord:
        try:
            result = self.executor.execute(
                spec, parameters, resource_snapshot=resource_snapshot
            )
        except CoordinatorError as error:
            self._finish_failure(action_id, parameters, error.code)
            raise
        except Exception as error:
            self._finish_failure(action_id, parameters, "action_execution_failed")
            raise CoordinatorError(
                "action_execution_failed", "Controlled action execution failed"
            ) from error
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE action_requests
                   SET status = 'succeeded', result_json = ?, completed_at = ?
                   WHERE action_id = ? AND status = 'executing'""",
                (json.dumps(result, sort_keys=True), utc_text(), action_id),
            )
            self._event(
                connection,
                getattr(parameters, "session_id", None),
                "action.succeeded",
                {"actionId": action_id, "kind": spec.kind.value},
            )
        return self.get(context, action_id)

    def cancel(
        self, context: ActionContext, action_id: str, *, reason: str
    ) -> ActionRecord:
        normalized_reason = reason.strip()
        if not normalized_reason or len(normalized_reason) > 500:
            raise CoordinatorError("action_reason_invalid", "Cancellation reason is required")
        with self.database.transaction() as connection:
            row = self._request_row(connection, action_id)
            context = self._context_for_request(context, row)
            self._require_request_identity(context, row)
            if row["status"] != ActionStatus.PREVIEWED.value:
                raise CoordinatorError("action_not_cancellable", f"Action is {row['status']}")
            connection.execute(
                """UPDATE action_requests
                   SET status = 'cancelled', reason = ?, completed_at = ?
                   WHERE action_id = ?""",
                (normalized_reason, utc_text(), action_id),
            )
            parameters = json.loads(row["parameters_json"])
            self._event(
                connection,
                parameters.get("sessionId"),
                "action.cancelled",
                {"actionId": action_id, "kind": row["action_kind"]},
            )
        return self.get(context, action_id)

    def get(self, context: ActionContext, action_id: str) -> ActionRecord:
        with self.database.connect() as connection:
            row = self._request_row(connection, action_id)
        context = self._context_for_request(context, row)
        self._require_request_identity(context, row)
        return self._record(row)

    def _record_denial(self, context, spec, parameters, code: str) -> None:
        action_id = uuid.uuid4().hex
        now = utc_text()
        session_id = parameters.get("sessionId")
        with self.database.transaction() as connection:
            connection.execute(
                """INSERT INTO action_requests(
                       action_id, action_kind, risk, required_role, actor,
                       web_session_id, bound_session_id, daemon_instance_id,
                       parameters_json, impact_json, warnings_json,
                       state_fingerprint, confirmation_phrase_hash, status,
                       error_code, created_at, expires_at, completed_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, '[]', ?, '', '', 'denied', ?, ?, ?, ?)""",
                (
                    action_id,
                    spec.kind.value,
                    spec.risk.value,
                    spec.required_role.value,
                    context.actor,
                    context.web_session_id,
                    context.bound_session_id,
                    self.daemon_instance_id,
                    json.dumps(parameters, sort_keys=True),
                    json.dumps(spec.warnings, ensure_ascii=False),
                    code,
                    now,
                    now,
                    now,
                ),
            )
            self._event(
                connection,
                session_id if self._session_exists(connection, session_id) else None,
                "action.denied",
                {"actionId": action_id, "kind": spec.kind.value, "code": code},
            )

    def _finish_failure(self, action_id, parameters, code: str) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """UPDATE action_requests
                   SET status = 'failed', error_code = ?, completed_at = ?
                   WHERE action_id = ? AND status = 'executing'""",
                (code, utc_text(), action_id),
            )
            self._event(
                connection,
                getattr(parameters, "session_id", None),
                "action.failed",
                {"actionId": action_id, "code": code},
            )

    def _require_request_identity(self, context: ActionContext, row) -> None:
        self._require_instance(context)
        if row["daemon_instance_id"] != self.daemon_instance_id:
            raise CoordinatorError("action_instance_mismatch", "Action belongs to another daemon")
        if row["actor"] != context.actor or row["web_session_id"] != context.web_session_id:
            raise CoordinatorError("action_identity_mismatch", "Action belongs to another actor")
        if row["bound_session_id"] != context.bound_session_id:
            raise CoordinatorError("action_session_scope_mismatch", "Action Session binding changed")

    @staticmethod
    def _context_for_request(context: ActionContext, row) -> ActionContext:
        if context.web_session_id is None and row["web_session_id"] is None:
            return replace(context, bound_session_id=row["bound_session_id"])
        return context

    def _require_instance(self, context: ActionContext) -> None:
        if context.daemon_instance_id != self.daemon_instance_id:
            raise CoordinatorError("action_instance_mismatch", "Action identity is stale after restart")

    @staticmethod
    def _request_row(connection, action_id: str):
        row = connection.execute(
            "SELECT * FROM action_requests WHERE action_id = ?", (action_id,)
        ).fetchone()
        if row is None:
            raise CoordinatorError("action_not_found", "Controlled action was not found")
        return row

    @staticmethod
    def _session_exists(connection, session_id) -> bool:
        if not session_id:
            return False
        return connection.execute(
            "SELECT 1 FROM sessions WHERE session_id = ?", (session_id,)
        ).fetchone() is not None

    @staticmethod
    def _event(connection, session_id, event_type: str, payload: dict[str, object]) -> None:
        connection.execute(
            "INSERT INTO events(session_id, event_type, payload_json, created_at) VALUES (?, ?, ?, ?)",
            (session_id, event_type, json.dumps(payload, sort_keys=True), utc_text()),
        )

    @staticmethod
    def _confirmation_phrase(kind: ActionKind) -> str:
        return f"CONFIRM {kind.value.upper()}"

    @staticmethod
    def _digest(value: str) -> str:
        return hashlib.sha256(value.encode("utf-8")).hexdigest()

    @staticmethod
    def _record(row) -> ActionRecord:
        return ActionRecord(
            action_id=row["action_id"],
            kind=ActionKind(row["action_kind"]),
            risk=ActionRisk(row["risk"]),
            required_role=WebControlRole(row["required_role"]),
            actor=row["actor"],
            web_session_id=row["web_session_id"],
            bound_session_id=row["bound_session_id"],
            parameters=json.loads(row["parameters_json"]),
            impact=tuple(json.loads(row["impact_json"])),
            warnings=tuple(json.loads(row["warnings_json"])),
            state_fingerprint=row["state_fingerprint"],
            status=ActionStatus(row["status"]),
            created_at=row["created_at"],
            expires_at=row["expires_at"],
            reason=row["reason"],
            result=json.loads(row["result_json"]) if row["result_json"] else None,
            error_code=row["error_code"],
        )
