from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Callable, Mapping
from urllib.parse import parse_qs, unquote, urlsplit

from ..models import CoordinatorError, WebControlRole
from ..workflows.projections import WorkflowProjectionService
from .actions.models import ActionContext
from .actions.service import ActionService
from .auth import WebControlAuth, WebSessionRecord
from .contracts import CONTROL_API_VERSION, ControlResponse
from .snapshot import ControlSnapshotService


@dataclass(frozen=True, slots=True)
class ControlIdentity:
    actor: str
    role: str
    web_session_id: str | None
    bound_session_id: str | None = None


class ControlPlaneRouter:
    """Routes bounded control contracts without depending on HTTP server internals."""

    def __init__(
        self,
        *,
        instance_id: str,
        auth: WebControlAuth,
        snapshot: ControlSnapshotService,
        workflows: WorkflowProjectionService,
        database,
        actions: ActionService | None = None,
        maintenance_authorizer: Callable[[dict[str, object]], None] | None = None,
        live_workflow_eligibility: Callable[[str, str], dict[str, object]] | None = None,
    ):
        self.instance_id = instance_id
        self.auth = auth
        self.snapshot = snapshot
        self.workflows = workflows
        self.database = database
        self.actions = actions
        self.maintenance_authorizer = maintenance_authorizer
        self.live_workflow_eligibility = live_workflow_eligibility

    def dispatch(
        self,
        method: str,
        raw_path: str,
        headers: Mapping[str, str],
        body: bytes,
        *,
        runtime_authorized: bool,
    ) -> ControlResponse:
        path = urlsplit(raw_path).path
        if method == "GET" and path.startswith("/ui/bootstrap/"):
            ticket = unquote(path.removeprefix("/ui/bootstrap/"))
            raw_session, _session = self.auth.consume_bootstrap_ticket(
                ticket, self.instance_id
            )
            return ControlResponse(
                status=303,
                data={"status": "authenticated"},
                headers={
                    "Location": "/ui/",
                    "Set-Cookie": self.auth.cookie_header(raw_session),
                    "Cache-Control": "no-store",
                },
            )

        if method == "POST" and path == "/control/v1/bootstrap-tickets":
            if not runtime_authorized:
                raise CoordinatorError(
                    "runtime_auth_required",
                    "Bootstrap tickets require the local runtime credential",
                )
            payload = self._json_body(body)
            role = WebControlRole(str(payload.get("role") or "observer"))
            actor = str(payload.get("actor") or "local-cli")
            ticket = self.auth.issue_bootstrap_ticket(
                actor, self.instance_id, role=role
            )
            return ControlResponse(
                201,
                {
                    "ticket": ticket,
                    "bootstrapPath": f"/ui/bootstrap/{ticket}",
                    "expiresInSeconds": 30,
                    "role": role.value,
                },
                headers={"Cache-Control": "no-store"},
            )

        if method == "POST" and path == "/control/v1/elevation-grants":
            if not runtime_authorized:
                raise CoordinatorError(
                    "runtime_auth_required",
                    "Elevation grants require the local runtime credential",
                )
            payload = self._json_body(body)
            role = WebControlRole(str(payload.get("role") or "operator"))
            maintenance_authorized = False
            if role is WebControlRole.MAINTAINER:
                if self.maintenance_authorizer is None:
                    raise CoordinatorError(
                        "maintenance_unauthorized", "Maintainer elevation is unavailable"
                    )
                self.maintenance_authorizer(payload)
                maintenance_authorized = True
            grant = self.auth.issue_elevation_grant(
                str(payload.get("actor") or "local-cli"),
                role,
                self.instance_id,
                bound_session_id=(
                    str(payload["sessionId"]) if payload.get("sessionId") else None
                ),
                maintenance_authorized=maintenance_authorized,
            )
            return ControlResponse(
                201,
                {
                    "grant": grant,
                    "expiresInSeconds": 60,
                    "role": role.value,
                    "boundSessionId": payload.get("sessionId"),
                },
                headers={"Cache-Control": "no-store"},
            )

        if method == "POST" and path == "/control/v1/auth/elevate":
            if runtime_authorized:
                raise CoordinatorError(
                    "browser_session_required", "Elevation consumption requires a browser cookie"
                )
            payload = self._json_body(body)
            csrf, session = self.auth.consume_elevation_grant(
                str(payload.get("grant") or ""),
                headers.get("Cookie", ""),
                self.instance_id,
            )
            return ControlResponse(
                200,
                {
                    "actor": session.actor,
                    "role": session.role,
                    "boundSessionId": session.bound_session_id,
                    "elevatedUntil": session.elevated_until,
                    "csrfToken": csrf,
                },
                headers={"Cache-Control": "no-store"},
            )

        identity = self.authenticate(headers, runtime_authorized=runtime_authorized)
        if method not in {"GET", "HEAD"} and not runtime_authorized:
            session = self.auth.validate_csrf(
                headers.get("Cookie", ""),
                headers.get("X-CSRF-Token", ""),
                self.instance_id,
            )
            identity = ControlIdentity(
                session.actor, session.role, session.session_id, session.bound_session_id
            )
        if method == "GET" and path == "/control/v1/meta":
            return ControlResponse(
                200,
                {
                    "apiVersion": CONTROL_API_VERSION,
                    "instanceId": self.instance_id,
                    "role": identity.role,
                    "actor": identity.actor,
                    "boundSessionId": identity.bound_session_id,
                    "mutationEnabled": identity.role != WebControlRole.OBSERVER.value,
                },
            )
        if method == "GET" and path == "/control/v1/auth/session":
            return ControlResponse(
                200,
                {
                    "actor": identity.actor,
                    "role": identity.role,
                    "boundSessionId": identity.bound_session_id,
                    "mutationEnabled": identity.role != WebControlRole.OBSERVER.value,
                },
            )
        if method == "GET" and path == "/control/v1/actions/catalog":
            return ControlResponse(200, self._actions().catalog())
        if method == "POST" and path == "/control/v1/actions/preview":
            payload = self._json_body(body)
            parameters = payload.get("parameters") or {}
            if not isinstance(parameters, dict):
                raise CoordinatorError("invalid_request", "Action parameters must be an object")
            context = self._action_context(identity, runtime_authorized, parameters)
            record = self._actions().preview(
                context, str(payload.get("kind") or ""), parameters
            )
            return ControlResponse(201, {"action": record.to_dict()})
        if path.startswith("/control/v1/actions/"):
            suffix = unquote(path.removeprefix("/control/v1/actions/"))
            parts = suffix.split("/")
            action_id = parts[0]
            context = self._action_context(identity, runtime_authorized, {})
            if method == "GET" and len(parts) == 1:
                return ControlResponse(
                    200, {"action": self._actions().get(context, action_id).to_dict()}
                )
            payload = self._json_body(body)
            if method == "POST" and parts[1:] == ["confirm"]:
                record = self._actions().confirm(
                    context,
                    action_id,
                    phrase=str(payload.get("phrase") or ""),
                    reason=str(payload.get("reason") or ""),
                )
                return ControlResponse(200, {"action": record.to_dict()})
            if method == "POST" and parts[1:] == ["cancel"]:
                record = self._actions().cancel(
                    context, action_id, reason=str(payload.get("reason") or "")
                )
                return ControlResponse(200, {"action": record.to_dict()})
        if method == "GET" and path == "/control/v1/snapshot":
            return ControlResponse(200, self.snapshot.build())
        if method == "GET" and path == "/control/v1/logs":
            return ControlResponse(200, self._logs(raw_path))
        if method == "GET" and path.startswith("/control/v1/workflows/"):
            run_id = unquote(path.removeprefix("/control/v1/workflows/"))
            try:
                with self.database.connect() as connection:
                    detail = self.workflows.workflow_detail(connection, run_id)
                if self.live_workflow_eligibility is not None:
                    for node in detail["nodes"]:
                        if node["kind"] == "milestone":
                            node["commitEligibility"] = self.live_workflow_eligibility(
                                run_id, str(node["nodeKey"])
                            )
            except KeyError as error:
                raise CoordinatorError("workflow_not_found", "Workflow run was not found") from error
            return ControlResponse(200, detail)
        raise CoordinatorError("not_found", "Unknown control endpoint")

    def _logs(self, raw_path: str) -> dict[str, object]:
        query = parse_qs(urlsplit(raw_path).query)
        try:
            limit = min(500, max(1, int(query.get("limit", ["250"])[0])))
            before_text = query.get("before", [""])[0]
            before = int(before_text) if before_text else None
        except ValueError as error:
            raise CoordinatorError("invalid_request", "Log range must use integer values") from error
        if before is not None and before < 1:
            raise CoordinatorError("invalid_request", "Log range before cursor must be positive")
        where = "WHERE event_id < ?" if before is not None else ""
        parameters: tuple[object, ...] = (before, limit + 1) if before is not None else (limit + 1,)
        with self.database.connect() as connection:
            rows = connection.execute(
                f"""SELECT event_id, session_id, event_type, payload_json, created_at
                    FROM events {where} ORDER BY event_id DESC LIMIT ?""",
                parameters,
            ).fetchall()
        truncated = len(rows) > limit
        selected = rows[:limit]
        events = [
            {
                "eventId": int(row["event_id"]),
                "sessionId": row["session_id"],
                "type": row["event_type"],
                "payload": json.loads(row["payload_json"]),
                "createdAt": row["created_at"],
            }
            for row in reversed(selected)
        ]
        return {
            "events": events,
            "truncated": truncated,
            "nextBefore": int(selected[-1]["event_id"]) if truncated and selected else None,
        }

    def authenticate(
        self, headers: Mapping[str, str], *, runtime_authorized: bool
    ) -> ControlIdentity:
        if runtime_authorized:
            return ControlIdentity("local-runtime", "maintainer", None, None)
        session: WebSessionRecord = self.auth.authenticate_cookie(
            headers.get("Cookie", ""), self.instance_id
        )
        return ControlIdentity(
            session.actor, session.role, session.session_id, session.bound_session_id
        )

    def _actions(self) -> ActionService:
        if self.actions is None:
            raise CoordinatorError("action_unavailable", "Controlled actions are unavailable")
        return self.actions

    def _action_context(
        self,
        identity: ControlIdentity,
        runtime_authorized: bool,
        parameters: Mapping[str, object],
    ) -> ActionContext:
        bound_session_id = identity.bound_session_id
        if runtime_authorized and parameters.get("sessionId"):
            bound_session_id = str(parameters["sessionId"])
        return ActionContext(
            actor=identity.actor,
            role=WebControlRole(identity.role),
            web_session_id=identity.web_session_id,
            bound_session_id=bound_session_id,
            daemon_instance_id=self.instance_id,
        )

    @staticmethod
    def _json_body(body: bytes) -> dict[str, object]:
        if not body:
            return {}
        try:
            payload = json.loads(body.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise CoordinatorError("invalid_json", "Request body must be valid JSON") from error
        if not isinstance(payload, dict):
            raise CoordinatorError("invalid_request", "Request body must be a JSON object")
        return payload
