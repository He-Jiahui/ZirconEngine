from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineHealth, BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.watch import WorkspaceWatcher


class WatcherTests(unittest.TestCase):
    def test_external_edit_is_preserved_and_marks_baseline_degraded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            baselines = BaselineService(database, repo)
            baselines.initialize()
            watcher = WorkspaceWatcher(baselines)
            (repo / "README.md").write_text("external\n", encoding="utf-8")

            changes = watcher.scan_once()

            self.assertEqual(["README.md"], [change.path for change in changes])
            self.assertEqual(BaselineHealth.DEGRADED, baselines.current().health)
            self.assertEqual("external\n", (repo / "README.md").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
