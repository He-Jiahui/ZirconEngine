from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from datetime import timedelta
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus, utc_now
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
        self.assertNotIn("README.md", self.sessions.get("session-b").write_scope)
        self.leases.release("session-a", ["README.md"])

        processed = self.patches.process_queue()

        self.assertEqual(PatchStatus.QUEUED, queued.status)
        self.assertEqual(PatchStatus.APPLIED, processed[0].status)
        self.assertEqual("patched\n", (self.repo / "README.md").read_text(encoding="utf-8"))
        self.assertEqual([], WorkspaceWatcher(self.baselines).scan_once())
        self.assertEqual("healthy", self.baselines.current().health.value)
        self.assertIn("README.md", self.sessions.get("session-b").write_scope)

    def test_applied_patch_adds_exact_target_to_existing_scope_and_audits_it(self) -> None:
        self.sessions.register(session_id="session-c", write_scope=("docs/owned",))
        self.sessions.set_status("session-c", SessionStatus.ACTIVE)

        applied = self.patches.submit(
            "session-c", replacement_patch("baseline", "patched"), ["README.md"]
        )

        self.assertEqual(PatchStatus.APPLIED, applied.status)
        self.assertEqual(
            ("docs/owned", "README.md"), self.sessions.get("session-c").write_scope
        )
        with self.database.connect() as connection:
            attribution = connection.execute(
                "SELECT session_id FROM attributions WHERE path_key='readme.md'"
            ).fetchone()
            event = connection.execute(
                "SELECT payload_json FROM events WHERE session_id=? AND event_type=?",
                ("session-c", "session.write_scope_transferred"),
            ).fetchone()
        self.assertEqual("session-c", attribution["session_id"])
        self.assertEqual(
            {"paths": ["README.md"], "transferFingerprint": applied.patch_object_hash},
            json.loads(event["payload_json"]),
        )

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
        self.assertNotIn("README.md", self.sessions.get("session-b").write_scope)

    def test_immediate_patch_rechecks_hash_after_acquiring_lease(self) -> None:
        original_acquire = self.leases.acquire_in_connection

        def acquire_then_external_edit(*args, **kwargs):
            result = original_acquire(*args, **kwargs)
            (self.repo / "README.md").write_text("external race\n", encoding="utf-8")
            return result

        with mock.patch.object(
            self.leases, "acquire_in_connection", side_effect=acquire_then_external_edit
        ):
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

    def test_process_queue_can_pin_the_previewed_patch_set(self) -> None:
        self.assertTrue(self.leases.acquire("session-a", ["README.md"]).acquired)
        previewed = self.patches.submit(
            "session-b", replacement_patch("baseline", "previewed"), ["README.md"]
        )
        later = self.patches.submit(
            "session-b", replacement_patch("baseline", "later"), ["README.md"]
        )
        self.leases.release("session-a", ["README.md"])

        processed = self.patches.process_queue(
            session_id="session-b", patch_ids=(previewed.patch_id,)
        )

        self.assertEqual([previewed.patch_id], [item.patch_id for item in processed])
        self.assertEqual(PatchStatus.QUEUED, self.patches.get(later.patch_id).status)

    def test_after_snapshot_error_records_rebase_without_replaying_git(self) -> None:
        create = self.snapshots.create

        def fail_after_snapshot(**kwargs):
            if kwargs["purpose"].startswith("after patch "):
                raise OSError("injected snapshot failure")
            return create(**kwargs)

        with mock.patch.object(self.snapshots, "create", side_effect=fail_after_snapshot):
            with self.assertRaisesRegex(OSError, "injected snapshot failure"):
                self.patches.submit(
                    "session-a", replacement_patch("baseline", "patched"), ["README.md"]
                )

        interrupted = self.patches.list(session_id="session-a")[0]
        self.assertEqual(PatchStatus.NEEDS_REBASE, interrupted.status)
        self.assertEqual("patched\n", (self.repo / "README.md").read_text(encoding="utf-8"))
        self.assertEqual(
            (self.repo / "README.md").read_bytes(),
            self.store.get(interrupted.current_objects["README.md"]),
        )
        self.assertIn("injected snapshot failure", interrupted.error_text)
        self.assertEqual([], self.patches.process_queue())
        self.assertNotIn("README.md", self.sessions.get("session-a").write_scope)

    def test_submit_object_failure_does_not_acquire_an_unrecorded_lease(self) -> None:
        with mock.patch.object(self.store, "put", side_effect=OSError("object write failed")):
            with self.assertRaisesRegex(OSError, "object write failed"):
                self.patches.submit(
                    "session-a", replacement_patch("baseline", "patched"), ["README.md"]
                )

        self.assertEqual([], self.patches.list())
        self.assertEqual([], self.leases.owned_paths("session-a"))
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_application_start_error_rolls_back_its_lease(self) -> None:
        with mock.patch.object(
            self.patches, "_application_event", side_effect=OSError("start audit failed")
        ):
            with self.assertRaisesRegex(OSError, "start audit failed"):
                self.patches.submit(
                    "session-a", replacement_patch("baseline", "patched"), ["README.md"]
                )

        self.assertEqual(PatchStatus.QUEUED, self.patches.list()[0].status)
        self.assertEqual([], self.leases.owned_paths("session-a"))
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_application_lock_error_leaves_retryable_request_without_lease(self) -> None:
        with mock.patch.object(
            self.patches, "_application_lock", side_effect=OSError("lock unavailable")
        ):
            with self.assertRaisesRegex(OSError, "lock unavailable"):
                self.patches.submit(
                    "session-a", replacement_patch("baseline", "patched"), ["README.md"]
                )

        self.assertEqual(PatchStatus.QUEUED, self.patches.list()[0].status)
        self.assertEqual([], self.leases.owned_paths("session-a"))
        self.assertEqual(PatchStatus.APPLIED, self.patches.process_queue()[0].status)

    def test_queued_hash_error_records_rebase_and_releases_its_lease(self) -> None:
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)
        queued = self.patches.submit(
            "session-a", replacement_patch("baseline", "patched"), ["README.md"]
        )
        self.leases.release("session-b", ["README.md"])

        with mock.patch(
            "tools.session_coordinator.patches.hash_file", side_effect=OSError("hash read failed")
        ):
            with self.assertRaisesRegex(OSError, "hash read failed"):
                self.patches.process_queue()

        self.assertEqual(PatchStatus.NEEDS_REBASE, self.patches.get(queued.patch_id).status)
        self.assertEqual("baseline\n", (self.repo / "README.md").read_text(encoding="utf-8"))
        self.assertEqual([], self.leases.owned_paths("session-a"))
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_scope_transaction_error_rolls_back_attribution_and_applied_state(self) -> None:
        extend_scope = self.sessions.extend_write_scope_in_connection

        def fail_after_scope(*args, **kwargs):
            extend_scope(*args, **kwargs)
            raise RuntimeError("injected scope transaction failure")

        with mock.patch.object(
            self.sessions, "extend_write_scope_in_connection", side_effect=fail_after_scope
        ):
            with self.assertRaisesRegex(RuntimeError, "injected scope transaction failure"):
                self.patches.submit(
                    "session-a", replacement_patch("baseline", "patched"), ["README.md"]
                )

        self.assertEqual(PatchStatus.NEEDS_REBASE, self.patches.list()[0].status)
        self.assertNotIn("README.md", self.sessions.get("session-a").write_scope)
        with self.database.connect() as connection:
            self.assertIsNone(connection.execute("SELECT 1 FROM attributions").fetchone())

    def test_hard_exit_after_git_is_recovered_once_without_changing_worktree(self) -> None:
        self._crash_patch_application("after_snapshot")
        interrupted = self.patches.list()[0]
        self.assertEqual(PatchStatus.APPLYING, interrupted.status)
        self.assertEqual(["README.md"], self.leases.owned_paths("session-a"))

        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named", return_value=()
        ):
            reconstructed = self._reconstruct_patches()

        recovered = reconstructed.get(interrupted.patch_id)
        self.assertEqual(PatchStatus.NEEDS_REBASE, recovered.status)
        self.assertEqual(
            (self.repo / "README.md").read_bytes(),
            self.store.get(recovered.current_objects["README.md"]),
        )
        self.assertEqual("patched\n", (self.repo / "README.md").read_text(encoding="utf-8"))
        self.assertNotIn("README.md", self.sessions.get("session-a").write_scope)
        self.assertEqual([], reconstructed.process_queue())
        self._reconstruct_patches()
        with self.database.connect() as connection:
            count = connection.execute(
                "SELECT COUNT(*) FROM events WHERE event_type='patch.application_interrupted'"
            ).fetchone()[0]
        self.assertEqual(1, count)
        self.assertEqual([], self.leases.owned_paths("session-a"))
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_hard_exit_after_applied_transaction_keeps_complete_ownership(self) -> None:
        self._crash_patch_application("after_attribution_commit")

        self.assertEqual([], self.leases.owned_paths("session-a"))
        reconstructed = self._reconstruct_patches()

        self.assertEqual(PatchStatus.APPLIED, reconstructed.list()[0].status)
        self.assertIn("README.md", self.sessions.get("session-a").write_scope)
        with self.database.connect() as connection:
            owner = connection.execute(
                "SELECT session_id FROM attributions WHERE path_key='readme.md'"
            ).fetchone()[0]
        self.assertEqual("session-a", owner)
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_interrupted_recovery_preserves_a_renewed_session_lease(self) -> None:
        self._crash_patch_application("after_snapshot")
        self.leases.heartbeat("session-a", now=utc_now() + timedelta(seconds=1))
        renewed = self.leases.list()

        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named", return_value=()
        ):
            reconstructed = self._reconstruct_patches()

        self.assertEqual(PatchStatus.NEEDS_REBASE, reconstructed.list()[0].status)
        self.assertEqual(renewed, self.leases.list())
        self.assertFalse(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_interrupted_recovery_preserves_a_replacement_lease(self) -> None:
        self._crash_patch_application("after_snapshot")
        self.leases.release("session-a", ["README.md"])
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)
        replacement = self.leases.list()

        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named", return_value=()
        ):
            reconstructed = self._reconstruct_patches()

        self.assertEqual(PatchStatus.NEEDS_REBASE, reconstructed.list()[0].status)
        self.assertEqual(replacement, self.leases.list())

    def test_interrupted_recovery_defers_while_a_git_child_may_still_write(self) -> None:
        self._crash_patch_application("after_snapshot")
        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named", return_value=(123,)
        ):
            reconstructed = self._reconstruct_patches()
        self.assertEqual(PatchStatus.APPLYING, reconstructed.list()[0].status)
        self.assertEqual(["README.md"], self.leases.owned_paths("session-a"))
        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named", return_value=()
        ):
            self.assertEqual([], reconstructed.process_queue())
        self.assertEqual(PatchStatus.NEEDS_REBASE, reconstructed.list()[0].status)
        self.assertTrue(self.leases.acquire("session-b", ["README.md"]).acquired)

    def test_interrupted_recovery_retains_state_when_git_liveness_is_unknown(self) -> None:
        self._crash_patch_application("after_snapshot")
        with mock.patch(
            "tools.session_coordinator.patches.live_process_ids_named",
            side_effect=OSError("process enumeration unavailable"),
        ):
            reconstructed = self._reconstruct_patches()
        self.assertEqual(PatchStatus.APPLYING, reconstructed.list()[0].status)
        self.assertEqual(["README.md"], self.leases.owned_paths("session-a"))

    def test_reconstruction_does_not_interrupt_a_live_applying_process(self) -> None:
        create = self.snapshots.create

        def reconstruct_during_apply(**kwargs):
            if kwargs["purpose"].startswith("after patch "):
                other = self._reconstruct_patches()
                self.assertEqual(PatchStatus.APPLYING, other.list()[0].status)
                self.assertFalse(self.leases.acquire("session-b", ["README.md"]).acquired)
            return create(**kwargs)

        with mock.patch.object(self.snapshots, "create", side_effect=reconstruct_during_apply):
            result = self.patches.submit(
                "session-a", replacement_patch("baseline", "patched"), ["README.md"]
            )
        self.assertEqual(PatchStatus.APPLIED, result.status)

    def _reconstruct_patches(self):
        return PatchService(
            self.database, self.repo, self.store, self.snapshots, self.leases, self.sessions
        )

    def _crash_patch_application(self, phase: str) -> None:
        script = r'''
import os, sys
from contextlib import contextmanager
from pathlib import Path
from tools.session_coordinator.database import Database
from tools.session_coordinator.leases import LeaseService, PathPolicy
from tools.session_coordinator.patches import PatchService
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.snapshots import ObjectStore, SnapshotService
from tools.session_coordinator.tests.test_patches import replacement_patch

repo, database_path, object_root, phase = sys.argv[1:]
database = Database(database_path)
store = ObjectStore(database, object_root)
snapshots = SnapshotService(database, repo, store)
sessions = SessionService(database, repo)
leases = LeaseService(database, PathPolicy(repo), ttl_seconds=300, grace_seconds=120)
patches = PatchService(database, repo, store, snapshots, leases, sessions)
create = snapshots.create
def create_or_exit(**kwargs):
    if kwargs["purpose"].startswith("after patch "):
        os._exit(73)
    return create(**kwargs)
if phase == "after_snapshot":
    snapshots.create = create_or_exit
else:
    transaction = database.transaction
    @contextmanager
    def exit_after_attribution_commit(**kwargs):
        with transaction(**kwargs) as connection:
            yield connection
        with database.connect() as connection:
            if connection.execute("SELECT 1 FROM attributions").fetchone():
                os._exit(73)
    database.transaction = exit_after_attribution_commit
patches.submit("session-a", replacement_patch("baseline", "patched"), ["README.md"])
'''
        result = subprocess.run(
            [
                sys.executable, "-B", "-c", script, str(self.repo),
                str(self.config.database_path), str(self.config.object_root), phase,
            ],
            cwd=Path(__file__).resolve().parents[3],
            check=False,
            capture_output=True,
            timeout=30,
        )
        self.assertEqual(73, result.returncode, result.stderr.decode(errors="replace"))


if __name__ == "__main__":
    unittest.main()
