from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineHealth, BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class BaselineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.service = BaselineService(self.database, self.repo)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_external_change_degrades_then_accept_opens_new_healthy_epoch(self) -> None:
        first = self.service.initialize()
        (self.repo / "README.md").write_text("external\n", encoding="utf-8")

        changes = self.service.scan()
        degraded = self.service.current()
        self.service.attribute("session-a", ["README.md"])
        accepted = self.service.accept(reason="attribute known Session change")

        self.assertEqual(BaselineHealth.HEALTHY, first.health)
        self.assertEqual(["README.md"], [change.path for change in changes])
        self.assertEqual(BaselineHealth.DEGRADED, degraded.health)
        self.assertGreater(accepted.epoch_id, first.epoch_id)
        self.assertEqual(BaselineHealth.HEALTHY, accepted.health)

    def test_head_change_creates_new_epoch_without_losing_manifest(self) -> None:
        first = self.service.initialize()
        (self.repo / "second.txt").write_text("second\n", encoding="utf-8")
        subprocess.run(["git", "add", "second.txt"], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: second"], cwd=self.repo, check=True)

        second = self.service.refresh_for_head_change()

        self.assertGreater(second.epoch_id, first.epoch_id)
        self.assertIn("second.txt", second.manifest)
        self.assertEqual(BaselineHealth.HEALTHY, second.health)


if __name__ == "__main__":
    unittest.main()
