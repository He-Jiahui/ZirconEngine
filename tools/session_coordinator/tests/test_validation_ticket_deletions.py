from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.tests.test_validation_tickets import _FakeWorkspaceCopy
from tools.session_coordinator.validation_ticket_worker import ValidationTicketWorker
from tools.session_coordinator.validation_tickets import ValidationTicketService
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


class ValidationTicketDeletionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.database = Database(Path(self.temporary.name) / "coordinator.sqlite3")
        self.repo = Path(self.temporary.name) / "repo"
        self.repo.mkdir()
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at, last_heartbeat_at
                ) VALUES (
                    'primary', 'docs/plans/mvp/00-current-source-baseline-recovery.md', 'active',
                    '2026-08-03T00:00:00+00:00', '2026-08-03T00:00:00+00:00',
                    '2026-08-03T00:00:00+00:00'
                )
                """
            )
        self.service = ValidationTicketService(self.database)
        self.workspace_copy = _FakeWorkspaceCopy(Path(self.temporary.name) / "copies")
        self.worker = ValidationTicketWorker(
            self.database,
            self.repo,
            self.service,
            self.workspace_copy,
            run_result_lookup=self.workspace_copy.run_result,
        )

    def test_tombstone_is_persisted_and_deduplicated_deterministically(self) -> None:
        first = self._submit(
            "deleted-request-a",
            {
                "zircon_runtime/src/owned.rs": "A" * 64,
                "zircon_runtime/src/deleted.rs": None,
            },
        )
        merged = self._submit(
            "deleted-request-b",
            {
                "zircon_runtime/src/deleted.rs": None,
                "zircon_runtime/src/owned.rs": "a" * 64,
            },
        )
        with self.database.connect() as connection:
            stored = connection.execute(
                "SELECT source_manifest_json FROM validation_tickets WHERE ticket_id=?",
                (first.ticket.ticket_id,),
            ).fetchone()[0]

        self.assertTrue(merged.reused)
        self.assertEqual(first.ticket.ticket_id, merged.ticket.ticket_id)
        self.assertEqual(
            {
                "zircon_runtime/src/deleted.rs": None,
                "zircon_runtime/src/owned.rs": "a" * 64,
            },
            json.loads(stored),
        )

    def test_manifest_rejects_non_null_non_sha_values(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self._submit(
                "invalid-manifest-value",
                {"zircon_runtime/src/owned.rs": 1},  # type: ignore[dict-item]
            )

        self.assertEqual("validation_ticket_manifest_invalid", rejected.exception.code)

    def test_reappearance_during_copy_is_snapshot_stale(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self._submit("copy-race-request", {deleted_path: None})
        self.worker.tick()
        copy = next(iter(self.workspace_copy.records.values()))
        copy.status = "failed"
        copy.error_code = "validation_copy_owned_source_reappeared"
        copy.error_stage = "owned_overlay"
        copy.error_path = deleted_path

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )

    def test_reappearance_in_materialized_copy_is_snapshot_stale(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self._submit("materialized-copy-race", {deleted_path: None})
        self.worker.tick()
        copy = next(iter(self.workspace_copy.records.values()))
        reappeared = copy.source_root / deleted_path
        reappeared.parent.mkdir(parents=True)
        reappeared.write_text("legacy shim\n", encoding="utf-8")
        copy.status = "materialized"

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )
        with self.database.connect() as connection:
            payload = json.loads(
                connection.execute(
                    """
                    SELECT payload_json FROM validation_ticket_events
                    WHERE ticket_id=? AND event_type='validation.ticket_status_changed'
                    ORDER BY event_id DESC LIMIT 1
                    """,
                    (receipt.ticket.ticket_id,),
                ).fetchone()[0]
            )
        self.assertEqual("materialized_copy", payload["evidence"]["phase"])
        self.assertEqual([deleted_path], payload["evidence"]["driftPaths"])

    def test_reappearing_directory_in_materialized_copy_is_snapshot_stale(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self._submit("materialized-copy-directory-race", {deleted_path: None})
        self.worker.tick()
        copy = next(iter(self.workspace_copy.records.values()))
        reappeared_directory = copy.source_root / deleted_path
        reappeared_directory.mkdir(parents=True)
        copy.status = "materialized"

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )
        with self.database.connect() as connection:
            payload = json.loads(
                connection.execute(
                    """
                    SELECT payload_json FROM validation_ticket_events
                    WHERE ticket_id=? AND event_type='validation.ticket_status_changed'
                    ORDER BY event_id DESC LIMIT 1
                    """,
                    (receipt.ticket.ticket_id,),
                ).fetchone()[0]
            )
        self.assertEqual("materialized_copy", payload["evidence"]["phase"])
        self.assertEqual([deleted_path], payload["evidence"]["driftPaths"])

    def test_reappearing_dangling_symlink_in_materialized_copy_is_snapshot_stale(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self._submit("materialized-copy-symlink-race", {deleted_path: None})
        self.worker.tick()
        copy = next(iter(self.workspace_copy.records.values()))
        reappeared_link = copy.source_root / deleted_path
        copy.status = "materialized"

        # Model a dangling link even on Windows hosts without symlink privileges.
        with mock.patch(
            "tools.session_coordinator.validation_ticket_worker.os.path.lexists",
            return_value=True,
        ) as lexists:
            result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        lexists.assert_called_once_with(reappeared_link)
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )

    def _submit(self, request_id: str, source_manifest: dict[str, str | None]):
        return self.service.submit(
            session_id="primary",
            request_id=request_id,
            source_manifest=source_manifest,
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )


class ValidationCopyDeletionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        target_root = root / "drive/targets/zircon-engine"
        target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            self.service = WorkspaceCopyService(
                self.database, self.repo, (target_root,)
            )

    def test_tombstone_removes_a_file_extracted_from_the_baseline(self) -> None:
        deleted = self.repo / "README.md"
        deleted.unlink()
        self.baselines.attribute("session-a", ["README.md"])

        result = self.service.materialize("session-a", include_paths=("README.md",))

        self.assertFalse((result.source_root / "README.md").exists())

    def test_tombstone_rejects_a_file_recreated_before_overlay(self) -> None:
        deleted = self.repo / "README.md"
        deleted.unlink()
        self.baselines.attribute("session-a", ["README.md"])
        deleted.write_text("reappeared\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.materialize("session-a", include_paths=("README.md",))

        self.assertEqual(
            "validation_copy_owned_source_reappeared", rejected.exception.code
        )


if __name__ == "__main__":
    unittest.main()
