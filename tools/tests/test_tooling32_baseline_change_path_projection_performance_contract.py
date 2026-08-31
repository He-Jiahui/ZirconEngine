from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService, WorkspaceChange
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate


class CountingPath(str):
    casefold_calls = 0

    def casefold(self) -> str:
        type(self).casefold_calls += 1
        return super().casefold()


class BaselineChangePathProjectionPerformanceContractTests(unittest.TestCase):
    def test_reuses_sql_path_keys_when_filtering_unattributed_changes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            database = Database(root / "coordinator.sqlite3")
            migrate(database)
            service = BaselineService(database, root)
            changes = [
                WorkspaceChange(
                    path=CountingPath(f"crates/member-{index:04d}/src/lib.rs"),
                    kind="modified",
                    baseline_hash="before",
                    current_hash="after",
                )
                for index in range(256)
            ]
            CountingPath.casefold_calls = 0

            unattributed = service._unattributed_changes(changes, baseline_epoch=1)

            self.assertEqual(changes, unattributed)
            self.assertEqual(len(changes), CountingPath.casefold_calls)


if __name__ == "__main__":
    unittest.main()
