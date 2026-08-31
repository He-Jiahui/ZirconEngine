from __future__ import annotations

import tempfile
import unittest
from unittest import mock
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.helpers import init_repo


class SnapshotTests(unittest.TestCase):
    def test_reconcile_removes_dead_writer_temporary_and_keeps_known_object(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            store = ObjectStore(database, config.object_root)
            object_hash = store.put(b"known object")
            known_path = store.path_for_hash(object_hash)
            temporary = known_path.with_suffix(".tmp-2147483647-123")
            temporary.write_bytes(b"crash residue")

            with mock.patch(
                "tools.session_coordinator.snapshots.process_is_alive",
                return_value=False,
            ):
                removed = store.reconcile_orphan_files()

            self.assertEqual(1, removed)
            self.assertFalse(temporary.exists())
            self.assertEqual(b"known object", store.get(object_hash))

    def test_reconcile_preserves_a_live_writer_temporary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            store = ObjectStore(database, config.object_root)
            object_hash = "a" * 64
            temporary = store.path_for_hash(object_hash).with_suffix(
                ".tmp-1234-5678"
            )
            temporary.parent.mkdir(parents=True)
            temporary.write_bytes(b"in flight")

            with mock.patch(
                "tools.session_coordinator.snapshots.process_is_alive",
                return_value=True,
            ):
                removed = store.reconcile_orphan_files()

            self.assertEqual(0, removed)
            self.assertTrue(temporary.exists())

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

    def test_put_repairs_existing_same_size_corrupt_object(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            store = ObjectStore(database, config.object_root)
            content = b"sealed validation input"
            object_hash = store.put(content)
            path = store.path_for_hash(object_hash)
            compressed_size = path.stat().st_size
            path.write_bytes(b"x" * compressed_size)

            repaired_hash = store.put(content)

            self.assertEqual(object_hash, repaired_hash)
            self.assertEqual(content, store.get(object_hash))


if __name__ == "__main__":
    unittest.main()
