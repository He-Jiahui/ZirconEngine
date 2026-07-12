from __future__ import annotations

import http.cookiejar
import json
import tempfile
import time
import unittest
import urllib.error
import urllib.request
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.database import Database
from tools.session_coordinator.models import WebControlRole
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class ControlHttpTests(unittest.TestCase):
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

    def test_observer_bootstrap_opens_cookie_snapshot_without_bearer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                client.command("session.register", {"session_id": "session-a"})
                ticket = client.issue_ui_ticket(actor="test")
                cookie_jar = http.cookiejar.CookieJar()
                opener = urllib.request.build_opener(
                    urllib.request.HTTPCookieProcessor(cookie_jar),
                    _NoRedirectHandler(),
                )
                bootstrap = opener.open(
                    f"{running.base_url}{ticket['bootstrapPath']}", timeout=2
                )
                self.assertEqual(303, bootstrap.status)
                bootstrap.close()

                snapshot_request = urllib.request.Request(
                    f"{running.base_url}/control/v1/snapshot",
                    headers={"Origin": running.base_url},
                )
                snapshot = json.loads(opener.open(snapshot_request, timeout=2).read())

            self.assertTrue(snapshot["ok"])
            self.assertEqual("session-a", snapshot["data"]["sessions"][0]["sessionId"])
            self.assertNotIn("token", json.dumps(snapshot).lower())

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
                    f"{running.base_url}/control/v1/logs?limit=1",
                    headers={"Authorization": f"Bearer {running.token}"},
                )

                payload = json.loads(urllib.request.urlopen(request, timeout=2).read())["data"]

            event = payload["events"][0]
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

    def test_artifact_http_requires_browser_origin_and_session(self) -> None:
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
                with self.assertRaises(urllib.error.HTTPError) as rejected_session:
                    urllib.request.urlopen(without_session, timeout=2)
                self.assertEqual(401, rejected_session.exception.code)
                rejected_session.exception.close()

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
