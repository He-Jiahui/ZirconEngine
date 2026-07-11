from __future__ import annotations

import json
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any

from .config import CoordinatorConfig


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
    timeout_seconds: float = 3.0
    command_timeout_seconds: float = 300.0

    @classmethod
    def from_runtime(cls, config: CoordinatorConfig) -> "CoordinatorClient":
        try:
            runtime = json.loads(config.runtime_path.read_text(encoding="utf-8"))
            host = str(runtime["host"])
            port = int(runtime["port"])
            token = str(runtime["token"])
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise CoordinatorClientError("offline", "Coordinator runtime descriptor is unavailable") from error
        return cls(base_url=f"http://{host}:{port}", token=token)

    def health(self) -> dict[str, Any]:
        return self._request("GET", "/health")

    def command(self, command: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
        return self._request(
            "POST",
            "/command",
            {"command": command, "arguments": arguments or {}},
            timeout_seconds=self.command_timeout_seconds,
        )

    def shutdown(self) -> dict[str, Any]:
        return self._request("POST", "/shutdown", {})

    def issue_ui_ticket(self, *, actor: str, role: str = "observer") -> dict[str, Any]:
        return self.control_request(
            "POST",
            "/control/v1/bootstrap-tickets",
            {"actor": actor, "role": role},
        )

    def control_snapshot(self) -> dict[str, Any]:
        return self.control_request("GET", "/control/v1/snapshot")

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
        envelope = self._request(method, path, payload)
        if envelope.get("ok") is not True or not isinstance(envelope.get("data"), dict):
            raise CoordinatorClientError(
                "invalid_response", "Coordinator returned an invalid control envelope"
            )
        return envelope["data"]

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
                "Authorization": f"Bearer {self.token}",
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
        except (urllib.error.URLError, TimeoutError, OSError) as error:
            raise CoordinatorClientError("offline", "Coordinator service is offline") from error
        if not isinstance(body, dict):
            raise CoordinatorClientError("invalid_response", "Coordinator returned a non-object response")
        return body
