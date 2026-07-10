from __future__ import annotations

import json
import subprocess
import time
import tempfile
import unittest
import urllib.error
import urllib.request
from pathlib import Path

from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.tests.helpers import init_repo


class ServerTests(unittest.TestCase):
    def test_second_instance_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                with self.assertRaises(CoordinatorError) as duplicate:
                    RunningCoordinator.start(config)
            self.assertEqual("already_running", duplicate.exception.code)

    def test_health_and_session_commands_require_runtime_token(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                health = client.health()
                registered = client.command("session.register", {"session_id": "session-a"})
                active = client.command(
                    "session.set_status", {"session_id": "session-a", "status": "active"}
                )

                self.assertEqual("ok", health["status"])
                self.assertEqual("registered", registered["session"]["status"])
                self.assertEqual("active", active["session"]["status"])

                request = urllib.request.Request(
                    f"{running.base_url}/command",
                    data=json.dumps({"command": "session.list", "arguments": {}}).encode("utf-8"),
                    headers={"Content-Type": "application/json", "Authorization": "Bearer wrong"},
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as unauthorized:
                    urllib.request.urlopen(request, timeout=2)
                self.assertEqual(401, unauthorized.exception.code)
                unauthorized.exception.close()

    def test_stale_runtime_descriptor_is_reported_as_offline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            config.state_root.mkdir(parents=True)
            config.runtime_path.write_text(
                json.dumps({"host": "127.0.0.1", "port": 1, "token": "stale", "pid": 999999}),
                encoding="utf-8",
            )

            with self.assertRaises(CoordinatorClientError) as offline:
                CoordinatorClient.from_runtime(config).health()
            self.assertEqual("offline", offline.exception.code)

    def test_non_main_checkout_is_read_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            subprocess.run(["git", "switch", "-q", "-c", "temporary-test"], cwd=repo, check=True)
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                self.assertEqual("read_only", client.health()["mode"])
                with self.assertRaises(CoordinatorClientError) as rejected:
                    client.command("session.register", {"session_id": "session-a"})
            self.assertEqual("not_on_main", rejected.exception.code)

    def test_background_watcher_marks_external_drift_degraded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                (repo / "README.md").write_text("external\n", encoding="utf-8")
                health = "healthy"
                for _ in range(50):
                    health = client.command("baseline.status")["baseline"]["health"]
                    if health == "degraded":
                        break
                    time.sleep(0.05)
            self.assertEqual("degraded", health)


if __name__ == "__main__":
    unittest.main()
