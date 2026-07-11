from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.patches import PatchService, PatchStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.watch import WorkspaceWatcher


def replacement_patch(old: str, new: str) -> str:
    return (
        "diff --git a/README.md b/README.md\n"
        "--- a/README.md\n"
        "+++ b/README.md\n"
        "@@ -1 +1 @@\n"
        f"-{old}\n"
        f"+{new}\n"
    )


class PatchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(self.config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        for session_id in ("session-a", "session-b"):
            self.sessions.register(session_id=session_id)
            self.sessions.set_status(session_id, SessionStatus.ACTIVE)
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        self.store = ObjectStore(self.database, self.config.object_root)
        self.snapshots = SnapshotService(self.database, self.repo, self.store)
        self.leases = LeaseService(self.database, PathPolicy(self.repo), ttl_seconds=300, grace_seconds=120)
        self.patches = PatchService(
            self.database,
            self.repo,
            self.store,
            self.snapshots,
            self.leases,
            self.sessions,
        )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def test_queued_patch_applies_after_owner_releases_unchanged_file(self) -> None:
        self.assertTrue(self.leases.acquire("session-a", ["README.md"]).acquired)
        queued = self.patches.submit(
            "session-b", replacement_patch("baseline", "patched"), ["README.md"]
        )
        self.leases.release("session-a", ["README.md"])

        processed = self.patches.process_queue()

        self.assertEqual(PatchStatus.QUEUED, queued.status)
        self.assertEqual(PatchStatus.APPLIED, processed[0].status)
        self.assertEqual("patched\n", (self.repo / "README.md").read_text(encoding="utf-8"))
        self.assertEqual([], WorkspaceWatcher(self.baselines).scan_once())
        self.assertEqual("healthy", self.baselines.current().health.value)

    def test_queued_patch_never_overwrites_changed_base(self) -> None:
        self.assertTrue(self.leases.acquire("session-a", ["README.md"]).acquired)
        queued = self.patches.submit(
            "session-b", replacement_patch("baseline", "patched"), ["README.md"]
        )
        (self.repo / "README.md").write_text("owner change\n", encoding="utf-8")
        self.leases.release("session-a", ["README.md"])

        processed = self.patches.process_queue()
        current = self.patches.get(queued.patch_id)

        self.assertEqual(PatchStatus.NEEDS_REBASE, processed[0].status)
        self.assertIsNotNone(current.current_objects)
        self.assertEqual("owner change\n", (self.repo / "README.md").read_text(encoding="utf-8"))

    def test_immediate_patch_rechecks_hash_after_acquiring_lease(self) -> None:
        original_acquire = self.leases.acquire

        def acquire_then_external_edit(*args, **kwargs):
            result = original_acquire(*args, **kwargs)
            (self.repo / "README.md").write_text("external race\n", encoding="utf-8")
            return result

        with mock.patch.object(self.leases, "acquire", side_effect=acquire_then_external_edit):
            patch = self.patches.submit(
                "session-a", replacement_patch("baseline", "patched"), ["README.md"]
            )

        self.assertEqual(PatchStatus.NEEDS_REBASE, patch.status)
        self.assertEqual("external race\n", (self.repo / "README.md").read_text(encoding="utf-8"))

    def test_queued_patch_survives_service_reconstruction(self) -> None:
        self.assertTrue(self.leases.acquire("session-a", ["README.md"]).acquired)
        queued = self.patches.submit(
            "session-b", replacement_patch("baseline", "after restart"), ["README.md"]
        )
        self.leases.release("session-a", ["README.md"])
        reconstructed_leases = LeaseService(
            self.database, PathPolicy(self.repo), ttl_seconds=300, grace_seconds=120
        )
        reconstructed = PatchService(
            self.database,
            self.repo,
            ObjectStore(self.database, self.config.object_root),
            SnapshotService(
                self.database,
                self.repo,
                ObjectStore(self.database, self.config.object_root),
            ),
            reconstructed_leases,
            SessionService(self.database, self.repo),
        )

        processed = reconstructed.process_queue()

        self.assertEqual(PatchStatus.QUEUED, queued.status)
        self.assertEqual(PatchStatus.APPLIED, processed[0].status)
        self.assertEqual(
            "after restart\n", (self.repo / "README.md").read_text(encoding="utf-8")
        )


if __name__ == "__main__":
    unittest.main()
