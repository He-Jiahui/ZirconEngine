from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.helpers import init_repo


class SnapshotTests(unittest.TestCase):
    def test_objects_deduplicate_and_restore_is_preview_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            SessionService(database, repo).register(session_id="session-a")
            epoch = BaselineService(database, repo).initialize()
            store = ObjectStore(database, config.object_root)
            snapshots = SnapshotService(database, repo, store)

            first_hash = store.put(b"same content")
            second_hash = store.put(b"same content")
            snapshot = snapshots.create(
                session_id="session-a",
                paths=["README.md"],
                baseline_epoch=epoch.epoch_id,
                purpose="before patch",
            )
            (repo / "README.md").write_text("changed\n", encoding="utf-8")
            preview = snapshots.restore_preview(snapshot.snapshot_id)

            self.assertEqual(first_hash, second_hash)
            self.assertEqual(b"same content", store.get(first_hash))
            self.assertEqual("README.md", preview[0].path)
            self.assertTrue(preview[0].would_change)
            self.assertEqual("changed\n", (repo / "README.md").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
