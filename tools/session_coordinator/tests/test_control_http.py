from __future__ import annotations

import http.cookiejar
from email.message import Message
from io import BytesIO
import json
import tempfile
import time
import unittest
import urllib.error
import urllib.request
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.http import ControlPlaneHttp
from tools.session_coordinator.control_plane.router import ControlPlaneRouter
from tools.session_coordinator.database import Database
from tools.session_coordinator.models import CoordinatorError, WebControlRole
from tools.session_coordinator.server import CoordinatorRequestHandler, RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class ControlHttpTests(unittest.TestCase):
    def test_json_body_types_extremely_large_numbers(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            ControlPlaneRouter._json_body(b'{"value":' + b"9" * 5000 + b"}")

        self.assertEqual("invalid_json", rejected.exception.code)
        self.assertEqual("Request body must be valid JSON", rejected.exception.message)

    def test_history_limit_errors_map_to_bad_request(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            ControlPlaneRouter._history_limit(
                "/control/v1/failures/history?limit=not-an-integer",
                default=100,
            )

        self.assertEqual("history_limit_invalid", rejected.exception.code)
        self.assertEqual(400, ControlPlaneHttp._status_for(rejected.exception.code))

    def test_history_limit_error_projects_as_400(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/failures/history?limit=invalid",
                    headers={"Origin": running.base_url},
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(request, timeout=2)
                body = json.loads(rejected.exception.read())
                rejected.exception.close()

        self.assertEqual(400, rejected.exception.code)
        self.assertEqual("history_limit_invalid", body["error"]["code"])

    def test_runtime_role_endpoints_type_malformed_roles(self) -> None:
        router = ControlPlaneRouter(
            instance_id="instance-a",
            auth=SimpleNamespace(),
            snapshot=SimpleNamespace(),
            workflows=SimpleNamespace(),
            database=SimpleNamespace(),
        )
        for path, role in (
            ("/control/v1/bootstrap-tickets", "not-a-role"),
            ("/control/v1/elevation-grants", ["operator"]),
        ):
            with self.subTest(path=path, role=role):
                with self.assertRaises(CoordinatorError) as rejected:
                    router.dispatch(
                        "POST",
                        path,
                        {},
                        json.dumps({"role": role}).encode("utf-8"),
                        runtime_authorized=True,
                    )

                self.assertEqual("invalid_request", rejected.exception.code)
                self.assertEqual("Control role is invalid", rejected.exception.message)

    def test_runtime_role_endpoint_projects_malformed_role_as_400(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/bootstrap-tickets",
                    data=json.dumps({"role": ["observer"]}).encode("utf-8"),
                    headers={
                        "Authorization": f"Bearer {running.token}",
                        "Content-Type": "application/json",
                    },
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(request, timeout=2)
                body = json.loads(rejected.exception.read())
                rejected.exception.close()

        self.assertEqual(400, rejected.exception.code)
        self.assertEqual("invalid_request", body["error"]["code"])
        self.assertEqual("Control role is invalid", body["error"]["message"])

    def test_invalid_content_length_is_typed(self) -> None:
        headers = Message()
        headers["Content-Length"] = "not-a-number"
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"{}"))

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("invalid_content_length", raised.exception.code)

    def test_conflicting_content_lengths_are_typed(self) -> None:
        headers = Message()
        headers["Content-Length"] = "2"
        headers["Content-Length"] = "3"
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"{}"))

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("invalid_content_length", raised.exception.code)

    def test_unsupported_transfer_encoding_is_typed(self) -> None:
        headers = Message()
        headers["Transfer-Encoding"] = "chunked"
        headers["Content-Length"] = "0"
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"0\r\n\r\n"))

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("unsupported_transfer_encoding", raised.exception.code)
        self.assertEqual(400, ControlPlaneHttp._status_for("unsupported_transfer_encoding"))

    def test_empty_transfer_encoding_is_not_treated_as_unframed(self) -> None:
        headers = Message()
        headers["Transfer-Encoding"] = ""
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"{}"))

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("unsupported_transfer_encoding", raised.exception.code)

    def test_bounded_content_length_reads_exact_body(self) -> None:
        headers = Message()
        headers["Content-Length"] = "2"
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"{}tail"))

        self.assertEqual(b"{}", ControlPlaneHttp._read_body(handler))
        self.assertEqual(400, ControlPlaneHttp._status_for("invalid_content_length"))

    def test_truncated_content_length_is_typed(self) -> None:
        headers = Message()
        headers["Content-Length"] = "3"
        handler = SimpleNamespace(headers=headers, rfile=BytesIO(b"{}"))

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("incomplete_request_body", raised.exception.code)
        self.assertEqual(400, ControlPlaneHttp._status_for("incomplete_request_body"))

    def test_content_length_limit_remains_one_mib(self) -> None:
        headers = Message()
        headers["Content-Length"] = str(1024 * 1024 + 1)
        handler = SimpleNamespace(headers=headers, rfile=BytesIO())

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("request_too_large", raised.exception.code)

    def test_extremely_long_numeric_content_length_is_typed(self) -> None:
        headers = Message()
        headers["Content-Length"] = "9" * 5000
        handler = SimpleNamespace(headers=headers, rfile=BytesIO())

        with self.assertRaises(CoordinatorError) as raised:
            ControlPlaneHttp._read_body(handler)

        self.assertEqual("request_too_large", raised.exception.code)

    def test_legacy_command_endpoint_uses_same_framing_boundary(self) -> None:
        headers = Message()
        headers["Content-Length"] = "malformed"
        handler = CoordinatorRequestHandler.__new__(CoordinatorRequestHandler)
        handler.path = "/command"
        handler.headers = headers
        handler.rfile = BytesIO(b"{}")
        handler.server = SimpleNamespace(
            control_http=SimpleNamespace(handles=lambda _path: False),
            token="test-token",
        )
        handler._authorized = lambda: True
        handler._write_json = Mock()
        handler._write_error = Mock()

        handler.do_POST()

        handler._write_json.assert_called_once()
        status, payload = handler._write_json.call_args.args
        self.assertEqual(400, status)
        self.assertEqual("invalid_content_length", payload["error"]["code"])

    def test_legacy_command_endpoint_rejects_truncated_body(self) -> None:
        headers = Message()
        headers["Content-Length"] = "3"
        handler = CoordinatorRequestHandler.__new__(CoordinatorRequestHandler)
        handler.path = "/command"
        handler.headers = headers
        handler.rfile = BytesIO(b"{}")
        handler.server = SimpleNamespace(
            control_http=SimpleNamespace(handles=lambda _path: False),
            token="test-token",
        )
        handler._authorized = lambda: True
        handler._write_json = Mock()
        handler._write_error = Mock()

        handler.do_POST()

        handler._write_json.assert_called_once()
        status, payload = handler._write_json.call_args.args
        self.assertEqual(400, status)
        self.assertEqual("incomplete_request_body", payload["error"]["code"])

    def test_legacy_command_endpoint_types_invalid_json(self) -> None:
        headers = Message()
        headers["Content-Length"] = "1"
        handler = CoordinatorRequestHandler.__new__(CoordinatorRequestHandler)
        handler.path = "/command"
        handler.headers = headers
        handler.rfile = BytesIO(b"{")
        handler.server = SimpleNamespace(
            control_http=SimpleNamespace(handles=lambda _path: False),
            token="test-token",
        )
        handler._authorized = lambda: True
        handler._write_json = Mock()
        handler._write_error = Mock()

        handler.do_POST()

        handler._write_json.assert_called_once()
        status, payload = handler._write_json.call_args.args
        self.assertEqual(400, status)
        self.assertEqual("invalid_json", payload["error"]["code"])
        self.assertEqual("Request body must be valid JSON", payload["error"]["message"])

    def test_legacy_command_endpoint_types_invalid_utf8(self) -> None:
        headers = Message()
        headers["Content-Length"] = "1"
        handler = CoordinatorRequestHandler.__new__(CoordinatorRequestHandler)
        handler.path = "/command"
        handler.headers = headers
        handler.rfile = BytesIO(b"\xff")
        handler.server = SimpleNamespace(
            control_http=SimpleNamespace(handles=lambda _path: False),
            token="test-token",
        )
        handler._authorized = lambda: True
        handler._write_json = Mock()
        handler._write_error = Mock()

        handler.do_POST()

        handler._write_json.assert_called_once()
        status, payload = handler._write_json.call_args.args
        self.assertEqual(400, status)
        self.assertEqual("invalid_json", payload["error"]["code"])

    def test_legacy_command_endpoint_types_extremely_large_numbers(self) -> None:
        body = b'{"value":' + b"9" * 5000 + b"}"
        headers = Message()
        headers["Content-Length"] = str(len(body))
        handler = CoordinatorRequestHandler.__new__(CoordinatorRequestHandler)
        handler.path = "/command"
        handler.headers = headers
        handler.rfile = BytesIO(body)
        handler.server = SimpleNamespace(
            control_http=SimpleNamespace(handles=lambda _path: False),
            token="test-token",
        )
        handler._authorized = lambda: True
        handler._write_json = Mock()
        handler._write_error = Mock()

        handler.do_POST()

        handler._write_json.assert_called_once()
        status, payload = handler._write_json.call_args.args
        self.assertEqual(400, status)
        self.assertEqual("invalid_json", payload["error"]["code"])
        self.assertEqual("Request body must be valid JSON", payload["error"]["message"])

    def test_direct_browser_session_query_requires_origin_and_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(
                        f"{running.base_url}/control/v1/auth/session", timeout=2
                    )
                payload = json.loads(rejected.exception.read())
                rejected.exception.close()

        self.assertEqual(403, rejected.exception.code)
        self.assertEqual("origin_required", payload["error"]["code"])

    def test_codex_wake_is_runtime_authenticated_exact_and_non_blocking(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                worker = running.httpd.application.codex_worker
                deadline = time.monotonic() + 2
                while worker.snapshot()["successfulRuns"] < 1 and time.monotonic() < deadline:
                    time.sleep(0.01)
                before = int(worker.snapshot()["successfulRuns"])
                payload = json.dumps(
                    {
                        "repositoryKey": running.httpd.application.repository_identity.key,
                        "schemaVersion": 1,
                    }
                ).encode("utf-8")
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/codex-sync/wake",
                    data=payload,
                    headers={
                        "Authorization": f"Bearer {running.token}",
                        "Content-Type": "application/json",
                    },
                    method="POST",
                )
                started = time.perf_counter()
                response = urllib.request.urlopen(request, timeout=2)
                elapsed = time.perf_counter() - started
                status = response.status
                body = json.loads(response.read())
                response.close()
                self.assertEqual(202, status)
                self.assertTrue(body["data"]["queued"])
                self.assertLess(elapsed, 0.5)

                deadline = time.monotonic() + 2
                while int(worker.snapshot()["successfulRuns"]) <= before and time.monotonic() < deadline:
                    time.sleep(0.01)
                self.assertGreater(int(worker.snapshot()["successfulRuns"]), before)

                unauthenticated = urllib.request.Request(
                    f"{running.base_url}/control/v1/codex-sync/wake",
                    data=payload,
                    headers={"Content-Type": "application/json", "Origin": running.base_url},
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as denied:
                    urllib.request.urlopen(unauthenticated, timeout=2)
                self.assertEqual(401, denied.exception.code)
                denied.exception.close()

                mismatched = urllib.request.Request(
                    f"{running.base_url}/control/v1/codex-sync/wake",
                    data=json.dumps(
                        {"repositoryKey": "0" * 64, "schemaVersion": 1}
                    ).encode("utf-8"),
                    headers={
                        "Authorization": f"Bearer {running.token}",
                        "Content-Type": "application/json",
                    },
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(mismatched, timeout=2)
                self.assertEqual(409, rejected.exception.code)
                rejected.exception.close()

            self.assertFalse(worker.is_alive())

    def test_ui_assets_are_served_without_exposing_api_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            dist = repo / "tools" / "session_coordinator" / "web" / "dist"
            dist.mkdir(parents=True)
            (dist / "index.html").write_text("<main>control console</main>", encoding="utf-8")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                page = urllib.request.urlopen(f"{running.base_url}/ui/workflows/run-a", timeout=2)
                self.assertEqual("no-store", page.headers["Cache-Control"])
                self.assertIn(b"control console", page.read())
                page.close()

                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(
                        f"{running.base_url}/control/v1/not-a-page", timeout=2
                    )
                self.assertEqual(403, rejected.exception.code)
                self.assertEqual(
                    "application/json; charset=utf-8",
                    rejected.exception.headers["Content-Type"],
                )
                rejected.exception.close()

    def test_root_redirects_to_the_control_console(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            dist = repo / "tools" / "session_coordinator" / "web" / "dist"
            dist.mkdir(parents=True)
            (dist / "index.html").write_text("<main>control console</main>", encoding="utf-8")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                page = urllib.request.urlopen(f"{running.base_url}/", timeout=2)
                self.assertEqual(f"{running.base_url}/ui/", page.url)
                self.assertIn(b"control console", page.read())
                page.close()

    def test_loopback_read_projection_does_not_require_browser_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                client.command("session.register", {"session_id": "session-a"})
                snapshot_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/snapshot",
                    headers={"Origin": running.base_url},
                )
                snapshot = json.loads(urllib.request.urlopen(snapshot_request, timeout=2).read())
                logs_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/logs?limit=1",
                    headers={"Origin": running.base_url},
                )
                logs = json.loads(urllib.request.urlopen(logs_request, timeout=2).read())
                details = {}
                for name, path in {
                    "failures": "/control/v1/failures",
                    "failureHistory": "/control/v1/failures/history?limit=10",
                    "git": "/control/v1/git",
                    "codex": "/control/v1/codex-sessions",
                    "validation": "/control/v1/validation",
                    "validationHistory": "/control/v1/validation/history?limit=10",
                    "continuations": "/control/v1/continuations",
                }.items():
                    detail_request = urllib.request.Request(
                        f"{running.base_url}{path}",
                        headers={"Origin": running.base_url},
                    )
                    details[name] = json.loads(
                        urllib.request.urlopen(detail_request, timeout=2).read()
                    )
                auth_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/auth/session",
                    headers={"Origin": running.base_url},
                )
                auth = json.loads(urllib.request.urlopen(auth_request, timeout=2).read())
                catalog_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/actions/catalog",
                    headers={"Origin": running.base_url},
                )
                catalog = json.loads(urllib.request.urlopen(catalog_request, timeout=2).read())
                mutation_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/actions/preview",
                    data=json.dumps(
                        {
                            "kind": ActionKind.SESSION_HEARTBEAT.value,
                            "parameters": {"sessionId": "session-a"},
                        }
                    ).encode("utf-8"),
                    headers={
                        "Content-Type": "application/json",
                        "Origin": running.base_url,
                    },
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected_mutation:
                    urllib.request.urlopen(mutation_request, timeout=2)

            self.assertTrue(snapshot["ok"])
            self.assertEqual("session-a", snapshot["data"]["sessions"][0]["sessionId"])
            self.assertTrue(logs["ok"])
            self.assertTrue(all(detail["ok"] for detail in details.values()))
            self.assertEqual("loopback-viewer", auth["data"]["actor"])
            self.assertEqual("observer", auth["data"]["role"])
            self.assertFalse(auth["data"]["mutationEnabled"])
            self.assertTrue(catalog["data"]["actions"])
            self.assertEqual(401, rejected_mutation.exception.code)
            rejected_mutation.exception.close()
            self.assertNotIn("token", json.dumps(snapshot).lower())

    def test_loopback_console_can_start_whitelisted_validation_without_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                started = SimpleNamespace(
                    to_dict=lambda: {"actionId": "queued-validation", "status": "executing"}
                )
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/validation/queue/start",
                    data=json.dumps(
                        {
                            "sessionId": "session-a",
                            "template": "web-check",
                            "runId": "run-a",
                            "milestoneId": "M1",
                        }
                    ).encode("utf-8"),
                    headers={
                        "Content-Type": "application/json",
                        "Origin": running.base_url,
                    },
                    method="POST",
                )
                with patch.object(
                    running.httpd.control_http.router.actions,
                    "start_loopback_validation",
                    return_value=started,
                ) as launch:
                    response = json.loads(urllib.request.urlopen(request, timeout=2).read())

                context, parameters = launch.call_args.args

            self.assertTrue(response["ok"])
            self.assertEqual("queued-validation", response["data"]["action"]["actionId"])
            self.assertEqual(WebControlRole.OPERATOR, context.role)
            self.assertEqual("loopback-console", context.actor)
            self.assertIsNone(context.web_session_id)
            self.assertEqual("session-a", context.bound_session_id)
            self.assertEqual("web-check", parameters["template"])

    def test_loopback_console_advances_next_validation_ticket_without_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/validation/queue/continue",
                    data=b"{}",
                    headers={
                        "Content-Type": "application/json",
                        "Origin": running.base_url,
                    },
                    method="POST",
                )
                selected = {
                    "ticket": {
                        "ticketId": "ticket-next",
                        "sessionId": "session-a",
                        "status": "materializing",
                    },
                    "progress": {"materializing": 1},
                }
                with patch.object(
                    running.httpd.control_http.router,
                    "validation_queue_continue",
                    return_value=selected,
                    create=True,
                ) as advance:
                    response = json.loads(urllib.request.urlopen(request, timeout=2).read())

            advance.assert_called_once_with()
        self.assertTrue(response["ok"])
        self.assertEqual("ticket-next", response["data"]["ticket"]["ticketId"])
        self.assertEqual("materializing", response["data"]["ticket"]["status"])

    def test_manual_queue_advance_uses_the_shared_non_reentrant_worker_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                selected = SimpleNamespace(
                    ticket_id="ticket-next",
                    session_id="session-a",
                    plan_path="docs/plans/tooling/01.md",
                    status="materializing",
                )
                tickets = SimpleNamespace(
                    active_ticket=Mock(side_effect=[None, selected]),
                    get=Mock(),
                )
                application.validation_ticket_worker = SimpleNamespace(
                    tickets=tickets,
                    tick=Mock(return_value={"materializing": 1}),
                )

                with patch.object(
                    application.supervision, "require_mutation_allowed"
                ) as require_admission:
                    # The Codex worker may check admission while this global mock is installed.
                    self.assertTrue(application._codex_sync_writable())
                    result = application.advance_validation_queue(
                        actor="loopback-console", require_admission=True
                    )
                    self.assertTrue(
                        application._validation_ticket_tick_lock.acquire(False)
                    )
                    try:
                        with self.assertRaises(CoordinatorError) as rejected:
                            application.advance_validation_queue(
                                actor="loopback-console", require_admission=True
                            )
                    finally:
                        application._validation_ticket_tick_lock.release()

            admission_operations = [item.args for item in require_admission.call_args_list]
            self.assertEqual(
                2,
                admission_operations.count(("validation.queue_continue",)),
            )
            self.assertIn(("codex.sessions.reconcile",), admission_operations)
            self.assertEqual("validation_queue_busy", rejected.exception.code)
            application.validation_ticket_worker.tick.assert_called_once_with()
            self.assertEqual("ticket-next", result["ticket"]["ticketId"])
            self.assertEqual({"materializing": 1}, result["progress"])

    def test_loopback_validation_start_still_requires_same_origin(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/validation/queue/start",
                    data=b"{}",
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(request, timeout=2)

            self.assertEqual(403, rejected.exception.code)
            rejected.exception.close()

    def test_runtime_descriptor_exposes_instance_and_api_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config):
                runtime = json.loads(config.runtime_path.read_text(encoding="utf-8"))
            self.assertEqual([1], runtime["control_api_versions"])
            self.assertTrue(runtime["instance_id"])
            self.assertTrue(runtime["started_at"])

    def test_action_activity_restores_only_the_current_browser_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                client.command("session.register", {"session_id": "session-a"})
                client.command("session.register", {"session_id": "session-b"})
                ticket = client.issue_ui_ticket(actor="browser-a")
                cookie_jar = http.cookiejar.CookieJar()
                opener = urllib.request.build_opener(
                    urllib.request.HTTPCookieProcessor(cookie_jar),
                    _NoRedirectHandler(),
                )
                opener.open(f"{running.base_url}{ticket['bootstrapPath']}", timeout=2).close()
                grant = client.issue_elevation_grant(
                    actor="browser-a", role="operator", session_id="session-a"
                )
                elevate = urllib.request.Request(
                    f"{running.base_url}/control/v1/auth/elevate",
                    data=json.dumps({"grant": grant["grant"]}).encode("utf-8"),
                    headers={
                        "Content-Type": "application/json",
                        "Origin": running.base_url,
                    },
                    method="POST",
                )
                csrf = json.loads(opener.open(elevate, timeout=2).read())["data"]["csrfToken"]

                def preview() -> dict[str, object]:
                    request = urllib.request.Request(
                        f"{running.base_url}/control/v1/actions/preview",
                        data=json.dumps(
                            {
                                "kind": ActionKind.SESSION_HEARTBEAT.value,
                                "parameters": {"sessionId": "session-a"},
                            }
                        ).encode("utf-8"),
                        headers={
                            "Content-Type": "application/json",
                            "Origin": running.base_url,
                            "X-CSRF-Token": csrf,
                        },
                        method="POST",
                    )
                    return json.loads(opener.open(request, timeout=2).read())["data"]["action"]

                first = preview()
                second = preview()
                running.httpd.control_http.router.actions.preview(
                    ActionContext(
                        actor="foreign-browser",
                        role=WebControlRole.OPERATOR,
                        web_session_id="foreign-web",
                        bound_session_id="session-b",
                        daemon_instance_id=running.instance_id,
                    ),
                    ActionKind.SESSION_HEARTBEAT.value,
                    {"sessionId": "session-b"},
                )

                activity_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/actions?limit=1",
                    headers={"Origin": running.base_url},
                )
                activity = json.loads(opener.open(activity_request, timeout=2).read())["data"]
                detail_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/actions/{first['actionId']}",
                    headers={"Origin": running.base_url},
                )
                detail = json.loads(opener.open(detail_request, timeout=2).read())["data"]
                invalid_limit_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/actions?limit=101",
                    headers={"Origin": running.base_url},
                )
                with self.assertRaises(urllib.error.HTTPError) as invalid_limit:
                    opener.open(invalid_limit_request, timeout=2)
                invalid_limit_body = json.loads(invalid_limit.exception.read())
                invalid_limit.exception.close()

            self.assertEqual([second["actionId"]], [item["actionId"] for item in activity["actions"]])
            self.assertTrue(activity["truncated"])
            self.assertNotIn("confirmationPhrase", activity["actions"][0])
            self.assertEqual(first["actionId"], detail["action"]["actionId"])
            self.assertIn("confirmationPhrase", detail["action"])
            self.assertEqual(400, invalid_limit.exception.code)
            self.assertEqual("action_limit_invalid", invalid_limit_body["error"]["code"])

    def test_log_range_is_bounded_and_cursor_based(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                for suffix in ("a", "b", "c"):
                    client.command("session.register", {"session_id": f"session-{suffix}"})
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/logs?limit=2",
                    headers={"Authorization": f"Bearer {running.token}"},
                )
                payload = json.loads(urllib.request.urlopen(request, timeout=2).read())["data"]
                self.assertEqual(2, len(payload["events"]))
                self.assertTrue(payload["truncated"])
                before = payload["events"][0]["eventId"]
                older = urllib.request.Request(
                    f"{running.base_url}/control/v1/logs?limit=2&before={before}",
                    headers={"Authorization": f"Bearer {running.token}"},
                )
                older_payload = json.loads(urllib.request.urlopen(older, timeout=2).read())["data"]
                self.assertTrue(all(event["eventId"] < before for event in older_payload["events"]))

    def test_log_range_projects_legacy_oversized_event_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                marker = "must-not-cross-log-boundary"
                oversized = json.dumps({"value": marker * 1024})
                with Database(config.database_path).transaction() as connection:
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) VALUES (?, ?, 'now')",
                        ("legacy.oversized", oversized),
                    )
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/logs?limit=500",
                    headers={"Authorization": f"Bearer {running.token}"},
                )

                payload = json.loads(urllib.request.urlopen(request, timeout=2).read())["data"]

            event = next(
                item for item in payload["events"] if item["type"] == "legacy.oversized"
            )
            self.assertEqual(True, event["payload"]["truncated"])
            self.assertGreater(event["payload"]["originalBytes"], 16 * 1024)
            self.assertNotIn(marker, json.dumps(payload))

    def test_malicious_host_is_rejected_before_authentication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                request = urllib.request.Request(
                    f"{running.base_url}/control/v1/meta",
                    headers={"Host": "example.com"},
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(request, timeout=2)
                self.assertEqual(403, rejected.exception.code)
                body = rejected.exception.read().decode("utf-8")
                self.assertNotIn("Traceback", body)
                self.assertNotIn(running.token, body)
                rejected.exception.close()

    def test_artifact_http_requires_browser_origin_but_not_a_cookie(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                without_origin = urllib.request.Request(
                    f"{running.base_url}/control/v1/artifacts/opaque"
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected_origin:
                    urllib.request.urlopen(without_origin, timeout=2)
                self.assertEqual(403, rejected_origin.exception.code)
                rejected_origin.exception.close()
                without_session = urllib.request.Request(
                    f"{running.base_url}/control/v1/artifacts/opaque",
                    headers={"Origin": running.base_url},
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected_missing:
                    urllib.request.urlopen(without_session, timeout=2)
                self.assertEqual(404, rejected_missing.exception.code)
                rejected_missing.exception.close()

    def test_non_loopback_bind_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", host="0.0.0.0", port=0
            )

            with self.assertRaisesRegex(Exception, "loopback"):
                RunningCoordinator.start(config)

    def test_all_control_methods_use_sanitized_v1_envelopes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                for method in ("PUT", "PATCH", "DELETE", "OPTIONS"):
                    with self.subTest(method=method):
                        request = urllib.request.Request(
                            f"{running.base_url}/control/v1/meta",
                            method=method,
                            headers={"Authorization": f"Bearer {running.token}"},
                        )
                        with self.assertRaises(urllib.error.HTTPError) as rejected:
                            urllib.request.urlopen(request, timeout=2)
                        body = json.loads(rejected.exception.read())
                        self.assertEqual("application/json; charset=utf-8", rejected.exception.headers["Content-Type"])
                        self.assertFalse(body["ok"])
                        self.assertEqual(1, body["meta"]["apiVersion"])
                        rejected.exception.close()

                head = urllib.request.Request(
                    f"{running.base_url}/control/v1/meta",
                    method="HEAD",
                    headers={"Authorization": f"Bearer {running.token}"},
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected_head:
                    urllib.request.urlopen(head, timeout=2)
                self.assertEqual(b"", rejected_head.exception.read())
                self.assertEqual(
                    "application/json; charset=utf-8",
                    rejected_head.exception.headers["Content-Type"],
                )
                rejected_head.exception.close()

    def test_windows_sse_disconnect_is_a_normal_transport_close(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                control = running.httpd.control_http
                handler = SimpleNamespace(
                    headers={
                        "Host": f"127.0.0.1:{running.httpd.server_address[1]}",
                        "Authorization": f"Bearer {running.token}",
                    },
                    server=running.httpd,
                    path="/control/v1/events/stream",
                    command="GET",
                )
                with patch.object(
                    control,
                    "_stream_events",
                    side_effect=ConnectionAbortedError(10053, "client disconnected"),
                ), patch.object(control, "_write_response") as write_response:
                    control.handle(handler)
                write_response.assert_not_called()


class _NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def http_error_303(self, request, response, code, message, headers):
        return response


if __name__ == "__main__":
    unittest.main()
