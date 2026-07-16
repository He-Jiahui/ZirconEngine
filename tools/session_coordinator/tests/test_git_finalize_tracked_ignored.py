from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.git_finalize import GitFinalizeService
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class TrackedIgnoredFinalizeTests(unittest.TestCase):
    skill_path = ".codex/skills/runtime-owned/SKILL.md"

    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")

        plan_path = self.repo / "docs" / "plans" / "runtime" / "01-feature.md"
        plan_path.parent.mkdir(parents=True, exist_ok=True)
        plan_path.write_text("# Runtime feature plan\n", encoding="utf-8")
        skill = self.repo / self.skill_path
        skill.parent.mkdir(parents=True, exist_ok=True)
        skill.write_text("---\nname: runtime-owned\n---\n", encoding="utf-8")
        subprocess.run(
            ["git", "add", "--", "docs/plans/runtime/01-feature.md"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "add", "-f", "--", self.skill_path],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add tracked runtime skill"],
            cwd=self.repo,
            check=True,
        )
        exclude = self.repo / ".git" / "info" / "exclude"
        with exclude.open("a", encoding="utf-8") as stream:
            stream.write("/.codex/\n")

        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(
            session_id="session-a", plan_path="docs/plans/runtime/01-feature.md"
        )
        self.sessions.set_status("session-a", SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.leases = LeaseService(
            self.database,
            PathPolicy(self.repo),
            ttl_seconds=config.lease_ttl_seconds,
            grace_seconds=config.lease_grace_seconds,
        )
        self.service = GitFinalizeService(
            self.database, self.repo, self.baselines, self.sessions
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _modify_skill(self) -> None:
        (self.repo / self.skill_path).write_text(
            "---\nname: runtime-owned\nversion: 2\n---\n", encoding="utf-8"
        )

    def _staged_names(self) -> list[str]:
        return subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()

    def test_ignore_scan_classifies_a_tracked_path_ignored_by_parent_rule(self) -> None:
        self._modify_skill()

        ignored = self.service._ignored_paths((self.skill_path,))

        self.assertEqual({self.skill_path}, ignored)

    def test_git_add_failure_records_the_exact_path_chunk(self) -> None:
        paths = (self.skill_path, "docs/runtime-owned-skill.md")
        failure = subprocess.CalledProcessError(
            1,
            ["git", "add"],
            stderr="fatal: simulated add failure",
        )

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run",
            side_effect=failure,
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._git_add_paths(paths, force=True)

        self.assertEqual("finalize_git_command_failed", rejected.exception.code)
        self.assertEqual(list(paths), rejected.exception.details["path_chunk"])
        self.assertEqual(1, rejected.exception.details["exit_code"])
        self.assertIn("simulated add failure", rejected.exception.details["stderr"])

    def test_maintenance_finalize_force_adds_only_the_approved_tracked_skill(self) -> None:
        self._modify_skill()
        foreign_path = "src/foreign_staged.py"
        foreign = self.repo / foreign_path
        foreign.parent.mkdir(parents=True, exist_ok=True)
        foreign.write_text("foreign = True\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", foreign_path], cwd=self.repo, check=True)

        result = self.service.finalize(
            "session-a",
            paths=[self.skill_path],
            message="fix(tooling): finalize tracked ignored skill",
            maintenance=True,
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual([self.skill_path], [path for path in committed if path])
        self.assertEqual([foreign_path], self._staged_names())

    def test_milestone_commits_tracked_ignored_skill_with_untracked_docs_only(self) -> None:
        self._modify_skill()
        docs_path = "docs/runtime-owned-skill.md"
        docs = self.repo / docs_path
        docs.parent.mkdir(parents=True, exist_ok=True)
        docs.write_text("# Runtime-owned skill\n", encoding="utf-8")
        foreign_path = "src/foreign_staged.py"
        foreign = self.repo / foreign_path
        foreign.parent.mkdir(parents=True, exist_ok=True)
        foreign.write_text("foreign = True\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", foreign_path], cwd=self.repo, check=True)
        paths = [self.skill_path, docs_path]
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        self.baselines.attribute("session-a", paths)

        result = self.service.commit_milestone(
            "session-a",
            paths=paths,
            message="fix(tooling): commit tracked ignored skill milestone",
            failure_workflow_node_keys=("M1",),
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(paths), sorted(path for path in committed if path))
        self.assertEqual([foreign_path], self._staged_names())


if __name__ == "__main__":
    unittest.main()
