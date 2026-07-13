from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineHealth, BaselineService
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
        original_build = self.service.build_manifest
        call_count = 0

        def changing_manifest():
            nonlocal call_count
            call_count += 1
            if call_count == 2:
                (self.repo / "foreign.txt").write_text("foreign\n", encoding="utf-8")
            return original_build()

        with mock.patch.object(
            self.service, "build_manifest", side_effect=changing_manifest
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


if __name__ == "__main__":
    unittest.main()
