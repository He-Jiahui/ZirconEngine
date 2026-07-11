from __future__ import annotations

import json
import os
import subprocess
import time
import tempfile
import unittest
import urllib.error
import urllib.request
from pathlib import Path
from unittest import mock

from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.server import CoordinatorApplication, RunningCoordinator
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture


class ServerTests(unittest.TestCase):
    def test_maintenance_requires_separate_local_capability(self) -> None:
        with mock.patch.dict("os.environ", {}, clear=True):
            with self.assertRaises(CoordinatorError) as rejected:
                CoordinatorApplication._authorize_maintenance({"maintenance": True})
        self.assertEqual("maintenance_unauthorized", rejected.exception.code)

        with mock.patch.dict(
            "os.environ", {"ZIRCON_COORDINATOR_MAINTENANCE_TOKEN": "local-only"}
        ):
            self.assertTrue(
                CoordinatorApplication._authorize_maintenance(
                    {"maintenance": True, "maintenance_capability": "local-only"}
                )
            )

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
                client.command("baseline.init")
                (repo / "README.md").write_text("external\n", encoding="utf-8")
                health = "healthy"
                for _ in range(200):
                    health = client.command("baseline.status")["baseline"]["health"]
                    if health == "degraded":
                        break
                    time.sleep(0.05)
            self.assertEqual("degraded", health)

    def test_daemon_runs_retention_maintenance_without_external_scheduler(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                tick_count = 0
                for _ in range(200):
                    with Database(config.database_path).connect() as connection:
                        tick_count = int(
                            connection.execute(
                                "SELECT COUNT(*) FROM maintenance_ticks WHERE status = 'succeeded'"
                            ).fetchone()[0]
                        )
                    if tick_count:
                        break
                    time.sleep(0.05)

            self.assertGreaterEqual(tick_count, 1)

    def test_daemon_periodically_imports_and_archives_inactive_root_note(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            session_root = repo / ".codex/sessions"
            session_root.mkdir(parents=True)
            note = session_root / "old.md"
            note.write_text(
                "---\nsession: old\nstatus: stale\n---\n\n# Old\n",
                encoding="utf-8",
            )
            old_time = time.time() - 2 * 86400
            os.utime(note, (old_time, old_time))
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                archived = session_root / "archive/old.md"
                for _ in range(100):
                    if archived.exists():
                        break
                    time.sleep(0.02)

            self.assertTrue(archived.exists())
            self.assertFalse(note.exists())

    def test_daemon_never_stales_or_archives_live_pid_root_note(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            session_root = repo / ".codex/sessions"
            session_root.mkdir(parents=True)
            note = session_root / "live.md"
            note.write_text(
                f"---\nsession: live\nstatus: completed\npid: {os.getpid()}\n---\n",
                encoding="utf-8",
            )
            old_time = time.time() - 2 * 86400
            os.utime(note, (old_time, old_time))
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                status = None
                for _ in range(100):
                    with Database(config.database_path).connect() as connection:
                        row = connection.execute(
                            "SELECT status FROM sessions WHERE session_id = 'live'"
                        ).fetchone()
                    if row is not None:
                        status = row[0]
                        break
                    time.sleep(0.02)

            self.assertTrue(note.exists())
            self.assertEqual("active", status)

    def test_destructive_legacy_import_requires_operator_capability(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            note_root = repo / ".codex/sessions"
            note_root.mkdir(parents=True)
            (note_root / "legacy.md").write_text(
                "---\nsession: legacy\nstatus: stale\n---\n",
                encoding="utf-8",
            )
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )

            with mock.patch.dict("os.environ", {}, clear=True):
                with self.assertRaises(CoordinatorError) as rejected:
                    application.command("legacy.import", {"apply": True})

            self.assertEqual("maintenance_unauthorized", rejected.exception.code)

    def test_registration_prioritizes_open_failure_for_numbered_plan(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/editor/01-editor.md")
            fixing = fixture.add_plan("docs/plans/runtime/02-runtime.md")
            fixture.add_handoff(origin, fixing, "provider")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                result = CoordinatorClient.from_runtime(config).command(
                    "session.register",
                    {
                        "session_id": "session-a",
                        "plan_path": fixing.path.relative_to(repo).as_posix(),
                    },
                )

            self.assertEqual("resolving_failure", result["session"]["status"])
            self.assertEqual(["provider"], [item["summary_slug"] for item in result["open_failures"]])


if __name__ == "__main__":
    unittest.main()
