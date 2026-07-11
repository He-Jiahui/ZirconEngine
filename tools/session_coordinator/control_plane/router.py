from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Mapping
from urllib.parse import unquote, urlsplit

from ..models import CoordinatorError, WebControlRole
from ..workflows.projections import WorkflowProjectionService
from .auth import WebControlAuth, WebSessionRecord
from .contracts import CONTROL_API_VERSION, ControlResponse
from .snapshot import ControlSnapshotService


@dataclass(frozen=True, slots=True)
class ControlIdentity:
    actor: str
    role: str
    web_session_id: str | None


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
    ):
        self.instance_id = instance_id
        self.auth = auth
        self.snapshot = snapshot
        self.workflows = workflows
        self.database = database

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

        identity = self.authenticate(headers, runtime_authorized=runtime_authorized)
        if method == "GET" and path == "/control/v1/meta":
            return ControlResponse(
                200,
                {
                    "apiVersion": CONTROL_API_VERSION,
                    "instanceId": self.instance_id,
                    "role": identity.role,
                    "actor": identity.actor,
                    "mutationEnabled": False,
                },
            )
        if method == "GET" and path == "/control/v1/snapshot":
            return ControlResponse(200, self.snapshot.build())
        if method == "GET" and path.startswith("/control/v1/workflows/"):
            run_id = unquote(path.removeprefix("/control/v1/workflows/"))
            try:
                with self.database.connect() as connection:
                    detail = self.workflows.workflow_detail(connection, run_id)
            except KeyError as error:
                raise CoordinatorError("workflow_not_found", "Workflow run was not found") from error
            return ControlResponse(200, detail)
        raise CoordinatorError("not_found", "Unknown control endpoint")

    def authenticate(
        self, headers: Mapping[str, str], *, runtime_authorized: bool
    ) -> ControlIdentity:
        if runtime_authorized:
            return ControlIdentity("local-runtime", "maintainer", None)
        session: WebSessionRecord = self.auth.authenticate_cookie(
            headers.get("Cookie", ""), self.instance_id
        )
        return ControlIdentity(session.actor, session.role, session.session_id)

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
