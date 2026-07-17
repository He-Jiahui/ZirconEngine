from __future__ import annotations

import json
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any

from .config import CoordinatorConfig


_PENDING_ACTION_STATUSES = frozenset({"previewed", "executing"})
_ACTION_POLL_INTERVAL_SECONDS = 0.25


class CoordinatorClientError(RuntimeError):
    def __init__(self, code: str, message: str, *, details: dict[str, Any] | None = None):
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or {}

    def to_dict(self) -> dict[str, Any]:
        return {"code": self.code, "message": self.message, "details": self.details}


@dataclass(frozen=True, slots=True)
class CoordinatorClient:
    base_url: str
    token: str
    expected_repository_key: str | None = None
    timeout_seconds: float = 3.0
    control_timeout_seconds: float = 30.0
    command_timeout_seconds: float = 300.0

    @classmethod
    def from_runtime(cls, config: CoordinatorConfig) -> "CoordinatorClient":
        try:
            runtime = json.loads(config.runtime_path.read_text(encoding="utf-8"))
            host = str(runtime["host"])
            port = int(runtime["port"])
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise CoordinatorClientError(
                "offline",
                "Coordinator runtime descriptor is unavailable",
                details={"transport": "descriptor_absent"},
            ) from error
        descriptor_key = runtime.get("repository_key")
        if descriptor_key is not None and descriptor_key != config.repository_key:
            raise CoordinatorClientError(
                "repository_mismatch",
                "Coordinator runtime descriptor belongs to another repository",
                details={
                    "expectedRepositoryKey": config.repository_key,
                    "actualRepositoryKey": descriptor_key,
                },
            )
        return cls(
            base_url=f"http://{host}:{port}",
            token="",
            expected_repository_key=config.repository_key,
        )

    def health(self) -> dict[str, Any]:
        health = self._request("GET", "/health")
        self._require_expected_repository(health)
        return health

    def command(self, command: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
        self._verify_endpoint_repository()
        try:
            return self._request(
                "POST",
                "/command",
                {"command": command, "arguments": arguments or {}},
                timeout_seconds=self.command_timeout_seconds,
            )
        except CoordinatorClientError as error:
            if error.code != "command_timeout":
                raise
            details = dict(error.details)
            details.update(
                {
                    "command": command,
                    "timeoutSeconds": self.command_timeout_seconds,
                    "recovery": "query health and the typed job/session status before retrying",
                }
            )
            raise CoordinatorClientError(error.code, error.message, details=details) from error

    def shutdown(self) -> dict[str, Any]:
        preview = self.control_request(
            "POST",
            "/control/v1/actions/preview",
            {"kind": "service.stop", "parameters": {"timeoutSeconds": 30}},
        )
        action = preview.get("action") or {}
        action_id = str(action.get("actionId") or "")
        phrase = str(action.get("confirmationPhrase") or "")
        if not action_id or not phrase:
            raise CoordinatorClientError(
                "invalid_response", "Stop preview omitted controlled confirmation data"
            )
        confirmed = self.control_request(
            "POST",
            f"/control/v1/actions/{action_id}/confirm",
            {"phrase": phrase, "reason": "explicit local CLI stop"},
        )
        return {
            "status": "stopping",
            "action": confirmed.get("action") or {},
        }

    def issue_ui_ticket(self, *, actor: str, role: str = "observer") -> dict[str, Any]:
        return self.control_request(
            "POST",
            "/control/v1/bootstrap-tickets",
            {"actor": actor, "role": role},
        )

    def control_snapshot(self) -> dict[str, Any]:
        return self.control_request("GET", "/control/v1/snapshot")

    def execute_control_action(
        self,
        kind: str,
        parameters: dict[str, Any],
        *,
        reason: str,
    ) -> dict[str, Any]:
        """Run one local controlled action through its preview/confirm boundary."""
        preview = self.control_request(
            "POST",
            "/control/v1/actions/preview",
            {"kind": kind, "parameters": parameters},
        )
        action = preview.get("action")
        if not isinstance(action, dict):
            raise CoordinatorClientError(
                "invalid_response", "Coordinator action preview omitted its action record"
            )
        action_id = action.get("actionId")
        phrase = action.get("confirmationPhrase")
        if not isinstance(action_id, str) or not action_id or not isinstance(phrase, str) or not phrase:
            raise CoordinatorClientError(
                "invalid_response", "Coordinator action preview omitted its confirmation data"
            )
        confirmed = self.control_request(
            "POST",
            f"/control/v1/actions/{action_id}/confirm",
            {"phrase": phrase, "reason": reason},
        )
        action = confirmed.get("action")
        if not isinstance(action, dict):
            raise CoordinatorClientError(
                "invalid_response", "Coordinator action confirmation omitted its result"
            )
        deadline = time.monotonic() + self.command_timeout_seconds
        while action.get("status") in _PENDING_ACTION_STATUSES:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise CoordinatorClientError(
                    "command_timeout",
                    "Coordinator action did not reach a terminal state before its deadline",
                    details={"actionId": action_id, "kind": kind},
                )
            time.sleep(min(_ACTION_POLL_INTERVAL_SECONDS, remaining))
            detail = self.control_request("GET", f"/control/v1/actions/{action_id}")
            action = detail.get("action")
            if not isinstance(action, dict):
                raise CoordinatorClientError(
                    "invalid_response", "Coordinator action detail omitted its action record"
                )
        return action

    def issue_elevation_grant(
        self,
        *,
        actor: str,
        role: str,
        session_id: str | None = None,
        maintenance_capability: str | None = None,
    ) -> dict[str, Any]:
        payload: dict[str, Any] = {"actor": actor, "role": role}
        if session_id:
            payload["sessionId"] = session_id
        if maintenance_capability:
            payload["maintenance_capability"] = maintenance_capability
        return self.control_request("POST", "/control/v1/elevation-grants", payload)

    def control_request(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        self._verify_endpoint_repository()
        envelope = self._request(
            method,
            path,
            payload,
            timeout_seconds=self.control_timeout_seconds,
        )
        if envelope.get("ok") is not True or not isinstance(envelope.get("data"), dict):
            raise CoordinatorClientError(
                "invalid_response", "Coordinator returned an invalid control envelope"
            )
        return envelope["data"]

    def _verify_endpoint_repository(self) -> None:
        if self.expected_repository_key is None:
            return
        self._require_expected_repository(self._request("GET", "/health"))

    def _require_expected_repository(self, health: dict[str, Any]) -> None:
        if self.expected_repository_key is None:
            return
        actual = health.get("repository_key")
        if actual == self.expected_repository_key:
            return
        raise CoordinatorClientError(
            "repository_mismatch",
            "Coordinator endpoint belongs to another repository",
            details={
                "expectedRepositoryKey": self.expected_repository_key,
                "actualRepositoryKey": actual,
            },
        )

    def _request(
        self,
        method: str,
        path: str,
        payload: dict[str, Any] | None = None,
        *,
        timeout_seconds: float | None = None,
    ) -> dict[str, Any]:
        data = json.dumps(payload).encode("utf-8") if payload is not None else None
        request = urllib.request.Request(
            f"{self.base_url}{path}",
            data=data,
            headers={
                "Content-Type": "application/json",
            },
            method=method,
        )
        try:
            with urllib.request.urlopen(
                request, timeout=timeout_seconds or self.timeout_seconds
            ) as response:
                body = json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as error:
            try:
                body = json.loads(error.read().decode("utf-8"))
                issue = body.get("error", {})
                raise CoordinatorClientError(
                    str(issue.get("code", "http_error")),
                    str(issue.get("message", error.reason)),
                    details=issue.get("details") or {},
                ) from error
            except (ValueError, AttributeError):
                raise CoordinatorClientError("http_error", str(error.reason)) from error
            finally:
                error.close()
        except TimeoutError as error:
            raise CoordinatorClientError(
                "command_timeout",
                "Coordinator request exceeded its deadline; the service may still be processing it",
            ) from error
        except urllib.error.URLError as error:
            if isinstance(error.reason, TimeoutError):
                raise CoordinatorClientError(
                    "command_timeout",
                    "Coordinator request exceeded its deadline; the service may still be processing it",
                ) from error
            transport = (
                "connection_refused"
                if isinstance(error.reason, ConnectionRefusedError)
                else "connection_uncertain"
            )
            raise CoordinatorClientError(
                "offline",
                "Coordinator service is offline",
                details={"transport": transport},
            ) from error
        except OSError as error:
            raise CoordinatorClientError("offline", "Coordinator service is offline") from error
        if not isinstance(body, dict):
            raise CoordinatorClientError("invalid_response", "Coordinator returned a non-object response")
        return body
