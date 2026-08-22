from __future__ import annotations

import http.client
import json
import os
import time
import urllib.error
import urllib.request
import uuid
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any

from .config import CoordinatorConfig


_PENDING_ACTION_STATUSES = frozenset({"previewed", "executing"})
_ACTION_POLL_INTERVAL_SECONDS = 0.25
_COMMAND_PREFLIGHT_ATTEMPTS = 2
_COMMAND_RECONCILIATION_TIMEOUT_SECONDS = 1.0
_COMMAND_RECONCILIATION_POLL_INTERVAL_SECONDS = 0.05
_RUNTIME_DESCRIPTOR_RETRY_SECONDS = 3.0
_RUNTIME_DESCRIPTOR_POLL_INTERVAL_SECONDS = 0.05


def _environment_timeout_seconds(name: str, default: float) -> float:
    raw = os.environ.get(name)
    if raw is None:
        return default
    try:
        timeout = float(raw)
    except ValueError:
        return default
    return timeout if timeout > 0 else default


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
    runtime_path: Path | None = None
    timeout_seconds: float = 3.0
    control_timeout_seconds: float = 30.0
    command_timeout_seconds: float = 300.0
    reconciliation_timeout_seconds: float = _COMMAND_RECONCILIATION_TIMEOUT_SECONDS

    @classmethod
    def from_runtime(cls, config: CoordinatorConfig) -> "CoordinatorClient":
        deadline = time.monotonic() + _RUNTIME_DESCRIPTOR_RETRY_SECONDS
        while True:
            try:
                runtime = json.loads(config.runtime_path.read_text(encoding="utf-8"))
                host = str(runtime["host"])
                port = int(runtime["port"])
                token = str(runtime["token"])
                break
            except (OSError, ValueError, KeyError, TypeError) as error:
                if time.monotonic() >= deadline:
                    raise CoordinatorClientError(
                        "offline",
                        "Coordinator runtime descriptor is unavailable",
                        details={"transport": "descriptor_absent"},
                    ) from error
                # A controlled rollover removes the predecessor descriptor before
                # the successor atomically publishes its own. Retry only this
                # read boundary; command requests are never replayed here.
                time.sleep(_RUNTIME_DESCRIPTOR_POLL_INTERVAL_SECONDS)
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
            token=token,
            expected_repository_key=config.repository_key,
            runtime_path=config.runtime_path,
            command_timeout_seconds=_environment_timeout_seconds(
                "ZIRCON_COORDINATOR_COMMAND_TIMEOUT_SECONDS", 300.0
            ),
        )

    def health(self) -> dict[str, Any]:
        health = self._request("GET", "/health")
        self._require_expected_repository(health)
        return health

    def command_request_status(self, request_id: str) -> dict[str, Any]:
        return self._validated_command_request_status(request_id)

    def _validated_command_request_status(
        self, request_id: str, *, timeout_seconds: float | None = None
    ) -> dict[str, Any]:
        if timeout_seconds is None:
            result = self._request("GET", f"/command/requests/{request_id}")
        else:
            result = self._request(
                "GET", f"/command/requests/{request_id}", timeout_seconds=timeout_seconds
            )
        if self.expected_repository_key is not None:
            actual = result.get("repositoryKey")
            if actual != self.expected_repository_key:
                raise CoordinatorClientError(
                    "repository_mismatch",
                    "Command request status belongs to another repository",
                    details={
                        "expectedRepositoryKey": self.expected_repository_key,
                        "actualRepositoryKey": actual,
                    },
                )
        return result

    def _reconcile_command_request(self, request_id: str) -> dict[str, Any]:
        """Recover one timed-out command without replaying its mutation."""
        deadline = time.monotonic() + max(0.0, self.reconciliation_timeout_seconds)
        last_query_error: CoordinatorClientError | None = None
        submission = "unknown"

        while True:
            remaining = deadline - time.monotonic()
            query_timeout = min(
                self.timeout_seconds,
                max(0.001, remaining),
            )
            try:
                query = self._validated_command_request_status(
                    request_id, timeout_seconds=query_timeout
                )
            except CoordinatorClientError as error:
                if error.code == "repository_mismatch":
                    raise
                last_query_error = error
            else:
                request = query.get("request")
                if isinstance(request, dict):
                    status = str(request.get("status") or "")
                    if status == "completed" and isinstance(query.get("result"), dict):
                        return query["result"]
                    if status == "failed" and isinstance(query.get("error"), dict):
                        issue = query["error"]
                        raise CoordinatorClientError(
                            str(issue.get("code", "command_failed")),
                            str(issue.get("message", "Coordinator command failed")),
                            details=issue.get("details") or {},
                        )
                    if status == "accepted":
                        submission = "accepted"

            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            time.sleep(min(_COMMAND_RECONCILIATION_POLL_INTERVAL_SECONDS, remaining))

        details = {
            "requestId": request_id,
            "phase": "post_response",
            "submission": submission,
            "recovery": f"GET /command/requests/{request_id}",
        }
        if last_query_error is not None:
            details["lastQueryError"] = last_query_error.code

        raise CoordinatorClientError(
            "command_post_timeout",
            "Coordinator command has no terminal result after reconciliation",
            details=details,
        )

    def command(self, command: str, arguments: dict[str, Any] | None = None) -> dict[str, Any]:
        request_id = uuid.uuid4().hex
        for attempt in range(1, _COMMAND_PREFLIGHT_ATTEMPTS + 1):
            try:
                self._verify_endpoint_repository()
                break
            except CoordinatorClientError as error:
                if error.code != "command_timeout":
                    raise
                if attempt < _COMMAND_PREFLIGHT_ATTEMPTS:
                    continue
                raise CoordinatorClientError(
                    "command_preflight_timeout",
                    "Coordinator health preflight exceeded its deadline; command was not submitted",
                    details={
                        "requestId": request_id,
                        "command": command,
                        "phase": "preflight",
                        "submission": "not_submitted",
                        "attempts": attempt,
                    },
                ) from error
        payload = {
            "request_id": request_id,
            "command": command,
            "arguments": arguments or {},
        }
        try:
            return self._request(
                "POST",
                "/command",
                payload,
                timeout_seconds=self.command_timeout_seconds,
            )
        except CoordinatorClientError as error:
            if error.code not in {"command_timeout", "offline", "invalid_response"}:
                raise
            try:
                return self._reconcile_command_request(request_id)
            except CoordinatorClientError as reconciliation_error:
                details = dict(reconciliation_error.details)
                details.setdefault("command", command)
                details.setdefault("timeoutSeconds", self.command_timeout_seconds)
                raise CoordinatorClientError(
                    reconciliation_error.code,
                    reconciliation_error.message,
                    details=details,
                ) from error

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
        polling_client = self
        while action.get("status") in _PENDING_ACTION_STATUSES:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise CoordinatorClientError(
                    "command_timeout",
                    "Coordinator action did not reach a terminal state before its deadline",
                    details={"actionId": action_id, "kind": kind},
                )
            time.sleep(min(_ACTION_POLL_INTERVAL_SECONDS, remaining))
            try:
                detail = polling_client.control_request(
                    "GET", f"/control/v1/actions/{action_id}"
                )
            except CoordinatorClientError as error:
                recoverable = kind == "service.rollover" and error.code in {
                    "offline",
                    "command_timeout",
                    "action_instance_mismatch",
                    "unauthorized",
                }
                if not recoverable:
                    raise
                if error.code == "unauthorized":
                    polling_client = polling_client._refresh_runtime_client()
                continue
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
        try:
            identity = self._request("GET", "/identity")
        except CoordinatorClientError as error:
            if error.code != "not_found":
                raise
            identity = self._request(
                "GET", "/health", timeout_seconds=self.control_timeout_seconds
            )
        self._require_expected_repository(identity)

    def _refresh_runtime_client(self) -> "CoordinatorClient":
        if self.runtime_path is None:
            raise CoordinatorClientError(
                "offline",
                "Coordinator successor descriptor is unavailable",
                details={"transport": "descriptor_absent"},
            )
        deadline = time.monotonic() + _RUNTIME_DESCRIPTOR_RETRY_SECONDS
        while True:
            try:
                runtime = json.loads(self.runtime_path.read_text(encoding="utf-8"))
                host = str(runtime["host"])
                port = int(runtime["port"])
                token = str(runtime["token"])
                descriptor_key = runtime.get("repository_key")
                if (
                    descriptor_key is not None
                    and self.expected_repository_key is not None
                    and descriptor_key != self.expected_repository_key
                ):
                    raise CoordinatorClientError(
                        "repository_mismatch",
                        "Coordinator successor descriptor belongs to another repository",
                        details={
                            "expectedRepositoryKey": self.expected_repository_key,
                            "actualRepositoryKey": descriptor_key,
                        },
                    )
                if not token:
                    raise ValueError("runtime token is empty")
                return replace(
                    self,
                    base_url=f"http://{host}:{port}",
                    token=token,
                )
            except CoordinatorClientError:
                raise
            except (OSError, ValueError, KeyError, TypeError) as error:
                if time.monotonic() >= deadline:
                    raise CoordinatorClientError(
                        "offline",
                        "Coordinator successor descriptor is unavailable",
                        details={"transport": "descriptor_absent"},
                    ) from error
                time.sleep(_RUNTIME_DESCRIPTOR_POLL_INTERVAL_SECONDS)

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
        except (json.JSONDecodeError, UnicodeDecodeError, http.client.IncompleteRead) as error:
            raise CoordinatorClientError(
                "invalid_response", "Coordinator response was truncated or invalid JSON"
            ) from error
        except urllib.error.HTTPError as error:
            try:
                body = json.loads(error.read().decode("utf-8"))
                issue = body.get("error", {})
                raise CoordinatorClientError(
                    str(issue.get("code", "http_error")),
                    str(issue.get("message", error.reason)),
                    details=issue.get("details") or {},
                ) from error
            except (
                ValueError,
                AttributeError,
                UnicodeDecodeError,
                http.client.IncompleteRead,
                OSError,
            ) as parse_error:
                raise CoordinatorClientError(
                    "invalid_response", "Coordinator error response was truncated or invalid JSON"
                ) from parse_error
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
