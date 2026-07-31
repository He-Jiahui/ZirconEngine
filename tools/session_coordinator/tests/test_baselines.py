from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineHealth, BaselineService, hash_bytes
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class BaselineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.service = BaselineService(self.database, self.repo)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_external_change_degrades_then_accept_opens_new_healthy_epoch(self) -> None:
        first = self.service.initialize()
        (self.repo / "README.md").write_text("external\n", encoding="utf-8")

        changes = self.service.scan()
        degraded = self.service.current()
        self.service.attribute("session-a", ["README.md"])
        accepted = self.service.accept(reason="attribute known Session change")

        self.assertEqual(BaselineHealth.HEALTHY, first.health)
        self.assertEqual(["README.md"], [change.path for change in changes])
        self.assertEqual(BaselineHealth.DEGRADED, degraded.health)
        self.assertGreater(accepted.epoch_id, first.epoch_id)
        self.assertEqual(BaselineHealth.HEALTHY, accepted.health)
        restarted = BaselineService(self.database, self.repo)
        self.assertEqual([], restarted.diff())

    def test_reconcile_restores_health_without_absorbing_attributed_change(self) -> None:
        initial = self.service.initialize()
        (self.repo / "README.md").write_text("attributed dirty\n", encoding="utf-8")
        self.service.scan()
        self.service.attribute("session-a", ["README.md"])

        reconciled = self.service.reconcile_health()

        self.assertEqual(initial.epoch_id, reconciled.epoch_id)
        self.assertEqual(BaselineHealth.HEALTHY, reconciled.health)
        self.assertEqual(["README.md"], [item.path for item in self.service.diff()])

    def test_head_change_creates_new_epoch_without_losing_manifest(self) -> None:
        first = self.service.initialize()
        (self.repo / "second.txt").write_text("second\n", encoding="utf-8")
        subprocess.run(["git", "add", "second.txt"], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-q", "-m", "test: second"], cwd=self.repo, check=True)

        second = self.service.refresh_for_head_change()

        self.assertGreater(second.epoch_id, first.epoch_id)
        self.assertIn("second.txt", second.manifest)
        self.assertEqual(BaselineHealth.HEALTHY, second.health)

    def test_head_refresh_hashes_only_changed_paths_instead_of_archiving_head(self) -> None:
        self.service.initialize()
        for index in range(3):
            (self.repo / f"archive-{index}.txt").write_text(
                f"archive {index}\n", encoding="utf-8"
            )
        subprocess.run(["git", "add", "archive-0.txt", "archive-1.txt", "archive-2.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add archive refresh fixture"],
            cwd=self.repo,
            check=True,
        )
        original_run = subprocess.run

        cat_file_paths: list[str] = []

        def guarded_run(arguments, *args, **kwargs):
            if len(arguments) > 1 and arguments[1] == "archive":
                raise AssertionError("head refresh must not rebuild the full archive")
            if len(arguments) > 1 and arguments[1] == "cat-file":
                cat_file_paths.append(arguments[-1])
            return original_run(arguments, *args, **kwargs)

        with mock.patch("tools.session_coordinator.baselines.subprocess.run", side_effect=guarded_run):
            refreshed = self.service.refresh_for_head_change()

        self.assertIn("archive-2.txt", refreshed.manifest)
        self.assertEqual(3, len(cat_file_paths))

    def test_archive_manifest_preserves_git_worktree_filters(self) -> None:
        (self.repo / ".gitattributes").write_text(
            "filtered.txt working-tree-encoding=UTF-16LE\n", encoding="utf-8"
        )
        (self.repo / "filtered.txt").write_text("line1\nline2\n", encoding="utf-8")
        subprocess.run(["git", "add", ".gitattributes", "filtered.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add filtered baseline fixture"],
            cwd=self.repo,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        filtered = subprocess.run(
            ["git", "cat-file", "--filters", "--path=filtered.txt", f"{commit}:filtered.txt"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        ).stdout

        manifest = self.service._commit_manifest(commit)

        self.assertEqual(hash_bytes(filtered), manifest["filtered.txt"])

    def test_head_refresh_never_absorbs_other_session_dirty_worktree(self) -> None:
        dirty = self.repo / "other-session.txt"
        dirty.write_text("stable\n", encoding="utf-8")
        subprocess.run(["git", "add", "other-session.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: tracked fixture"],
            cwd=self.repo,
            check=True,
        )
        self.service.initialize()
        dirty.write_text("other Session dirty\n", encoding="utf-8")
        (self.repo / "README.md").write_text("new committed head\n", encoding="utf-8")
        subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: advance head"],
            cwd=self.repo,
            check=True,
        )

        refreshed = self.service.refresh_for_head_change()
        changed = {item.path for item in self.service.diff()}

        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.assertEqual(head, refreshed.head_commit)
        self.assertIn("other-session.txt", changed)

    def test_accept_commit_refreshes_all_paths_advanced_by_shared_head(self) -> None:
        foreign = self.repo / "foreign.txt"
        foreign.write_text("foreign before\n", encoding="utf-8")
        subprocess.run(["git", "add", "foreign.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add foreign fixture"],
            cwd=self.repo,
            check=True,
        )
        self.service.initialize()

        (self.repo / "README.md").write_text("owned commit\n", encoding="utf-8")
        foreign.write_text("foreign commit\n", encoding="utf-8")
        subprocess.run(
            ["git", "add", "README.md", "foreign.txt"], cwd=self.repo, check=True
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: shared head advance"],
            cwd=self.repo,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

        accepted = self.service.accept_commit(
            ["README.md"], commit_sha=commit, reason="owned scope committed"
        )

        self.assertEqual(
            hash_bytes(foreign.read_bytes()), accepted.manifest["foreign.txt"]
        )
        self.assertEqual([], self.service.diff())

    def test_accept_commit_updates_from_changed_git_paths_without_full_archive(self) -> None:
        foreign = self.repo / "foreign.txt"
        removed = self.repo / "removed.txt"
        foreign.write_text("foreign before\n", encoding="utf-8")
        removed.write_text("removed before\n", encoding="utf-8")
        subprocess.run(["git", "add", "foreign.txt", "removed.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add incremental baseline fixtures"],
            cwd=self.repo,
            check=True,
        )
        self.service.initialize()

        (self.repo / "README.md").write_text("owned commit\n", encoding="utf-8")
        foreign.write_text("foreign after\n", encoding="utf-8")
        (self.repo / "added.txt").write_text("added after\n", encoding="utf-8")
        removed.unlink()
        subprocess.run(
            ["git", "add", "README.md", "foreign.txt", "added.txt", "removed.txt"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: incrementally advance baseline"],
            cwd=self.repo,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

        with mock.patch.object(
            self.service,
            "_commit_manifest",
            side_effect=AssertionError("managed commit must not rebuild the full archive"),
        ):
            accepted = self.service.accept_commit(
                ["README.md"], commit_sha=commit, reason="owned scope committed"
            )

        self.assertEqual(hash_bytes(foreign.read_bytes()), accepted.manifest["foreign.txt"])
        self.assertEqual(hash_bytes((self.repo / "added.txt").read_bytes()), accepted.manifest["added.txt"])
        self.assertNotIn("removed.txt", accepted.manifest)
        self.assertEqual([], self.service.diff())

    def test_accept_commit_rebuilds_when_attributes_change_filtered_hashes(self) -> None:
        filtered = self.repo / "filtered.txt"
        filtered.write_text("line1\nline2\n", encoding="utf-8")
        subprocess.run(["git", "add", "filtered.txt"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add filtered baseline fixture"],
            cwd=self.repo,
            check=True,
        )
        initial = self.service.initialize()

        (self.repo / ".gitattributes").write_text(
            "filtered.txt text eol=crlf\n", encoding="utf-8"
        )
        subprocess.run(["git", "add", ".gitattributes"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add commit filters"],
            cwd=self.repo,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        expected = subprocess.run(
            ["git", "cat-file", "--filters", "--path=filtered.txt", f"{commit}:filtered.txt"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        ).stdout

        with mock.patch.object(
            self.service,
            "_commit_manifest",
            wraps=self.service._commit_manifest,
        ) as archive:
            accepted = self.service.accept_commit(
                [".gitattributes"], commit_sha=commit, reason="attributes committed"
            )

        self.assertEqual(1, archive.call_count)
        self.assertEqual(initial.manifest["filtered.txt"], hash_bytes(expected))
        self.assertEqual(hash_bytes(expected), accepted.manifest["filtered.txt"])

    def test_prior_epoch_attribution_cannot_reconcile_reappearing_content(self) -> None:
        self.service.initialize()
        (self.repo / "README.md").write_text("old attributed\n", encoding="utf-8")
        self.service.scan()
        self.service.attribute("session-a", ["README.md"])
        subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: advance baseline"],
            cwd=self.repo,
            check=True,
        )
        self.service.refresh_for_head_change()
        (self.repo / "README.md").write_text("old attributed\n", encoding="utf-8")
        # Advance the committed baseline to different content while preserving
        # the old attribution row from the prior epoch.
        (self.repo / "README.md").write_text("new committed baseline\n", encoding="utf-8")
        subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: second baseline"],
            cwd=self.repo,
            check=True,
        )
        self.service.refresh_for_head_change()
        (self.repo / "README.md").write_text("old attributed\n", encoding="utf-8")
        self.service.scan()

        with self.assertRaises(Exception):
            self.service.reconcile_health()

        self.assertEqual(BaselineHealth.DEGRADED, self.service.current().health)

    def test_reconcile_refuses_workspace_change_between_hash_passes(self) -> None:
        self.service.initialize()
        (self.repo / "README.md").write_text("owned\n", encoding="utf-8")
        self.service.scan()
        self.service.attribute("session-a", ["README.md"])
        original_build = self.service._workspace_manifest_from_baseline
        call_count = 0

        def changing_manifest(*arguments):
            nonlocal call_count
            call_count += 1
            if call_count == 2:
                (self.repo / "foreign.txt").write_text("foreign\n", encoding="utf-8")
            return original_build(*arguments)

        with mock.patch.object(
            self.service,
            "_workspace_manifest_from_baseline",
            side_effect=changing_manifest,
        ):
            with self.assertRaises(Exception):
                self.service.reconcile_health()

        self.assertEqual(BaselineHealth.DEGRADED, self.service.current().health)

    def test_degraded_background_scan_reuses_baseline_until_head_changes(self) -> None:
        self.service.initialize()
        (self.repo / "README.md").write_text("external\n", encoding="utf-8")
        self.service.scan()

        with mock.patch.object(
            self.service,
            "build_manifest",
            side_effect=AssertionError("degraded background scan must not hash every file"),
        ):
            observation = self.service.prepare_scan()
        result = self.service.apply_scan(observation)

        self.assertTrue(result.applied)
        self.assertEqual((), result.changes)
        self.assertEqual(BaselineHealth.DEGRADED, self.service.current().health)

    def test_diff_hashes_only_git_reported_workspace_candidates(self) -> None:
        self.service.initialize()
        (self.repo / "README.md").write_text("dirty candidate\n", encoding="utf-8")

        with mock.patch.object(
            self.service,
            "build_manifest",
            side_effect=AssertionError("diff must not hash the full workspace"),
        ):
            changes = self.service.diff()

        self.assertEqual(["README.md"], [item.path for item in changes])

    def test_scan_repairs_a_stale_manifest_when_head_is_unchanged(self) -> None:
        initial = self.service.initialize()
        stale_manifest = dict(initial.manifest)
        stale_manifest["README.md"] = hash_bytes(b"obsolete baseline bytes\n")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE baseline_epochs SET manifest_json=? WHERE epoch_id=?",
                (json.dumps(stale_manifest, sort_keys=True), initial.epoch_id),
            )

        result = self.service.apply_scan(self.service.prepare_scan())
        repaired = self.service.current()

        self.assertTrue(result.applied)
        self.assertEqual((), result.changes)
        self.assertEqual(
            hash_bytes((self.repo / "README.md").read_bytes()),
            repaired.manifest["README.md"],
        )


if __name__ == "__main__":
    unittest.main()
