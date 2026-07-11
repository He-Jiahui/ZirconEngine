from __future__ import annotations

import http.cookiejar
import json
import tempfile
import unittest
import urllib.error
import urllib.request
from pathlib import Path

from tools.session_coordinator.client import CoordinatorClient
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.helpers import init_repo


class ControlHttpTests(unittest.TestCase):
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


class _NoRedirectHandler(urllib.request.HTTPRedirectHandler):
    def http_error_303(self, request, response, code, message, headers):
        return response


if __name__ == "__main__":
    unittest.main()
