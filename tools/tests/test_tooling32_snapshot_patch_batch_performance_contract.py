from __future__ import annotations

import inspect
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.patches import PatchService
from tools.session_coordinator.snapshots import ObjectStore


class SnapshotPatchBatchPerformanceContractTests(unittest.TestCase):
    def test_existing_object_reuses_compressed_file_size(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            database = Database(root / "coordinator.sqlite3")
            migrate(database)
            store = ObjectStore(database, root / "objects")
            content = b"repeated snapshot payload" * 4096
            expected_hash = store.put(content)

            with mock.patch(
                "tools.session_coordinator.snapshots.zlib.compress",
                side_effect=AssertionError("existing object was recompressed"),
            ):
                actual_hash = store.put(content)

            self.assertEqual(expected_hash, actual_hash)

    def test_patch_attributions_use_one_batch_upsert(self) -> None:
        source = inspect.getsource(PatchService._apply)

        self.assertIn("attribution_rows", source)
        self.assertEqual(1, source.count("connection.executemany("))
        self.assertEqual(1, source.count("INSERT INTO attributions("))
        self.assertNotIn("connection.execute(\n                    \"\"\"\n                    INSERT INTO attributions", source)


if __name__ == "__main__":
    unittest.main()
