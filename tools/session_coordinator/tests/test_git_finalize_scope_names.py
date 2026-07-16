from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class GitFinalizeScopeNameTests(unittest.TestCase):
    def test_staged_scope_keeps_rename_source_and_target_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            old_path = "docs/plans/runtime/failure-old.md"
            new_path = "docs/plans/runtime/fixed-new.md"
            old = repo / old_path
            old.parent.mkdir(parents=True, exist_ok=True)
            old.write_text("same failure payload\n", encoding="utf-8")
            subprocess.run(["git", "add", "--", old_path], cwd=repo, check=True)
            subprocess.run(
                ["git", "commit", "-q", "-m", "test: add failure"],
                cwd=repo,
                check=True,
            )

            old.unlink()
            new = repo / new_path
            new.write_text("same failure payload\n", encoding="utf-8")

            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            database = Database(config.database_path)
            migrate(database)
            sessions = SessionService(database, repo)
            baselines = BaselineService(database, repo)
            baselines.initialize()
            service = GitFinalizeService(database, repo, baselines, sessions)

            service._git("read-tree", baselines.current().head_commit)
            service._git_add_paths((old_path, new_path))

            self.assertEqual(
                {old_path, new_path},
                set(service._staged_scope_paths()),
            )


if __name__ == "__main__":
    unittest.main()
