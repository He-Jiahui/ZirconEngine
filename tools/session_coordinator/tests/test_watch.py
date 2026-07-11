from __future__ import annotations

import tempfile
import threading
import unittest
from pathlib import Path
from types import SimpleNamespace

from tools.session_coordinator.baselines import BaselineHealth, BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.server import RunningCoordinator
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

    def test_stale_prepared_scan_is_discarded_after_epoch_changes(self) -> None:
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

            observation = watcher.prepare_scan()
            replacement = baselines.accept(reason="concurrent baseline transition")
            result = watcher.apply_scan(observation)

            self.assertFalse(result.applied)
            self.assertEqual(replacement.epoch_id, baselines.current().epoch_id)
            self.assertEqual(BaselineHealth.HEALTHY, baselines.current().health)

    def test_maintenance_hashing_does_not_hold_global_mutation_lock(self) -> None:
        prepare_started = threading.Event()
        release_prepare = threading.Event()

        class BlockingWatcher:
            def scan_once(self):
                prepare_started.set()
                release_prepare.wait(timeout=2)
                return []

            def prepare_scan(self):
                prepare_started.set()
                release_prepare.wait(timeout=2)
                return object()

            def apply_scan(self, _observation):
                return SimpleNamespace(applied=True, changes=())

        application = SimpleNamespace(
            _mutation_lock=threading.RLock(),
            watcher=BlockingWatcher(),
            cargo_jobs=None,
            workspace_copy=None,
            read_only=True,
        )
        stop = threading.Event()
        worker = threading.Thread(
            target=RunningCoordinator._maintenance_loop,
            args=(application, 0.01, 60.0, stop),
            daemon=True,
        )
        worker.start()
        self.assertTrue(prepare_started.wait(timeout=1))

        acquired = application._mutation_lock.acquire(timeout=0.1)
        if acquired:
            application._mutation_lock.release()
        release_prepare.set()
        stop.set()
        worker.join(timeout=2)

        self.assertTrue(acquired, "workspace hashing monopolized the mutation lock")
        self.assertFalse(worker.is_alive())


if __name__ == "__main__":
    unittest.main()
