from __future__ import annotations

import http.client
import http.server
import io
import json
import os
import tempfile
import threading
import unittest
import urllib.error
from contextlib import redirect_stdout
from pathlib import Path
from unittest import mock

from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError
from tools.session_coordinator.cli import _parser, main
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.tests.helpers import init_repo


class CoordinatorClientTests(unittest.TestCase):
    def test_command_request_status_uses_named_read_only_endpoint(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        request_id = "a" * 32
        expected = {"request": {"requestId": request_id, "status": "accepted"}}
        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", return_value=expected) as request,
        ):
            result = client.command_request_status(request_id)

        self.assertEqual(expected, result)
        request.assert_called_once_with("GET", f"/command/requests/{request_id}")

    def test_command_request_status_bypasses_health_and_validates_repository(self) -> None:
        request_id = "c" * 32
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", expected_repository_key="repository-a"
        )
        expected = {
            "repositoryKey": "repository-a",
            "request": {"requestId": request_id, "status": "accepted"},
        }
        with (
            mock.patch.object(
                CoordinatorClient,
                "_verify_endpoint_repository",
                side_effect=AssertionError("request recovery must not depend on health"),
            ) as preflight,
            mock.patch.object(CoordinatorClient, "_request", return_value=expected),
        ):
            result = client.command_request_status(request_id)

        self.assertEqual(expected, result)
        preflight.assert_not_called()

    def test_cli_exposes_wrapper_safe_request_status_command(self) -> None:
        request_id = "b" * 32

        arguments = _parser().parse_args(["request-status", request_id])

        self.assertEqual("request-status", arguments.command)
        self.assertEqual(request_id, arguments.request_id)

    def test_plain_cli_preserves_post_response_recovery_identity(self) -> None:
        output = io.StringIO()
        error = CoordinatorClientError(
            "command_post_timeout",
            "Coordinator command remains accepted but has no terminal result",
            details={
                "requestId": "a" * 32,
                "submission": "accepted",
                "recovery": "GET /command/requests/" + "a" * 32,
            },
        )

        with (
            mock.patch("tools.session_coordinator.cli._run", side_effect=error),
            redirect_stdout(output),
        ):
            exit_code = main(["session", "list"])

        self.assertEqual(2, exit_code)
        payload = json.loads(output.getvalue())
        self.assertEqual("command_post_timeout", payload["error"]["code"])
        self.assertEqual("a" * 32, payload["error"]["details"]["requestId"])

    def test_command_preflight_timeout_is_explicitly_not_submitted(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        timeout = CoordinatorClientError(
            "command_timeout", "health probe exceeded its deadline"
        )

        with (
            mock.patch.object(
                CoordinatorClient,
                "_verify_endpoint_repository",
                side_effect=(timeout, timeout),
            ) as preflight,
            mock.patch.object(CoordinatorClient, "_request") as request,
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("session.register", {"session_id": "session-a"})

        self.assertEqual("command_preflight_timeout", rejected.exception.code)
        self.assertEqual("preflight", rejected.exception.details["phase"])
        self.assertEqual("not_submitted", rejected.exception.details["submission"])
        self.assertEqual(32, len(rejected.exception.details["requestId"]))
        self.assertEqual(2, preflight.call_count)
        request.assert_not_called()

    def test_command_retries_only_the_timed_out_preflight_before_one_submission(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        timeout = CoordinatorClientError(
            "command_timeout", "health probe exceeded its deadline"
        )
        expected = {"session": {"session_id": "session-a"}}

        with (
            mock.patch.object(
                CoordinatorClient,
                "_verify_endpoint_repository",
                side_effect=(timeout, None),
            ) as preflight,
            mock.patch.object(
                CoordinatorClient, "_request", return_value=expected
            ) as request,
        ):
            result = client.command("session.register", {"session_id": "session-a"})

        self.assertEqual(expected, result)
        self.assertEqual(2, preflight.call_count)
        request.assert_called_once()
        method, path, payload = request.call_args.args
        self.assertEqual(("POST", "/command"), (method, path))
        self.assertEqual("session.register", payload["command"])
        self.assertEqual({"session_id": "session-a"}, payload["arguments"])
        self.assertEqual(32, len(payload["request_id"]))

    def test_command_post_timeout_queries_durable_request_without_replay(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        calls: list[tuple[str, str, object]] = []

        def request(
            _client,
            method: str,
            path: str,
            payload=None,
            *,
            timeout_seconds=None,
        ):
            calls.append((method, path, payload))
            if method == "POST":
                raise CoordinatorClientError(
                    "command_timeout", "POST response exceeded its deadline"
                )
            return {
                "request": {
                    "requestId": path.rsplit("/", 1)[-1],
                    "status": "accepted",
                }
            }

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", new=request),
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("post_response", rejected.exception.details["phase"])
        self.assertEqual("accepted", rejected.exception.details["submission"])
        request_id = rejected.exception.details["requestId"]
        self.assertEqual(
            [
                ("POST", "/command", {"request_id": request_id, "command": "baseline.scan", "arguments": {}}),
                ("GET", f"/command/requests/{request_id}", None),
            ],
            calls,
        )

    def test_command_post_timeout_reconciles_the_same_request_to_terminal_result(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0.05
        )
        calls: list[tuple[str, str, object]] = []
        statuses = iter(("accepted", "completed"))

        def request(
            _client,
            method: str,
            path: str,
            payload=None,
            *,
            timeout_seconds=None,
        ):
            calls.append((method, path, payload))
            if method == "POST":
                raise CoordinatorClientError(
                    "command_timeout", "POST response exceeded its deadline"
                )
            status = next(statuses)
            result = {
                "request": {
                    "requestId": path.rsplit("/", 1)[-1],
                    "status": status,
                }
            }
            if status == "completed":
                result["result"] = {
                    "requestId": path.rsplit("/", 1)[-1],
                    "status": "reconciled",
                }
            return result

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", new=request),
        ):
            result = client.command("baseline.scan")

        self.assertEqual("reconciled", result["status"])
        request_id = result["requestId"]
        self.assertEqual(
            [
                ("POST", "/command", {"request_id": request_id, "command": "baseline.scan", "arguments": {}}),
                ("GET", f"/command/requests/{request_id}", None),
                ("GET", f"/command/requests/{request_id}", None),
            ],
            calls,
        )

    def test_command_post_timeout_keeps_unfenced_missing_request_unknown(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        calls: list[tuple[str, str, object]] = []

        def request(
            _client,
            method: str,
            path: str,
            payload=None,
            *,
            timeout_seconds=None,
        ):
            calls.append((method, path, payload))
            if method == "POST":
                raise CoordinatorClientError(
                    "command_timeout", "POST response exceeded its deadline"
                )
            raise CoordinatorClientError("command_request_not_found", "request not visible")

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", new=request),
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("unknown", rejected.exception.details["submission"])
        self.assertEqual(
            "command_request_not_found", rejected.exception.details["lastQueryError"]
        )
        request_id = rejected.exception.details["requestId"]
        self.assertEqual(
            [
                ("POST", "/command", {"request_id": request_id, "command": "baseline.scan", "arguments": {}}),
                ("GET", f"/command/requests/{request_id}", None),
            ],
            calls,
        )

    def test_command_post_recovery_rejects_another_repository(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", expected_repository_key="repository-a"
        )

        def request(
            _client,
            method: str,
            path: str,
            payload=None,
            *,
            timeout_seconds=None,
        ):
            if method == "POST":
                raise CoordinatorClientError(
                    "command_timeout", "POST response exceeded its deadline"
                )
            return {
                "repositoryKey": "repository-b",
                "request": {
                    "requestId": path.rsplit("/", 1)[-1],
                    "status": "completed",
                },
                "result": {"status": "wrong-repository-result"},
            }

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", new=request),
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("repository_mismatch", rejected.exception.code)
        self.assertEqual(
            "repository-a", rejected.exception.details["expectedRepositoryKey"]
        )
        self.assertEqual(
            "repository-b", rejected.exception.details["actualRepositoryKey"]
        )

    def test_command_post_disconnect_keeps_unfenced_missing_request_unknown(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )
        calls: list[tuple[str, str, object]] = []

        def request(
            _client,
            method: str,
            path: str,
            payload=None,
            *,
            timeout_seconds=None,
        ):
            calls.append((method, path, payload))
            if method == "POST":
                raise CoordinatorClientError(
                    "offline",
                    "connection closed before the response was read",
                    details={"transport": "connection_uncertain"},
                )
            raise CoordinatorClientError("command_request_not_found", "request not visible yet")

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch.object(CoordinatorClient, "_request", new=request),
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("cargo.run_reserved", {"job_id": "job-a"})

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("post_response", rejected.exception.details["phase"])
        self.assertEqual("unknown", rejected.exception.details["submission"])
        self.assertEqual(
            "command_request_not_found", rejected.exception.details["lastQueryError"]
        )
        request_id = rejected.exception.details["requestId"]
        self.assertEqual(
            [
                (
                    "POST",
                    "/command",
                    {
                        "request_id": request_id,
                        "command": "cargo.run_reserved",
                        "arguments": {"job_id": "job-a"},
                    },
                ),
                ("GET", f"/command/requests/{request_id}", None),
            ],
            calls,
        )

    def test_get_overtakes_late_post_without_proving_not_accepted(self) -> None:
        repository_key = "repository-a"
        post_received = threading.Event()
        allow_post = threading.Event()
        post_completed = threading.Event()
        state: dict[str, object] = {"postCount": 0, "requestId": None}

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self) -> None:
                if self.path == "/health":
                    self._write_json(200, {"status": "ok", "repository_key": repository_key})
                    return
                if self.path.startswith("/command/requests/"):
                    self._write_json(
                        404,
                        {
                            "error": {
                                "code": "command_request_not_found",
                                "message": "request is not visible yet",
                                "details": {},
                            }
                        },
                    )
                    return
                self._write_json(404, {"error": {"code": "not_found"}})

            def do_POST(self) -> None:
                length = int(self.headers.get("Content-Length", "0"))
                payload = json.loads(self.rfile.read(length).decode("utf-8"))
                state["requestId"] = payload["request_id"]
                post_received.set()
                allow_post.wait(timeout=5)
                state["postCount"] = int(state["postCount"]) + 1
                post_completed.set()
                self._write_json(200, {"status": "late-post-completed"})

            def _write_json(self, status: int, payload: dict[str, object]) -> None:
                encoded = json.dumps(payload).encode("utf-8")
                try:
                    self.send_response(status)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(encoded)))
                    self.end_headers()
                    self.wfile.write(encoded)
                except OSError:
                    pass

            def log_message(self, _format: str, *_args: object) -> None:
                return

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        server_thread = threading.Thread(target=server.serve_forever, daemon=True)
        server_thread.start()
        client = CoordinatorClient(
            f"http://127.0.0.1:{server.server_port}",
            "",
            expected_repository_key=repository_key,
            timeout_seconds=0.1,
            command_timeout_seconds=0.05,
            reconciliation_timeout_seconds=0,
        )
        try:
            with self.assertRaises(CoordinatorClientError) as rejected:
                client.command("baseline.scan")

            self.assertTrue(post_received.is_set())
            self.assertEqual("command_post_timeout", rejected.exception.code)
            self.assertEqual("unknown", rejected.exception.details["submission"])
            self.assertEqual(
                "command_request_not_found", rejected.exception.details["lastQueryError"]
            )
            self.assertEqual(0, state["postCount"])

            allow_post.set()
            self.assertTrue(post_completed.wait(timeout=2))
            self.assertEqual(1, state["postCount"])
            self.assertEqual(rejected.exception.details["requestId"], state["requestId"])
        finally:
            allow_post.set()
            server.shutdown()
            server.server_close()
            server_thread.join(timeout=2)

    def test_command_truncated_post_response_queries_durable_request(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )

        class Response:
            def __init__(self, body: bytes):
                self.body = body

            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return False

            def read(self):
                return self.body

        accepted = json.dumps(
            {"request": {"requestId": "ignored", "status": "accepted"}}
        ).encode("utf-8")
        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch(
                "tools.session_coordinator.client.urllib.request.urlopen",
                side_effect=[Response(b"{"), Response(accepted)],
            ) as urlopen,
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("accepted", rejected.exception.details["submission"])
        self.assertEqual(2, urlopen.call_count)

    def test_command_truncated_http_error_queries_durable_request(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )

        class Response:
            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return False

            @staticmethod
            def read():
                return b'{"request":{"status":"accepted"}}'

        malformed = urllib.error.HTTPError(
            "http://127.0.0.1:43123/command",
            500,
            "Internal Server Error",
            None,
            io.BytesIO(b"{"),
        )
        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch(
                "tools.session_coordinator.client.urllib.request.urlopen",
                side_effect=[malformed, Response()],
            ) as urlopen,
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("accepted", rejected.exception.details["submission"])
        self.assertEqual(2, urlopen.call_count)

    def test_command_incomplete_http_error_body_queries_durable_request(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )

        class IncompleteErrorBody:
            def read(self):
                raise http.client.IncompleteRead(b'{"error"', 32)

            def close(self):
                return None

        class AcceptedResponse:
            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return False

            @staticmethod
            def read():
                return b'{"request":{"status":"accepted"}}'

        truncated = urllib.error.HTTPError(
            "http://127.0.0.1:43123/command",
            500,
            "Internal Server Error",
            None,
            IncompleteErrorBody(),
        )
        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch(
                "tools.session_coordinator.client.urllib.request.urlopen",
                side_effect=[truncated, AcceptedResponse()],
            ) as urlopen,
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("accepted", rejected.exception.details["submission"])
        self.assertEqual(2, urlopen.call_count)

    def test_command_incomplete_body_queries_durable_request(self) -> None:
        client = CoordinatorClient(
            "http://127.0.0.1:43123", "", reconciliation_timeout_seconds=0
        )

        class IncompleteResponse:
            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return False

            @staticmethod
            def read():
                raise http.client.IncompleteRead(b'{"request"', 20)

        class AcceptedResponse:
            def __enter__(self):
                return self

            def __exit__(self, *_args):
                return False

            @staticmethod
            def read():
                return b'{"request":{"status":"accepted"}}'

        with (
            mock.patch.object(CoordinatorClient, "_verify_endpoint_repository"),
            mock.patch(
                "tools.session_coordinator.client.urllib.request.urlopen",
                side_effect=[IncompleteResponse(), AcceptedResponse()],
            ) as urlopen,
            self.assertRaises(CoordinatorClientError) as rejected,
        ):
            client.command("baseline.scan")

        self.assertEqual("command_post_timeout", rejected.exception.code)
        self.assertEqual("accepted", rejected.exception.details["submission"])
        self.assertEqual(2, urlopen.call_count)

    def test_from_runtime_waits_for_descriptor_published_during_rollover(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=43123
            )

            def publish_successor_descriptor() -> None:
                config.state_root.mkdir(parents=True, exist_ok=True)
                config.runtime_path.write_text(
                    json.dumps(
                        {
                            "host": "127.0.0.1",
                            "port": config.port,
                            "repository_key": config.repository_key,
                        }
                    ),
                    encoding="utf-8",
                )

            publisher = threading.Timer(0.02, publish_successor_descriptor)
            publisher.start()
            try:
                client = CoordinatorClient.from_runtime(config)
            finally:
                publisher.cancel()
                publisher.join(timeout=1)

        self.assertEqual("http://127.0.0.1:43123", client.base_url)

    def test_from_runtime_honors_the_wrapper_command_deadline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = CoordinatorConfig.for_repo(
                init_repo(root / "repo"), state_root=root / "state", port=43123
            )
            config.state_root.mkdir(parents=True, exist_ok=True)
            config.runtime_path.write_text(
                json.dumps(
                    {
                        "host": "127.0.0.1",
                        "port": config.port,
                        "repository_key": config.repository_key,
                    }
                ),
                encoding="utf-8",
            )
            with mock.patch.dict(
                os.environ,
                {"ZIRCON_COORDINATOR_COMMAND_TIMEOUT_SECONDS": "15"},
                clear=False,
            ):
                client = CoordinatorClient.from_runtime(config)

        self.assertEqual(15.0, client.command_timeout_seconds)


if __name__ == "__main__":
    unittest.main()
