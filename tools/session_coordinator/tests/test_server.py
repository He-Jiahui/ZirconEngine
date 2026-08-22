from __future__ import annotations

import base64
import json
import hashlib
import os
import sqlite3
import subprocess
import time
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from datetime import date
from pathlib import Path
from unittest import mock

from tools.session_coordinator import cli, server
from tools.session_coordinator.client import CoordinatorClient, CoordinatorClientError
from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.codex_sync.evidence import CodexEvidenceProjector
from tools.session_coordinator.codex_sync.models import CodexReconcileResult
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.server import (
    CoordinatorApplication,
    RunningCoordinator,
    validate_proof_bound_handoff,
)
from tools.session_coordinator.models import CoordinatorError, SessionStatus, SupervisionState
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.watch import WorkspaceWatcher
from tools.session_coordinator.workspace_copy import WorkspaceCopyRecord


class ServerTests(unittest.TestCase):
    def test_artifact_fixture_commands_route_process_bound_lifecycle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            governance = mock.Mock()
            acquired = mock.Mock()
            acquired.to_dict.return_value = {
                "leaseId": "a" * 32,
                "path": r"D:\ZirconBuilds\mvp-test-fixtures-42\server-a",
                "ownerPid": 42,
                "status": "active",
            }
            released = mock.Mock()
            released.to_dict.return_value = {
                "leaseId": "a" * 32,
                "path": r"D:\ZirconBuilds\mvp-test-fixtures-42\server-a",
                "ownerPid": 42,
                "status": "released",
            }
            governance.acquire_fixture.return_value = acquired
            governance.release_fixture.return_value = released
            application.artifact_governance = governance

            acquire_result = application.command(
                "artifact.fixture_acquire", {"prefix": "server", "owner_pid": 42}
            )
            release_result = application.command(
                "artifact.fixture_release", {"lease_id": "a" * 32, "owner_pid": 42}
            )

        governance.acquire_fixture.assert_called_once_with("server", owner_pid=42)
        governance.release_fixture.assert_called_once_with("a" * 32, owner_pid=42)
        self.assertEqual("active", acquire_result["lease"]["status"])
        self.assertEqual("released", release_result["lease"]["status"])

    def test_artifact_fixture_cli_sends_only_prefix_lease_and_owner_pid(self) -> None:
        parser = cli._parser()
        acquired = parser.parse_args(
            [
                "artifact",
                "fixture-acquire",
                "--prefix",
                "build-editor",
                "--owner-pid",
                "42",
            ]
        )
        released = parser.parse_args(
            [
                "artifact",
                "fixture-release",
                "--lease-id",
                "a" * 32,
                "--owner-pid",
                "42",
            ]
        )
        client = mock.Mock()
        client.command.side_effect = (
            {"lease": {"status": "active"}},
            {"lease": {"status": "released"}},
        )

        with mock.patch.object(cli.CoordinatorClient, "from_runtime", return_value=client):
            acquire_result = cli._run(acquired)
            release_result = cli._run(released)

        self.assertEqual("active", acquire_result["lease"]["status"])
        self.assertEqual("released", release_result["lease"]["status"])
        self.assertEqual(
            [
                mock.call(
                    "artifact.fixture_acquire",
                    {"prefix": "build-editor", "owner_pid": 42},
                ),
                mock.call(
                    "artifact.fixture_release",
                    {"lease_id": "a" * 32, "owner_pid": 42},
                ),
            ],
            client.command.call_args_list,
        )

    def test_artifact_product_staging_commands_route_closed_lifecycle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            governance = mock.Mock()
            states = []
            for status in ("active", "publishing", "published", "released"):
                state = mock.Mock()
                state.to_dict.return_value = {
                    "leaseId": "b" * 32,
                    "stagingPath": r"D:\ZirconBuilds\mvp-product-inputs-build-editor-b",
                    "finalPath": r"D:\ZirconBuilds\editor-current",
                    "ownerPid": 42,
                    "status": status,
                }
                states.append(state)
            governance.acquire_product_staging.return_value = states[0]
            governance.begin_product_staging_publish.return_value = states[1]
            governance.complete_product_staging_publish.return_value = states[2]
            governance.release_product_staging.return_value = states[3]
            application.artifact_governance = governance

            acquired = application.command(
                "artifact.staging_acquire",
                {
                    "purpose": "build-editor",
                    "final_path": r"D:\ZirconBuilds\editor-current",
                    "owner_pid": 42,
                },
            )
            publishing = application.command(
                "artifact.staging_begin_publish",
                {"lease_id": "b" * 32, "owner_pid": 42},
            )
            published = application.command(
                "artifact.staging_complete_publish",
                {"lease_id": "b" * 32, "owner_pid": 42},
            )
            released = application.command(
                "artifact.staging_release",
                {"lease_id": "b" * 32, "owner_pid": 42},
            )
            for invalid_owner in (True, 42.5, "42"):
                with self.assertRaises(CoordinatorError) as invalid:
                    application.command(
                        "artifact.staging_acquire",
                        {
                            "purpose": "build-editor",
                            "final_path": r"D:\ZirconBuilds\editor-current",
                            "owner_pid": invalid_owner,
                        },
                    )
                self.assertEqual(
                    "artifact_product_staging_arguments_invalid",
                    invalid.exception.code,
                )

        governance.acquire_product_staging.assert_called_once_with(
            "build-editor",
            final_path=r"D:\ZirconBuilds\editor-current",
            owner_pid=42,
        )
        governance.begin_product_staging_publish.assert_called_once_with(
            "b" * 32, owner_pid=42
        )
        governance.complete_product_staging_publish.assert_called_once_with(
            "b" * 32, owner_pid=42
        )
        governance.release_product_staging.assert_called_once_with(
            "b" * 32, owner_pid=42
        )
        self.assertEqual(
            ["active", "publishing", "published", "released"],
            [
                acquired["lease"]["status"],
                publishing["lease"]["status"],
                published["lease"]["status"],
                released["lease"]["status"],
            ],
        )

    def test_artifact_product_staging_cli_has_no_caller_staging_path(self) -> None:
        parser = cli._parser()
        commands = (
            parser.parse_args(
                [
                    "artifact",
                    "staging-acquire",
                    "--purpose",
                    "build-editor",
                    "--final-path",
                    r"D:\ZirconBuilds\editor-current",
                    "--owner-pid",
                    "42",
                ]
            ),
            parser.parse_args(
                [
                    "artifact",
                    "staging-begin-publish",
                    "--lease-id",
                    "b" * 32,
                    "--owner-pid",
                    "42",
                ]
            ),
            parser.parse_args(
                [
                    "artifact",
                    "staging-complete-publish",
                    "--lease-id",
                    "b" * 32,
                    "--owner-pid",
                    "42",
                ]
            ),
            parser.parse_args(
                [
                    "artifact",
                    "staging-release",
                    "--lease-id",
                    "b" * 32,
                    "--owner-pid",
                    "42",
                ]
            ),
        )
        client = mock.Mock()
        client.command.side_effect = (
            {"lease": {"status": "active"}},
            {"lease": {"status": "publishing"}},
            {"lease": {"status": "published"}},
            {"lease": {"status": "released"}},
        )

        with mock.patch.object(cli.CoordinatorClient, "from_runtime", return_value=client):
            results = [cli._run(arguments) for arguments in commands]

        self.assertEqual(
            ["active", "publishing", "published", "released"],
            [result["lease"]["status"] for result in results],
        )
        self.assertEqual(
            [
                mock.call(
                    "artifact.staging_acquire",
                    {
                        "purpose": "build-editor",
                        "final_path": r"D:\ZirconBuilds\editor-current",
                        "owner_pid": 42,
                    },
                ),
                mock.call(
                    "artifact.staging_begin_publish",
                    {"lease_id": "b" * 32, "owner_pid": 42},
                ),
                mock.call(
                    "artifact.staging_complete_publish",
                    {"lease_id": "b" * 32, "owner_pid": 42},
                ),
                mock.call(
                    "artifact.staging_release",
                    {"lease_id": "b" * 32, "owner_pid": 42},
                ),
            ],
            client.command.call_args_list,
        )

    def test_governance_converge_routes_preview_and_apply_through_audited_service(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            with application.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO sessions(
                        session_id, status, created_at, updated_at, last_heartbeat_at
                    ) VALUES (
                        'expired-owner', 'active', '2026-07-29T00:00:00+00:00',
                        '2026-07-29T00:00:00+00:00', '2026-07-29T00:00:00+00:00'
                    )
                    """
                )

            preview = application.command(
                "governance.converge.preview", {"actor": "server-test"}
            )["preview"]
            result = application.command(
                "governance.converge.apply",
                {"fingerprint": preview["fingerprint"], "actor": "server-test"},
            )["result"]

            self.assertEqual("converge", preview["operation"])
            self.assertEqual(["session:expired-owner"], result["applied"])
            with application.database.connect() as connection:
                status = connection.execute(
                    "SELECT status FROM sessions WHERE session_id='expired-owner'"
                ).fetchone()["status"]
                apply_audit = connection.execute(
                    """
                    SELECT applied_count, conflict_count FROM governance_applies
                    WHERE fingerprint=?
                    """,
                    (preview["fingerprint"],),
                ).fetchone()
            self.assertEqual("stale", status)
            self.assertEqual((1, 0), tuple(apply_audit))

    def test_governance_manifest_retention_routes_verified_retirement_and_compact(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            with application.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO baseline_epochs(
                        head_commit, index_tree, health, manifest_json, created_at
                    ) VALUES ('old', 'old', 'healthy', '{"old.rs":"a"}',
                              '2026-07-01T00:00:00+00:00')
                    """
                )
                old_epoch = int(connection.execute("SELECT last_insert_rowid()").fetchone()[0])
                connection.execute(
                    """
                    INSERT INTO baseline_epochs(
                        head_commit, index_tree, health, manifest_json, created_at
                    ) VALUES ('new', 'new', 'healthy', '{"new.rs":"b"}',
                              '2026-07-30T00:00:00+00:00')
                    """
                )

            preview = application.command(
                "governance.retention.preview", {"actor": "server-test"}
            )["preview"]
            self.assertEqual(
                [{"table": "baseline_epochs", "identity": str(old_epoch), "sha256": preview["candidates"][0]["sha256"], "entryCount": 1, "byteCount": 14}],
                preview["candidates"],
            )
            applied = application.command(
                "governance.retention.apply",
                {"fingerprint": preview["fingerprint"], "actor": "server-test"},
            )["result"]
            queued = application.command(
                "governance.retention.compact",
                {"batch_id": applied["batchId"], "actor": "server-test"},
            )["receipt"]
            maintenance = application._maintenance_tick_unlocked({})

            self.assertTrue(Path(applied["archivePath"]).is_file())
            self.assertTrue(Path(applied["backupPath"]).is_file())
            self.assertEqual("compact_pending", queued["status"])
            self.assertIn(applied["batchId"], maintenance["compacted_manifest_batches"])
            with application.database.connect() as connection:
                row = connection.execute(
                    "SELECT manifest_json, manifest_archive_path FROM baseline_epochs WHERE epoch_id=?",
                    (old_epoch,),
                ).fetchone()
            self.assertEqual("{}", row["manifest_json"])
            self.assertIsNotNone(row["manifest_archive_path"])

    def test_ownership_matrix_routes_only_currently_attributed_live_leased_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            source = repo / "tools" / "owned.py"
            source.parent.mkdir(parents=True)
            source.write_text("value = 1\n", encoding="utf-8")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.baselines.accept(reason="ownership matrix fixture")
            application.sessions.register(session_id="owner")
            source.write_text("value = 2\n", encoding="utf-8")
            application.leases.acquire("owner", ["tools/owned.py"])
            application.baselines.attribute("owner", ["tools/owned.py"])

            matrix = application.command(
                "ownership.matrix", {"prefix": "tools"}
            )["matrix"]

            self.assertEqual(
                [{"sessionId": "owner", "paths": ["tools/owned.py"]}],
                matrix["candidates"],
            )
            self.assertEqual("integration_ready", matrix["entries"][0]["state"])

    def test_ownership_transfer_route_requires_the_reviewed_fingerprint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            source = repo / "tools" / "owned.py"
            source.parent.mkdir(parents=True)
            source.write_text("value = 1\n", encoding="utf-8")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.baselines.accept(reason="ownership transfer fixture")
            application.sessions.register(session_id="source")
            application.sessions.register(session_id="target")
            source.write_text("value = 2\n", encoding="utf-8")
            application.leases.acquire("source", ["tools/owned.py"])
            application.baselines.attribute("source", ["tools/owned.py"])
            application.leases.release("source")
            application.sessions.set_status(
                "source", SessionStatus.STALE, reason="ownership transfer fixture"
            )

            preview = application.command(
                "ownership.transfer.preview",
                {"target_session_id": "target", "paths": ["tools/owned.py"]},
            )["preview"]
            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "ownership.transfer.apply",
                    {
                        "fingerprint": preview["fingerprint"],
                        "confirm_fingerprint": "not-the-preview",
                    },
                )
            result = application.command(
                "ownership.transfer.apply",
                {
                    "fingerprint": preview["fingerprint"],
                    "confirm_fingerprint": preview["fingerprint"],
                    "actor": "server-test",
                },
            )["result"]

            self.assertEqual(
                "ownership_transfer_confirmation_required", rejected.exception.code
            )
            self.assertEqual(["tools/owned.py"], result["paths"])
            self.assertEqual(["tools/owned.py"], application.leases.owned_paths("target"))

    def test_local_validation_failure_command_persists_and_indexes_a_forward_repair(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="validation-owner",
                plan_path=plan.path.relative_to(repo).as_posix(),
            )

            result = application.command(
                "failure.materialize_local_validation",
                {
                    "session_id": "validation-owner",
                    "summary_slug": "focused-validation-failed",
                    "source_slice": "M2 ticket 99",
                    "reproduction": "python -m unittest focused_case",
                    "lowest_known_cause": "The focused validation reported a repairable failure.",
                    "acceptance_criteria": ["The focused validation passes after repair."],
                    "related_code": ["tools/session_coordinator/failures.py"],
                    "created_at": "2026-07-31",
                },
            )

            self.assertEqual("local", result["failure_scope"])
            self.assertEqual("forward_fix_required", result["integration_disposition"])
            self.assertEqual(plan.path.relative_to(repo).as_posix(), result["repair_plan"])
            self.assertEqual(
                ["focused-validation-failed"],
                [node.summary_slug for node in application.failures.open_for_plan(plan.path)],
            )

    def test_failed_validation_result_creates_a_forward_repair_without_reverting_work(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="validation-owner",
                plan_path=plan.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("validation-owner", SessionStatus.ACTIVE)
            receipt = application.command(
                "validation.submit",
                {
                    "session_id": "validation-owner",
                    "request_id": "focused-test-request",
                    "source_manifest": {"tools/session_coordinator/server.py": "a" * 64},
                    "command": ["python", "-m", "unittest", "focused_case"],
                    "toolchain": {"python": "3.14"},
                    "coverage": {"kind": "focused"},
                },
            )

            ticket_id = receipt["receipt"]["ticket"]["ticket_id"]
            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "validation.record_result",
                    {
                        "ticket_id": ticket_id,
                        "status": "failed",
                        "evidence": {"exitCode": 1},
                        "failure": {
                            "summary_slug": "focused-test-failed",
                            "source_slice": "M2 validation ticket",
                            "reproduction": "python -m unittest focused_case",
                            "lowest_known_cause": "The focused test reported a repairable failure.",
                            "acceptance_criteria": ["The focused test passes after repair."],
                            "related_code": ["tools/session_coordinator/server.py"],
                            "created_at": "2026-07-31",
                        },
                    },
                )

            self.assertEqual("validation_ticket_result_worker_only", rejected.exception.code)
            self.assertEqual("queued", application.validation_tickets.get(ticket_id).status)
            self.assertEqual([], application.failures.open_for_plan(plan.path))
            self.assertEqual(
                "active", application.sessions.get("validation-owner").status.value
            )

    def test_failed_validation_without_handoff_context_keeps_the_ticket_queueable(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="validation-owner",
                plan_path=plan.path.relative_to(repo).as_posix(),
            )
            receipt = application.command(
                "validation.submit",
                {
                    "session_id": "validation-owner",
                    "request_id": "missing-handoff-request",
                    "source_manifest": {"tools/session_coordinator/server.py": "a" * 64},
                    "command": ["python", "-m", "unittest", "focused_case"],
                    "toolchain": {"python": "3.14"},
                    "coverage": {"kind": "focused"},
                },
            )
            ticket_id = receipt["receipt"]["ticket"]["ticket_id"]

            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "validation.record_result",
                    {"ticket_id": ticket_id, "status": "failed", "evidence": {}},
                )

            self.assertEqual("validation_ticket_result_worker_only", rejected.exception.code)
            self.assertEqual("queued", application.validation_tickets.get(ticket_id).status)

    def test_external_validation_result_cannot_mark_queued_ticket_passed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="validation-owner",
                plan_path=plan.path.relative_to(repo).as_posix(),
            )
            receipt = application.command(
                "validation.submit",
                {
                    "session_id": "validation-owner",
                    "request_id": "caller-written-pass",
                    "source_manifest": {"tools/session_coordinator/server.py": "a" * 64},
                    "command": ["python", "-m", "unittest", "focused_case"],
                    "toolchain": {"python": "3.14"},
                    "coverage": {"kind": "focused"},
                },
            )
            ticket_id = receipt["receipt"]["ticket"]["ticket_id"]

            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "validation.record_result",
                    {
                        "ticket_id": ticket_id,
                        "status": "passed",
                        "evidence": {"exitCode": 0, "claimedBy": "external-caller"},
                    },
                )

            self.assertEqual("validation_ticket_result_worker_only", rejected.exception.code)
            self.assertEqual("queued", application.validation_tickets.get(ticket_id).status)

    def test_compile_ticket_can_finalize_a_sealed_candidate_through_the_coordinator(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            source = repo / "tools" / "candidate.py"
            source.parent.mkdir(parents=True)
            source.write_text("value = 1\n", encoding="utf-8")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="candidate-owner",
                plan_path=plan.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("candidate-owner", SessionStatus.ACTIVE)
            self.assertTrue(
                application.leases.acquire("candidate-owner", ["tools/candidate.py"]).acquired
            )
            ticket = application.command(
                "validation.submit",
                {
                    "session_id": "candidate-owner",
                    "request_id": "compile-ticket",
                    "source_manifest": {
                        "tools/candidate.py": hashlib.sha256(source.read_bytes()).hexdigest()
                    },
                    "command": ["python", "-m", "py_compile", "tools/candidate.py"],
                    "toolchain": {"python": "3.14"},
                    "coverage": {"kind": "compile"},
                },
            )
            application.validation_tickets.record_result(
                ticket["receipt"]["ticket"]["ticket_id"],
                "passed",
                evidence={"exitCode": 0},
            )
            candidate = application.command(
                "integration.submit",
                {
                    "session_id": "candidate-owner",
                    "request_id": "candidate-request",
                    "compile_ticket_id": ticket["receipt"]["ticket"]["ticket_id"],
                    "paths": ["tools/candidate.py"],
                },
            )
            source.write_text("value = 2\n", encoding="utf-8")

            finalized = application.command(
                "integration.finalize",
                {
                    "candidate_id": candidate["candidate"]["candidate_id"],
                    "message": "integration: sealed candidate",
                },
            )

            commit_sha = finalized["candidate"]["commit_sha"]
            self.assertEqual("integrated_validation_pending", finalized["candidate"]["status"])
            committed = subprocess.run(
                ["git", "show", f"{commit_sha}:tools/candidate.py"],
                cwd=repo,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
            self.assertEqual("value = 1\n", committed)
            self.assertEqual("value = 2\n", source.read_text(encoding="utf-8"))

    def test_isolated_patch_route_is_maintenance_only_and_has_no_compile_ticket(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            result = mock.Mock()
            result.to_dict.return_value = {
                "requestId": "isolated-request",
                "derivedBlob": "d" * 40,
            }
            service = mock.Mock()
            service.finalize.return_value = result
            application.isolated_patch_finalize = service

            response = application.command(
                "maintenance.finalize_patch",
                {
                    "session_id": "render-owner",
                    "target": "tools/construct.rs",
                    "expected_head": "a" * 40,
                    "expected_blob": "b" * 40,
                    "patch_base64": base64.b64encode(b"patch-bytes").decode("ascii"),
                    "message": "fix(coordinator): isolate target patch",
                    "validation_commands": [
                        ["python", "-m", "unittest", "focused.case"]
                    ],
                },
            )

            self.assertEqual("isolated-request", response["result"]["requestId"])
            service.finalize.assert_called_once_with(
                session_id="render-owner",
                target="tools/construct.rs",
                patch=b"patch-bytes",
                expected_head="a" * 40,
                expected_blob="b" * 40,
                message="fix(coordinator): isolate target patch",
                validation_commands=(("python", "-m", "unittest", "focused.case"),),
            )

    def test_pending_git_recovery_still_requires_the_daemon_process_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            database = Database(config.database_path)
            migrate(database)
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO git_mutex(lock_name, owner_id, acquired_at) VALUES ('index', ?, datetime('now'))",
                    ("interrupted-owner",),
                )

            with self.assertRaises(CoordinatorError) as rejected:
                CoordinatorApplication(config)

        self.assertEqual(
            "finalize_recovery_process_unproven", rejected.exception.code
        )

    def test_cpu_burst_eligibility_defaults_for_safe_checks_and_allows_opt_out(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(session_id="owner")
            application.sessions.set_status("owner", SessionStatus.ACTIVE)
            arguments = {
                "session_id": "owner",
                "compatibility": {
                    "platform": "windows",
                    "toolchain": "stable-x86_64-pc-windows-msvc",
                    "target_architecture": "x86_64-pc-windows-msvc",
                    "workspace": "Cargo.toml",
                    "build_config": "profile=dev",
                },
                "target_dir": None,
                "ttl_seconds": 900,
                "command": ["cargo", "check", "-p", "zircon_runtime"],
            }

            result = application.command("cargo.reserve_cpu", arguments)

            self.assertTrue(result["reservation"]["burstEligible"])
            disabled = application.command(
                "cargo.reserve_cpu", {**arguments, "burst_eligible": False}
            )
            self.assertFalse(disabled["reservation"]["burstEligible"])
            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "cargo.reserve_cpu", {**arguments, "burst_eligible": "true"}
                )
            self.assertEqual("cargo_cpu_burst_eligibility_invalid", rejected.exception.code)

    def test_cpu_reservation_forwards_typed_failure_dependency_barrier(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(session_id="owner")
            application.sessions.set_status("owner", SessionStatus.ACTIVE)
            application.cargo_jobs.reserve_cpu = mock.Mock(
                return_value={"reservationId": "reservation-a"}
            )

            result = application.command(
                "cargo.reserve_cpu",
                {
                    "session_id": "owner",
                    "compatibility": {
                        "platform": "windows",
                        "toolchain": "stable-x86_64-pc-windows-msvc",
                        "target_architecture": "x86_64-pc-windows-msvc",
                        "workspace": "Cargo.toml",
                        "build_config": "profile=dev",
                    },
                    "target_dir": None,
                    "ttl_seconds": 900,
                    "command": ["cargo", "test", "-p", "zircon_runtime"],
                    "dependency_lifecycle_key": "failure-key",
                    "dependency_fixed_sha256": "A" * 64,
                },
            )

            self.assertEqual("reservation-a", result["reservation"]["reservationId"])
            self.assertEqual(
                "failure-key",
                application.cargo_jobs.reserve_cpu.call_args.kwargs[
                    "dependency_lifecycle_key"
                ],
            )
            self.assertEqual(
                "A" * 64,
                application.cargo_jobs.reserve_cpu.call_args.kwargs[
                    "dependency_fixed_sha256"
                ],
            )

    def test_reserve_cpu_cli_preserves_auto_default_and_explicit_opt_out(self) -> None:
        parser = cli._parser()
        automatic = parser.parse_args(
            ["cargo", "reserve-cpu", "--compatibility-json", "{}", "--", "cargo", "check"]
        )
        disabled = parser.parse_args(
            ["cargo", "reserve-cpu", "--compatibility-json", "{}", "--no-burst", "--", "cargo", "check"]
        )

        self.assertIsNone(automatic.burst_eligible)
        self.assertFalse(disabled.burst_eligible)

    def test_lease_release_succeeds_when_a_stale_session_has_a_queued_patch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            for session_id in ("owner", "queued-session"):
                application.sessions.register(session_id=session_id)
                application.sessions.set_status(session_id, SessionStatus.ACTIVE)
            self.assertTrue(application.leases.acquire("owner", ["README.md"]).acquired)
            queued = application.patches.submit(
                "queued-session",
                "diff --git a/README.md b/README.md\n"
                "--- a/README.md\n"
                "+++ b/README.md\n"
                "@@ -1 +1 @@\n"
                "-baseline\n"
                "+queued\n",
                ["README.md"],
            )
            application.sessions.set_status("queued-session", SessionStatus.STALE)

            result = application.command(
                "lease.release", {"session_id": "owner", "paths": ["README.md"]}
            )

            self.assertEqual(1, result["released"])
            self.assertEqual([], result["processed_patches"])
            self.assertEqual("queued", application.patches.get(queued.patch_id).status.value)
            self.assertEqual("baseline\n", (repo / "README.md").read_text(encoding="utf-8"))

    def test_scoped_failure_return_requires_leases_for_generated_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/runtime/04-runtime.md")
            fixing = fixture.add_plan("docs/plans/tooling/01-tooling.md")
            failure = fixture.add_handoff(origin, fixing, "child-return")
            failure.write_text(
                failure.read_text(encoding="utf-8").replace(
                    "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
                ),
                encoding="utf-8",
            )
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            node = application.failures.import_repository().nodes[0]
            application.sessions.register(session_id="owner")
            generated = [
                failure.relative_to(repo).as_posix(),
                (origin.child / "fixed-2026-07-16-child-return.md").relative_to(repo).as_posix(),
                (fixing.child / "2026-07-16-child-return-return.md").relative_to(repo).as_posix(),
            ]
            application.leases.acquire("owner", generated)

            application._require_scoped_failure_return_leases(
                "owner", node.lifecycle_key, date(2026, 7, 16)
            )
            application.leases.release("owner", [generated[-1]])
            with self.assertRaises(CoordinatorError) as rejected:
                application._require_scoped_failure_return_leases(
                    "owner", node.lifecycle_key, date(2026, 7, 16)
                )

        self.assertEqual("failure_return_lease_missing", rejected.exception.code)

    def test_scoped_failure_return_allows_waiting_validation_origin_destination_lease(self) -> None:
        """A child-only return may use the origin lease while its gate waits in FIFO."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/plugins/02-sound.md")
            fixing = fixture.add_plan("docs/plans/runtime/12-input.md")
            failure = fixture.add_handoff(origin, fixing, "origin-destination")
            failure.write_text(
                failure.read_text(encoding="utf-8").replace(
                    "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
                ),
                encoding="utf-8",
            )
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            node = application.failures.import_repository().nodes[0]
            application.sessions.register(
                session_id="origin-owner",
                plan_path=origin.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("origin-owner", SessionStatus.ACTIVE)
            application.sessions.set_status("origin-owner", SessionStatus.WAITING_VALIDATION)
            application.sessions.register(
                session_id="fixer",
                plan_path=fixing.path.relative_to(repo).as_posix(),
            )
            receipt = fixing.child / "2026-07-16-origin-destination-return.md"
            application.leases.acquire(
                "origin-owner", [origin.child.relative_to(repo).as_posix()]
            )
            application.leases.acquire(
                "fixer",
                [failure.relative_to(repo).as_posix(), receipt.relative_to(repo).as_posix()],
            )

            application._require_scoped_failure_return_leases(
                "fixer", node.lifecycle_key, date(2026, 7, 16)
            )
            with application.database.connect() as connection:
                event = connection.execute(
                    "SELECT payload_json FROM events WHERE event_type='failure.return_origin_destination_authorized'"
                ).fetchone()
            self.assertEqual("origin-owner", json.loads(event["payload_json"])["originOwnerSessionId"])

    def test_failure_return_seals_origin_destination_authorization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/plugins/02-sound.md")
            fixing = fixture.add_plan("docs/plans/runtime/12-input.md")
            failure = fixture.add_handoff(origin, fixing, "sealed-origin-destination")
            failure.write_text(
                failure.read_text(encoding="utf-8").replace(
                    "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
                ),
                encoding="utf-8",
            )
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.baselines.initialize()
            node = application.failures.import_repository().nodes[0]
            application.sessions.register(
                session_id="origin-owner",
                plan_path=origin.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("origin-owner", SessionStatus.ACTIVE)
            application.sessions.register(
                session_id="fixer",
                plan_path=fixing.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("fixer", SessionStatus.ACTIVE)
            application.sessions.set_status("fixer", SessionStatus.RESOLVING_FAILURE)
            receipt = fixing.child / "2026-07-16-sealed-origin-destination-return.md"
            application.leases.acquire(
                "origin-owner", [origin.child.relative_to(repo).as_posix()]
            )
            application.leases.acquire(
                "fixer",
                [failure.relative_to(repo).as_posix(), receipt.relative_to(repo).as_posix()],
            )

            result = application.command(
                "failure.return",
                {
                    "session_id": "fixer",
                    "lifecycle_key": node.lifecycle_key,
                    "resolved_at": "2026-07-16",
                    "root_cause": "origin-owned destination lacked a commit proof",
                    "architecture_fix": "seal an exact delegated return proof",
                    "validation": "server regression passed",
                    "return_summary": "the closeout may consume the sealed proof",
                },
            )

            with application.database.connect() as connection:
                proof = connection.execute(
                    "SELECT * FROM failure_return_delegation_proofs"
                ).fetchone()
            self.assertEqual(proof["proof_id"], result["delegated_return_proof_id"])
            self.assertEqual("fixer", proof["fixing_session_id"])
            self.assertEqual("origin-owner", proof["origin_session_id"])
            self.assertEqual(result["fixed_artifact"], proof["destination_path"])
            self.assertEqual(application.baselines.current().epoch_id, proof["baseline_epoch"])

    def test_scoped_failure_return_rejects_unrelated_destination_lease(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/plugins/02-sound.md")
            fixing = fixture.add_plan("docs/plans/runtime/12-input.md")
            failure = fixture.add_handoff(origin, fixing, "unrelated-destination")
            failure.write_text(
                failure.read_text(encoding="utf-8").replace(
                    "summary_slug:", "plan_link_mode: child_record_only\nsummary_slug:"
                ),
                encoding="utf-8",
            )
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            node = application.failures.import_repository().nodes[0]
            application.sessions.register(
                session_id="unrelated-owner",
                plan_path=fixing.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("unrelated-owner", SessionStatus.ACTIVE)
            application.sessions.register(session_id="fixer")
            receipt = fixing.child / "2026-07-16-unrelated-destination-return.md"
            application.leases.acquire(
                "unrelated-owner", [origin.child.relative_to(repo).as_posix()]
            )
            application.leases.acquire(
                "fixer",
                [failure.relative_to(repo).as_posix(), receipt.relative_to(repo).as_posix()],
            )

            with self.assertRaises(CoordinatorError) as rejected:
                application._require_scoped_failure_return_leases(
                    "fixer", node.lifecycle_key, date(2026, 7, 16)
                )

        self.assertEqual("failure_return_lease_missing", rejected.exception.code)

    def test_default_config_uses_the_fixed_local_control_port(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            config = CoordinatorConfig.for_repo(Path(directory) / "repo")

        self.assertEqual(6518, config.port)
        self.assertTrue(config.unmanaged_artifact_sweep_enabled)

    def test_maintenance_tick_expires_elapsed_pending_cpu_reservations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            application = CoordinatorApplication(config)
            application.sessions.register(session_id="expired-owner")
            reservation = application.cargo_jobs.reserve_cpu(
                "expired-owner",
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="rustc-test",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=metadata;expired-reservation-test",
                ),
                command=("cargo", "metadata"),
            )
            with application.database.transaction() as connection:
                connection.execute(
                    "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                    ("2000-01-01T00:00:00+00:00", reservation["reservationId"]),
                )

            application._maintenance_tick_unlocked({})

            with application.database.connect() as connection:
                row = connection.execute(
                    "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                    (reservation["reservationId"],),
                ).fetchone()
            self.assertEqual("expired", row["status"])

    def test_startup_expires_pending_reservations_before_listener_publish(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=60,
                maintenance_interval_seconds=60,
            )
            bootstrap = CoordinatorApplication(config)
            bootstrap.sessions.register(session_id="expired-owner")
            bootstrap.sessions.set_status("expired-owner", SessionStatus.ACTIVE)
            reservation = bootstrap.cargo_jobs.reserve_cpu(
                "expired-owner",
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="rustc-test",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=metadata;startup-expiry-test",
                ),
                command=("cargo", "metadata"),
            )
            with bootstrap.database.transaction() as connection:
                connection.execute(
                    "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                    ("2000-01-01T00:00:00+00:00", reservation["reservationId"]),
                )

            with RunningCoordinator.start(config):
                with Database(config.database_path).connect() as connection:
                    row = connection.execute(
                        "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                        (reservation["reservationId"],),
                    ).fetchone()

            self.assertEqual("expired", row["status"])

    def test_application_wires_codex_sync_to_sanitized_evidence_projection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            self.assertIsInstance(application.codex_evidence, CodexEvidenceProjector)
            application.codex_worker._project(
                CodexReconcileResult(
                    run_id="sync-a", scanned_count=0, changed_count=0,
                    diagnostic_count=0, unavailable_count=0,
                )
            )
            self.assertTrue(
                any((root / "state" / "codex-source" / "sessions").rglob("*.md"))
            )

    def test_startup_audits_gpu_lease_that_predates_the_latest_reservation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            target_root = root / "D" / "cargo-targets"
            target_root.mkdir(parents=True)
            database = Database(config.database_path)
            migrate(database)
            SessionService(database, repo).register(session_id="gpu-owner")
            job = CargoJobService(
                database,
                TargetPathPolicy((target_root,)),
                repo_root=repo,
            ).acquire("gpu-owner", CargoLaneKind.GPU)
            with database.transaction() as connection:
                connection.execute(
                    """INSERT INTO action_requests(
                           action_id, action_kind, risk, required_role, actor,
                           daemon_instance_id, parameters_json, impact_json, warnings_json,
                           state_fingerprint, confirmation_phrase_hash, status, created_at,
                           expires_at, completed_at
                       ) VALUES (
                           'later-resume', 'service.resume', 'yellow', 'operator', 'operator',
                           'daemon', ?, '[]', '[]', 'fingerprint', 'phrase', 'succeeded',
                           '2099-01-01T00:00:00+00:00', '2099-01-01T00:00:00+00:00',
                           '2099-01-01T00:00:00+00:00'
                       )""",
                    (json.dumps({"timeoutSeconds": 30, "gpuReservationSessionId": "other"}),),
                )

            with (
                mock.patch.object(
                    CoordinatorConfig,
                    "enabled_target_roots",
                    new_callable=mock.PropertyMock,
                    return_value=(target_root,),
                ),
                mock.patch("tools.session_coordinator.server.WorkspaceCopyService"),
            ):
                CoordinatorApplication(config)

            with database.connect() as connection:
                event = connection.execute(
                    """SELECT payload_json FROM events
                       WHERE event_type='cargo.gpu_lane_startup_audit'
                       ORDER BY event_id DESC LIMIT 1"""
                ).fetchone()
            self.assertIsNotNone(event)
            payload = json.loads(event["payload_json"])
            self.assertEqual("2099-01-01T00:00:00+00:00", payload["reservationCompletedAt"])
            self.assertEqual(
                [{
                    "jobId": job.job_id,
                    "sessionId": "gpu-owner",
                    "status": "leased",
                    "targetDir": job.target_dir,
                    "createdAt": job.created_at.isoformat(),
                    "preReservation": True,
                }],
                payload["jobs"],
            )

    def test_isolated_configs_can_request_ephemeral_listeners(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first_repo = init_repo(root / "first")
            second_repo = init_repo(root / "second")
            first = CoordinatorConfig.for_repo(first_repo, port=0)
            second = CoordinatorConfig.for_repo(second_repo, port=0)

            with RunningCoordinator.start(first) as first_running:
                with RunningCoordinator.start(second) as second_running:
                    first_runtime = json.loads(first.runtime_path.read_text(encoding="utf-8"))
                    second_runtime = json.loads(second.runtime_path.read_text(encoding="utf-8"))

                    self.assertNotEqual(first_runtime["port"], second_runtime["port"])
                    self.assertEqual(first_running.base_url, f"http://127.0.0.1:{first_runtime['port']}")
                    self.assertEqual(second_running.base_url, f"http://127.0.0.1:{second_runtime['port']}")
                    self.assertEqual(
                        str(first_repo),
                        CoordinatorClient.from_runtime(first).health()["repo_root"],
                    )
                    self.assertEqual(
                        str(second_repo),
                        CoordinatorClient.from_runtime(second).health()["repo_root"],
                    )

    def test_fixed_listener_rejects_a_second_repository(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first_repo = init_repo(root / "first")
            second_repo = init_repo(root / "second")
            first = CoordinatorConfig.for_repo(
                first_repo,
                state_root=root / "first-state",
                port=0,
            )

            with RunningCoordinator.start(first) as first_running:
                second = CoordinatorConfig.for_repo(
                    second_repo,
                    state_root=root / "second-state",
                    port=first_running.httpd.server_address[1],
                )
                with self.assertRaises(OSError):
                    RunningCoordinator.start(second)

    def test_client_rejects_foreign_repository_at_descriptor_endpoint(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected_repo = init_repo(root / "expected")
            foreign_repo = init_repo(root / "foreign")
            expected = CoordinatorConfig.for_repo(
                expected_repo,
                state_root=root / "expected-state",
                port=0,
            )
            foreign = CoordinatorConfig.for_repo(
                foreign_repo,
                state_root=root / "foreign-state",
                port=0,
            )

            with RunningCoordinator.start(foreign) as foreign_running:
                expected.runtime_path.parent.mkdir(parents=True, exist_ok=True)
                expected.runtime_path.write_text(
                    json.dumps(
                        {
                            "host": "127.0.0.1",
                            "port": foreign_running.httpd.server_address[1],
                            "repository_key": expected.repository_key,
                            "token": foreign_running.token,
                        }
                    ),
                    encoding="utf-8",
                )

                with self.assertRaises(CoordinatorClientError) as rejected:
                    CoordinatorClient.from_runtime(expected).health()

            self.assertEqual("repository_mismatch", rejected.exception.code)

    def test_command_preflight_recovers_once_while_a_cargo_job_is_running(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", port=0
            )

            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                application.sessions.register(session_id="cargo-owner")
                with application.database.transaction() as connection:
                    connection.execute(
                        """
                        INSERT INTO cargo_jobs(
                            job_id, session_id, lane_kind, target_dir, target_key,
                            status, pid, command_json, created_at, last_heartbeat_at,
                            started_at
                        ) VALUES (
                            'active-job', 'cargo-owner', 'test', 'D:/cargo-targets/active',
                            'd:\\cargo-targets\\active', 'running', ?, '["cargo", "test"]',
                            '2026-08-03T00:00:00+00:00', '2026-08-03T00:00:00+00:00',
                            '2026-08-03T00:00:00+00:00'
                        )
                        """,
                        (os.getpid(),),
                    )

                original_identity = application.identity
                identity_calls = 0
                identity_lock = threading.Lock()
                first_identity_complete = threading.Event()

                def delayed_identity() -> dict[str, object]:
                    nonlocal identity_calls
                    with identity_lock:
                        identity_calls += 1
                        ordinal = identity_calls
                    if ordinal == 1:
                        time.sleep(0.3)
                    try:
                        return original_identity()
                    finally:
                        if ordinal == 1:
                            first_identity_complete.set()

                client = CoordinatorClient(
                    running.base_url,
                    running.token,
                    expected_repository_key=config.repository_key,
                    timeout_seconds=0.1,
                    command_timeout_seconds=2,
                )
                with mock.patch.object(
                    application, "identity", side_effect=delayed_identity
                ):
                    registered = client.command(
                        "session.register", {"session_id": "recovered-session"}
                    )
                    self.assertTrue(first_identity_complete.wait(1))

                with application.database.connect() as connection:
                    count = connection.execute(
                        "SELECT COUNT(*) FROM sessions WHERE session_id='recovered-session'"
                    ).fetchone()[0]

            self.assertEqual("recovered-session", registered["session"]["session_id"])
            self.assertEqual(2, identity_calls)
            self.assertEqual(1, count)

    def test_isolated_config_disables_host_artifact_sweeps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

        self.assertFalse(config.unmanaged_artifact_sweep_enabled)

    def test_maintenance_uses_local_runtime_when_no_capability_is_configured(self) -> None:
        maintenance_name = "ZIRCON_COORDINATOR_" + "MAINTENANCE_TOKEN"
        with mock.patch.dict("os.environ", {}, clear=True):
            self.assertTrue(CoordinatorApplication._authorize_maintenance({"maintenance": True}))

        with mock.patch.dict(
            "os.environ", {maintenance_name: "local-only"}
        ):
            self.assertTrue(
                CoordinatorApplication._authorize_maintenance(
                    {"maintenance": True, "maintenance_capability": "local-only"}
                )
            )

    def test_second_instance_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                with self.assertRaises(CoordinatorError) as duplicate:
                    RunningCoordinator.start(config)
            self.assertEqual("already_running", duplicate.exception.code)

    def test_startup_keeps_a_durable_maintenance_hold_in_draining_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            prepared = CoordinatorApplication(config)
            prepared.supervision.transition(
                SupervisionState.HEALTHY,
                reason_code="test.maintenance_hold",
                actor="test",
                updates={"maintenance_hold": 1},
            )

            with RunningCoordinator.start(config) as running:
                health = CoordinatorClient.from_runtime(config).health()
                self.assertEqual("draining", health["supervision"]["state"])
                self.assertTrue(health["supervision"]["maintenanceHold"])

    def test_runtime_descriptor_is_published_after_durable_hold_enters_draining(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            prepared = CoordinatorApplication(config)
            prepared.supervision.transition(
                SupervisionState.HEALTHY,
                reason_code="test.descriptor_order",
                actor="test",
                updates={"maintenance_hold": 1},
            )
            original_write = server._atomic_json_write
            published_states: list[str] = []

            def capture_runtime_state(path, payload) -> None:
                if path == config.runtime_path:
                    with Database(config.database_path).connect() as connection:
                        published_states.append(
                            connection.execute(
                                "SELECT state FROM service_recovery_state LIMIT 1"
                            ).fetchone()["state"]
                        )
                original_write(path, payload)

            with mock.patch.object(
                server, "_atomic_json_write", side_effect=capture_runtime_state
            ):
                with RunningCoordinator.start(config):
                    pass

        self.assertEqual(["draining"], published_states)

    def test_successor_rehydrates_scoped_maintenance_hold_from_drain_action(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            prepared = CoordinatorApplication(config)
            with prepared.database.transaction() as connection:
                connection.execute(
                    """INSERT INTO action_requests(
                           action_id, action_kind, risk, required_role, actor,
                           daemon_instance_id, parameters_json, impact_json, warnings_json,
                           state_fingerprint, confirmation_phrase_hash, status, created_at,
                           expires_at, completed_at
                       ) VALUES (
                           'scoped-drain', 'service.drain', 'red', 'maintainer', 'operator',
                           'daemon-a', ?, '[]', '[]', 'fingerprint', 'phrase', 'succeeded',
                           '2099-01-01T00:00:00+00:00', '2099-01-01T00:00:00+00:00',
                           '2099-01-01T00:00:00+00:00'
                       )""",
                    (
                        json.dumps(
                            {
                                "timeoutSeconds": 30,
                                "maintenanceSessionIds": [
                                    "executor-session",
                                    "reviewer-session",
                                ],
                            },
                            sort_keys=True,
                        ),
                    ),
                )
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status, requested_by,
                        source_daemon_instance_id, created_at, updated_at, completed_at, result_json
                    ) VALUES (
                        'scoped-drain-intent', ?, 'scoped-drain', 'service.drain', 'succeeded',
                        'operator', 'daemon-a', '2099-01-01T00:00:00+00:00',
                        '2099-01-01T00:00:00+00:00', '2099-01-01T00:00:00+00:00',
                        '{"admissionOpen": false, "proofBound": true}'
                    )
                    """,
                    (prepared.repository_identity.key,),
                )
                connection.execute(
                    """
                    INSERT INTO action_requests(
                        action_id, action_kind, risk, required_role, actor,
                        daemon_instance_id, parameters_json, impact_json, warnings_json,
                        state_fingerprint, confirmation_phrase_hash, status, created_at,
                        expires_at, completed_at
                    ) VALUES (
                        'stale-restart', 'service.restart', 'red', 'maintainer', 'operator',
                        'daemon-a', ?, '[]', '[]', 'fingerprint', 'phrase', 'succeeded',
                        '2101-01-01T00:00:00+00:00', '2101-01-01T00:00:00+00:00',
                        '2101-01-01T00:00:00+00:00'
                    )
                    """,
                    (
                        json.dumps(
                            {
                                "timeoutSeconds": 30,
                                "maintenanceSessionIds": ["restart-session"],
                            },
                            sort_keys=True,
                        ),
                    ),
                )
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status, requested_by,
                        source_daemon_instance_id, created_at, updated_at, completed_at, result_json
                    ) VALUES (
                        'stale-restart-intent', ?, 'stale-restart', 'service.restart', 'succeeded',
                        'operator', 'daemon-a', '2101-01-01T00:00:00+00:00',
                        '2101-01-01T00:00:00+00:00', '2101-01-01T00:00:00+00:00',
                        '{"state": "healthy"}'
                    )
                    """,
                    (prepared.repository_identity.key,),
                )
            prepared.supervision.transition(
                SupervisionState.DRAINING,
                reason_code="lifecycle.drain.accepted",
                actor="test",
                action_id="scoped-drain",
                updates={"maintenance_hold": 1},
            )
            prepared.supervision.transition(
                SupervisionState.DRAINING,
                reason_code="lifecycle.restart.accepted",
                actor="test",
            )

            with mock.patch.dict(
                "os.environ",
                {
                    "ZIRCON_COORDINATOR_MAINTENANCE_SESSION": "",
                    "ZIRCON_COORDINATOR_MAINTENANCE_SESSIONS": "",
                },
            ):
                successor = CoordinatorApplication(config)
            successor.supervision.mark_healthy()
            successor.supervision.transition(
                SupervisionState.DRAINING,
                reason_code="test.startup_maintenance_hold",
                actor="test",
            )

            successor.supervision.require_mutation_allowed(
                "lease.claim@executor-session"
            )
            with self.assertRaises(CoordinatorError) as rejected:
                successor.supervision.require_mutation_allowed("lease.claim@other-session")
            self.assertEqual("maintenance_hold_active", rejected.exception.code)
            with self.assertRaises(CoordinatorError) as stale_restart:
                successor.supervision.require_mutation_allowed(
                    "cargo.consume_cpu_reservation@restart-session"
                )
            self.assertEqual("maintenance_hold_active", stale_restart.exception.code)

    def test_successor_uses_only_a_proof_bound_drain_not_a_newer_legacy_drain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            prepared = CoordinatorApplication(config)
            with prepared.database.transaction() as connection:
                for action_id, completed_at, session_id in (
                    ("proof-drain", "2099-01-01T00:00:00+00:00", "hgi-session"),
                    ("legacy-drain", "2100-01-01T00:00:00+00:00", "legacy-session"),
                ):
                    connection.execute(
                        """
                        INSERT INTO action_requests(
                            action_id, action_kind, risk, required_role, actor,
                            daemon_instance_id, parameters_json, impact_json, warnings_json,
                            state_fingerprint, confirmation_phrase_hash, status, created_at,
                            expires_at, completed_at
                        ) VALUES (?, 'service.drain', 'red', 'maintainer', 'operator',
                                  'daemon-a', ?, '[]', '[]', 'fingerprint', 'phrase',
                                  'succeeded', ?, ?, ?)
                        """,
                        (
                            action_id,
                            json.dumps(
                                {
                                    "timeoutSeconds": 30,
                                    "maintenanceSessionIds": [session_id],
                                },
                                sort_keys=True,
                            ),
                            completed_at,
                            completed_at,
                            completed_at,
                        ),
                    )
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status, requested_by,
                        source_daemon_instance_id, created_at, updated_at, completed_at,
                        result_json
                    ) VALUES (
                        'proof-intent', ?, 'proof-drain', 'service.drain', 'succeeded',
                        'operator', 'daemon-a', '2099-01-01T00:00:00+00:00',
                        '2099-01-01T00:00:00+00:00', '2099-01-01T00:00:00+00:00',
                        '{"admissionOpen": false, "proofBound": true, "reservationId": "hgi-reservation"}'
                    )
                    """,
                    (prepared.repository_identity.key,),
                )
                connection.execute(
                    """
                    INSERT INTO service_lifecycle_intents(
                        intent_id, repository_key, action_id, kind, status, requested_by,
                        source_daemon_instance_id, created_at, updated_at, completed_at,
                        result_json
                    ) VALUES (
                        'legacy-intent', ?, 'legacy-drain', 'service.drain', 'succeeded',
                        'operator', 'daemon-a', '2100-01-01T00:00:00+00:00',
                        '2100-01-01T00:00:00+00:00', '2100-01-01T00:00:00+00:00',
                        '{"admissionOpen": false, "proofBound": false}'
                    )
                    """,
                    (prepared.repository_identity.key,),
                )
            prepared.supervision.transition(
                SupervisionState.DRAINING,
                reason_code="test.bootstrap_hold",
                actor="test",
                updates={"maintenance_hold": 1},
            )

            with mock.patch.dict(
                "os.environ",
                {
                    "ZIRCON_COORDINATOR_MAINTENANCE_SESSION": "",
                    "ZIRCON_COORDINATOR_MAINTENANCE_SESSIONS": "",
                },
            ):
                successor = CoordinatorApplication(config)
            successor.supervision.mark_healthy()
            successor.supervision.transition(
                SupervisionState.DRAINING,
                reason_code="test.startup_maintenance_hold",
                actor="test",
            )

            successor.supervision.require_mutation_allowed(
                "cargo.consume_cpu_reservation@hgi-session"
            )
            with self.assertRaises(CoordinatorError) as legacy:
                successor.supervision.require_mutation_allowed(
                    "cargo.consume_cpu_reservation@legacy-session"
                )
            self.assertEqual("maintenance_hold_active", legacy.exception.code)

    def test_bootstrap_proof_allows_only_its_exact_cpu_reservation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            application = CoordinatorApplication(config)
            application.supervision.mark_healthy()
            for session_id in ("hgi-owner", "repair-owner"):
                application.sessions.register(session_id=session_id)
                application.sessions.set_status(session_id, SessionStatus.ACTIVE)
            with application.database.transaction() as connection:
                for reservation_id, session_id in (
                    ("hgi-reservation", "hgi-owner"),
                    ("other-reservation", "repair-owner"),
                ):
                    connection.execute(
                        """
                        INSERT INTO cargo_lane_reservations(
                            reservation_id, session_id, lane_scope, compatibility_key,
                            compatibility_json, command_fingerprint, job_id, status, created_at, expires_at,
                            execution_mode, burst_eligible, priority_rank
                        ) VALUES (?, ?, 'cpu', 'compat',
                                  '{"source_manifest":{"owned.txt":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}}',
                                  'command', NULL, 'pending',
                                  '2026-07-19T00:00:00+00:00', '2099-07-19T00:00:00+00:00',
                                  'warm', 0, 1000)
                        """,
                        (reservation_id, session_id),
                    )
            application.supervision.bootstrap_proof_bound_handoff(
                reservation_id="hgi-reservation",
                maintenance_session_ids=("repair-owner", "hgi-owner"),
                actor="bootstrap-owner",
            )

            with self.assertRaises(CoordinatorError) as generic:
                application.command(
                    "cargo.reserve_cpu",
                    {
                        "session_id": "hgi-owner",
                        "compatibility": {
                            "platform": "windows",
                            "toolchain": "rustc 1.94.1",
                            "target_architecture": "x86_64-pc-windows-msvc",
                            "workspace": "zircon-engine-root",
                            "build_config": "profile=test",
                        },
                        "target_dir": None,
                        "command": ["cargo", "test"],
                    },
                )
            self.assertEqual("maintenance_hold_active", generic.exception.code)
            with self.assertRaises(CoordinatorError) as other:
                application.command(
                    "cargo.consume_cpu_reservation",
                    {
                        "session_id": "repair-owner",
                        "reservation_id": "other-reservation",
                        "lane_kind": "test",
                    },
                )
            self.assertEqual("maintenance_proof_reservation_mismatch", other.exception.code)

    def test_bootstrap_invalidates_a_generic_reservation_authorized_before_the_hold(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            owned = repo / "owned.txt"
            owned.write_text("owned\n", encoding="utf-8")
            owned_hash = hashlib.sha256(owned.read_bytes()).hexdigest().upper()
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            application = CoordinatorApplication(config)
            application.supervision.mark_healthy()
            for session_id in ("hgi-owner", "repair-owner", "generic-owner"):
                application.sessions.register(session_id=session_id)
                application.sessions.set_status(session_id, SessionStatus.ACTIVE)
            with application.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_lane_reservations(
                        reservation_id, session_id, lane_scope, compatibility_key,
                        compatibility_json, command_fingerprint, job_id, status, created_at,
                        expires_at, execution_mode, burst_eligible, priority_rank
                    ) VALUES (
                        'hgi-reservation', 'hgi-owner', 'cpu', 'hgi-compat',
                        ?, 'hgi-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                        '2099-07-19T00:00:00+00:00', 'warm', 0, 1000
                    )
                    """,
                    (
                        json.dumps(
                            {"source_manifest": {"owned.txt": owned_hash}}, sort_keys=True
                        ),
                    ),
                )
            checkpoint = application.supervision.require_mutation_allowed(
                "cargo.reserve_cpu@generic-owner"
            )
            application.supervision.bootstrap_proof_bound_handoff(
                reservation_id="hgi-reservation",
                maintenance_session_ids=("repair-owner", "hgi-owner"),
                actor="bootstrap-owner",
            )

            with self.assertRaises(CoordinatorError) as stale:
                application._command_unlocked(
                    "cargo.reserve_cpu",
                    {
                        "session_id": "generic-owner",
                        "compatibility": {
                            "platform": "windows",
                            "toolchain": "rustc 1.94.1",
                            "target_architecture": "x86_64-pc-windows-msvc",
                            "workspace": "zircon-engine-root",
                            "build_config": "profile=test",
                            "source_manifest": {"owned.txt": owned_hash},
                        },
                        "target_dir": None,
                        "command": ["cargo", "test"],
                    },
                    admission_checkpoint=checkpoint,
                )
            self.assertEqual("admission_checkpoint_stale", stale.exception.code)
            with application.database.connect() as connection:
                generic_reservations = connection.execute(
                    "SELECT count(*) FROM cargo_lane_reservations WHERE session_id='generic-owner'"
                ).fetchone()[0]
            self.assertEqual(0, generic_reservations)
            self.assertTrue(application.supervision.snapshot().maintenance_hold)

    def test_post_handoff_audit_keeps_hold_when_a_legacy_request_lands_after_proof(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            application = CoordinatorApplication(config)
            application.supervision.mark_healthy()
            for session_id in ("hgi-owner", "repair-owner", "legacy-owner"):
                application.sessions.register(session_id=session_id)
                application.sessions.set_status(session_id, SessionStatus.ACTIVE)
            with application.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_lane_reservations(
                        reservation_id, session_id, lane_scope, compatibility_key,
                        compatibility_json, command_fingerprint, job_id, status, created_at,
                        expires_at, execution_mode, burst_eligible, priority_rank
                    ) VALUES (
                        'hgi-reservation', 'hgi-owner', 'cpu', 'hgi-compat',
                        '{"source_manifest":{"owned.txt":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}}',
                        'hgi-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                        '2099-07-19T00:00:00+00:00', 'warm', 0, 1000
                    )
                    """
                )
            handoff = application.supervision.bootstrap_proof_bound_handoff(
                reservation_id="hgi-reservation",
                maintenance_session_ids=("repair-owner", "hgi-owner"),
                actor="bootstrap-owner",
            )
            with application.database.transaction() as connection:
                connection.execute(
                    """
                    INSERT INTO cargo_lane_reservations(
                        reservation_id, session_id, lane_scope, compatibility_key,
                        compatibility_json, command_fingerprint, job_id, status, created_at,
                        expires_at, execution_mode, burst_eligible, priority_rank
                    ) VALUES (
                        'legacy-pending', 'legacy-owner', 'cpu', 'legacy-compat',
                        '{}', 'legacy-command', NULL, 'pending', '2026-07-19T00:01:00+00:00',
                        '2099-07-19T00:01:00+00:00', 'warm', 0, 1000
                    )
                    """
                )
                connection.execute(
                    """
                    INSERT INTO cargo_jobs(
                        job_id, session_id, lane_kind, target_dir, status, command_json,
                        created_at, last_heartbeat_at
                    ) VALUES (
                        'legacy-job', 'legacy-owner', 'test', 'D:/cargo-targets/legacy',
                        'leased', '[]', '2026-07-19T00:01:00+00:00', '2026-07-19T00:01:00+00:00'
                    )
                    """
                )
                connection.execute(
                    """
                    INSERT INTO events(session_id, event_type, payload_json, created_at)
                    VALUES ('legacy-owner', 'cargo.acquired', '{}', '2026-07-19T00:01:00+00:00')
                    """
                )

            audit = validate_proof_bound_handoff(
                config,
                action_id=handoff["actionId"],
                reservation_id="hgi-reservation",
            )

            self.assertFalse(audit["ready"])
            self.assertTrue(application.supervision.snapshot().maintenance_hold)
            self.assertTrue(any(item["kind"] == "cargo" for item in audit["blockers"]))
            self.assertTrue(
                any(item["kind"] == "post_proof_cargo_event" for item in audit["blockers"])
            )
            self.assertTrue(
                any(
                    item["kind"] == "post_proof_reservation_ledger_drift"
                    for item in audit["blockers"]
                )
            )

    def test_local_health_identity_and_session_commands_require_runtime_token(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            (repo / "owned.txt").write_text("owned\n", encoding="utf-8")

            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                health = client.health()
                identity = client._request("GET", "/identity")
                registered = client.command(
                    "session.register",
                    {"session_id": "session-a", "write_scope": ["owned.txt"]},
                )
                active = client.command(
                    "session.set_status", {"session_id": "session-a", "status": "active"}
                )
                claimed = client.command(
                    "lease.claim", {"session_id": "session-a", "paths": ["owned.txt"]}
                )
                heartbeat = client.command("session.heartbeat", {"session_id": "session-a"})

                self.assertEqual("ok", health["status"])
                self.assertEqual(
                    {
                        "instance_id",
                        "process_creation_time",
                        "repository_key",
                        "schema_version",
                        "status",
                    },
                    set(identity),
                )
                self.assertEqual(config.repository_key, identity["repository_key"])
                self.assertNotIn("supervision", identity)
                self.assertNotIn("token", identity)
                self.assertEqual("registered", registered["session"]["status"])
                self.assertEqual("active", active["session"]["status"])
                self.assertTrue(claimed["lease"]["acquired"])
                self.assertEqual(1, heartbeat["leases"]["renewed"])

                request = urllib.request.Request(
                    f"{running.base_url}/command",
                    data=json.dumps({"command": "session.list", "arguments": {}}).encode("utf-8"),
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                with self.assertRaises(urllib.error.HTTPError) as rejected:
                    urllib.request.urlopen(request, timeout=2)
                self.assertEqual(401, rejected.exception.code)
                self.assertEqual(
                    "unauthorized",
                    json.loads(rejected.exception.read())["error"]["code"],
                )
                rejected.exception.close()

    def test_baseline_attribution_requires_the_session_live_lease(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            target = repo / "owned.txt"
            target.write_text("owned change\n", encoding="utf-8")

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                client.command("session.register", {"session_id": "session-a"})
                client.command(
                    "session.set_status", {"session_id": "session-a", "status": "active"}
                )
                with self.assertRaises(CoordinatorClientError) as rejected:
                    client.command(
                        "baseline.attribute",
                        {"session_id": "session-a", "paths": ["owned.txt"]},
                    )
                client.command(
                    "lease.claim", {"session_id": "session-a", "paths": ["owned.txt"]}
                )
                attributed = client.command(
                    "baseline.attribute",
                    {"session_id": "session-a", "paths": ["owned.txt"]},
                )

            self.assertEqual("baseline_lease_missing", rejected.exception.code)
            self.assertEqual("attributed", attributed["status"])

    def test_authenticated_tray_recovery_command_updates_health_projection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                result = client.command(
                    "supervision.recovery_record",
                    {
                        "failureCount": 2,
                        "failureWindowStartedAt": 100,
                        "nextRetryAt": 105,
                        "circuitOpenUntil": None,
                        "healthySince": None,
                    },
                )
                health = client.health()

            self.assertEqual(2, result["supervision"]["failureCount"])
            self.assertEqual(2, health["supervision"]["failureCount"])

    def test_stale_runtime_descriptor_is_reported_as_offline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            config.state_root.mkdir(parents=True)
            config.runtime_path.write_text(
                json.dumps({"host": "127.0.0.1", "port": 1, "token": "stale", "pid": 999999}),
                encoding="utf-8",
            )

            with self.assertRaises(CoordinatorClientError) as offline:
                CoordinatorClient.from_runtime(config).health()
            self.assertEqual("offline", offline.exception.code)

    def test_non_main_checkout_is_read_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            subprocess.run(["git", "switch", "-q", "-c", "temporary-test"], cwd=repo, check=True)
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                self.assertEqual("read_only", client.health()["mode"])
                with self.assertRaises(CoordinatorClientError) as rejected:
                    client.command("session.register", {"session_id": "session-a"})
            self.assertEqual("not_on_main", rejected.exception.code)

    def test_non_main_startup_and_maintenance_do_not_mutate_shared_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            subprocess.run(["git", "switch", "-q", "-c", "temporary-test"], cwd=repo, check=True)
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            target_root = root / "D" / "cargo-targets"
            target_root.mkdir(parents=True)
            governance = mock.Mock()
            cargo_jobs = mock.Mock()
            cargo_jobs.audit_active_gpu_jobs.return_value = ()
            cargo_runner = mock.Mock()
            cleanup = mock.Mock()
            workspace_copy = mock.Mock()
            validation_ticket_worker = mock.Mock()
            benchmark_validation_grants = mock.Mock()
            milestone_workflows = mock.Mock()

            with (
                mock.patch.object(
                    CoordinatorConfig,
                    "enabled_target_roots",
                    new_callable=mock.PropertyMock,
                    return_value=(target_root,),
                ),
                mock.patch.object(
                    CoordinatorConfig,
                    "unmanaged_artifact_sweep_enabled",
                    new_callable=mock.PropertyMock,
                    return_value=True,
                ),
                mock.patch(
                    "tools.session_coordinator.server.ArtifactGovernanceService",
                    return_value=governance,
                ),
                mock.patch(
                    "tools.session_coordinator.server.CargoJobService",
                    return_value=cargo_jobs,
                ),
                mock.patch(
                    "tools.session_coordinator.server.CargoJobRunner",
                    return_value=cargo_runner,
                ),
                mock.patch(
                    "tools.session_coordinator.server.CleanupService",
                    return_value=cleanup,
                ),
                mock.patch(
                    "tools.session_coordinator.server.WorkspaceCopyService",
                    return_value=workspace_copy,
                ),
                mock.patch(
                    "tools.session_coordinator.server.ValidationTicketWorker",
                    return_value=validation_ticket_worker,
                ),
                mock.patch(
                    "tools.session_coordinator.server.BenchmarkValidationGrantService",
                    return_value=benchmark_validation_grants,
                ),
                mock.patch(
                    "tools.session_coordinator.server.MilestoneWorkflowService",
                    return_value=milestone_workflows,
                ),
            ):
                application = CoordinatorApplication(config)

            self.assertTrue(application.read_only)
            governance.recover_reservations.assert_not_called()
            cargo_jobs.reconcile_orphans.assert_not_called()
            cargo_runner.reconcile_terminal_runs.assert_not_called()
            cleanup.recover_reservations.assert_not_called()
            workspace_copy.recover_interrupted_jobs.assert_not_called()
            benchmark_validation_grants.reconcile_interrupted_consumed.assert_not_called()
            milestone_workflows.recover_validation_results.assert_not_called()
            stop_event = mock.Mock()
            stop_event.wait.side_effect = (False, True)
            stop_event.is_set.return_value = False

            RunningCoordinator._maintenance_loop(application, 0.05, 60, stop_event)

            governance.cleanup.assert_not_called()
            cargo_jobs.reconcile_orphans.assert_not_called()
            cargo_runner.reconcile_terminal_runs.assert_not_called()
            cargo_jobs.reconcile_pending_reservations.assert_not_called()
            cleanup.retry_pending_jobs.assert_not_called()
            cleanup.evict_idle_pools_under_pressure.assert_not_called()
            workspace_copy.recover_interrupted_jobs.assert_not_called()
            validation_ticket_worker.tick.assert_not_called()

    def test_background_watcher_marks_external_drift_degraded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                client = CoordinatorClient.from_runtime(config)
                client.command("baseline.init")
                (repo / "README.md").write_text("external\n", encoding="utf-8")
                health = "healthy"
                for _ in range(200):
                    health = client.command("baseline.status")["baseline"]["health"]
                    if health == "degraded":
                        break
                    time.sleep(0.05)
            self.assertEqual("degraded", health)

    def test_daemon_runs_retention_maintenance_without_external_scheduler(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            with RunningCoordinator.start(config):
                tick_count = 0
                for _ in range(200):
                    with Database(config.database_path).connect() as connection:
                        tick_count = int(
                            connection.execute(
                                "SELECT COUNT(*) FROM maintenance_ticks WHERE status = 'succeeded'"
                            ).fetchone()[0]
                        )
                    if tick_count:
                        break
                    time.sleep(0.05)

            self.assertGreaterEqual(tick_count, 1)

    def test_stop_waits_for_inflight_maintenance_and_skips_later_phases(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=60,
            )
            running = RunningCoordinator.start(config)
            entered = threading.Event()
            release = threading.Event()
            stopped = threading.Event()
            stop_errors: list[BaseException] = []
            original_scan = running.httpd.application.watcher.prepare_scan

            def blocked_scan():
                entered.set()
                if not release.wait(30):
                    raise AssertionError("maintenance phase was never released")
                return original_scan()

            def stop() -> None:
                try:
                    running.stop()
                except BaseException as error:
                    stop_errors.append(error)
                finally:
                    stopped.set()

            try:
                with (
                    mock.patch.object(
                        running.httpd.application.watcher,
                        "prepare_scan",
                        side_effect=blocked_scan,
                    ),
                    mock.patch.object(
                        running.httpd.application.cargo_jobs,
                        "reconcile_orphans",
                    ) as later_phase,
                ):
                    self.assertTrue(entered.wait(5))
                    later_phase.reset_mock()
                    stopper = threading.Thread(target=stop, daemon=True)
                    stopper.start()
                    self.assertFalse(stopped.wait(5.5))
                    release.set()
                    stopper.join(10)

                    self.assertTrue(stopped.is_set())
                    self.assertEqual([], stop_errors)
                    self.assertFalse(running.maintenance_thread.is_alive())
                    later_phase.assert_not_called()
            finally:
                release.set()
                running.maintenance_stop.set()
                running.maintenance_thread.join(10)

    def test_daemon_periodically_imports_and_archives_inactive_root_note(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            session_root = repo / ".codex/sessions"
            session_root.mkdir(parents=True)
            note = session_root / "old.md"
            note.write_text(
                "---\nsession: old\nstatus: stale\n---\n\n# Old\n",
                encoding="utf-8",
            )
            old_time = time.time() - 2 * 86400
            os.utime(note, (old_time, old_time))
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            # Legacy-note maintenance is the contract under test.  Do not spend its
            # short synchronization window scanning host target pools or building a
            # real workspace observation first.
            with (
                mock.patch.object(
                    CoordinatorConfig,
                    "enabled_target_roots",
                    new_callable=mock.PropertyMock,
                    return_value=(),
                ),
                mock.patch.object(WorkspaceWatcher, "prepare_scan", return_value=object()),
                mock.patch.object(WorkspaceWatcher, "apply_scan", return_value=None),
                RunningCoordinator.start(config),
            ):
                archived = session_root / "archive/old.md"
                for _ in range(100):
                    if archived.exists():
                        break
                    time.sleep(0.02)

            self.assertTrue(archived.exists())
            self.assertFalse(note.exists())

    def test_daemon_never_stales_or_archives_live_pid_root_note(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            session_root = repo / ".codex/sessions"
            session_root.mkdir(parents=True)
            note = session_root / "live.md"
            note.write_text(
                f"---\nsession: live\nstatus: completed\npid: {os.getpid()}\n---\n",
                encoding="utf-8",
            )
            old_time = time.time() - 2 * 86400
            os.utime(note, (old_time, old_time))
            config = CoordinatorConfig.for_repo(
                repo,
                state_root=root / "state",
                port=0,
                watch_interval_seconds=0.02,
                maintenance_interval_seconds=0.05,
            )

            # An isolated coordinator must not spend the test's synchronization
            # window reconciling real host target pools before it reaches legacy
            # note maintenance.
            with mock.patch.object(
                CoordinatorConfig,
                "enabled_target_roots",
                new_callable=mock.PropertyMock,
                return_value=(),
            ), mock.patch.object(WorkspaceWatcher, "prepare_scan", return_value=object()), mock.patch.object(
                WorkspaceWatcher, "apply_scan", return_value=None
            ):
                with RunningCoordinator.start(config) as running:
                    status = None
                    tick = None
                    application = running.httpd.application
                    sync_before = application.codex_worker.snapshot()["successfulRuns"]
                    application.codex_worker.wake("controlled")
                    sync = None
                    sync_deadline = time.monotonic() + 5.0
                    while time.monotonic() < sync_deadline:
                        candidate = application.codex_worker.snapshot()
                        if (
                            candidate["successfulRuns"] > sync_before
                            and candidate["lastRunId"]
                            and candidate["state"] != "running"
                        ):
                            sync = candidate
                            break
                        time.sleep(0.02)
                    self.assertIsNotNone(sync, "Codex discovery did not become idle after wake")
                    with Database(config.database_path).connect() as connection:
                        sync_run = connection.execute(
                            "SELECT source_revision FROM codex_sync_runs WHERE run_id=?",
                            (sync["lastRunId"],),
                        ).fetchone()
                    self.assertIsNotNone(sync_run)
                    self.assertTrue(sync_run[0])

                    tick_deadline = time.monotonic() + 5.0
                    while time.monotonic() < tick_deadline:
                        with Database(config.database_path).connect() as connection:
                            tick = connection.execute(
                                "SELECT 1 FROM maintenance_ticks WHERE status='succeeded' LIMIT 1"
                            ).fetchone()
                            row = connection.execute(
                                "SELECT status FROM sessions WHERE session_id = 'live'"
                            ).fetchone()
                        if tick is not None and row is not None:
                            status = row[0]
                            break
                        time.sleep(0.02)

            self.assertTrue(note.exists())
            self.assertIsNotNone(tick, "daemon maintenance did not complete")
            self.assertEqual("active", status)

    def test_destructive_legacy_import_requires_configured_operator_capability(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            note_root = repo / ".codex/sessions"
            note_root.mkdir(parents=True)
            (note_root / "legacy.md").write_text(
                "---\nsession: legacy\nstatus: stale\n---\n",
                encoding="utf-8",
            )
            maintenance_name = "ZIRCON_COORDINATOR_" + "MAINTENANCE_TOKEN"
            with mock.patch.dict(
                "os.environ",
                {maintenance_name: "local-only"},
            ):
                application = CoordinatorApplication(
                    CoordinatorConfig.for_repo(repo, state_root=root / "state")
                )
                with self.assertRaises(CoordinatorError) as rejected:
                    application.command("legacy.import", {"apply": True})

            self.assertEqual("maintenance_unauthorized", rejected.exception.code)

    def test_registration_prioritizes_open_failure_for_numbered_plan(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/editor/01-editor.md")
            fixing = fixture.add_plan("docs/plans/runtime/02-runtime.md")
            fixture.add_handoff(origin, fixing, "provider")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

            with RunningCoordinator.start(config):
                result = CoordinatorClient.from_runtime(config).command(
                    "session.register",
                    {
                        "session_id": "session-a",
                        "plan_path": fixing.path.relative_to(repo).as_posix(),
                    },
                )

            self.assertEqual("resolving_failure", result["session"]["status"])
            self.assertEqual(["provider"], [item["summary_slug"] for item in result["open_failures"]])

    def test_registration_reopens_a_stale_owner_into_failure_resolution_without_partial_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            origin = fixture.add_plan("docs/plans/editor/01-editor.md")
            fixing = fixture.add_plan("docs/plans/runtime/02-runtime.md")
            fixture.add_handoff(origin, fixing, "provider")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="session-a",
                display_name="before",
                plan_path=fixing.path.relative_to(repo).as_posix(),
            )
            application.sessions.set_status("session-a", SessionStatus.ACTIVE)
            application.sessions.set_status("session-a", SessionStatus.STALE)

            result = application.command(
                "session.register",
                {
                    "session_id": "session-a",
                    "display_name": "after",
                    "plan_path": fixing.path.relative_to(repo).as_posix(),
                },
            )

            self.assertEqual("resolving_failure", result["session"]["status"])
        self.assertEqual("after", result["session"]["display_name"])
        self.assertEqual(["provider"], [item["summary_slug"] for item in result["open_failures"]])

    def test_registration_snapshot_parse_does_not_hold_the_database_writer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/runtime/01-runtime.md")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(session_id="copy-session")
            parse_started = threading.Event()
            release_parse = threading.Event()
            registration_done = threading.Event()
            materialize_done = threading.Event()
            errors: list[BaseException] = []
            original_parse = application.failures._parse_immutable_snapshot

            def blocked_parse(*args, **kwargs):
                parse_started.set()
                release_parse.wait(timeout=5)
                return original_parse(*args, **kwargs)

            def register() -> None:
                try:
                    application.execute_command_request(
                        "session.register",
                        {
                            "session_id": "registering-session",
                            "plan_path": plan.path.relative_to(repo).as_posix(),
                        },
                        request_id="a" * 32,
                    )
                except BaseException as error:
                    errors.append(error)
                finally:
                    registration_done.set()

            def materialize() -> None:
                try:
                    application.execute_command_request(
                        "validation_copy.materialize",
                        {"session_id": "copy-session", "paths": ["README.md"]},
                        request_id="b" * 32,
                    )
                except BaseException as error:
                    errors.append(error)
                finally:
                    materialize_done.set()

            record = WorkspaceCopyRecord(
                "copy-job",
                "copy-session",
                root / "copy-job",
                root / "copy-job/source",
                root / "copy-job/target",
                ("README.md",),
                "materializing",
            )
            with (
                mock.patch.object(
                    application.failures,
                    "_parse_immutable_snapshot",
                    side_effect=blocked_parse,
                ),
                mock.patch.object(
                    application, "_require_artifact_governance_clean"
                ),
                mock.patch.object(
                    application.workspace_copy,
                    "materialize_async",
                    return_value=record,
                ),
            ):
                registration = threading.Thread(target=register, daemon=True)
                registration.start()
                self.assertTrue(parse_started.wait(timeout=1))
                copy_admission = threading.Thread(target=materialize, daemon=True)
                copy_admission.start()
                try:
                    self.assertTrue(
                        materialize_done.wait(timeout=1),
                        "validation-copy admission waited behind failure snapshot parsing",
                    )
                finally:
                    release_parse.set()
                    registration.join(timeout=5)
                    copy_admission.join(timeout=5)

            self.assertTrue(registration_done.is_set())
            self.assertEqual([], errors)

    def test_planless_registration_does_not_prepare_or_import_failure_snapshot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()

            with (
                mock.patch.object(
                    application.failures,
                    "prepare_import_snapshot",
                    side_effect=AssertionError(
                        "planless registration parsed the repository failure graph"
                    ),
                ) as prepared,
                mock.patch.object(
                    application.failures,
                    "import_repository",
                    side_effect=AssertionError(
                        "planless registration imported the repository failure graph"
                    ),
                ) as imported,
            ):
                result = application.execute_command_request(
                    "session.register",
                    {"session_id": "planless-maintenance"},
                    request_id="e" * 32,
                )
                repeated = application.execute_command_request(
                    "session.register",
                    {"session_id": "planless-maintenance"},
                    request_id="d" * 32,
                )

            self.assertEqual("registered", result["session"]["status"])
            self.assertEqual([], result["open_failures"])
            self.assertEqual("planless-maintenance", repeated["session"]["session_id"])
            self.assertEqual([], repeated["open_failures"])
            prepared.assert_not_called()
            imported.assert_not_called()

    def test_existing_plan_registration_still_prepares_failure_snapshot(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            fixture = FailureGraphFixture(repo)
            plan = fixture.add_plan("docs/plans/runtime/01-runtime.md")
            plan_path = plan.path.relative_to(repo).as_posix()
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()
            application.sessions.register(
                session_id="plan-owner",
                plan_path=plan_path,
                write_scope=[plan_path],
            )

            with (
                mock.patch.object(
                    application.failures,
                    "prepare_import_snapshot",
                    wraps=application.failures.prepare_import_snapshot,
                ) as prepared,
                mock.patch.object(
                    application.failures,
                    "import_prepared_snapshot",
                    wraps=application.failures.import_prepared_snapshot,
                ) as imported,
            ):
                result = application.execute_command_request(
                    "session.register",
                    {"session_id": "plan-owner"},
                    request_id="f" * 32,
                )

            self.assertEqual(plan_path, result["session"]["plan_path"])
            prepared.assert_called_once_with()
            imported.assert_called_once()

    def test_registration_race_to_plan_fails_closed_without_writer_parse(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            )
            application.supervision.mark_healthy()

            with (
                mock.patch.object(
                    application,
                    "_session_registration_effective_plan",
                    side_effect=(None, "docs/plans/runtime/01-runtime.md"),
                ),
                mock.patch.object(
                    application.failures, "prepare_import_snapshot"
                ) as prepared,
                mock.patch.object(application.failures, "import_repository") as imported,
                self.assertRaises(CoordinatorError) as rejected,
            ):
                application.execute_command_request(
                    "session.register",
                    {"session_id": "racing-maintenance"},
                    request_id="1" * 32,
                )

            self.assertEqual("failure_snapshot_missing", rejected.exception.code)
            prepared.assert_not_called()
            imported.assert_not_called()

    def test_database_busy_diagnostic_does_not_terminate_maintenance_loop(self) -> None:
        application = mock.Mock()
        application.read_only = False
        application.cargo_jobs = None
        application.artifact_governance = None
        application.validation_ticket_worker = None
        application.watcher.prepare_scan.return_value = mock.sentinel.observation
        recovery_attempted = threading.Event()

        def fail_recovery(*_args, **_kwargs):
            recovery_attempted.set()
            raise sqlite3.OperationalError("database is locked")

        application.workspace_copy.recover_interrupted_jobs.side_effect = fail_recovery
        application.database.transaction.side_effect = sqlite3.OperationalError(
            "database is locked"
        )
        stop = threading.Event()
        worker = threading.Thread(
            target=RunningCoordinator._maintenance_loop,
            args=(application, 0.01, 60, stop),
            daemon=True,
        )
        worker.start()
        try:
            self.assertTrue(recovery_attempted.wait(timeout=1))
            time.sleep(0.05)
            self.assertTrue(
                worker.is_alive(),
                "maintenance loop exited after its DB-busy diagnostic write failed",
            )
        finally:
            stop.set()
            worker.join(timeout=1)

    def test_foreground_mutation_is_not_blocked_by_slow_workspace_observation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state")
            application = CoordinatorApplication(config)
            application.supervision.mark_healthy()
            started = threading.Event()
            release = threading.Event()
            stop = threading.Event()
            observation = application.watcher.prepare_scan()
            original_apply = application.watcher.apply_scan

            def slow_apply(received):
                started.set()
                release.wait()
                return original_apply(received)

            with (
                mock.patch.object(application.watcher, "prepare_scan", return_value=observation),
                mock.patch.object(application.watcher, "apply_scan", side_effect=slow_apply),
            ):
                worker = threading.Thread(
                    target=RunningCoordinator._maintenance_loop,
                    args=(application, 0.01, 60, stop),
                    daemon=True,
                )
                worker.start()
                self.assertTrue(started.wait(timeout=1))
                completed = threading.Event()
                outcome: dict[str, object] = {}
                errors: list[BaseException] = []

                def register() -> None:
                    try:
                        outcome["result"] = application.command(
                            "session.register", {"session_id": "session-a"}
                        )
                    except BaseException as error:
                        errors.append(error)
                    finally:
                        completed.set()

                foreground = threading.Thread(target=register, daemon=True)
                foreground.start()
                try:
                    self.assertTrue(
                        completed.wait(timeout=5),
                        "session.register remained blocked until workspace observation released",
                    )
                    self.assertEqual([], errors)
                finally:
                    release.set()
                    stop.set()
                    foreground.join(timeout=1)
                    worker.join(timeout=1)

            self.assertEqual("registered", outcome["result"]["session"]["status"])

    def test_legacy_finalize_milestone_is_rejected_before_it_can_bypass_workflow(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()

            with self.assertRaises(CoordinatorError) as rejected:
                application.command("finalize.milestone", {})

        self.assertEqual("legacy_milestone_finalize_forbidden", rejected.exception.code)

    def test_numbered_plan_session_cannot_be_completed_by_generic_status_command(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            plan = repo / "docs/plans/runtime/01-runtime.md"
            plan.parent.mkdir(parents=True)
            plan.write_text("# Runtime\n", encoding="utf-8")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            application.command(
                "session.register",
                {"session_id": "session-a", "plan_path": "docs/plans/runtime/01-runtime.md"},
            )
            application.command(
                "session.set_status", {"session_id": "session-a", "status": "active"}
            )

            with self.assertRaises(CoordinatorError) as rejected:
                application.command(
                    "session.set_status", {"session_id": "session-a", "status": "completed"}
                )

        self.assertEqual("session_goal_close_requires_milestone", rejected.exception.code)

    def test_foreground_mutation_is_not_blocked_by_long_control_action(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            action_lock = application.control_actions._confirmation_lock
            self.assertIsNot(action_lock, application._mutation_lock)
            acquired = threading.Event()
            release = threading.Event()
            completed = threading.Event()
            result: dict[str, object] = {}
            errors: list[BaseException] = []

            def occupy_control_action() -> None:
                with action_lock:
                    acquired.set()
                    release.wait(timeout=10)

            def register_session() -> None:
                try:
                    result.update(
                        application.command("session.register", {"session_id": "session-a"})
                    )
                except BaseException as error:
                    errors.append(error)
                finally:
                    completed.set()

            worker = threading.Thread(target=occupy_control_action, daemon=True)
            worker.start()
            self.assertTrue(acquired.wait(timeout=1))
            foreground = threading.Thread(target=register_session, daemon=True)
            foreground.start()
            try:
                self.assertTrue(
                    completed.wait(timeout=5),
                    "session.register remained blocked by the control action lock",
                )
                self.assertEqual([], errors)
            finally:
                release.set()
                foreground.join(timeout=1)
                worker.join(timeout=1)

        self.assertEqual("registered", result["session"]["status"])

    def test_foreground_mutation_is_not_blocked_by_manual_workspace_scan(self) -> None:
        """An on-demand diagnostic scan must not own the foreground command mutex."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            scan_started = threading.Event()
            release_scan = threading.Event()
            register_entered = threading.Event()
            mutation_finished = threading.Event()
            result: dict[str, object] = {}

            def slow_scan():
                scan_started.set()
                release_scan.wait(timeout=2)
                return []

            def register_session() -> None:
                result.update(application.command("session.register", {"session_id": "session-a"}))
                mutation_finished.set()

            original_register = application.sessions.register

            def observe_register(*args, **kwargs):
                register_entered.set()
                return original_register(*args, **kwargs)

            with (
                mock.patch.object(application.watcher, "scan_once", side_effect=slow_scan),
                mock.patch.object(application.sessions, "register", side_effect=observe_register),
            ):
                scan_worker = threading.Thread(
                    target=lambda: application.command("watch.scan", {}), daemon=True
                )
                scan_worker.start()
                self.assertTrue(scan_started.wait(timeout=1))
                foreground_worker = threading.Thread(target=register_session, daemon=True)
                foreground_worker.start()
                try:
                    # This is the exact boundary protected by the foreground mutex.
                    # Do not include SQLite scheduling in the non-blocking assertion.
                    self.assertTrue(register_entered.wait(timeout=1))
                finally:
                    release_scan.set()
                    scan_worker.join(timeout=1)
                    foreground_worker.join(timeout=1)

        self.assertEqual("registered", result["session"]["status"])

    def test_foreground_mutation_is_not_blocked_by_baseline_scan(self) -> None:
        """A HEAD refresh prepares outside the foreground mutation mutex."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            scan_started = threading.Event()
            release_scan = threading.Event()
            mutation_finished = threading.Event()
            result: dict[str, object] = {}

            def slow_scan():
                scan_started.set()
                release_scan.wait()
                return []

            def register_session() -> None:
                try:
                    result.update(
                        application.command("session.register", {"session_id": "session-a"})
                    )
                except BaseException as error:
                    foreground_error.append(error)
                finally:
                    mutation_finished.set()

            with mock.patch.object(application.baselines, "scan", side_effect=slow_scan):
                scan_worker = threading.Thread(
                    target=lambda: application.command("baseline.scan", {}), daemon=True
                )
                scan_worker.start()
                self.assertTrue(scan_started.wait(timeout=1))
                foreground_error: list[BaseException] = []
                foreground_worker = threading.Thread(target=register_session, daemon=True)
                foreground_worker.start()
                try:
                    self.assertTrue(
                        mutation_finished.wait(timeout=5),
                        "session.register did not finish while baseline.scan remained blocked",
                    )
                    self.assertFalse(foreground_error, repr(foreground_error))
                finally:
                    release_scan.set()
                    scan_worker.join(timeout=1)
                    foreground_worker.join(timeout=1)

        self.assertEqual("registered", result["session"]["status"])

    def test_disconnected_baseline_scan_does_not_block_finish_or_attribution(self) -> None:
        """A timed-out HTTP caller must not retain the foreground mutation lane."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", port=0, watch_interval_seconds=60
            )
            target = repo / "owned.txt"
            target.write_text("owned\n", encoding="utf-8")

            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                application = running.httpd.application
                client.command("session.register", {"session_id": "cargo-session"})
                client.command("session.register", {"session_id": "owner-session"})
                client.command("baseline.init")
                client.command(
                    "lease.claim", {"session_id": "owner-session", "paths": ["owned.txt"]}
                )
                cargo_jobs = mock.Mock()
                cargo_jobs.acquire.return_value.to_dict.return_value = {"status": "leased"}
                cargo_jobs.finish.return_value.to_dict.return_value = {"status": "failed"}
                application.cargo_jobs = cargo_jobs
                application.cleanup = mock.Mock()
                application.cleanup.schedule_pending_cleanup.return_value = 0
                scan_started = threading.Event()
                release_scan = threading.Event()
                action_started = threading.Event()
                release_action = threading.Event()

                def hold_control_action() -> None:
                    with application.control_actions._confirmation_lock:
                        action_started.set()
                        release_action.wait()

                def slow_scan():
                    scan_started.set()
                    release_scan.wait()
                    return []

                timed_client = CoordinatorClient(
                    running.base_url, running.token, command_timeout_seconds=0.05
                )
                action_worker = threading.Thread(target=hold_control_action, daemon=True)
                action_worker.start()
                self.assertTrue(action_started.wait(timeout=1))
                foreground_finished = threading.Event()
                foreground_errors: list[BaseException] = []
                foreground_results: dict[str, object] = {}
                foreground_worker: threading.Thread | None = None

                def finish_and_attribute() -> None:
                    try:
                        foreground_results["acquired"] = client.command(
                            "cargo.acquire",
                            {
                                "session_id": "cargo-session",
                                "lane_kind": "test",
                                "target_dir": None,
                                "dry_run": False,
                                "pid": None,
                                "ephemeral": True,
                                "compatibility": None,
                            },
                        )
                        foreground_results["finished"] = client.command(
                            "cargo.finish",
                            {
                                "job_id": "job-a",
                                "session_id": "cargo-session",
                                "exit_code": 1,
                            },
                        )
                        foreground_results["attributed"] = client.command(
                            "baseline.attribute",
                            {"session_id": "owner-session", "paths": ["owned.txt"]},
                        )
                        foreground_results["heartbeat"] = client.command(
                            "session.heartbeat", {"session_id": "cargo-session"}
                        )
                    except BaseException as error:
                        foreground_errors.append(error)
                    finally:
                        foreground_finished.set()

                try:
                    with mock.patch.object(application.baselines, "scan", side_effect=slow_scan):
                        with self.assertRaises(CoordinatorClientError) as timed_out:
                            timed_client.command("baseline.scan")
                        self.assertTrue(scan_started.wait(timeout=1))
                        foreground_worker = threading.Thread(
                            target=finish_and_attribute, daemon=True
                        )
                        foreground_worker.start()
                        self.assertTrue(
                            foreground_finished.wait(timeout=5),
                            "foreground lifecycle mutations waited for the disconnected scan",
                        )
                        self.assertFalse(foreground_errors, repr(foreground_errors))
                finally:
                    release_scan.set()
                    release_action.set()
                    action_worker.join(timeout=1)
                    if foreground_worker is not None:
                        foreground_worker.join(timeout=1)

            self.assertEqual("command_post_timeout", timed_out.exception.code)
            self.assertEqual("baseline.scan", timed_out.exception.details["command"])
            self.assertEqual(0.05, timed_out.exception.details["timeoutSeconds"])
            self.assertEqual("post_response", timed_out.exception.details["phase"])
            self.assertEqual("accepted", timed_out.exception.details["submission"])
            acquired = foreground_results["acquired"]
            finished = foreground_results["finished"]
            attributed = foreground_results["attributed"]
            heartbeat = foreground_results["heartbeat"]
            self.assertEqual("leased", acquired["job"]["status"])
            self.assertEqual("failed", finished["job"]["status"])
            self.assertEqual("attributed", attributed["status"])
            self.assertEqual("cargo-session", heartbeat["session"]["session_id"])
            cargo_jobs.finish.assert_called_once_with(
                "job-a", session_id="cargo-session", exit_code=1
            )
            cargo_jobs.acquire.assert_called_once()

    def test_cargo_and_session_lifecycle_commands_do_not_use_global_mutex(self) -> None:
        commands = {
            "session.heartbeat",
            "lease.claim",
            "lease.release",
            "cargo.acquire",
            "cargo.consume_cpu_reservation",
            "cargo.run_reserved",
            "cargo.start",
            "cargo.heartbeat",
            "cargo.finish",
            "cargo.release",
        }

        self.assertTrue(commands <= CoordinatorApplication.NON_BLOCKING_MUTATION_COMMANDS)

    def test_foreground_mutation_is_not_blocked_by_validation_copy_materialize(self) -> None:
        """Long copy work may not own the global foreground mutation mutex."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            application.command("session.register", {"session_id": "copy-session"})
            started = threading.Event()
            release_copy = threading.Event()
            mutation_finished = threading.Event()
            result: dict[str, object] = {}

            def slow_materialize(*_args, **_kwargs):
                started.set()
                release_copy.wait()
                return WorkspaceCopyRecord(
                    "copy-job",
                    "copy-session",
                    root / "copy-job",
                    root / "copy-job/source",
                    root / "copy-job/target",
                    ("README.md",),
                    "materializing",
                )

            def register_session() -> None:
                try:
                    result.update(application.command("session.register", {"session_id": "session-a"}))
                except BaseException as error:
                    foreground_error.append(error)
                finally:
                    mutation_finished.set()

            with mock.patch.object(
                application.workspace_copy, "materialize_async", side_effect=slow_materialize
            ):
                copy_worker = threading.Thread(
                    target=lambda: application.command(
                        "validation_copy.materialize",
                        {"session_id": "copy-session", "paths": ["README.md"]},
                    ),
                    daemon=True,
                )
                copy_worker.start()
                self.assertTrue(started.wait(timeout=1))
                foreground_error: list[BaseException] = []
                foreground_worker = threading.Thread(target=register_session, daemon=True)
                foreground_worker.start()
                try:
                    self.assertTrue(
                        mutation_finished.wait(timeout=5),
                        "session.register did not finish while validation-copy materialization remained blocked",
                    )
                    self.assertFalse(foreground_error, repr(foreground_error))
                finally:
                    release_copy.set()
                    copy_worker.join(timeout=1)
                    foreground_worker.join(timeout=1)

        self.assertEqual("registered", result["session"]["status"])

    def test_cargo_materialize_ack_is_bounded_job_metadata_not_full_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            application.command("session.register", {"session_id": "copy-session"})
            oversized = WorkspaceCopyRecord(
                "copy-job",
                "copy-session",
                root / "copy-job",
                root / "copy-job/source",
                root / "copy-job/target",
                tuple(f"src/{index}.rs" for index in range(20_000)),
                "materializing",
            )

            with mock.patch.object(
                application.workspace_copy,
                "materialize_cargo_async",
                return_value=oversized,
            ):
                result = application.command(
                    "validation_copy.materialize_cargo",
                    {
                        "session_id": "copy-session",
                        "command": ["cargo", "test", "-p", "zircon_runtime", "--lib"],
                    },
                )

        self.assertEqual("copy-job", result["copy"]["job_id"])
        self.assertNotIn("manifest", result["copy"])
        self.assertLess(len(json.dumps(result)), 1024)

    def test_cargo_materialize_ack_precedes_artifact_governance_scan(self) -> None:
        """A slow artifact scan must become a durable worker failure, not block HTTP ACK."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            application = CoordinatorApplication(
                CoordinatorConfig.for_repo(repo, state_root=root / "state")
            )
            application.supervision.mark_healthy()
            application.command("session.register", {"session_id": "copy-session"})
            scan_started = threading.Event()
            release_scan = threading.Event()
            response_ready = threading.Event()
            result: dict[str, object] = {}
            errors: list[BaseException] = []

            def slow_governance_scan() -> None:
                scan_started.set()
                release_scan.wait(timeout=5)
                raise CoordinatorError(
                    "artifact_governance_dirty", "Fixture artifact scan rejected the copy"
                )

            def submit() -> None:
                try:
                    result.update(
                        application.command(
                            "validation_copy.materialize_cargo",
                            {
                                "session_id": "copy-session",
                                "command": ["cargo", "test", "--lib"],
                            },
                        )
                    )
                except BaseException as error:
                    errors.append(error)
                finally:
                    response_ready.set()

            with mock.patch.object(
                application,
                "_require_artifact_governance_clean",
                side_effect=slow_governance_scan,
            ):
                request = threading.Thread(target=submit, daemon=True)
                request.start()
                try:
                    self.assertTrue(scan_started.wait(timeout=1))
                    self.assertTrue(
                        response_ready.wait(timeout=1),
                        "Cargo materialize acknowledgement waited on artifact governance",
                    )
                    self.assertFalse(errors, repr(errors))
                    job_id = str(result["copy"]["job_id"])
                    release_scan.set()
                    for _ in range(100):
                        status = application.workspace_copy.status("copy-session", job_id)
                        if status.status == "failed":
                            break
                        threading.Event().wait(0.02)
                finally:
                    release_scan.set()
                    request.join(timeout=1)

        self.assertEqual("failed", status.status)
        self.assertEqual("artifact_governance_dirty", status.error_code)
        self.assertEqual("artifact_governance", status.error_stage)


if __name__ == "__main__":
    unittest.main()
