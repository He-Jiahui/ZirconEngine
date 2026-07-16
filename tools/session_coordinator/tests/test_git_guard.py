from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.git_guard import remove_commit_guard
from tools.session_coordinator.tests.helpers import init_repo


class GitCommitGuardTests(unittest.TestCase):
    def test_removes_managed_hooks_and_allows_a_direct_commit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = init_repo(Path(directory) / "repo")
            hooks = repo / ".git/hooks"
            for name in ("pre-commit", "prepare-commit-msg"):
                (hooks / name).write_text(
                    "#!/bin/sh\n# zircon-session-coordinator-managed-commit-guard\nexit 1\n",
                    encoding="utf-8",
                )

            removed = remove_commit_guard(repo)

            self.assertEqual(("pre-commit", "prepare-commit-msg"), removed)
            self.assertFalse((hooks / "pre-commit").exists())
            self.assertFalse((hooks / "prepare-commit-msg").exists())
            (repo / "owned.txt").write_text("owned\n", encoding="utf-8")
            subprocess.run(["git", "add", "owned.txt"], cwd=repo, check=True)
            subprocess.run(
                ["git", "commit", "-m", "feat(test): direct commit"],
                cwd=repo,
                check=True,
            )

    def test_restores_preserved_user_hooks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = init_repo(Path(directory) / "repo")
            hooks = repo / ".git/hooks"
            user_hook = "#!/bin/sh\necho user hook\n"
            (hooks / "pre-commit").write_text(
                "#!/bin/sh\n# zircon-session-coordinator-managed-commit-guard\nexit 1\n",
                encoding="utf-8",
            )
            (hooks / "pre-commit.zircon-user").write_text(user_hook, encoding="utf-8")

            removed = remove_commit_guard(repo)

            self.assertEqual(("pre-commit",), removed)
            self.assertEqual(user_hook, (hooks / "pre-commit").read_text(encoding="utf-8"))
            self.assertFalse((hooks / "pre-commit.zircon-user").exists())

    def test_leaves_unmanaged_user_hooks_unchanged(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = init_repo(Path(directory) / "repo")
            hook = repo / ".git/hooks/pre-commit"
            user_hook = "#!/bin/sh\necho user hook\n"
            hook.write_text(user_hook, encoding="utf-8")

            removed = remove_commit_guard(repo)

            self.assertEqual((), removed)
            self.assertEqual(user_hook, hook.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
