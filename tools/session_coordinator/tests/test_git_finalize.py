from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineHealth, BaselineService
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
        plan_path = self.repo / "docs" / "plans" / "runtime" / "01-feature.md"
        plan_path.parent.mkdir(parents=True, exist_ok=True)
        plan_path.write_text("# Runtime feature plan\n", encoding="utf-8")
        subprocess.run(
            ["git", "add", "--", "docs/plans/runtime/01-feature.md"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add runtime plan"],
            cwd=self.repo,
            check=True,
        )
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

    def _authorize_recovery_process(self) -> None:
        lock_path = self.database.path.parent / "coordinator.lock"
        lock_path.write_text(json.dumps({"pid": os.getpid()}), encoding="utf-8")

    def _mutex_owner(self) -> str | None:
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
            ).fetchone()
        return None if row is None else str(row["owner_id"])

    def _commit_milestone(self, *args, **kwargs):
        return self.service.commit_milestone(
            *args,
            failure_workflow_node_keys=("M1",),
            **kwargs,
        )

    def test_completed_session_never_commits_without_explicit_finalize(self) -> None:
        before = self._head()
        self._complete_with_changes()

        self.assertEqual(before, self._head())

    def test_cleanup_shared_index_restores_head_index_without_changing_worktree(self) -> None:
        paths = ("src/stale_stage.py", "docs/stale-stage.md")
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"staged but retained worktree: {path}\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)
        before_worktree = {
            path: (self.repo / path).read_bytes()
            for path in paths
        }

        result = self.service.cleanup_shared_index("maintenance:index-cleanup")

        self.assertEqual(self._head(), result["head"])
        self.assertEqual(sorted(paths), result["paths"])
        self.assertEqual(0, result["remaining_staged_count"])
        self.assertEqual("", self._staged_names())
        self.assertEqual(before_worktree, {
            path: (self.repo / path).read_bytes()
            for path in paths
        })

    def test_maintenance_finalize_preserves_foreign_staged_index_on_degraded_baseline(self) -> None:
        maintenance_path = "tools/coordinator_repair.py"
        foreign_path = "src/foreign_staged.py"
        for path, content in (
            (maintenance_path, "repair = True\n"),
            (foreign_path, "foreign = True\n"),
        ):
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", foreign_path], cwd=self.repo, check=True)
        self.assertEqual(foreign_path, self._staged_names())

        result = self.service.finalize(
            "session-a",
            paths=[maintenance_path],
            message="fix(tooling): preserve shared index during maintenance finalize",
            maintenance=True,
        )

        committed = subprocess.run(
            ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual([maintenance_path], committed)
        self.assertEqual(foreign_path, self._staged_names())
        self.assertEqual("foreign = True\n", (self.repo / foreign_path).read_text(encoding="utf-8"))

    def test_maintenance_finalize_snapshots_foreign_leased_work_without_blocking_later_edits(self) -> None:
        path = "src/leased_snapshot.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("snapshot = 1\n", encoding="utf-8")
        self.sessions.register(session_id="session-b")
        self.sessions.set_status("session-b", SessionStatus.ACTIVE)
        self.assertTrue(self.leases.acquire("session-b", [path]).acquired)
        original_stage = self.service._git_add_partition

        def stage_then_continue(*args) -> None:
            original_stage(*args)
            target.write_text("snapshot = 2\n", encoding="utf-8")

        with mock.patch.object(
            self.service, "_git_add_partition", side_effect=stage_then_continue
        ):
            result = self.service.finalize(
                "session-a",
                paths=[path],
                message="feat(runtime): snapshot active leased work",
                maintenance=True,
            )

        committed = subprocess.run(
            ["git", "show", f"{result.commit_sha}:{path}"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        self.assertEqual("snapshot = 1\n", committed)
        self.assertEqual("snapshot = 2\n", target.read_text(encoding="utf-8"))
        self.assertEqual([path], self.leases.owned_paths("session-b"))

    def test_maintenance_restore_failure_keeps_recoverable_index_snapshot(self) -> None:
        maintenance_path = "tools/coordinator_repair.py"
        foreign_path = "src/foreign_staged.py"
        for path, content in (
            (maintenance_path, "repair = True\n"),
            (foreign_path, "foreign = True\n"),
        ):
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", foreign_path], cwd=self.repo, check=True)
        original_index = self.service._index_path().read_bytes()

        with mock.patch.object(
            self.service,
            "_restore_index",
            side_effect=RuntimeError("injected maintenance restore failure"),
        ):
            with self.assertRaises(RuntimeError):
                self.service.finalize(
                    "session-a",
                    paths=[maintenance_path],
                    message="fix(tooling): preserve recoverable maintenance index",
                    maintenance=True,
                )

        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT status, commit_sha, ref_updated_sha, index_snapshot
                   FROM finalize_requests ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertEqual("finalizing", request["status"])
        self.assertIsNone(request["commit_sha"])
        self.assertEqual(self._head(), request["ref_updated_sha"])
        self.assertEqual(original_index, bytes(request["index_snapshot"]))
        self.assertEqual("session-a", self._mutex_owner())

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

        result = self._commit_milestone(
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
        self.assertEqual("feat(runtime): complete M2 milestone", result.message)

    def test_milestone_restore_failure_keeps_recoverable_index_snapshot(self) -> None:
        path = "src/recoverable_milestone.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("milestone = True\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])
        original_index = self.service._index_path().read_bytes()

        with mock.patch.object(
            self.service,
            "_restore_index",
            side_effect=RuntimeError("injected milestone restore failure"),
        ):
            with self.assertRaises(RuntimeError):
                self._commit_milestone(
                    "session-a",
                    paths=[path],
                    message="fix(runtime): preserve recoverable milestone index",
                )

        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT status, commit_sha, ref_updated_sha, index_snapshot
                   FROM finalize_requests ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertEqual("finalizing", request["status"])
        self.assertIsNone(request["commit_sha"])
        self.assertEqual(self._head(), request["ref_updated_sha"])
        self.assertEqual(original_index, bytes(request["index_snapshot"]))
        self.assertEqual("session-a", self._mutex_owner())

    def test_milestone_pre_cas_restore_failure_keeps_recoverable_snapshot(self) -> None:
        path = "src/failed_milestone.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("milestone = False\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])
        original_index = self.service._index_path().read_bytes()

        with mock.patch.object(
            self.service,
            "_restore_index",
            side_effect=RuntimeError("injected failed milestone restore"),
        ):
            with self.assertRaises(RuntimeError):
                self._commit_milestone(
                    "session-a",
                    paths=[path],
                    message="fix(runtime): preserve failed milestone snapshot",
                    validation_commands=((sys.executable, "-c", "raise SystemExit(7)"),),
                )

        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT status, commit_sha, ref_updated_sha, index_snapshot
                   FROM finalize_requests ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertEqual("finalizing", request["status"])
        self.assertIsNone(request["commit_sha"])
        self.assertIsNone(request["ref_updated_sha"])
        self.assertEqual(original_index, bytes(request["index_snapshot"]))
        self.assertEqual("session-a", self._mutex_owner())

    def test_milestone_commit_uses_pathspec_file_add_and_chunked_reset(self) -> None:
        paths = [f"src/chunked/path_{index}.py" for index in range(8)]
        self.assertTrue(self.leases.acquire("session-a", paths).acquired)
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"value = {path!r}\n", encoding="utf-8")
        self.baselines.attribute("session-a", paths)
        original_git = self.service._git

        with mock.patch(
            "tools.session_coordinator.git_finalize._GIT_PATHSPEC_CHUNK_CHARS", 80
        ), mock.patch.object(self.service, "_git", wraps=original_git) as git_call:
            self._commit_milestone(
                "session-a",
                paths=paths,
                message="fix(runtime): chunk milestone pathspec mutations",
            )

        add_calls = [
            call
            for call in git_call.call_args_list
            if call.args and call.args[0] == "add"
        ]
        reset_calls = [
            call
            for call in git_call.call_args_list
            if call.args and call.args[0] == "reset" and "--quiet" in call.args
        ]
        self.assertEqual(1, len(add_calls))
        self.assertTrue(
            any(
                str(argument).startswith("--pathspec-from-file=")
                for argument in add_calls[0].args
            )
        )
        self.assertIn("--pathspec-file-nul", add_calls[0].args)
        self.assertGreater(len(reset_calls), 1)

    def test_milestone_commit_keeps_attributed_tracked_change_after_global_baseline_absorbs_hash(
        self,
    ) -> None:
        path = "src/session_owned.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("committed\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", path], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add tracked ownership fixture"],
            cwd=self.repo,
            check=True,
        )
        self.baselines.refresh_for_head_change()

        target.write_text("session change\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])
        self.baselines.accept(reason="simulate a later global baseline capture")
        self.assertNotIn(path, {change.path for change in self.baselines.diff()})

        result = self._commit_milestone(
            "session-a",
            paths=[path],
            message="fix(runtime): preserve attributed tracked ownership",
        )

        committed = subprocess.run(
            ["git", "show", f"{result.commit_sha}:{path}"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        self.assertEqual("session change\n", committed)

    def test_milestone_commit_owned_scope_does_not_scan_global_baseline(self) -> None:
        """Finalize must inspect only this Session's attributed paths under its mutex."""
        path = "src/session_local_scope.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("owned = True\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])

        with mock.patch.object(
            self.baselines,
            "diff",
            side_effect=AssertionError("finalize must not scan the global workspace"),
        ):
            result = self._commit_milestone(
                "session-a",
                paths=[path],
                message="fix(runtime): scope finalize ownership to session paths",
            )

        self.assertEqual(result.commit_sha, self._head())

    def test_milestone_commit_ignores_clean_attributed_lf_file_with_autocrlf(self) -> None:
        clean_path = "src/already_committed.py"
        clean_target = self.repo / clean_path
        clean_target.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "config", "core.autocrlf", "true"], cwd=self.repo, check=True
        )
        clean_target.write_bytes(b"first line\nsecond line\n")
        self.baselines.attribute("session-a", [clean_path])
        subprocess.run(["git", "add", "--", clean_path], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add LF ownership fixture"],
            cwd=self.repo,
            check=True,
        )
        self.baselines.refresh_for_head_change()
        self.assertEqual(
            "",
            subprocess.run(
                ["git", "status", "--short", "--", clean_path],
                cwd=self.repo,
                check=True,
                capture_output=True,
                text=True,
            ).stdout,
        )
        self.assertEqual(
            set(), self.service._worktree_paths_differing_from_head([clean_path])
        )

        changed_path = "src/current_milestone.py"
        changed_target = self.repo / changed_path
        changed_target.write_text("milestone change\n", encoding="utf-8", newline="\n")
        self.assertTrue(self.leases.acquire("session-a", [changed_path]).acquired)
        self.baselines.attribute("session-a", [changed_path])

        result = self._commit_milestone(
            "session-a",
            paths=[changed_path],
            message="fix(runtime): ignore clean attributed LF paths",
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual([changed_path], [item for item in committed if item])

    def test_session_dirty_scan_batches_git_queries(self) -> None:
        tracked_paths = [f"src/batch_{index}.py" for index in range(20)]
        tracked_paths.append(" leading_batch.py")
        for path in tracked_paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"value = {path!r}\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", *tracked_paths], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add batch dirty fixtures"],
            cwd=self.repo,
            check=True,
        )
        (self.repo / tracked_paths[0]).write_text("modified = True\n", encoding="utf-8")
        (self.repo / tracked_paths[1]).unlink()
        untracked_path = "src/untracked_batch.py"
        (self.repo / untracked_path).write_text("untracked = True\n", encoding="utf-8")
        paths = [*tracked_paths, untracked_path, "src/missing_batch.py"]

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", wraps=subprocess.run
        ) as runner:
            dirty = self.service._worktree_paths_differing_from_head(paths)

        git_calls = [
            call
            for call in runner.call_args_list
            if call.args and call.args[0] and call.args[0][0] == "git"
        ]
        self.assertEqual({tracked_paths[0], tracked_paths[1], untracked_path}, dirty)
        self.assertEqual(2, len(git_calls))

    def test_session_dirty_scan_surfaces_git_failures(self) -> None:
        failure = subprocess.CalledProcessError(
            128, ["git", "ls-tree"], stderr="fatal: object database unavailable"
        )
        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", side_effect=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._worktree_paths_differing_from_head(["src/feature.py"])

        self.assertEqual("finalize_head_content_failed", rejected.exception.code)

    def test_git_command_failure_preserves_bounded_stderr_for_finalize_audit(self) -> None:
        failure = subprocess.CalledProcessError(
            128,
            ["git", "write-tree"],
            stderr="fatal: index file is corrupt\n" + ("x" * 4_096),
        )
        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", side_effect=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._git("write-tree")

        self.assertEqual("finalize_git_command_failed", rejected.exception.code)
        self.assertEqual("git write-tree", rejected.exception.details["command"])
        self.assertEqual(128, rejected.exception.details["exit_code"])
        self.assertIn("fatal: index file is corrupt", rejected.exception.details["stderr"])
        self.assertLessEqual(len(rejected.exception.details["stderr"]), 2_048)

    def test_staged_blob_scan_batches_git_queries_and_preserves_path_bytes(self) -> None:
        paths = [f"src/staged_batch_{index}.py" for index in range(20)]
        paths.append(" leading_staged_batch.py")
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"value = {path!r}\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)
        missing_path = "src/deleted_staged_batch.py"

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", wraps=subprocess.run
        ) as runner:
            blobs = self.service._staged_blobs((*paths, missing_path))

        git_calls = [
            call
            for call in runner.call_args_list
            if call.args and call.args[0] and call.args[0][0] == "git"
        ]
        self.assertEqual(1, len(git_calls))
        self.assertIn("ls-files", git_calls[0].args[0])
        self.assertIn("--stage", git_calls[0].args[0])
        self.assertEqual(set(paths), {path for path, blob in blobs.items() if blob})
        self.assertIsNone(blobs[missing_path])
        self.assertRegex(blobs[" leading_staged_batch.py"] or "", r"^[0-9a-f]{40,64}$")

    def test_staged_blob_scan_surfaces_git_failures(self) -> None:
        failure = subprocess.CalledProcessError(
            128, ["git", "ls-files"], stderr="fatal: index unavailable"
        )
        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", side_effect=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._staged_blobs(("src/feature.py",))

        self.assertEqual("finalize_index_blob_scan_failed", rejected.exception.code)

    def test_staged_blob_scan_batches_a_large_milestone_manifest(self) -> None:
        paths = tuple(f"src/large_manifest/path_{index:03}.py" for index in range(320))
        for path in paths:
            target = self.repo / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"value = {path!r}\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", *paths], cwd=self.repo, check=True)

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", wraps=subprocess.run
        ) as runner:
            blobs = self.service._staged_blobs(paths)

        scan_calls = [
            call
            for call in runner.call_args_list
            if call.args
            and call.args[0]
            and call.args[0][0] == "git"
            and "ls-files" in call.args[0]
            and "--stage" in call.args[0]
        ]
        self.assertEqual(set(paths), {path for path, blob in blobs.items() if blob})
        self.assertLessEqual(
            len(scan_calls),
            2,
            "staged blob verification must scale by Windows pathspec chunks, not paths",
        )

    def test_ignored_path_scan_batches_git_queries_and_preserves_path_bytes(self) -> None:
        exclude = self.repo / ".git" / "info" / "exclude"
        with exclude.open("a", encoding="utf-8") as stream:
            stream.write("/ignored_batch/\n/ leading_ignored_batch.py\n")
        paths = [f"ignored_batch/path_{index}.py" for index in range(20)]
        paths.extend([" leading_ignored_batch.py", "src/not_ignored_batch.py"])

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", wraps=subprocess.run
        ) as runner:
            ignored = self.service._ignored_paths(tuple(paths))

        git_calls = [
            call
            for call in runner.call_args_list
            if call.args and call.args[0] and call.args[0][0] == "git"
        ]
        self.assertEqual(1, len(git_calls))
        self.assertIn("check-ignore", git_calls[0].args[0])
        self.assertEqual(set(paths[:-1]), ignored)

    def test_ignored_path_scan_surfaces_git_failures(self) -> None:
        failure = subprocess.CompletedProcess(
            ["git", "check-ignore"], 128, stdout=b"", stderr=b"fatal: index unavailable"
        )
        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", return_value=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._ignored_paths(("src/feature.py",))

        self.assertEqual("finalize_ignore_scan_failed", rejected.exception.code)

    def test_git_path_output_redacts_git_stderr_details(self) -> None:
        secret = "api" + "_key=do-not-log"
        failure = subprocess.CalledProcessError(
            128, ["git", "ls-tree"], stderr=f"fatal: {secret}"
        )

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", side_effect=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._git_path_output("ls-tree", "--name-only", "HEAD")

        self.assertEqual("finalize_head_content_failed", rejected.exception.code)
        self.assertNotIn(secret, str(rejected.exception.details))
        self.assertIn("<redacted>", rejected.exception.details["stderr"])

    def test_ignored_path_scan_redacts_git_stderr_details(self) -> None:
        secret = "WECOM_" + "WEBHOOK_KEY=do-not-log"
        failure = subprocess.CompletedProcess(
            ["git", "check-ignore"],
            128,
            stdout=b"",
            stderr=f"fatal: {secret}".encode("utf-8"),
        )

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", return_value=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._ignored_paths(("src/feature.py",))

        self.assertEqual("finalize_ignore_scan_failed", rejected.exception.code)
        self.assertNotIn(secret, str(rejected.exception.details))
        self.assertIn("<redacted>", rejected.exception.details["error"])

    def test_index_worktree_scan_redacts_git_stderr_details(self) -> None:
        secret = (
            "https://"
            + "qyapi"
            + ".weixin.qq.com/cgi-bin/"
            + "webhook/send?"
            + "key=do-not-log"
        )
        failure = subprocess.CompletedProcess(
            ["git", "diff"],
            128,
            stdout=b"",
            stderr=f"fatal: {secret}".encode("utf-8"),
        )

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.run", return_value=failure
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._require_index_matches_worktree(("src/feature.py",))

        self.assertEqual("finalize_index_worktree_scan_failed", rejected.exception.code)
        self.assertNotIn(secret, str(rejected.exception.details))
        self.assertIn("<redacted>", rejected.exception.details["error"])

    def test_pathspec_chunks_use_utf16_budget_and_reject_oversize_path(self) -> None:
        astral_path = "\U0001f600" * 7_000
        chunks = list(self.service._pathspec_chunks((astral_path, astral_path)))
        self.assertEqual(2, len(chunks))

        with self.assertRaises(CoordinatorError) as rejected:
            list(self.service._pathspec_chunks(("\U0001f600" * 12_000,)))

        self.assertEqual("finalize_pathspec_too_long", rejected.exception.code)

    def test_milestone_commit_includes_an_explicitly_owned_ignored_skill(self) -> None:
        path = ".codex/skills/runtime-new/SKILL.md"
        exclude = self.repo / ".git" / "info" / "exclude"
        with exclude.open("a", encoding="utf-8") as stream:
            stream.write("/.codex/\n")
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("---\nname: runtime-new\n---\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])

        result = self._commit_milestone(
            "session-a", paths=[path], message="feat(runtime): add managed skill"
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual([path], [item for item in committed if item])

    def test_milestone_commit_rejects_ignored_session_notes(self) -> None:
        path = ".codex/sessions/temporary.md"
        exclude = self.repo / ".git" / "info" / "exclude"
        with exclude.open("a", encoding="utf-8") as stream:
            stream.write("/.codex/\n")
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("temporary session note\n", encoding="utf-8")
        self.assertTrue(self.leases.acquire("session-a", [path]).acquired)
        self.baselines.attribute("session-a", [path])

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit_milestone(
                "session-a", paths=[path], message="feat(runtime): add local state"
            )

        self.assertEqual("milestone_ignored_path_forbidden", rejected.exception.code)

    def test_preview_preserves_conventional_commit_without_module_prefix(self) -> None:
        paths = self._complete_with_changes()

        preview = self.service.preview(
            "session-a", paths=paths, message="feat(runtime): add feature"
        )

        self.assertEqual("feat(runtime): add feature", preview.message)

    def test_preview_rejects_any_module_prefix(self) -> None:
        paths = self._complete_with_changes()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.preview(
                "session-a",
                paths=paths,
                message="【editor_ui】feat(runtime): add feature",
            )

        self.assertEqual("finalize_message_prefix_forbidden", rejected.exception.code)

    def test_milestone_commit_requires_live_owned_leases(self) -> None:
        path = "src/milestone.py"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("content\n", encoding="utf-8")
        self.baselines.attribute("session-a", [path])
        subprocess.run(["git", "add", "--", path], cwd=self.repo, check=True)

        with self.assertRaises(CoordinatorError) as rejected:
            self._commit_milestone(
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
            self._commit_milestone(
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

        result = self._commit_milestone(
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
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT index_snapshot FROM finalize_requests WHERE request_id = ?",
                (result.request_id,),
            ).fetchone()
        self.assertIsNone(request["index_snapshot"])

    def test_explicit_finalize_allows_owned_scope_when_global_baseline_is_degraded(self) -> None:
        paths = self._complete_with_changes()
        foreign = self.repo / "foreign-unattributed.txt"
        foreign.write_text("unrelated workspace change\n", encoding="utf-8")
        self.baselines.scan()
        self.assertEqual(BaselineHealth.DEGRADED, self.baselines.current().health)

        result = self.service.finalize(
            "session-a", paths=paths, message="feat(runtime): finalize against degraded baseline"
        )

        committed = subprocess.run(
            ["git", "show", "--pretty=", "--name-only", result.commit_sha],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        self.assertEqual(sorted(paths), sorted(item for item in committed if item))
        self.assertEqual("unrelated workspace change\n", foreign.read_text(encoding="utf-8"))
        self.assertEqual(SessionStatus.COMPLETED, self.sessions.get("session-a").status)

    def test_finalize_rejects_an_owned_dirty_path_omitted_from_manifest(self) -> None:
        paths = self._complete_with_changes()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.preview(
                "session-a", paths=paths[:-1], message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_owned_path_omitted", rejected.exception.code)

    def test_owned_scope_ignores_git_ignored_codex_session_state(self) -> None:
        ignore = self.repo / ".gitignore"
        ignore.write_text("/.codex\n", encoding="utf-8")
        subprocess.run(["git", "add", ".gitignore"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: ignore local codex state"],
            cwd=self.repo,
            check=True,
        )
        feature = self.repo / "src" / "feature.py"
        feature.parent.mkdir(parents=True, exist_ok=True)
        feature.write_text("accepted feature\n", encoding="utf-8")
        note = self.repo / ".codex" / "sessions" / "active.md"
        note.parent.mkdir(parents=True, exist_ok=True)
        note.write_text("live session state\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["src/feature.py", ".codex/sessions/active.md"])

        self.service._require_owned_scope(
            "session-a", ("src/feature.py",), maintenance=False
        )

    def test_plan_output_allows_owned_fixed_return_in_origin_child(self) -> None:
        fixed_path = "docs/plans/render/18/fixed-2026-07-16-contract-drift.md"
        target = self.repo / fixed_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("---\nhandoff_kind: fixed\nstatus: fixed\n---\n", encoding="utf-8")
        self.baselines.attribute("session-a", [fixed_path])
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO failure_nodes(
                    lifecycle_key, artifact_path, kind, status, created_at, resolved_at,
                    summary_slug, origin_plan, fixing_plan, origin_child_dir,
                    fixing_child_dir, priority, imported_at, origin_workflow_node
                ) VALUES (?, ?, 'fixed', 'fixed', ?, ?, ?, ?, ?, ?, ?, 100, ?, NULL)
                """,
                (
                    "render18-contract-drift",
                    fixed_path,
                    "2026-07-16T00:00:00+00:00",
                    "2026-07-16T00:00:00+00:00",
                    "contract-drift",
                    "docs/plans/render/18-plan.md",
                    "docs/plans/runtime/01-feature.md",
                    "docs/plans/render/18",
                    "docs/plans/runtime/01",
                    "2026-07-16T00:00:00+00:00",
                ),
            )

        self.service._require_plan_outputs(
            self.sessions.get("session-a"), (fixed_path,), maintenance=False
        )

    def test_finalize_rejects_staged_wecom_webhook_url(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        endpoint = "https://" + "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?"
        secret_value = endpoint + "key=do-not-commit"
        secret.write_text(
            secret_value + "\n",
            encoding="utf-8",
        )
        self.baselines.attribute("session-a", [paths[0]])
        before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertNotIn(secret_value, str(rejected.exception.details))
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_staged_wecom_webhook_key_configuration(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        secret_value = "WECOM_" + "WEBHOOK_KEY=do-not-commit"
        secret.write_text(secret_value + "\n", encoding="utf-8")
        self.baselines.attribute("session-a", [paths[0]])
        before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertNotIn(secret_value, str(rejected.exception.details))
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_staged_maintenance_capability(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        secret.write_text(
            "ZIRCON_COORDINATOR_" + "MAINTENANCE_TOKEN=do-not-commit\n",
            encoding="utf-8",
        )
        self.baselines.attribute("session-a", [paths[0]])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_staged_generic_credential(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        secret.write_text("api" + "_key=do-not-commit\n", encoding="utf-8")
        self.baselines.attribute("session-a", [paths[0]])

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_staged_credential_in_binary_blob(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        endpoint = "https://" + "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?"
        secret.write_bytes(
            b"\x00binary-prefix\xff"
            + (b"x" * 65_520)
            + (endpoint + "key=binary-secret").encode()
        )
        self.baselines.attribute("session-a", [paths[0]])
        before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add binary feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertEqual("", self._staged_names())

    def test_finalize_allows_binary_blob_without_credential_marker(self) -> None:
        paths = self._complete_with_changes()
        binary = self.repo / paths[0]
        binary.write_bytes(b"\x00safe-binary\xff" + (b"x" * 70_000))
        self.baselines.attribute("session-a", [paths[0]])

        result = self.service.finalize(
            "session-a", paths=paths, message="feat(runtime): add safe binary feature"
        )

        self.assertEqual(result.commit_sha, self._head())
        self.assertEqual("", self._staged_names())

    def test_finalize_allows_sensitive_names_without_concrete_assignments(self) -> None:
        paths = self._complete_with_changes()
        source = self.repo / paths[0]
        source.write_text(
            "SENSITIVE_PATTERN = r'ZIRCON_COORDINATOR_MAINTENANCE_TOKEN|password'\n"
            "def api_key_contract():\n"
            "    return 'names are not credential values'\n"
            + "api"
            + "_key: str\n"
            + "pass"
            + "word: Optional[str]\n"
            + "pass"
            + "word: String,\n",
            encoding="utf-8",
        )
        self.baselines.attribute("session-a", [paths[0]])

        result = self.service.finalize(
            "session-a", paths=paths, message="test(runtime): document credential fields"
        )

        self.assertEqual(result.commit_sha, self._head())
        self.assertEqual("", self._staged_names())

    def test_finalize_allows_sensitive_source_values_from_non_literal_expressions(self) -> None:
        paths = ["src/credential_forwarding.rs", "tools/credential-forwarding.ps1"]
        rust = self.repo / paths[0]
        rust.parent.mkdir(parents=True, exist_ok=True)
        rust.write_text(
            "let pass" + "word = request.pass" + "word;\n"
            "Route::ShowPass" + "word => {}\n"
            "Credentials { pass" + "word: String::new() }\n",
            encoding="utf-8",
        )
        powershell = self.repo / paths[1]
        powershell.parent.mkdir(parents=True, exist_ok=True)
        powershell.write_text(
            "$env:ZIRCON_COORDINATOR_"
            + "MAINTENANCE_TOKEN = $generatedToken\n",
            encoding="utf-8",
        )

        result = self.service.finalize(
            "session-a",
            paths=paths,
            message="test(tooling): allow credential value forwarding",
            maintenance=True,
        )

        self.assertEqual(result.commit_sha, self._head())
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_utf16_staged_credential(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        before = self._head()

        for encoding in ("utf-16", "utf-16-le", "utf-16-be"):
            with self.subTest(encoding=encoding):
                prefix = "" if encoding == "utf-16" else "漢" * 4_096
                secret.write_bytes(
                    (prefix + "api" + "_key=utf16-secret\n").encode(encoding)
                )
                self.baselines.attribute("session-a", [paths[0]])
                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.finalize(
                        "session-a",
                        paths=paths,
                        message="feat(runtime): reject utf16 secret",
                    )

                self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_yaml_scalar_that_matches_a_source_type_name(self) -> None:
        path = "config/runtime.yaml"
        target = self.repo / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("api" + "_key: str\n", encoding="utf-8")
        self.baselines.attribute("session-a", [path])
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=[path], message="fix(runtime): reject yaml secret"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual("", self._staged_names())

    def test_finalize_rejects_boundary_marker_before_long_separator(self) -> None:
        paths = self._complete_with_changes()
        secret = self.repo / paths[0]
        secret.write_bytes(
            (b"x" * 65_533)
            + b"api"
            + b"_key"
            + (b" " * 70_000)
            + b"= boundary-secret\n"
        )
        self.baselines.attribute("session-a", [paths[0]])
        before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): reject boundary marker"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertEqual("", self._staged_names())

    def test_secret_scan_skips_staged_gitlink_object_payload(self) -> None:
        gitlink_path = "vendor/dependency"
        missing_commit = "1" * 40
        subprocess.run(
            [
                "git",
                "update-index",
                "--add",
                "--cacheinfo",
                "160000",
                missing_commit,
                gitlink_path,
            ],
            cwd=self.repo,
            check=True,
        )

        self.service._require_no_staged_secrets()

        self.assertEqual(gitlink_path, self._staged_names())

    def test_secret_scan_uses_file_backed_stderr_to_avoid_pipe_backpressure(self) -> None:
        target = self.repo / "src" / "feature.py"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("safe staged content\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", "src/feature.py"], cwd=self.repo, check=True)
        original_popen = subprocess.Popen
        observed_stderr: list[object] = []

        def capture_stderr(*args, **kwargs):
            command = args[0] if args else kwargs.get("args")
            if command == ["git", "cat-file", "--batch"]:
                observed_stderr.append(kwargs.get("stderr"))
            return original_popen(*args, **kwargs)

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.Popen",
            side_effect=capture_stderr,
        ):
            self.service._require_no_staged_secrets()

        self.assertEqual(1, len(observed_stderr))
        self.assertIsNot(subprocess.PIPE, observed_stderr[0])
        self.assertTrue(hasattr(observed_stderr[0], "fileno"))

    def test_secret_scan_cleanup_survives_broken_stdin_pipe(self) -> None:
        target = self.repo / "src" / "feature.py"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("safe staged content\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", "src/feature.py"], cwd=self.repo, check=True)
        original_popen = subprocess.Popen
        process = mock.Mock()
        process.stdin = mock.Mock()
        process.stdin.close.side_effect = BrokenPipeError("closed batch input")
        process.stdout = mock.Mock()
        process.stdout.readline.side_effect = OSError("batch read failed")
        process.poll.return_value = None

        def fail_cat_file(*args, **kwargs):
            command = args[0] if args else kwargs.get("args")
            if command == ["git", "cat-file", "--batch"]:
                return process
            return original_popen(*args, **kwargs)

        with mock.patch(
            "tools.session_coordinator.git_finalize.subprocess.Popen",
            side_effect=fail_cat_file,
        ):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._require_no_staged_secrets()

        self.assertEqual("finalize_secret_scan_failed", rejected.exception.code)
        process.kill.assert_called_once_with()
        process.wait.assert_called_once_with()
        process.stdout.close.assert_called_once_with()

    def test_finalize_rejects_staged_credential_when_attributes_disable_diff(self) -> None:
        paths = self._complete_with_changes()
        attributes_path = ".gitattributes"
        (self.repo / attributes_path).write_text("src/feature.py -diff\n", encoding="utf-8")
        secret = self.repo / paths[0]
        secret.write_text("api" + "_key=attributes-secret\n", encoding="utf-8")
        paths.append(attributes_path)
        self.baselines.attribute("session-a", [paths[0], attributes_path])
        before = self._head()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add attributed feature"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(before, self._head())
        self.assertEqual("", self._staged_names())

    def test_secret_scan_decodes_utf8_independently_of_host_locale(self) -> None:
        target = self.repo / "docs" / "utf8.md"
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text("里程碑验证通过\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", "docs/utf8.md"], cwd=self.repo, check=True)

        with mock.patch("locale.getencoding", return_value="ascii"):
            self.service._require_no_staged_secrets()

    def test_safe_git_stderr_redacts_complete_wecom_webhook(self) -> None:
        endpoint = "https://" + "qyapi" + ".weixin.qq.com/cgi-bin/" + "webhook/send?"

        sanitized = self.service._safe_git_stderr(endpoint + "key=do-not-log")

        self.assertEqual("https://<redacted>", sanitized)

    def test_safe_git_stderr_redacts_complete_typed_and_yaml_assignments(self) -> None:
        cases = (
            ('fatal: api' + '_key: str = "typed-secret"', "typed-secret"),
            ("fatal: api" + "_key: !!str yaml-secret", "yaml-secret"),
            (
                'fatal: api' + '_key: Literal["prod"] = "typed-literal-secret"',
                "typed-literal-secret",
            ),
            (
                "fatal: api" + "_key: !!str yaml secret value",
                "yaml secret value",
            ),
            (
                'fatal: api' + '_key = {"nested": "dict-secret"}',
                "dict-secret",
            ),
        )

        for stderr, secret in cases:
            with self.subTest(stderr=stderr):
                sanitized = self.service._safe_git_stderr(stderr)
                self.assertEqual("fatal: <redacted>", sanitized)
                self.assertNotIn(secret, sanitized)

    def test_git_failure_redacts_complete_typed_assignment_from_message_and_details(self) -> None:
        stderr = 'fatal: api' + '_key: str = "direct-git-secret"'
        failure = subprocess.CalledProcessError(
            128,
            ["git", "status"],
            stderr=stderr,
        )

        with mock.patch("subprocess.run", side_effect=failure):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service._git("status")

        self.assertEqual("finalize_git_command_failed", rejected.exception.code)
        self.assertNotIn("direct-git-secret", str(rejected.exception))
        self.assertNotIn("direct-git-secret", str(rejected.exception.details))
        self.assertEqual("fatal: <redacted>", rejected.exception.details["stderr"])

    def test_secret_scan_owns_git_mutex_and_rejection_restores_raw_index_and_head(self) -> None:
        paths = self._complete_with_changes()
        staged_path = self.repo / paths[0]
        staged_path.write_text("approved pre-existing stage\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", paths[0]], cwd=self.repo, check=True)
        index_path = self.service._index_path()
        before_index = index_path.read_bytes()
        before_head = self._head()
        staged_path.write_text("api" + "_key=restore-secret\n", encoding="utf-8")
        self.baselines.attribute("session-a", [paths[0]])
        observed_mutex_owners: list[str | None] = []
        observed_restore_owners: list[str | None] = []
        original_scan = self.service._require_no_staged_secrets
        original_restore = self.service._restore_index

        def scan_while_owned() -> None:
            with self.database.connect() as connection:
                row = connection.execute(
                    "SELECT owner_id FROM git_mutex WHERE lock_name = 'index'"
                ).fetchone()
            observed_mutex_owners.append(None if row is None else row["owner_id"])
            original_scan()

        def restore_while_owned(path, existed, content) -> None:
            with self.database.connect() as connection:
                row = connection.execute(
                    "SELECT owner_id FROM git_mutex WHERE lock_name = 'index'"
                ).fetchone()
            observed_restore_owners.append(None if row is None else row["owner_id"])
            original_restore(path, existed, content)

        self.service._require_no_staged_secrets = scan_while_owned  # type: ignore[method-assign]
        self.service._restore_index = restore_while_owned  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): reject staged secret"
            )

        self.assertEqual("finalize_secret_detected", rejected.exception.code)
        self.assertEqual(["session-a"], observed_mutex_owners)
        self.assertEqual(["session-a"], observed_restore_owners)
        self.assertEqual(before_head, self._head())
        self.assertEqual(before_index, index_path.read_bytes())
        self.assertEqual(paths[0], self._staged_names())
        self.assertIsNone(self._mutex_owner())

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
        self.assertIsNone(self._mutex_owner())

    def test_pre_cas_restore_failure_keeps_recoverable_index_snapshot(self) -> None:
        paths = self._complete_with_changes()
        original_index = self.service._index_path().read_bytes()

        with mock.patch.object(
            self.service,
            "_restore_index",
            side_effect=RuntimeError("injected pre-cas restore failure"),
        ):
            with self.assertRaises(RuntimeError):
                self.service.finalize(
                    "session-a",
                    paths=paths,
                    message="fix(runtime): preserve failed finalize snapshot",
                    validation_commands=((sys.executable, "-c", "raise SystemExit(7)"),),
                )

        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT status, commit_sha, ref_updated_sha, index_snapshot
                   FROM finalize_requests ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertEqual("finalizing", request["status"])
        self.assertIsNone(request["commit_sha"])
        self.assertIsNone(request["ref_updated_sha"])
        self.assertEqual(original_index, bytes(request["index_snapshot"]))
        self.assertEqual("session-a", self._mutex_owner())

    def test_content_changed_between_preview_and_stage_is_rejected(self) -> None:
        paths = self._complete_with_changes()
        original_git = self.service._git

        def racing_git(*arguments: str) -> str:
            if arguments[:2] == ("add", "-A"):
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

    def test_stage_blob_injection_restored_to_attributed_worktree_is_rejected(self) -> None:
        paths = self._complete_with_changes()
        original_git = self.service._git

        def injecting_git(*arguments: str) -> str:
            if arguments and arguments[0] == "add":
                target = self.repo / paths[0]
                approved = target.read_text(encoding="utf-8")
                target.write_text("injected staged blob\n", encoding="utf-8")
                try:
                    return original_git(*arguments)
                finally:
                    target.write_text(approved, encoding="utf-8")
            return original_git(*arguments)

        self.service._git = injecting_git  # type: ignore[method-assign]
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        self.assertEqual("finalize_staged_attribution_mismatch", rejected.exception.code)
        self.assertEqual("", self._staged_names())
        self.assertEqual(
            "content for src/feature.py\n",
            (self.repo / paths[0]).read_text(encoding="utf-8"),
        )

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

    def test_recovery_rejects_unproven_process_owner_without_releasing_mutex(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, datetime('now'))",
                ("active-owner",),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.recover_stale_mutex()

        self.assertEqual("finalize_recovery_process_unproven", rejected.exception.code)
        with self.database.connect() as connection:
            owner = connection.execute(
                "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
            ).fetchone()["owner_id"]
        self.assertEqual("active-owner", owner)

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
        self._authorize_recovery_process()
        observed_restore_owners: list[str] = []
        original_restore = self.service._restore_index

        def restore_while_recovery_owns_mutex(path, existed, content) -> None:
            with self.database.connect() as connection:
                owner = connection.execute(
                    "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
                ).fetchone()["owner_id"]
            observed_restore_owners.append(owner)
            original_restore(path, existed, content)

        self.service._restore_index = restore_while_recovery_owns_mutex  # type: ignore[method-assign]

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(1, recovered)
        self.assertEqual(1, len(observed_restore_owners))
        self.assertTrue(observed_restore_owners[0].startswith("recovery:"))
        self.assertEqual("", self._staged_names())
        self.assertEqual(SessionStatus.COMPLETED, self.sessions.get("session-a").status)
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT status, index_snapshot FROM finalize_requests WHERE request_id = ?",
                (preview.request_id,),
            ).fetchone()
        self.assertEqual("failed", request["status"])
        self.assertIsNone(request["index_snapshot"])

    def test_recovery_preserves_snapshot_and_mutex_when_head_is_ambiguous(self) -> None:
        historical_path = "src/historical_finalize.py"
        historical = self.repo / historical_path
        historical.parent.mkdir(parents=True, exist_ok=True)
        historical.write_text("historical = True\n", encoding="utf-8")
        self.baselines.attribute("session-a", [historical_path])
        self.sessions.set_status("session-a", SessionStatus.COMPLETED)
        self.service.finalize(
            "session-a",
            paths=[historical_path],
            message="feat(runtime): historical finalize fixture",
        )
        paths = self._complete_with_changes()
        preview = self.service.preview(
            "session-a", paths=paths, message="feat(runtime): ambiguous recovery"
        )
        index_path = self.service._index_path()
        snapshot = index_path.read_bytes()
        start_head = self._head()
        self.service._persist_finalize_start(
            preview.request_id,
            start_head=start_head,
            index_existed=True,
            index_content=snapshot,
        )
        with self.database.transaction() as connection:
            connection.execute(
                "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, datetime('now'))",
                ("interrupted-owner",),
            )
        foreign = self.repo / "foreign.txt"
        foreign.write_text("foreign head\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", "foreign.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: move head outside finalize"],
            cwd=self.repo,
            check=True,
        )
        self.assertNotEqual(start_head, self._head())
        self.sessions.set_status("session-a", SessionStatus.FINALIZING)
        self._authorize_recovery_process()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.recover_stale_mutex()

        self.assertEqual("finalize_recovery_head_ambiguous", rejected.exception.code)
        self.assertTrue(self._mutex_owner().startswith("recovery:"))
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT status, index_snapshot FROM finalize_requests WHERE request_id=?",
                (preview.request_id,),
            ).fetchone()
        self.assertEqual("finalizing", request["status"])
        self.assertEqual(snapshot, bytes(request["index_snapshot"]))
        self.assertEqual(SessionStatus.FINALIZING, self.sessions.get("session-a").status)

    def test_recovery_completes_session_for_already_committed_request(self) -> None:
        paths = self._complete_with_changes()
        result = self.service.finalize(
            "session-a", paths=paths, message="feat(runtime): committed recovery fixture"
        )
        with self.database.connect() as connection:
            request_id = connection.execute(
                "SELECT request_id FROM finalize_requests WHERE commit_sha=?",
                (result.commit_sha,),
            ).fetchone()["request_id"]
        self.sessions.set_status("session-a", SessionStatus.FINALIZING)
        self._authorize_recovery_process()

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(0, recovered)
        self.assertEqual(SessionStatus.COMPLETED, self.sessions.get("session-a").status)
        self.assertIsNone(self._mutex_owner())
        with self.database.connect() as connection:
            request = connection.execute(
                "SELECT status, commit_sha FROM finalize_requests WHERE request_id=?",
                (request_id,),
            ).fetchone()
        self.assertEqual("committed", request["status"])
        self.assertEqual(result.commit_sha, request["commit_sha"])

    def test_restart_reconciles_commit_when_baseline_update_failed(self) -> None:
        paths = self._complete_with_changes()
        staged_path = self.repo / paths[0]
        staged_path.write_text("pre-existing staged bytes\n", encoding="utf-8")
        subprocess.run(["git", "add", "--", paths[0]], cwd=self.repo, check=True)
        index_path = self.service._index_path()
        original_index = index_path.read_bytes()
        original_head = self._head()
        staged_path.write_text("final attributed worktree bytes\n", encoding="utf-8")
        self.baselines.attribute("session-a", [paths[0]])
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("injected baseline failure"))

        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )

        moved_head = self._head()
        self.assertNotEqual(original_head, moved_head)
        self.assertEqual(original_index, index_path.read_bytes())
        with self.database.connect() as connection:
            pending = connection.execute(
                "SELECT status, ref_updated_sha, commit_sha FROM finalize_requests ORDER BY created_at DESC LIMIT 1"
            ).fetchone()
        self.assertEqual("finalizing", pending["status"])
        self.assertEqual(moved_head, pending["ref_updated_sha"])
        self.assertIsNone(pending["commit_sha"])
        self.baselines.accept_commit = original_accept
        self._authorize_recovery_process()
        observed_recovery_owners: list[str] = []
        original_restore = self.service._restore_index

        def restore_recovered_commit_while_owned(path, existed, content) -> None:
            with self.database.connect() as connection:
                owner = connection.execute(
                    "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
                ).fetchone()["owner_id"]
            observed_recovery_owners.append(owner)
            original_restore(path, existed, content)

        self.service._restore_index = restore_recovered_commit_while_owned  # type: ignore[method-assign]

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(0, recovered)
        self.assertEqual(1, len(observed_recovery_owners))
        self.assertTrue(observed_recovery_owners[0].startswith("recovery:"))
        with self.database.connect() as connection:
            committed = connection.execute(
                "SELECT status, commit_sha, index_snapshot FROM finalize_requests WHERE ref_updated_sha = ?",
                (moved_head,),
            ).fetchone()
        self.assertEqual("committed", committed["status"])
        self.assertEqual(moved_head, committed["commit_sha"])
        self.assertIsNone(committed["index_snapshot"])

    def test_restart_recovers_recreated_index_lock_before_resetting_paths(self) -> None:
        paths = self._complete_with_changes()
        unrelated_path = self.repo / "unrelated-staged.txt"
        unrelated_path.write_text("preserve this staged blob\n", encoding="utf-8")
        subprocess.run(
            ["git", "add", "--", unrelated_path.name], cwd=self.repo, check=True
        )
        original_staged_blob = subprocess.run(
            ["git", "show", f":{unrelated_path.name}"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        ).stdout
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(
            side_effect=RuntimeError("injected baseline failure")
        )
        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a",
                paths=paths,
                message="feat(runtime): recovery lock fixture",
                maintenance=True,
            )
        self.baselines.accept_commit = original_accept
        lock_path = self.service._index_path().with_name("index.lock")
        lock_path.write_bytes(b"")
        recovered_locks: list[Path] = []

        def recover_recreated_lock(path: Path):
            recovered_locks.append(path)
            path.unlink()
            return None

        self.service.index_lock_recoverer = recover_recreated_lock
        self._authorize_recovery_process()

        recovered = self.service.recover_stale_mutex()

        self.assertEqual(0, recovered)
        self.assertEqual([lock_path], recovered_locks)
        self.assertFalse(lock_path.exists())
        self.assertEqual(unrelated_path.name, self._staged_names())
        recovered_staged_blob = subprocess.run(
            ["git", "show", f":{unrelated_path.name}"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        ).stdout
        self.assertEqual(original_staged_blob, recovered_staged_blob)
        self.assertIsNone(self._mutex_owner())

    def test_recovery_keeps_pending_when_baseline_retry_fails(self) -> None:
        paths = self._complete_with_changes()
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("first baseline failure"))
        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): add feature"
            )
        self.baselines.accept_commit = mock.Mock(side_effect=RuntimeError("retry failure"))
        self._authorize_recovery_process()

        with self.assertRaises(RuntimeError):
            self.service.recover_stale_mutex()

        with self.database.connect() as connection:
            recovery_owner = connection.execute(
                "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
            ).fetchone()["owner_id"]
            pending = connection.execute(
                "SELECT status, commit_sha, ref_updated_sha FROM finalize_requests ORDER BY created_at DESC LIMIT 1"
            ).fetchone()
        self.assertTrue(recovery_owner.startswith("recovery:"))
        self.assertEqual("finalizing", pending["status"])
        self.assertIsNone(pending["commit_sha"])
        self.assertEqual(self._head(), pending["ref_updated_sha"])
        self.baselines.accept_commit = original_accept

    def test_forward_reconcile_restores_index_before_clearing_snapshot(self) -> None:
        paths = self._complete_with_changes()
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(
            side_effect=RuntimeError("injected reconcile fixture failure")
        )
        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): reconcile fixture"
            )
        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT request_id, index_snapshot FROM finalize_requests
                   ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertIsNotNone(request["index_snapshot"])
        self.baselines.accept_commit = original_accept
        observed_owners: list[str] = []
        original_restore = self.service._restore_index

        def restore_while_reconcile_owns_mutex(path, existed, content) -> None:
            with self.database.connect() as connection:
                owner = connection.execute(
                    "SELECT owner_id FROM git_mutex WHERE lock_name='index'"
                ).fetchone()["owner_id"]
            observed_owners.append(owner)
            original_restore(path, existed, content)

        self.service._restore_index = restore_while_reconcile_owns_mutex  # type: ignore[method-assign]

        result = self.service.reconcile_request(request["request_id"])

        self.assertIsNotNone(result)
        self.assertEqual([f"reconcile:{request['request_id']}"], observed_owners)
        with self.database.connect() as connection:
            reconciled = connection.execute(
                "SELECT status, index_snapshot FROM finalize_requests WHERE request_id=?",
                (request["request_id"],),
            ).fetchone()
        self.assertEqual("committed", reconciled["status"])
        self.assertIsNone(reconciled["index_snapshot"])

    def test_forward_reconcile_restore_failure_retains_mutex_and_snapshot(self) -> None:
        paths = self._complete_with_changes()
        original_accept = self.baselines.accept_commit
        self.baselines.accept_commit = mock.Mock(
            side_effect=RuntimeError("injected reconcile fixture failure")
        )
        with self.assertRaises(RuntimeError):
            self.service.finalize(
                "session-a", paths=paths, message="feat(runtime): retained reconcile fixture"
            )
        with self.database.connect() as connection:
            request = connection.execute(
                """SELECT request_id, index_snapshot FROM finalize_requests
                   ORDER BY created_at DESC LIMIT 1"""
            ).fetchone()
        self.assertIsNotNone(request["index_snapshot"])
        self.baselines.accept_commit = original_accept

        with mock.patch.object(
            self.service,
            "_restore_index",
            side_effect=RuntimeError("injected reconcile restore failure"),
        ):
            with self.assertRaises(RuntimeError):
                self.service.reconcile_request(request["request_id"])

        self.assertEqual(
            f"reconcile:{request['request_id']}", self._mutex_owner()
        )
        with self.database.connect() as connection:
            pending = connection.execute(
                "SELECT status, index_snapshot FROM finalize_requests WHERE request_id=?",
                (request["request_id"],),
            ).fetchone()
        self.assertEqual("finalizing", pending["status"])
        self.assertIsNotNone(pending["index_snapshot"])

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
