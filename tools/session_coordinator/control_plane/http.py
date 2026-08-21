from __future__ import annotations

import json
import secrets
import time
from http import HTTPStatus
from urllib.parse import parse_qs, urlsplit

from ..models import CoordinatorError
from .contracts import ControlResponse, error_payload, new_correlation_id
from .artifact_downloads import ArtifactDownloadService
from .assets import BinaryResponse, StaticAssetService
from .events import EventStreamService
from .http_security import (
    validate_browser_read_origin,
    validate_loopback_host,
    validate_loopback_origin,
)
from .router import ControlPlaneRouter


class ControlPlaneHttp:
    """Adapts BaseHTTPRequestHandler to the transport-neutral control router."""

    def __init__(
        self,
        router: ControlPlaneRouter,
        events: EventStreamService,
        *,
        runtime_token: str,
        assets: StaticAssetService,
        artifact_downloads: ArtifactDownloadService,
    ):
        self.router = router
        self.events = events
        self.runtime_token = runtime_token
        self.assets = assets
        self.artifact_downloads = artifact_downloads

    @staticmethod
    def handles(path: str) -> bool:
        route = urlsplit(path).path
        return route.startswith("/control/v1/") or route.startswith("/ui/")

    def handle(self, handler) -> None:
        correlation_id = new_correlation_id()
        port = int(handler.server.server_address[1])
        authorization = handler.headers.get("Authorization", "")
        runtime_authorized = secrets.compare_digest(
            authorization, f"Bearer {self.runtime_token}"
        )
        try:
            validate_loopback_host(handler.headers.get("Host"), port)
            route = urlsplit(handler.path).path
            browser_control = route.startswith("/control/v1/") and not runtime_authorized
            if browser_control:
                if handler.command in {"GET", "HEAD"}:
                    validate_browser_read_origin(
                        handler.headers.get("Origin"),
                        handler.headers.get("Referer"),
                        handler.headers.get("Sec-Fetch-Site"),
                        port,
                    )
                else:
                    validate_loopback_origin(handler.headers.get("Origin"), port)
            elif handler.headers.get("Origin"):
                validate_loopback_origin(handler.headers.get("Origin"), port)
            if handler.command == "GET" and route == "/control/v1/events/stream":
                self._stream_events(
                    handler,
                    runtime_authorized=runtime_authorized,
                    correlation_id=correlation_id,
                )
                return
            if handler.command in {"GET", "HEAD"} and route.startswith("/ui/"):
                asset = self.assets.resolve(handler.path)
                if asset is not None:
                    self._write_binary(handler, asset)
                    return
            if handler.command in {"GET", "HEAD"} and route.startswith("/control/v1/artifacts/"):
                self.router.authenticate(handler.headers, runtime_authorized=runtime_authorized)
                artifact_id = route.removeprefix("/control/v1/artifacts/")
                response = self.artifact_downloads.download(
                    artifact_id, handler.headers.get("Range")
                )
                self._write_binary(handler, response)
                return
            body = self._read_body(handler)
            response = self.router.dispatch(
                handler.command,
                handler.path,
                handler.headers,
                body,
                runtime_authorized=runtime_authorized,
            )
            self._write_response(handler, response, correlation_id)
        except CoordinatorError as error:
            status = self._status_for(error.code)
            self._write_response(
                handler,
                ControlResponse(status, error=error_payload(error)),
                correlation_id,
            )
        except (BrokenPipeError, ConnectionAbortedError, ConnectionResetError, TimeoutError):
            return
        except Exception:
            issue = CoordinatorError("internal_error", "Internal control service error")
            self._write_response(
                handler,
                ControlResponse(500, error=error_payload(issue)),
                correlation_id,
            )

    def close(self) -> None:
        self.events.close()

    def _stream_events(
        self, handler, *, runtime_authorized: bool, correlation_id: str
    ) -> None:
        self.router.authenticate(handler.headers, runtime_authorized=runtime_authorized)
        query = parse_qs(urlsplit(handler.path).query)
        cursor_text = handler.headers.get("Last-Event-ID") or query.get("cursor", ["0"])[0]
        try:
            cursor = max(0, int(cursor_text))
        except ValueError as error:
            raise CoordinatorError("invalid_cursor", "SSE cursor must be an integer") from error
        with self.events.client_slot():
            handler.connection.settimeout(5.0)
            handler.send_response(HTTPStatus.OK)
            handler.send_header("Content-Type", "text/event-stream; charset=utf-8")
            handler.send_header("Cache-Control", "no-store")
            handler.send_header("X-Accel-Buffering", "no")
            handler.send_header("X-Correlation-ID", correlation_id)
            handler.end_headers()
            last_heartbeat = time.monotonic()
            while not self.events.wait_for_close(0):
                replay = self.events.read_after(cursor)
                if replay.resync_required:
                    handler.wfile.write(b"event: resync_required\ndata: {}\n\n")
                    handler.wfile.flush()
                    return
                for event in replay.events:
                    handler.wfile.write(self.events.encode(event).encode("utf-8"))
                    cursor = event.event_id
                now = time.monotonic()
                if now - last_heartbeat >= 15:
                    handler.wfile.write(b": heartbeat\n\n")
                    last_heartbeat = now
                if replay.events or now - last_heartbeat < 0.1:
                    handler.wfile.flush()
                if self.events.wait_for_close(0.25):
                    return

    @staticmethod
    def _read_body(handler) -> bytes:
        length = int(handler.headers.get("Content-Length", "0"))
        if length < 0 or length > 1024 * 1024:
            raise CoordinatorError("request_too_large", "Control request exceeds one MiB")
        return handler.rfile.read(length) if length else b""

    @staticmethod
    def _write_response(handler, response: ControlResponse, correlation_id: str) -> None:
        encoded = json.dumps(response.body(correlation_id), sort_keys=True).encode("utf-8")
        handler.send_response(response.status)
        handler.send_header("Content-Type", "application/json; charset=utf-8")
        handler.send_header("Content-Length", str(len(encoded)))
        handler.send_header("Cache-Control", "no-store")
        handler.send_header("X-Content-Type-Options", "nosniff")
        handler.send_header("X-Correlation-ID", correlation_id)
        for name, value in response.headers.items():
            handler.send_header(name, value)
        handler.end_headers()
        if handler.command != "HEAD":
            handler.wfile.write(encoded)

    @staticmethod
    def _write_binary(handler, response: BinaryResponse) -> None:
        handler.send_response(response.status)
        handler.send_header("Content-Length", str(len(response.body)))
        for name, value in response.headers.items():
            handler.send_header(name, value)
        handler.end_headers()
        if handler.command != "HEAD":
            handler.wfile.write(response.body)

    @staticmethod
    def _status_for(code: str) -> int:
        if code in {
            "invalid_host",
            "invalid_origin",
            "origin_required",
            "csrf_invalid",
            "action_permission_denied",
            "action_session_scope_mismatch",
            "web_session_scope_mismatch",
        }:
            return HTTPStatus.FORBIDDEN
        if code.startswith("web_session") or code == "runtime_auth_required":
            return HTTPStatus.UNAUTHORIZED
        if code == "not_found" or code.endswith("_not_found"):
            return HTTPStatus.NOT_FOUND
        if code in {"action_expired", "elevation_grant_expired"}:
            return HTTPStatus.GONE
        if code in {
            "invalid_json",
            "invalid_request",
            "invalid_cursor",
            "request_too_large",
            "action_limit_invalid",
        }:
            return HTTPStatus.BAD_REQUEST
        if code == "invalid_range":
            return HTTPStatus.REQUESTED_RANGE_NOT_SATISFIABLE
        if code == "sse_capacity":
            return HTTPStatus.SERVICE_UNAVAILABLE
        return HTTPStatus.CONFLICT
