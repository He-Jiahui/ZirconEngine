from __future__ import annotations

import subprocess
import sys
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


class GitFinalizeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
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

    def _complete_with_changes(self) -> list[str]:
        paths = [
            "src/feature.py",
            "docs/feature.md",
            "tests/test_feature.py",
            "tools/check-feature.ps1",
        ]
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"content for {path}\n", encoding="utf-8")
        self.baselines.attribute("session-a", paths)
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)
        return paths

    def _head(self) -> str:
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    def test_completed_session_never_commits_without_explicit_finalize(self) -> None:
        before = self._head()
        self._complete_with_changes()

        self.assertEqual(before, self._head())

    def test_milestone_commit_is_scoped_atomic_and_keeps_session_active(self) -> None:
        paths = ["src/milestone.py", "tests/test_milestone.py"]
        acquisition = self.leases.acquire("session-a", paths)
        self.assertTrue(acquisition.acquired)
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"content for {path}\n", encoding="utf-8")
        self.baselines.attribute("session-a", paths)
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)

        result = self.service.commit_milestone(
            "session-a", paths=paths, message="feat(runtime): complete M2 milestone"
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(paths), sorted(item for item in committed if item))
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("session-a").status)
        self.assertEqual(result.commit_sha, self._head())

    def test_milestone_commit_requires_live_owned_leases(self) -> None:
        path = "src/milestone.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("content\n", encoding="utf-8")
        self.baselines.attribute("session-a", [path])
        subprocess.run(["git", "add", "--", path], cwd=self.repo, check=True)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.commit_milestone(
                "session-a", paths=[path], message="feat(runtime): complete M2 milestone"
            )

        self.assertEqual("milestone_lease_missing", rejected.exception.code)

    def test_milestone_commit_runs_acceptance_inside_git_mutex(self) -> None:
        path = "src/milestone.py"
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("content\n", encoding="utf-8")
        self.baselines.attribute("session-a", [path])
        subprocess.run(["git", "add", "--", path], cwd=self.repo, check=True)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.commit_milestone(
                "session-a",
                paths=[path],
                message="feat(runtime): complete M2 milestone",
                validation_commands=((sys.executable, "-c", "raise SystemExit(7)"),),
            )

        self.assertEqual("milestone_validation_failed", rejected.exception.code)
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("session-a").status)
        self.assertNotEqual("", self._staged_names())

    def test_milestone_commit_accepts_deletion_attributed_after_delete_with_lease_base(self) -> None:
        path = "src/delete_me.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("tracked\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", path], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: add deletion target"], cwd=self.repo, check=True)
        self.baselines.accept(reason="test deletion baseline")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        target.unlink()
        self.baselines.attribute("session-a", [path])
        subprocess.run(["git", "add", "-u", "--", path], cwd=self.repo, check=True)

        result = self.service.commit_milestone(
            "session-a", paths=[path], message="fix(runtime): remove obsolete milestone file"
        )

        self.assertFalse(target.exists())
        self.assertEqual(SessionStatus.ACTIVE, self.sessions.get("session-a").status)
        self.assertEqual(result.commit_sha, self._head())

    def test_preview_records_code_docs_tests_scripts_and_untracked_separately(self) -> None:
        paths = self._complete_with_changes()

        preview = self.service.preview(
            "session-a", paths=paths, message="feat(runtime): add feature"
        )

        self.assertEqual(("src/feature.py",), preview.categories["code"])
        self.assertEqual(("docs/feature.md",), preview.categories["docs"])
        self.assertEqual(("tests/test_feature.py",), preview.categories["tests"])
        self.assertEqual(("tools/check-feature.ps1",), preview.categories["scripts"])
        self.assertEqual(tuple(sorted(paths)), tuple(sorted(preview.untracked_paths)))

    def test_explicit_finalize_commits_only_owned_paths_with_ordinary_message(self) -> None:
        paths = self._complete_with_changes()
        self.sessions.register(session_id="session-b")
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        foreign = self.repo / "foreign.txt"
        foreign.write_text("other Session\n", encoding="utf-8")
        self.baselines.attribute("session-b", ["foreign.txt"])

        result = self.service.finalize(
            "session-a", paths=paths, message="feat(runtime): add feature"
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(paths), sorted(item for item in committed if item))
        self.assertTrue(foreign.exists())
        self.assertNotIn("[zircon-session:", result.message)
        self.assertEqual(SessionStatus.COMPLETED, self.sessions.get("session-a").status)

    def test_finalize_rejects_an_owned_dirty_path_omitted_from_manifest(self) -> None:
        paths = self._complete_with_changes()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.preview(
                "session-a", paths=paths[:-1], message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_owned_path_omitted", rejected.exception.code)

    def test_finalize_rejects_wecom_webhook_material(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        endpoint = "https://" + "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?"
        secret.write_text(
            endpoint + "key=" + "do-not-commit\n",
            encoding="utf-8",
        )
        self.baselines.attribute("session-a", [paths[0]])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual("", self._staged_names())

    def test_unattributed_path_is_rejected_before_index_mutation(self) -> None:
        paths = self._complete_with_changes()
        unowned = self.repo / "unowned.txt"
        unowned.write_text("foreign\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a",
                paths=[*paths, "unowned.txt"],
                message="feat(runtime): add feature",
            )

        self.assertEqual("finalize_unattributed_path", rejected.exception.code)
        self.assertEqual("", self._staged_names())

    def test_foreign_staged_path_aborts_and_preserves_prior_index(self) -> None:
        paths = self._complete_with_changes()
        self.sessions.register(session_id="session-b")
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        foreign = self.repo / "foreign.txt"
        foreign.write_text("foreign\n", encoding="utf-8")
        self.baselines.attribute("session-b", ["foreign.txt"])
        subprocess.run(["git", "add", "foreign.txt"], cwd=self.repo, check=True)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_foreign_index", rejected.exception.code)
        self.assertEqual("foreign.txt", self._staged_names())

    def test_validation_failure_restores_index_without_reverting_worktree(self) -> None:
        paths = self._complete_with_changes()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a",
                paths=paths,
                message="feat(runtime): add feature",
                validation_commands=(("python", "-c", "raise SystemExit(7)"),),
            )

        self.assertEqual("finalize_validation_failed", rejected.exception.code)
        self.assertEqual("", self._staged_names())
        self.assertTrue((self.repo / "src/feature.py").exists())

    def test_content_changed_between_preview_and_stage_is_rejected(self) -> None:
        paths = self._complete_with_changes()
        original_git = self.service._git

        def racing_git(*arguments: str) -> str:
            if arguments[:3] == ("add", "-A", "--"):
                (self.repo / paths[0]).write_text("foreign race\n", encoding="utf-8")
            return original_git(*arguments)

        self.service._git = racing_git  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_staged_attribution_mismatch", rejected.exception.code)
        self.assertEqual("", self._staged_names())
        self.assertEqual("foreign race\n", (self.repo / paths[0]).read_text(encoding="utf-8"))

    def test_validation_command_cannot_expand_staged_scope(self) -> None:
        paths = self._complete_with_changes()
        self.sessions.register(session_id="session-b")
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        foreign = self.repo / "foreign.txt"
        foreign.write_text("foreign\n", encoding="utf-8")
        self.baselines.attribute("session-b", ["foreign.txt"])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a",
                paths=paths,
                message="feat(runtime): add feature",
                validation_commands=(("git", "add", "foreign.txt"),),
            )

        self.assertEqual("finalize_foreign_index", rejected.exception.code)
        self.assertEqual("", self._staged_names())
        self.assertTrue(foreign.exists())

    def test_commit_epoch_does_not_absorb_another_session_dirty_file(self) -> None:
        paths = self._complete_with_changes()
        self.sessions.register(session_id="session-b")
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        foreign = self.repo / "foreign.txt"
        foreign.write_text("other Session\n", encoding="utf-8")
        self.baselines.attribute("session-b", ["foreign.txt"])

        self.service.finalize(
            "session-a", paths=paths, message="feat(runtime): add feature"
        )

        changed = {item.path for item in self.baselines.diff()}
        self.assertIn("foreign.txt", changed)

    def test_commit_epoch_uses_commit_content_not_post_validation_worktree(self) -> None:
        paths = self._complete_with_changes()
        approved = (self.repo / paths[0]).read_text(encoding="utf-8")
        command = (
            "python",
            "-c",
            f"from pathlib import Path; Path({paths[0]!r}).write_text('post-validation\\n')",
        )

        result = self.service.finalize(
            "session-a",
            paths=paths,
            message="feat(runtime): add feature",
            validation_commands=(command,),
        )

        committed = subprocess.run(
            ["git", "show", f"{result.commit_sha}:{paths[0]}"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        self.assertEqual(approved.replace("\r\n", "\n"), committed)
        self.assertEqual("post-validation\n", (self.repo / paths[0]).read_text())
        self.assertIn(paths[0], {item.path for item in self.baselines.diff()})

    def test_restart_recovers_index_and_session_from_interrupted_finalize(self) -> None:
        paths = self._complete_with_changes()
        preview = self.service.preview(
            "session-a", paths=paths, message="feat(runtime): add feature"
        )
        self.sessions.set_status("session-a", SessionStatus.FINALIZING)
        index_path = self.service._index_path()
        existed = index_path.exists()
        snapshot = index_path.read_bytes() if existed else b""
        self.service._persist_finalize_start(
            preview.request_id,
            start_head=self._head(),
            index_existed=existed,
            index_content=snapshot,
        )
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, datetime('now'))",
                ("session-a",),
            )
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(1, recovered)
        self.assertEqual("", self._staged_names())
        self.assertEqual(SessionStatus.COMPLETED, self.sessions.get("session-a").status)
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT status FROM finalize_requests WHERE request_id = ?",
                (preview.request_id,),
            ).fetchone()
        self.assertEqual("failed", request["status"])

    def test_restart_reconciles_commit_when_baseline_update_failed(self) -> None:
        paths = self._complete_with_changes()
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("injected baseline failure"))

        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        moved_head = self._head()
        with self.database.connect() as connection:
            pending = connection.execute(
                "SELECT status, ref_updated_sha, commit_sha FROM finalize_requests ORDER BY created_at DESC LIMIT 1"
            ).fetchone()
        self.assertEqual("finalizing", pending["status"])
        self.assertEqual(moved_head, pending["ref_updated_sha"])
        self.assertIsNone(pending["commit_sha"])
        self.baselines.accept_commit = original_accept

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(0, recovered)
        with self.database.connect() as connection:
            committed = connection.execute(
                "SELECT status, commit_sha FROM finalize_requests WHERE ref_updated_sha = ?",
                (moved_head,),
            ).fetchone()
        self.assertEqual("committed", committed["status"])
        self.assertEqual(moved_head, committed["commit_sha"])

    def test_recovery_keeps_pending_when_baseline_retry_fails(self) -> None:
        paths = self._complete_with_changes()
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("first baseline failure"))
        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("retry failure"))

        with self.assertRaises(RuntimeError):
            self.service.recover_stale_mutex()

        with self.database.connect() as connection:
            pending = connection.execute(
                "SELECT status, commit_sha, ref_updated_sha FROM finalize_requests ORDER BY created_at DESC LIMIT 1"
            ).fetchone()
        self.assertEqual("finalizing", pending["status"])
        self.assertIsNone(pending["commit_sha"])
        self.assertEqual(self._head(), pending["ref_updated_sha"])
        self.baselines.accept_commit = original_accept

    def test_session_tag_message_is_rejected(self) -> None:
        paths = self._complete_with_changes()
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.preview(
                "session-a", paths=paths, message="[zircon-session:bad] feature"
            )
        self.assertEqual("finalize_message_forbidden", rejected.exception.code)

    def _staged_names(self) -> str:
        return subprocess.run(
            ["git", "diff", "--cached", "--name-only"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()


if __name__ == "__main__":
    unittest.main()
