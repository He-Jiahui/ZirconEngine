from __future__ import annotations

import json
import hashlib
import os
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
                        }
                    ),
                    encoding="utf-8",
                )

                with self.assertRaises(CoordinatorClientError) as rejected:
                    CoordinatorClient.from_runtime(expected).health()

            self.assertEqual("repository_mismatch", rejected.exception.code)

    def test_isolated_config_disables_host_artifact_sweeps(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)

        self.assertFalse(config.unmanaged_artifact_sweep_enabled)

    def test_maintenance_uses_local_runtime_when_no_capability_is_configured(self) -> None:
        with mock.patch.dict("os.environ", {}, clear=True):
            self.assertTrue(CoordinatorApplication._authorize_maintenance({"maintenance": True}))

        with mock.patch.dict(
            "os.environ", {"ZIRCON_COORDINATOR_MAINTENANCE_TOKEN": "local-only"}
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

    def test_local_health_and_session_commands_accept_requests_without_token(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            config = CoordinatorConfig.for_repo(repo, state_root=root / "state", port=0)
            (repo / "owned.txt").write_text("owned\n", encoding="utf-8")

            with RunningCoordinator.start(config) as running:
                client = CoordinatorClient.from_runtime(config)
                health = client.health()
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
                response = urllib.request.urlopen(request, timeout=2)
                self.assertEqual(200, response.status)
                self.assertIn("sessions", json.loads(response.read()))
                response.close()

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

            with RunningCoordinator.start(config):
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
            with mock.patch.dict(
                "os.environ",
                {"ZIRCON_COORDINATOR_MAINTENANCE_TOKEN": "local-only"},
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
                release.wait(timeout=2)
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
                began = time.monotonic()
                result = application.command("session.register", {"session_id": "session-a"})
                elapsed = time.monotonic() - began
                release.set()
                stop.set()
                worker.join(timeout=1)

            self.assertEqual("registered", result["session"]["status"])
            self.assertLess(elapsed, 0.75)

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

            def occupy_control_action() -> None:
                with action_lock:
                    acquired.set()
                    release.wait(timeout=2)

            worker = threading.Thread(target=occupy_control_action, daemon=True)
            worker.start()
            self.assertTrue(acquired.wait(timeout=1))
            began = time.monotonic()
            result = application.command("session.register", {"session_id": "session-a"})
            elapsed = time.monotonic() - began
            release.set()
            worker.join(timeout=1)

        self.assertEqual("registered", result["session"]["status"])
        self.assertLess(elapsed, 0.75)

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
                        release_action.wait(timeout=2)

                def slow_scan():
                    scan_started.set()
                    release_scan.wait(timeout=2)
                    return []

                timed_client = CoordinatorClient(
                    running.base_url, "", command_timeout_seconds=0.05
                )
                action_worker = threading.Thread(target=hold_control_action, daemon=True)
                action_worker.start()
                self.assertTrue(action_started.wait(timeout=1))
                try:
                    with mock.patch.object(application.baselines, "scan", side_effect=slow_scan):
                        with self.assertRaises(CoordinatorClientError) as timed_out:
                            timed_client.command("baseline.scan")
                        self.assertTrue(scan_started.wait(timeout=1))
                        began = time.monotonic()
                        acquired = client.command(
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
                        finished = client.command(
                            "cargo.finish",
                            {
                                "job_id": "job-a",
                                "session_id": "cargo-session",
                                "exit_code": 1,
                            },
                        )
                        attributed = client.command(
                            "baseline.attribute",
                            {"session_id": "owner-session", "paths": ["owned.txt"]},
                        )
                        heartbeat = client.command(
                            "session.heartbeat", {"session_id": "cargo-session"}
                        )
                        elapsed = time.monotonic() - began
                finally:
                    release_scan.set()
                    release_action.set()
                    action_worker.join(timeout=1)

            self.assertEqual("command_timeout", timed_out.exception.code)
            self.assertEqual("baseline.scan", timed_out.exception.details["command"])
            self.assertEqual(0.05, timed_out.exception.details["timeoutSeconds"])
            self.assertEqual("leased", acquired["job"]["status"])
            self.assertEqual("failed", finished["job"]["status"])
            self.assertEqual("attributed", attributed["status"])
            self.assertEqual("cargo-session", heartbeat["session"]["session_id"])
            self.assertLess(elapsed, 0.75)
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


if __name__ == "__main__":
    unittest.main()
