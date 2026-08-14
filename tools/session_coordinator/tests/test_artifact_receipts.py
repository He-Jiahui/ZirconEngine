from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.artifact_receipts import ManagedArtifactReceiptService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError


SESSION_ID = "shader06"
OTHER_SESSION_ID = "shader07"
JOB_ID = "a" * 32
TICKET_ID = "b" * 32
RUN_ID = "c" * 32
INPUT_MANIFEST = "d" * 64
SOURCE_MANIFEST = "e" * 64
BUILD_COMMAND = (
    "cargo",
    "+1.94.1",
    "build",
    "-p",
    "zircon_app",
    "--bin",
    "zircon_shader_pbr_viewer",
    "--locked",
    "--release",
)


class ManagedArtifactReceiptServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.database = Database(self.root / "coordinator.sqlite3")
        migrate(self.database)
        self.job_root = self.root / "validation" / JOB_ID
        self.source_root = self.job_root / "source"
        self.target_root = self.job_root / "target"
        self.source_root.mkdir(parents=True)
        self.target_root.mkdir()
        self.artifact_root = self.root / "managed-artifacts"
        self.service = ManagedArtifactReceiptService(
            self.database, self.artifact_root
        )
        self._insert_session(SESSION_ID)
        self._insert_session(OTHER_SESSION_ID)
        self._insert_ticket()
        self._insert_copy()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _insert_session(self, session_id: str) -> None:
        now = "2026-08-14T00:00:00+00:00"
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, display_name, plan_path, status, base_head,
                    write_scope_json, created_at, updated_at, last_heartbeat_at
                ) VALUES (?, ?, ?, 'active', ?, '[]', ?, ?, ?)
                """,
                (
                    session_id,
                    session_id,
                    "docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md",
                    "1" * 40,
                    now,
                    now,
                    now,
                ),
            )

    def _insert_ticket(self) -> None:
        now = "2026-08-14T00:00:00+00:00"
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_tickets(
                    ticket_id, session_id, plan_path, status, dedupe_key,
                    source_manifest_hash, source_manifest_json, command_json,
                    toolchain_json, coverage_json, created_at, updated_at
                ) VALUES (?, ?, ?, 'passed', ?, ?, '{}', '[]', '{}', '{}', ?, ?)
                """,
                (
                    TICKET_ID,
                    SESSION_ID,
                    "docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md",
                    "2" * 64,
                    SOURCE_MANIFEST,
                    now,
                    now,
                ),
            )

    def _insert_copy(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root,
                    head_commit, manifest_json, status, created_at,
                    input_manifest_hash, materialization_kind
                ) VALUES (?, ?, ?, ?, ?, ?, '{}', 'materialized', ?, ?, 'cargo')
                """,
                (
                    JOB_ID,
                    SESSION_ID,
                    str(self.job_root),
                    str(self.source_root),
                    str(self.target_root),
                    "1" * 40,
                    "2026-08-14T00:00:00+00:00",
                    INPUT_MANIFEST,
                ),
            )

    def _insert_run(
        self,
        *,
        exit_code: int = 0,
        command: tuple[str, ...] = BUILD_COMMAND,
    ) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copy_runs(
                    run_id, job_id, session_id, command_json, exit_code,
                    stdout_text, stderr_text, started_at, completed_at
                ) VALUES (?, ?, ?, ?, ?, '', '', ?, ?)
                """,
                (
                    RUN_ID,
                    JOB_ID,
                    SESSION_ID,
                    json.dumps(command),
                    exit_code,
                    "2026-08-14T00:01:00+00:00",
                    "2026-08-14T00:02:00+00:00",
                ),
            )

    def _request(self):
        return self.service.request(
            session_id=SESSION_ID,
            job_id=JOB_ID,
            validation_ticket_id=TICKET_ID,
            artifact_kind="shader-pbr-viewer",
        )

    def _viewer_path(self) -> Path:
        path = self.target_root / "release" / "zircon_shader_pbr_viewer.exe"
        path.parent.mkdir(parents=True, exist_ok=True)
        return path

    def test_successful_terminal_build_emits_durable_exact_receipt(self) -> None:
        requested = self._request()
        viewer = self._viewer_path()
        viewer.write_bytes(b"current managed viewer")
        self._insert_run()

        receipt = self.service.finalize_run(RUN_ID)

        self.assertIsNotNone(receipt)
        assert receipt is not None
        self.assertEqual(requested.receipt_id, receipt.receipt_id)
        self.assertEqual("passed", receipt.status)
        self.assertEqual(JOB_ID, receipt.job_id)
        self.assertEqual(RUN_ID, receipt.run_id)
        self.assertEqual(TICKET_ID, receipt.validation_ticket_id)
        self.assertEqual(INPUT_MANIFEST, receipt.input_manifest_hash)
        self.assertEqual(SOURCE_MANIFEST, receipt.source_manifest_hash)
        self.assertEqual(
            "release/zircon_shader_pbr_viewer.exe", receipt.target_relative_path
        )
        self.assertEqual(len(b"current managed viewer"), receipt.byte_length)
        self.assertEqual(
            hashlib.sha256(b"current managed viewer").hexdigest(), receipt.sha256
        )
        self.assertEqual(
            hashlib.sha256(
                json.dumps(BUILD_COMMAND, separators=(",", ":")).encode("utf-8")
            ).hexdigest(),
            receipt.command_sha256,
        )
        durable = Path(receipt.artifact_path)
        self.assertTrue(durable.is_file())
        self.assertEqual(viewer.read_bytes(), durable.read_bytes())
        self.assertEqual(receipt, self.service.get(receipt.receipt_id))

    def test_nonzero_or_missing_artifact_rejects_receipt(self) -> None:
        for exit_code, create_artifact, expected_error in (
            (1, True, "managed_artifact_build_failed"),
            (0, False, "managed_artifact_missing"),
        ):
            with self.subTest(exit_code=exit_code, create_artifact=create_artifact):
                self._reset_receipt_and_run()
                self._request()
                if create_artifact:
                    self._viewer_path().write_bytes(b"failed build output")
                self._insert_run(exit_code=exit_code)

                receipt = self.service.finalize_run(RUN_ID)

                self.assertIsNotNone(receipt)
                assert receipt is not None
                self.assertEqual("rejected", receipt.status)
                self.assertEqual(expected_error, receipt.error_code)
                self.assertIsNone(receipt.artifact_path)

    def test_target_escape_is_rejected_before_hashing(self) -> None:
        self._request()
        outside = self.root / "outside.exe"
        outside.write_bytes(b"outside")
        self._insert_run()

        with mock.patch.object(
            self.service, "_artifact_path", return_value=outside
        ):
            receipt = self.service.finalize_run(RUN_ID)

        self.assertIsNotNone(receipt)
        assert receipt is not None
        self.assertEqual("rejected", receipt.status)
        self.assertEqual("managed_artifact_target_escape", receipt.error_code)

    def test_durable_artifact_mutation_invalidates_query(self) -> None:
        self._request()
        self._viewer_path().write_bytes(b"original")
        self._insert_run()
        receipt = self.service.finalize_run(RUN_ID)
        assert receipt is not None and receipt.artifact_path is not None
        Path(receipt.artifact_path).write_bytes(b"modified after build")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.get(receipt.receipt_id)

        self.assertEqual("managed_artifact_receipt_hash_mismatch", rejected.exception.code)

    def test_input_manifest_change_rejects_terminal_receipt(self) -> None:
        self._request()
        self._viewer_path().write_bytes(b"viewer")
        self._insert_run()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET input_manifest_hash=? WHERE job_id=?",
                ("f" * 64, JOB_ID),
            )

        receipt = self.service.finalize_run(RUN_ID)

        self.assertIsNotNone(receipt)
        assert receipt is not None
        self.assertEqual("rejected", receipt.status)
        self.assertEqual("managed_artifact_input_manifest_mismatch", receipt.error_code)

    def test_source_manifest_change_rejects_terminal_receipt(self) -> None:
        self._request()
        self._viewer_path().write_bytes(b"viewer")
        self._insert_run()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_tickets SET source_manifest_hash=? WHERE ticket_id=?",
                ("f" * 64, TICKET_ID),
            )

        receipt = self.service.finalize_run(RUN_ID)

        self.assertIsNotNone(receipt)
        assert receipt is not None
        self.assertEqual("rejected", receipt.status)
        self.assertEqual("managed_artifact_source_manifest_mismatch", receipt.error_code)

    def test_foreign_session_cannot_request_or_query_receipt(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.request(
                session_id=OTHER_SESSION_ID,
                job_id=JOB_ID,
                validation_ticket_id=TICKET_ID,
                artifact_kind="shader-pbr-viewer",
            )

        self.assertEqual("managed_artifact_cross_session", rejected.exception.code)

        receipt = self._request()
        with self.assertRaises(CoordinatorError) as query_rejected:
            self.service.get(receipt.receipt_id, session_id=OTHER_SESSION_ID)
        self.assertEqual("managed_artifact_cross_session", query_rejected.exception.code)

    def _reset_receipt_and_run(self) -> None:
        with self.database.transaction() as connection:
            connection.execute("DELETE FROM managed_artifact_receipts")
            connection.execute("DELETE FROM validation_copy_runs")
        viewer = self.target_root / "release" / "zircon_shader_pbr_viewer.exe"
        if viewer.exists():
            viewer.unlink()


class ManagedArtifactReceiptCliTests(unittest.TestCase):
    def test_status_command_is_available_through_the_read_only_surface(self) -> None:
        from tools.session_coordinator.server import CoordinatorApplication

        self.assertIn(
            "validation.artifact_receipt.status",
            CoordinatorApplication.READ_ONLY_COMMANDS,
        )

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_request_transport_exposes_no_caller_artifact_path_or_hash(
        self, from_runtime
    ) -> None:
        from tools.session_coordinator import cli

        arguments = cli._parser().parse_args(
            [
                "validation",
                "artifact-receipt-request",
                "--session-id",
                SESSION_ID,
                "--job-id",
                JOB_ID,
                "--ticket-id",
                TICKET_ID,
                "--artifact-kind",
                "shader-pbr-viewer",
            ]
        )
        from_runtime.return_value.command.return_value = {
            "artifactReceipt": {"receiptId": "f" * 32}
        }

        cli._run(arguments)

        command, payload = from_runtime.return_value.command.call_args.args
        self.assertEqual("validation.artifact_receipt.request", command)
        self.assertEqual(
            {
                "session_id": SESSION_ID,
                "job_id": JOB_ID,
                "ticket_id": TICKET_ID,
                "artifact_kind": "shader-pbr-viewer",
            },
            payload,
        )
        self.assertNotIn("path", payload)
        self.assertNotIn("sha256", payload)

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_status_transport_can_bind_query_to_requesting_session(
        self, from_runtime
    ) -> None:
        from tools.session_coordinator import cli

        arguments = cli._parser().parse_args(
            [
                "validation",
                "artifact-receipt-status",
                "--receipt-id",
                "f" * 32,
                "--session-id",
                SESSION_ID,
            ]
        )
        from_runtime.return_value.command.return_value = {
            "artifactReceipt": {"receiptId": "f" * 32}
        }

        cli._run(arguments)

        from_runtime.return_value.command.assert_called_once_with(
            "validation.artifact_receipt.status",
            {"receipt_id": "f" * 32, "session_id": SESSION_ID},
        )


class ManagedArtifactReceiptCompletionHookTests(unittest.TestCase):
    def test_artifact_is_sealed_before_milestone_terminal_projection(self) -> None:
        from tools.session_coordinator.server import CoordinatorApplication

        application = object.__new__(CoordinatorApplication)
        order: list[str] = []
        application.artifact_receipts = mock.Mock()
        application.milestone_workflows = mock.Mock()
        application.artifact_receipts.finalize_run.side_effect = (
            lambda run_id: order.append(f"artifact:{run_id}")
        )
        application.milestone_workflows.import_validation_result.side_effect = (
            lambda run_id: order.append(f"milestone:{run_id}")
        )

        application._complete_validation_run(RUN_ID)

        self.assertEqual(
            [f"artifact:{RUN_ID}", f"milestone:{RUN_ID}"], order
        )


if __name__ == "__main__":
    unittest.main()
