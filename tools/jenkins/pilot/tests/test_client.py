from __future__ import annotations

import hashlib
import http.server
import json
import socket
import tempfile
import threading
import unittest
from pathlib import Path

from tools.jenkins.pilot.client import JenkinsClient, JenkinsTransportError
from tools.jenkins.pilot.contracts import PilotError, RequestIdentity
from tools.jenkins.pilot.journal import SubmissionJournal
from tools.jenkins.pilot.tests.fixtures import temporary_directory, sealed_bundle


class ClientTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = temporary_directory(prefix="jenkins-http-")
        self.addCleanup(self.directory.cleanup)
        root = Path(self.directory.name)
        self.bundle, input_hash = sealed_bundle(root)
        self.journal = SubmissionJournal(root / "submissions.sqlite3")
        self.identity = RequestIdentity("session", "request", "attempt", 1, input_hash, "python-static")
        self.builds: list[dict] = []
        self.queue: list[dict] = []
        self.posts = 0
        self.drop_response = False
        self.foreign_redirect = False
        self.cancelled_item = None
        owner = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_GET(self):
                if self.headers.get("Authorization") != "Basic dXNlcjpwYXNzd29yZA==":
                    self.send_error(401)
                    return
                if self.path.startswith("/crumbIssuer/"):
                    self.send_response(200)
                    self.send_header("Set-Cookie", "JSESSIONID=bound; Path=/")
                    self.end_headers()
                    self.wfile.write(json.dumps({"crumbRequestField": "Jenkins-Crumb", "crumb": "crumb"}).encode())
                    return
                if self.path.startswith("/queue/item/7/"):
                    item = owner.cancelled_item or next((q for q in owner.queue if q["id"] == 7), None)
                    if item is None:
                        self.send_error(404)
                        return
                    data = item
                elif self.path.startswith("/queue/"):
                    data = {"items": owner.queue}
                elif self.path.startswith("/job/pilot/3/"):
                    data = owner.builds[0]
                elif self.path.startswith("/job/pilot/"):
                    data = {"builds": owner.builds}
                else:
                    data = {"mode": "NORMAL"}
                self.send_response(200)
                self.end_headers()
                self.wfile.write(json.dumps(data).encode())

            def do_POST(self):
                if self.headers.get("Cookie") != "JSESSIONID=bound" or self.headers.get("Jenkins-Crumb") != "crumb":
                    self.send_error(403)
                    return
                owner.posts += 1
                self.rfile.read(int(self.headers.get("Content-Length", "0")))
                if self.path.startswith("/queue/cancelItem"):
                    owner.cancelled_item = {**owner.queue[0], "cancelled": True}
                    owner.queue = []
                    self.send_response(200)
                    self.end_headers()
                    return
                owner.builds = [owner.build_record()]
                if owner.drop_response:
                    self.connection.shutdown(socket.SHUT_RDWR)
                    self.connection.close()
                    return
                self.send_response(201)
                location = "http://foreign.invalid/queue/item/7/" if owner.foreign_redirect else f"{owner.client.url}/queue/item/7/"
                self.send_header("Location", location)
                self.end_headers()

        self.server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.addCleanup(self.server.server_close)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.server.shutdown)
        self.client = JenkinsClient(f"http://127.0.0.1:{self.server.server_port}", "user", "password", timeout=2)

    def build_record(self) -> dict:
        parameters = {**self.identity.parameters(), "BUNDLE_HASH": hashlib.sha256(self.bundle.read_bytes()).hexdigest()}
        return {"number": 3, "building": False, "result": "SUCCESS", "actions": [
            {"parameters": [{"name": key, "value": value} for key, value in parameters.items()]}
        ]}

    def test_dropped_post_response_reconciles_exact_build_without_resubmission(self) -> None:
        self.drop_response = True
        first = self.client.submit("pilot", self.identity, self.bundle, self.journal)
        self.assertEqual(first["state"], "uncertain")
        second = self.client.submit("pilot", self.identity, self.bundle, self.journal)
        self.assertEqual(second["state"], "completed")
        self.assertEqual(second["build_number"], 3)
        self.assertEqual(self.posts, 1)

    def test_missing_observation_preserves_pending_request(self) -> None:
        self.journal.reserve(self.identity, hashlib.sha256(self.bundle.read_bytes()).hexdigest(), "pilot")
        self.journal.update(self.identity, "uncertain")
        result = self.client.submit("pilot", self.identity, self.bundle, self.journal)
        self.assertEqual(result["state"], "uncertain")
        self.assertEqual(self.posts, 0)

    def test_two_matching_builds_are_rejected(self) -> None:
        self.journal.reserve(self.identity, hashlib.sha256(self.bundle.read_bytes()).hexdigest(), "pilot")
        self.builds = [self.build_record(), {**self.build_record(), "number": 4}]
        with self.assertRaises(PilotError):
            self.client.reconcile("pilot", self.identity, self.journal)

    def test_older_generation_cannot_satisfy_current_request(self) -> None:
        self.journal.reserve(self.identity, hashlib.sha256(self.bundle.read_bytes()).hexdigest(), "pilot")
        self.builds = [self.build_record()]
        parameter = next(p for p in self.builds[0]["actions"][0]["parameters"] if p["name"] == "GENERATION")
        parameter["value"] = "0"
        with self.assertRaises(PilotError):
            self.client.reconcile("pilot", self.identity, self.journal)

    def test_foreign_queue_redirect_is_rejected_without_forwarding_credentials(self) -> None:
        self.foreign_redirect = True
        with self.assertRaises(PilotError):
            self.client.submit("pilot", self.identity, self.bundle, self.journal)
        self.assertEqual(self.posts, 1)

    def test_remote_controller_is_outside_isolated_pilot(self) -> None:
        for url in ["http://example.com:8080", "http://127.0.0.1:8080@evil.invalid", "http://127.0.0.1:8080/path"]:
            with self.assertRaises(PilotError):
                JenkinsClient(url, "user", "password")

    def test_normal_post_binds_exact_queue_identity(self) -> None:
        result = self.client.submit("pilot", self.identity, self.bundle, self.journal)
        self.assertEqual(result["queue_id"], 7)
        self.assertEqual(result["state"], "queued")

    def test_cancel_uses_bound_queue_terminal_proof_after_live_list_removal(self):
        self.journal.reserve(self.identity, hashlib.sha256(self.bundle.read_bytes()).hexdigest(), "pilot")
        self.journal.update(self.identity, "queued", queue_id=7)
        self.queue = [{"id": 7, "cancelled": False, "task": {"name": "pilot"}, "actions": self.build_record()["actions"]}]
        result = self.client.cancel("pilot", self.identity, self.journal)
        self.assertEqual(result["state"], "cancelled")
        self.assertEqual(result["result"], "CANCELLED")
        self.assertEqual(self.queue, [])


if __name__ == "__main__":
    unittest.main()
