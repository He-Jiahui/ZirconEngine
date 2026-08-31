from __future__ import annotations

import subprocess
import unittest
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig


class CoordinatorDatabaseAuthorityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve().parents[3]
        cls.source_placeholder = (
            cls.repo_root / "tools" / "session_coordinator" / "session_coordinator.db"
        )

    def test_source_tree_database_placeholder_is_absent(self) -> None:
        self.assertFalse(self.source_placeholder.exists())

    def test_source_tree_database_placeholder_is_ignored(self) -> None:
        result = subprocess.run(
            [
                "git",
                "check-ignore",
                "--no-index",
                "--quiet",
                str(self.source_placeholder),
            ],
            cwd=self.repo_root,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertEqual(0, result.returncode, result.stderr)

    def test_runtime_database_authority_is_under_coordinator_state(self) -> None:
        config = CoordinatorConfig.for_repo(self.repo_root)

        self.assertEqual(
            self.repo_root
            / ".codex"
            / "state"
            / "session-coordinator"
            / "coordinator.sqlite3",
            config.database_path,
        )
        self.assertNotEqual(self.source_placeholder, config.database_path)


if __name__ == "__main__":
    unittest.main()
