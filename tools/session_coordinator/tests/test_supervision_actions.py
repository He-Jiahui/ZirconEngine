from __future__ import annotations

import hashlib
import json
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.fingerprint import ActionFingerprinter
from tools.session_coordinator.control_plane.actions.models import ActionContext, ActionKind
from tools.session_coordinator.control_plane.actions.service import ActionService
from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import (
    CoordinatorError,
    SessionStatus,
    SupervisionState,
    WebControlRole,
)
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.supervision.lifecycle import LifecycleService
from tools.session_coordinator.supervision.models import LifecycleKind, LifecycleStatus
from tools.session_coordinator.supervision.service import SupervisionService
from tools.session_coordinator.tests.helpers import init_repo


class SupervisionActionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "cargo-targets"
        self.target_root.mkdir()
        self.database = Database(root / "coordinator.sqlite3")
        migrate(self.database)
        BaselineService(self.database, self.repo).initialize()
        self.sessions = SessionService(self.database, self.repo)
        for session_id in ("executor-session", "reviewer-session"):
            self.sessions.register(session_id=session_id)
            self.sessions.set_status(session_id, SessionStatus.ACTIVE, reason="test setup")
        self.maintenance_active = threading.Event()
        self.supervision = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-a",
            process_creation_time="created-a",
            maintenance_active=self.maintenance_active.is_set,
        )
        self.supervision.initialize()
        self.supervision.mark_healthy()
        self.shutdown = threading.Event()
        self.lifecycle = LifecycleService(
            self.supervision,
            shutdown=lambda _kind: self.shutdown.set(),
            poll_seconds=0.01,
            allow_global_shutdown=True,
        )
        self.addCleanup(self.lifecycle.close)
        executor = ActionExecutor(
            sessions=self.sessions,
            leases=None,
            patches=None,
            failures=None,
            workspace_copy=None,
            workflows=None,
            lifecycle=self.lifecycle,
        )
        self.actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-a",
                supervision=self.supervision,
            ),
            executor,
            daemon_instance_id="daemon-a",
            mutation_gate=self.supervision.require_mutation_allowed,
        )
        self.context = ActionContext(
            actor="tray",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-a",
        )

    def _confirm(self, kind: ActionKind, timeout: int = 5, **parameters):
        payload = {"timeoutSeconds": timeout, **parameters}
        preview = self.actions.preview(
            self.context, kind.value, payload
        )
        return self.actions.confirm(
            self.context,
            preview.action_id,
            phrase=preview.confirmation_phrase or "",
            reason=f"test {kind.value}",
        )

    def _insert_cargo_job(self, job_id: str, *, status: str, live_pids: list[int]) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, command_json,
                    created_at, last_heartbeat_at, process_tree_live_pids_json
                ) VALUES (?, 'executor-session', 'test', ?, ?, '[]', 'now', 'now', ?)
                """,
                (job_id, f"D:/cargo-targets/{job_id}", status, json.dumps(live_pids)),
            )

    def _source_bound_compatibility(self) -> CargoCompatibility:
        readme_hash = hashlib.sha256((self.repo / "README.md").read_bytes()).hexdigest()
        return CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="Cargo.toml",
            build_config="profile=test;features=default;rustflags=;incremental=0;debug=0",
            source_manifest={"README.md": readme_hash},
        )

    def test_drain_reports_blockers_without_closing_mutation_admission(self) -> None:
        self._insert_cargo_job("live-cargo", status="running", live_pids=[101])

        drained = self._confirm(ActionKind.SERVICE_DRAIN)

        self.assertEqual("succeeded", drained.status.value)
        self.assertTrue(drained.result["admissionOpen"])
        self.assertTrue(any(blocker["kind"] == "cargo" for blocker in drained.result["blockers"]))
        snapshot = self.supervision.snapshot()
        self.assertEqual("healthy", snapshot.state.value)
        self.assertFalse(snapshot.maintenance_hold)
        self.supervision.require_mutation_allowed("cargo.reserve_cpu@reviewer-session")

    def test_drain_with_a_maintenance_scope_keeps_all_sessions_admitted(self) -> None:
        drained = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )

        self.assertEqual("succeeded", drained.status.value)
        self.assertTrue(drained.result["admissionOpen"])
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)
        self.supervision.require_mutation_allowed("cargo.acquire@reviewer-session")

    def test_drain_does_not_limit_the_current_daemon_to_a_maintenance_scope(self) -> None:
        self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )

        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        self.supervision.require_mutation_allowed("cargo.run_reserved@executor-session")
        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@reviewer-session"
        )

    def test_bootstrap_rejects_a_burst_reservation(self) -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, command_fingerprint, job_id, status, created_at,
                    expires_at, execution_mode, burst_eligible, priority_rank
                ) VALUES (
                    'burst-reservation', 'executor-session', 'cpu', 'burst-compat',
                    '{"source_manifest":{"README.md":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}}',
                    'burst-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', 'burst', 1, 1000
                )
                """
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.supervision.bootstrap_proof_bound_handoff(
                reservation_id="burst-reservation",
                maintenance_session_ids=("executor-session",),
                actor="bootstrap-owner",
            )

        self.assertEqual("bootstrap_reservation_ineligible", rejected.exception.code)

    def test_proof_bound_runtime11_shape_consumes_once_and_retries_same_job(self) -> None:
        cargo = CargoJobService(
            self.database,
            TargetPathPolicy([self.target_root]),
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
        )
        compatibility = self._source_bound_compatibility()
        first = cargo.reserve_cpu(
            "executor-session",
            compatibility=compatibility,
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "runtime11"),
            burst_eligible=False,
        )
        later = cargo.reserve_cpu(
            "reviewer-session",
            compatibility=compatibility,
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "reviewer"),
            burst_eligible=False,
        )
        self.supervision.bootstrap_proof_bound_handoff(
            reservation_id=first["reservationId"],
            maintenance_session_ids=("executor-session",),
            actor="bootstrap-owner",
        )
        cargo.set_reservation_consume_guard(
            lambda connection, reservation_id, session_id, job_id=None: (
                self.supervision.require_proof_bound_reservation_in_connection(
                    connection,
                    reservation_id,
                    session_id=session_id,
                    job_id=job_id,
                )
            )
        )

        first_job = cargo.consume_cpu_reservation(
            first["reservationId"],
            session_id="executor-session",
            lane_kind=CargoLaneKind.TEST,
        )
        retried_job = cargo.consume_cpu_reservation(
            first["reservationId"],
            session_id="executor-session",
            lane_kind=CargoLaneKind.TEST,
        )

        self.assertEqual(first_job.job_id, retried_job.job_id)
        with self.database.connect() as connection:
            jobs = connection.execute(
                "SELECT job_id FROM cargo_jobs WHERE session_id='executor-session'"
            ).fetchall()
            reservations = connection.execute(
                """
                SELECT reservation_id, status, job_id
                FROM cargo_lane_reservations
                WHERE reservation_id IN (?, ?)
                ORDER BY created_at, reservation_id
                """,
                (first["reservationId"], later["reservationId"]),
            ).fetchall()
        self.assertEqual([first_job.job_id], [row["job_id"] for row in jobs])
        self.assertEqual("leased", reservations[0]["status"])
        self.assertEqual(first_job.job_id, reservations[0]["job_id"])
        self.assertEqual("pending", reservations[1]["status"])
        self.assertIsNone(reservations[1]["job_id"])

    def test_bootstrap_handoff_requires_a_quiet_lane_and_binds_one_existing_reservation(self) -> None:
        """An offline bootstrap cannot create a hold around a live Cargo tree."""
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'hgi-reservation', 'executor-session', 'cpu', 'hgi-compat',
                    '{"source_manifest":{"owned.txt":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}}',
                    'hgi-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, NULL, 'warm', 0, 1000, NULL
                )
                """
            )
        self._insert_cargo_job("live-cargo", status="running", live_pids=[101])

        with self.assertRaises(CoordinatorError) as blocked:
            self.supervision.bootstrap_proof_bound_handoff(
                reservation_id="hgi-reservation",
                maintenance_session_ids=("reviewer-session", "executor-session"),
                actor="bootstrap-owner",
            )
        self.assertEqual("bootstrap_cargo_active", blocked.exception.code)

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET status='released', finished_at='2026-07-19T00:01:00+00:00' "
                "WHERE job_id='live-cargo'"
            )

        handoff = self.supervision.bootstrap_proof_bound_handoff(
            reservation_id="hgi-reservation",
            maintenance_session_ids=("reviewer-session", "executor-session"),
            actor="bootstrap-owner",
        )

        self.assertFalse(handoff["admissionOpen"])
        self.assertTrue(handoff["proofBound"])
        self.assertEqual("hgi-reservation", handoff["reservationId"])
        self.assertTrue(self.supervision.snapshot().maintenance_hold)
        self.assertEqual("draining", self.supervision.snapshot().state.value)
        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        self.supervision.require_mutation_allowed("cargo.run_reserved@executor-session")
        with self.assertRaises(CoordinatorError) as generic:
            self.supervision.require_mutation_allowed("cargo.reserve_cpu@executor-session")
        self.assertEqual("maintenance_hold_active", generic.exception.code)
        with self.assertRaises(CoordinatorError) as mismatched:
            self.supervision.require_proof_bound_reservation(
                "another-reservation", session_id="executor-session"
            )
        self.assertEqual("maintenance_proof_reservation_mismatch", mismatched.exception.code)
        self.supervision.require_proof_bound_reservation(
            "hgi-reservation", session_id="executor-session"
        )
        with self.database.connect() as connection:
            proof = self.supervision.proof_bound_drain(connection)
            binding = self.supervision._proof_result(proof)["reservationBinding"]
        self.assertEqual(
            {
                "reservationId",
                "sessionId",
                "laneScope",
                "status",
                "jobId",
                "compatibilityKey",
                "compatibilityPayloadFingerprint",
                "commandFingerprint",
                "sourceManifestFingerprint",
                "createdAt",
                "priorityRank",
                "executionMode",
                "fifoPredecessor",
            },
            set(binding),
        )
        self.assertEqual("pending", binding["status"])
        self.assertIsNone(binding["jobId"])
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET status='leased' "
                "WHERE reservation_id='hgi-reservation'"
            )
        with self.assertRaises(CoordinatorError) as status_drift:
            self.supervision.require_proof_bound_reservation(
                "hgi-reservation", session_id="executor-session"
            )
        self.assertEqual("maintenance_proof_reservation_mismatch", status_drift.exception.code)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET status='pending' "
                "WHERE reservation_id='hgi-reservation'"
            )
        for column, replacement in (
            ("command_fingerprint", "drift-command"),
            ("compatibility_key", "drift-compatibility"),
            (
                "compatibility_json",
                '{"source_manifest":{"owned.txt":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"}}',
            ),
            ("created_at", "2026-07-20T00:00:00+00:00"),
            ("execution_mode", "burst"),
        ):
            with self.database.transaction() as connection:
                original = connection.execute(
                    f"SELECT {column} FROM cargo_lane_reservations WHERE reservation_id='hgi-reservation'"
                ).fetchone()[0]
                connection.execute(
                    f"UPDATE cargo_lane_reservations SET {column}=? WHERE reservation_id='hgi-reservation'",
                    (replacement,),
                )
            with self.assertRaises(CoordinatorError) as drifted:
                self.supervision.require_proof_bound_reservation(
                    "hgi-reservation", session_id="executor-session"
                )
            self.assertEqual("maintenance_proof_reservation_mismatch", drifted.exception.code)
            with self.database.transaction() as connection:
                connection.execute(
                    f"UPDATE cargo_lane_reservations SET {column}=? WHERE reservation_id='hgi-reservation'",
                    (original,),
                )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, command_fingerprint, job_id, status, created_at,
                    expires_at, execution_mode, burst_eligible, priority_rank
                ) VALUES (
                    'injected-predecessor', 'reviewer-session', 'cpu', 'other-compat',
                    '{"source_manifest":{"owned.txt":"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"}}',
                    'other-command', NULL, 'pending', '2020-01-01T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', 'warm', 0, 0
                )
                """
            )
        with self.assertRaises(CoordinatorError) as predecessor_drift:
            self.supervision.require_proof_bound_reservation(
                "hgi-reservation", session_id="executor-session"
            )
        self.assertEqual("maintenance_proof_reservation_mismatch", predecessor_drift.exception.code)
        with self.database.transaction() as connection:
            connection.execute(
                "DELETE FROM cargo_lane_reservations WHERE reservation_id='injected-predecessor'"
            )
        with self.assertRaises(CoordinatorError) as premature_resume:
            self._confirm(
                ActionKind.SERVICE_RESUME,
                releaseMaintenanceHold=True,
                maintenanceHoldActionId=handoff["actionId"],
            )
        self.assertEqual("maintenance_proof_reservation_active", premature_resume.exception.code)
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE cargo_lane_reservations
                SET status='released', completed_at='2026-07-19T00:02:00+00:00'
                WHERE reservation_id='hgi-reservation'
                """
            )
        resumed = self._confirm(
            ActionKind.SERVICE_RESUME,
            releaseMaintenanceHold=True,
            maintenanceHoldActionId=handoff["actionId"],
        )
        self.assertEqual("succeeded", resumed.status.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)

    def test_bootstrap_reconciles_legacy_priority_yield_before_selecting_fifo_head(self) -> None:
        """A legacy retry cannot become a proof-bound head ahead of its normal barrier."""
        lifecycle_key = "origin|runtime11|legacy-priority-yield"
        with self.database.transaction() as connection:
            connection.executemany(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    compatibility_json, command_fingerprint, job_id, status, created_at,
                    expires_at, execution_mode, burst_eligible, priority_rank,
                    failure_lifecycle_key
                ) VALUES (?, ?, 'cpu', 'compat',
                          '{"source_manifest":{"owned.txt":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}}',
                          'command', NULL, ?, ?, '2099-07-19T00:00:00+00:00',
                          'warm', 0, ?, ?)
                """,
                (
                    (
                        "completed-priority",
                        "executor-session",
                        "released",
                        "2026-07-19T00:00:00+00:00",
                        0,
                        lifecycle_key,
                    ),
                    (
                        "normal-barrier",
                        "reviewer-session",
                        "pending",
                        "2026-07-19T00:01:00+00:00",
                        1000,
                        None,
                    ),
                    (
                        "legacy-retry",
                        "executor-session",
                        "pending",
                        "2026-07-19T00:02:00+00:00",
                        0,
                        lifecycle_key,
                    ),
                ),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.supervision.bootstrap_proof_bound_handoff(
                reservation_id="legacy-retry",
                maintenance_session_ids=("executor-session", "reviewer-session"),
                actor="bootstrap-owner",
            )

        self.assertEqual("bootstrap_reservation_not_fifo_head", rejected.exception.code)

    def test_repeated_drain_does_not_scope_admission_to_either_session(self) -> None:
        first = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'first-proof-reservation', 'executor-session', 'cpu', 'proof-compat',
                    'proof-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, NULL, 'warm', 0, 1000, NULL
                )
                """
            )
        second = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["reviewer-session"],
        )

        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@reviewer-session"
        )
        self.assertEqual("succeeded", first.status.value)
        self.assertEqual("succeeded", second.status.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)

    def test_repeated_drain_cycles_leave_both_sessions_admitted(self) -> None:
        first = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'first-cycle-reservation', 'executor-session', 'cpu', 'proof-compat',
                    'proof-command', NULL, 'released', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, '2026-07-19T00:01:00+00:00',
                    'warm', 0, 1000, NULL
                )
                """
            )
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)
        second = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["reviewer-session"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'second-cycle-reservation', 'reviewer-session', 'cpu', 'proof-compat',
                    'proof-command', NULL, 'pending', '2026-07-19T00:02:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, NULL, 'warm', 0, 1000, NULL
                )
                """
            )

        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@reviewer-session"
        )
        self.supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        self.assertEqual("succeeded", first.status.value)
        self.assertEqual("succeeded", second.status.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)

    def test_restarted_daemon_keeps_all_sessions_admitted_across_drain_cycles(self) -> None:
        first = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'restart-first-cycle-reservation', 'executor-session', 'cpu', 'proof-compat',
                    'proof-command', NULL, 'released', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, '2026-07-19T00:01:00+00:00',
                    'warm', 0, 1000, NULL
                )
                """
            )

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-successor",
            process_creation_time="created-successor",
            maintenance_session_ids=("executor-session",),
        )
        successor.initialize(start_reason="recovery.proof_bound_cycle_a")
        successor.mark_healthy()
        successor.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        successor_lifecycle = LifecycleService(
            successor,
            shutdown=lambda _kind: self.shutdown.set(),
            poll_seconds=0.01,
            allow_global_shutdown=True,
        )
        self.addCleanup(successor_lifecycle.close)
        successor_actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-successor",
                supervision=successor,
            ),
            ActionExecutor(
                sessions=self.sessions,
                leases=None,
                patches=None,
                failures=None,
                workspace_copy=None,
                workflows=None,
                lifecycle=successor_lifecycle,
            ),
            daemon_instance_id="daemon-successor",
            mutation_gate=successor.require_mutation_allowed,
        )
        successor_context = ActionContext(
            actor="tray",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-successor",
        )

        def confirm(kind: ActionKind, **parameters):
            preview = successor_actions.preview(
                successor_context,
                kind.value,
                {"timeoutSeconds": 5, **parameters},
            )
            return successor_actions.confirm(
                successor_context,
                preview.action_id,
                phrase=preview.confirmation_phrase or "",
                reason=f"test successor {kind.value}",
            )

        self.assertEqual("healthy", successor.snapshot().state.value)
        self.assertFalse(successor.snapshot().maintenance_hold)
        second = confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["reviewer-session"],
        )

        successor.require_mutation_allowed(
            "cargo.consume_cpu_reservation@reviewer-session"
        )
        successor.require_mutation_allowed("cargo.run_reserved@reviewer-session")
        successor.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        self.assertEqual("succeeded", first.status.value)
        self.assertEqual("succeeded", second.status.value)
        self.assertFalse(successor.snapshot().maintenance_hold)

    def test_drain_does_not_gate_existing_or_new_reservations_after_restart(self) -> None:
        drained = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_lane_reservations(
                    reservation_id, session_id, lane_scope, compatibility_key,
                    command_fingerprint, job_id, status, created_at, expires_at,
                    started_at, completed_at, execution_mode, burst_eligible,
                    priority_rank, failure_lifecycle_key
                ) VALUES (
                    'proof-reservation', 'executor-session', 'cpu', 'proof-compat',
                    'proof-command', NULL, 'pending', '2026-07-19T00:00:00+00:00',
                    '2099-07-19T00:00:00+00:00', NULL, NULL, 'warm', 0, 1000, NULL
                )
                """
            )

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-successor",
            process_creation_time="created-successor",
            maintenance_session_ids=("executor-session",),
        )
        successor.initialize(start_reason="recovery.proof_bound_drain")
        successor.mark_healthy()
        self.assertFalse(successor.snapshot().maintenance_hold)
        successor.require_mutation_allowed(
            "cargo.consume_cpu_reservation@executor-session"
        )
        successor.require_mutation_allowed("cargo.run_reserved@executor-session")
        for operation in (
            "cargo.acquire@reviewer-session",
            "cargo.reserve_cpu@reviewer-session",
            "cargo.promote_failure_reservation@reviewer-session",
            "cargo.reserve_cpu@executor-session",
            "cargo.reserve_gpu@executor-session",
            "cargo.recover_expired_reservation@executor-session",
            "cargo.promote_failure_reservation@executor-session",
            "cargo.consume_gpu_reservation@executor-session",
        ):
            successor.require_mutation_allowed(operation)
        self.assertEqual("succeeded", drained.status.value)

    def test_drain_persists_its_maintenance_session_scope(self) -> None:
        drained = self._confirm(
            ActionKind.SERVICE_DRAIN,
            maintenanceSessionIds=["executor-session", "reviewer-session"],
        )

        self.assertEqual(
            ["executor-session", "reviewer-session"],
            drained.parameters["maintenanceSessionIds"],
        )
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT parameters_json FROM action_requests WHERE action_id=?",
                (drained.action_id,),
            ).fetchone()
        self.assertEqual(
            ["executor-session", "reviewer-session"],
            json.loads(row["parameters_json"])["maintenanceSessionIds"],
        )

    def test_explicit_release_preview_is_not_bound_to_a_maintenance_session(self) -> None:
        operations: list[str] = []
        actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-a",
                supervision=self.supervision,
            ),
            self.actions.executor,
            daemon_instance_id="daemon-a",
            mutation_gate=operations.append,
        )

        actions.preview(
            self.context,
            ActionKind.SERVICE_RESUME.value,
            {
                "timeoutSeconds": 5,
                "releaseMaintenanceHold": True,
                "maintenanceHoldActionId": "drain-action",
                "maintenanceSessionId": "executor-session",
            },
        )

        self.assertEqual(["service.resume.release"], operations)

    def test_explicit_release_preview_without_session_uses_drain_proof_gate(self) -> None:
        operations: list[str] = []
        actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-a",
                supervision=self.supervision,
            ),
            self.actions.executor,
            daemon_instance_id="daemon-a",
            mutation_gate=operations.append,
        )

        actions.preview(
            self.context,
            ActionKind.SERVICE_RESUME.value,
            {
                "timeoutSeconds": 5,
                "releaseMaintenanceHold": True,
                "maintenanceHoldActionId": "drain-action",
            },
        )

        self.assertEqual(["service.resume.release"], operations)

    def test_scoped_activation_bootstraps_a_new_session_with_exact_scope(self) -> None:
        scoped_supervision = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-scoped",
            process_creation_time="created-scoped",
            maintenance_session_ids=("executor-session",),
        )
        scoped_supervision.initialize()
        scoped_supervision.mark_healthy()
        scoped_supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.scoped_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )
        actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-scoped",
                supervision=scoped_supervision,
            ),
            self.actions.executor,
            daemon_instance_id="daemon-scoped",
            mutation_gate=scoped_supervision.require_mutation_allowed,
        )
        context = ActionContext(
            actor="tray",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-scoped",
        )
        preview = actions.preview(
            context,
            ActionKind.SESSION_ACTIVATE.value,
            {
                "sessionId": "render18-bootstrap",
                "displayName": "Render18 AF-M3 bootstrap",
                "planPath": "docs/plans/zircon_runtime/render/18-advanced-lighting-features.md",
                "writeScope": ["zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel"],
                "maintenanceSessionId": "executor-session",
            },
        )
        actions.confirm(
            context,
            preview.action_id,
            phrase=preview.confirmation_phrase or "",
            reason="test scoped session bootstrap",
        )

        created = self.sessions.get("render18-bootstrap")
        self.assertEqual(SessionStatus.ACTIVE, created.status)
        self.assertEqual(
            "docs/plans/zircon_runtime/render/18-advanced-lighting-features.md",
            created.plan_path,
        )
        self.assertEqual(
            ("zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel",),
            created.write_scope,
        )

    def test_scoped_activation_bootstrap_requires_maintainer_permission(self) -> None:
        scoped_supervision = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-scoped",
            process_creation_time="created-scoped",
            maintenance_session_ids=("executor-session",),
        )
        scoped_supervision.initialize()
        scoped_supervision.mark_healthy()
        scoped_supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.scoped_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )
        actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-scoped",
                supervision=scoped_supervision,
            ),
            self.actions.executor,
            daemon_instance_id="daemon-scoped",
            mutation_gate=scoped_supervision.require_mutation_allowed,
        )
        context = ActionContext(
            actor="operator",
            role=WebControlRole.OPERATOR,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-scoped",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            actions.preview(
                context,
                ActionKind.SESSION_ACTIVATE.value,
                {
                    "sessionId": "render18-bootstrap",
                    "displayName": "Render18 AF-M3 bootstrap",
                    "planPath": "docs/plans/zircon_runtime/render/18-advanced-lighting-features.md",
                    "writeScope": [
                        "zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel"
                    ],
                    "maintenanceSessionId": "executor-session",
                },
            )

        self.assertEqual("action_permission_denied", rejected.exception.code)

    def test_production_lifecycle_rejects_global_shutdown_without_draining(self) -> None:
        lifecycle = LifecycleService(self.supervision, poll_seconds=0.01)
        self.addCleanup(lifecycle.close)

        with self.assertRaises(CoordinatorError) as rejected:
            lifecycle.request(
                LifecycleKind.STOP,
                action_id="production-stop",
                actor="test",
                timeout_seconds=5,
            )

        self.assertEqual("lifecycle_global_shutdown_disabled", rejected.exception.code)
        snapshot = self.supervision.snapshot()
        self.assertEqual("healthy", snapshot.state.value)
        self.assertFalse(snapshot.maintenance_hold)
        self.assertFalse(snapshot.explicit_stop)
        with self.database.connect() as connection:
            intent_count = connection.execute(
                "SELECT COUNT(*) FROM service_lifecycle_intents WHERE action_id=?",
                ("production-stop",),
            ).fetchone()[0]
        self.assertEqual(0, intent_count)

    def test_rollover_preserves_admission_and_unstarted_work_for_successor(self) -> None:
        """A code reload must not turn an empty process window into a global drain."""
        self._insert_cargo_job("leased-job", status="leased", live_pids=[])
        rolling = self._confirm(ActionKind.SERVICE_ROLLOVER)

        self.assertEqual("executing", rolling.status.value)
        self.assertTrue(self.shutdown.wait(2))
        snapshot = self.supervision.snapshot()
        self.assertEqual("healthy", snapshot.state.value)
        self.assertFalse(snapshot.maintenance_hold)
        self.assertFalse(snapshot.explicit_stop)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status FROM service_lifecycle_intents WHERE action_id=?",
                (rolling.action_id,),
            ).fetchone()
        self.assertEqual("awaiting_restart", intent["status"])

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize()
        successor.mark_healthy()
        recovered = LifecycleService(successor, poll_seconds=0.01)
        self.addCleanup(recovered.close)
        successor_actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-b",
                supervision=successor,
            ),
            self.actions.executor,
            daemon_instance_id="daemon-b",
            mutation_gate=successor.require_mutation_allowed,
        )

        self.assertEqual(0, successor_actions.recover_interrupted_actions())
        self.assertEqual(1, recovered.recover_restart_intents())
        with self.database.connect() as connection:
            completed = connection.execute(
                "SELECT status, result_json, error_code FROM action_requests WHERE action_id=?",
                (rolling.action_id,),
            ).fetchone()
        self.assertEqual("succeeded", completed["status"], completed["error_code"])
        self.assertEqual("daemon-b", json.loads(completed["result_json"])["successorInstanceId"])
        with self.database.connect() as connection:
            job = connection.execute(
                "SELECT status, process_tree_live_pids_json FROM cargo_jobs WHERE job_id='leased-job'"
            ).fetchone()
        self.assertEqual("leased", job["status"])
        self.assertEqual([], json.loads(job["process_tree_live_pids_json"]))
        successor_snapshot = successor.snapshot()
        self.assertEqual("healthy", successor_snapshot.state.value)
        self.assertFalse(successor_snapshot.maintenance_hold)
        successor_record = successor_actions.get(
            ActionContext(
                actor="tray",
                role=WebControlRole.MAINTAINER,
                web_session_id=None,
                bound_session_id=None,
                daemon_instance_id="daemon-b",
            ),
            rolling.action_id,
        )
        self.assertEqual("succeeded", successor_record.status.value)
        self.assertEqual("daemon-b", successor_record.result["successorInstanceId"])

    def test_successor_coalesces_a_recent_rollover_without_a_second_shutdown(self) -> None:
        first = self._confirm(ActionKind.SERVICE_ROLLOVER)
        self.assertTrue(self.shutdown.wait(2))

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize()
        successor.mark_healthy()
        second_shutdown = threading.Event()
        successor_lifecycle = LifecycleService(
            successor,
            shutdown=lambda _kind: second_shutdown.set(),
            poll_seconds=0.01,
            allow_global_shutdown=True,
        )
        self.addCleanup(successor_lifecycle.close)
        successor_actions = ActionService(
            self.database,
            ActionFingerprinter(
                self.database,
                self.repo,
                daemon_instance_id="daemon-b",
                supervision=successor,
            ),
            ActionExecutor(
                sessions=self.sessions,
                leases=None,
                patches=None,
                failures=None,
                workspace_copy=None,
                workflows=None,
                lifecycle=successor_lifecycle,
            ),
            daemon_instance_id="daemon-b",
            mutation_gate=successor.require_mutation_allowed,
        )
        successor_context = ActionContext(
            actor="second-local-executor",
            role=WebControlRole.MAINTAINER,
            web_session_id=None,
            bound_session_id=None,
            daemon_instance_id="daemon-b",
        )
        self.assertEqual(1, successor_lifecycle.recover_restart_intents())

        preview = successor_actions.preview(
            successor_context,
            ActionKind.SERVICE_ROLLOVER.value,
            {"timeoutSeconds": 5},
        )
        coalesced = successor_actions.confirm(
            successor_context,
            preview.action_id,
            phrase=preview.confirmation_phrase or "",
            reason="a second local monitor raced the successor startup",
        )

        self.assertEqual("succeeded", coalesced.status.value)
        self.assertEqual(True, coalesced.result["coalesced"])
        self.assertEqual("daemon-b", coalesced.result["successorInstanceId"])
        self.assertFalse(second_shutdown.wait(0.1))
        with self.database.connect() as connection:
            rollover_count = connection.execute(
                "SELECT COUNT(*) FROM service_lifecycle_intents WHERE kind='service.rollover'"
            ).fetchone()[0]
        self.assertEqual(1, rollover_count)

    def test_rollover_defers_for_a_live_managed_cargo_tree_without_draining(self) -> None:
        self._insert_cargo_job("running-job", status="running", live_pids=[4242])
        rolling = self._confirm(ActionKind.SERVICE_ROLLOVER)

        self.assertEqual("executing", rolling.status.value)
        self.assertTrue(rolling.result["waitingForCargo"])
        self.assertEqual("healthy", rolling.result["state"])
        self.assertFalse(self.shutdown.wait(0.1))
        snapshot = self.supervision.snapshot()
        self.assertEqual("healthy", snapshot.state.value)
        self.assertFalse(snapshot.maintenance_hold)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, result_json FROM service_lifecycle_intents WHERE action_id=?",
                (rolling.action_id,),
            ).fetchone()
        self.assertEqual("accepted", intent["status"])
        self.assertTrue(json.loads(intent["result_json"])["waitingForCargo"])

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET process_tree_live_pids_json='[]' WHERE job_id='running-job'"
            )
        self.assertTrue(self.shutdown.wait(2))
        with self.database.connect() as connection:
            armed = connection.execute(
                "SELECT status FROM service_lifecycle_intents WHERE action_id=?",
                (rolling.action_id,),
            ).fetchone()
        self.assertEqual("awaiting_restart", armed["status"])

    def test_rollover_records_one_waiting_audit_until_live_cargo_changes(self) -> None:
        self._insert_cargo_job("running-job", status="running", live_pids=[4242])
        rolling = self._confirm(ActionKind.SERVICE_ROLLOVER)

        # 轮询重试保持准入开放，相同状态不能淹没操作审计。
        time.sleep(0.08)
        with self.database.connect() as connection:
            waiting_events = connection.execute(
                """
                SELECT COUNT(*) FROM events
                WHERE event_type='lifecycle.rollover_waiting_for_cargo'
                  AND json_extract(payload_json, '$.actionId')=?
                """,
                (rolling.action_id,),
            ).fetchone()[0]

        self.assertEqual("executing", rolling.status.value)
        self.assertEqual(1, waiting_events)
        self.assertFalse(self.shutdown.is_set())

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET process_tree_live_pids_json='[4242, 4343]' "
                "WHERE job_id='running-job'"
            )
        deadline = time.monotonic() + 1
        while time.monotonic() < deadline:
            with self.database.connect() as connection:
                waiting_events = connection.execute(
                    """
                    SELECT COUNT(*) FROM events
                    WHERE event_type='lifecycle.rollover_waiting_for_cargo'
                      AND json_extract(payload_json, '$.actionId')=?
                    """,
                    (rolling.action_id,),
                ).fetchone()[0]
            if waiting_events == 2:
                break
            time.sleep(0.01)
        self.assertEqual(2, waiting_events)

        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_jobs SET process_tree_live_pids_json='[4343, 4242]' "
                "WHERE job_id='running-job'"
            )
        time.sleep(0.08)
        with self.database.connect() as connection:
            reordered_events = connection.execute(
                """
                SELECT COUNT(*) FROM events
                WHERE event_type='lifecycle.rollover_waiting_for_cargo'
                  AND json_extract(payload_json, '$.actionId')=?
                """,
                (rolling.action_id,),
            ).fetchone()[0]
        self.assertEqual(2, reordered_events)

    def test_stop_remains_executing_until_critical_sections_drain(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        self.assertEqual("executing", stopping.status.value)
        self.assertFalse(self.shutdown.wait(0.05))
        self.maintenance_active.clear()
        self.assertTrue(self.shutdown.wait(2))
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            record = self.actions.get(self.context, stopping.action_id)
            if record.status.value == "succeeded":
                break
            time.sleep(0.01)
        else:
            self.fail("stop action did not reach succeeded")

        snapshot = self.supervision.snapshot()
        self.assertEqual("offline", snapshot.state.value)
        self.assertTrue(snapshot.explicit_stop)

    def test_confirmed_stop_can_be_cancelled_while_still_draining(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        cancelled = self.actions.cancel(
            self.context,
            stopping.action_id,
            reason="operator cancelled before shutdown",
        )
        self.maintenance_active.clear()

        self.assertEqual("cancelled", cancelled.status.value)
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.shutdown.wait(0.1))
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (stopping.action_id,),
            ).fetchone()
        self.assertEqual("cancelled", intent["status"])
        self.assertEqual("lifecycle_cancelled", intent["error_code"])

    def test_confirmed_force_stop_cannot_be_cancelled_while_draining(self) -> None:
        """The durable maintenance barrier must survive unrelated operator actions."""
        self.maintenance_active.set()
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)

        with self.assertRaises(CoordinatorError) as rejected:
            self.actions.cancel(
                self.context,
                forcing.action_id,
                reason="unrelated session wants to reopen mutations",
            )
        self.assertEqual("action_not_cancellable", rejected.exception.code)
        self.assertEqual("executing", self.actions.get(self.context, forcing.action_id).status.value)
        self.assertEqual("draining", self.supervision.snapshot().state.value)
        self.assertTrue(self.supervision.snapshot().maintenance_hold)
        with self.assertRaises(CoordinatorError) as resumed:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("lifecycle_restart_draining", resumed.exception.code)

        self.maintenance_active.clear()
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if self.supervision.snapshot().state.value == "offline":
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")
        self.lifecycle.acknowledge_force_stop(forcing.action_id)
        self.assertTrue(self.shutdown.wait(2))

    def test_failed_force_stop_preserves_maintenance_hold(self) -> None:
        preview = self.actions.preview(
            self.context, ActionKind.SERVICE_FORCE_STOP.value, {"timeoutSeconds": 5}
        )
        self.supervision.create_intent(
            LifecycleKind.FORCE_STOP,
            action_id=preview.action_id,
            actor="tray",
            deadline_at="later",
        )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )

        result = self.supervision.fail_lifecycle(
            preview.action_id,
            actor="daemon",
            error_code="lifecycle_orphan_recovered",
        )

        self.assertEqual("draining", result["state"])
        snapshot = self.supervision.snapshot()
        self.assertEqual(SupervisionState.DRAINING, snapshot.state)
        self.assertTrue(snapshot.maintenance_hold)
        with self.assertRaises(CoordinatorError) as rejected:
            self.supervision.require_mutation_allowed("cargo.acquire")
        self.assertEqual("maintenance_hold_active", rejected.exception.code)

    def test_resume_cannot_cancel_an_active_stop_drain(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        with self.assertRaises(CoordinatorError) as rejected:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("lifecycle_restart_draining", rejected.exception.code)
        self.assertEqual(
            "executing", self.actions.get(self.context, stopping.action_id).status.value
        )
        self.assertEqual("draining", self.supervision.snapshot().state.value)

        self.actions.cancel(
            self.context,
            stopping.action_id,
            reason="test cleanup of reversible stop",
        )
        self.maintenance_active.clear()

    def test_resume_cannot_cancel_an_active_restart_drain(self) -> None:
        restarting = self.actions.preview(
            self.context, ActionKind.SERVICE_RESTART.value, {"timeoutSeconds": 5}
        )
        self.supervision.create_intent(
            LifecycleKind.RESTART,
            action_id=restarting.action_id,
            actor="test",
            deadline_at="later",
        )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.restart_drain",
            actor="test",
            action_id=restarting.action_id,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self._confirm(ActionKind.SERVICE_RESUME)

        self.assertEqual("lifecycle_restart_draining", rejected.exception.code)
        self.assertEqual(
            "previewed", self.actions.get(self.context, restarting.action_id).status.value
        )
        self.assertEqual("draining", self.supervision.snapshot().state.value)
        self.supervision.fail_lifecycle(
            restarting.action_id,
            actor="test",
            error_code="test_cleanup",
        )

    def test_restart_establishes_a_persistent_maintenance_hold_while_draining(self) -> None:
        self.maintenance_active.set()
        restarting = self._confirm(ActionKind.SERVICE_RESTART)

        snapshot = self.supervision.snapshot()
        self.assertEqual("draining", snapshot.state.value)
        self.assertTrue(snapshot.maintenance_hold)
        self.assertEqual("executing", restarting.status.value)

        self.maintenance_active.clear()
        self.assertTrue(self.shutdown.wait(2))

    def test_unbound_legacy_maintenance_hold_fails_closed(self) -> None:
        drained = self._confirm(ActionKind.SERVICE_DRAIN)

        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.legacy_maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )

        with self.assertRaises(CoordinatorError) as blocked:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("maintenance_hold_active", blocked.exception.code)

        with self.assertRaises(CoordinatorError) as stale_release:
            self._confirm(
                ActionKind.SERVICE_RESUME,
                releaseMaintenanceHold=True,
                maintenanceHoldActionId=drained.action_id,
            )
        self.assertEqual("maintenance_hold_release_mismatch", stale_release.exception.code)
        self.assertIsNone(stale_release.exception.details["maintenanceHoldActionId"])

        self.assertEqual("succeeded", drained.status.value)
        self.assertTrue(self.supervision.snapshot().maintenance_hold)
        with self.assertRaises(CoordinatorError) as still_blocked:
            self.supervision.require_mutation_allowed("session.register@executor-session")
        self.assertEqual("maintenance_hold_active", still_blocked.exception.code)

    def test_drain_is_immediately_terminal_and_keeps_admission_open(self) -> None:
        drained = self._confirm(ActionKind.SERVICE_DRAIN, timeout=1)

        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        self.assertFalse(self.supervision.snapshot().maintenance_hold)

        with self.database.connect() as connection:
            intent = connection.execute(
                """SELECT status, completed_at, result_json
                   FROM service_lifecycle_intents WHERE action_id=?""",
                (drained.action_id,),
            ).fetchone()
        self.assertEqual("succeeded", intent["status"])
        self.assertIsNotNone(intent["completed_at"])
        self.assertTrue(json.loads(intent["result_json"])["admissionOpen"])

    def test_recovered_drain_deadline_cannot_reopen_proof_bound_admission(self) -> None:
        drained = self._confirm(ActionKind.SERVICE_DRAIN, timeout=30)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT intent_id FROM service_lifecycle_intents WHERE action_id=?",
                (drained.action_id,),
            ).fetchone()
        self.assertIsNotNone(row)
        intent_id = row["intent_id"]
        with self.database.transaction() as connection:
            connection.execute(
                """
                UPDATE service_lifecycle_intents
                SET deadline_at='2000-01-01T00:00:00+00:00'
                WHERE intent_id=?
                """,
                (intent_id,),
            )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.legacy_drain",
            actor="test",
            updates={"maintenance_hold": 1},
        )
        self.supervision.update_intent(intent_id, LifecycleStatus.DRAINING)
        self.lifecycle.close()
        successor = LifecycleService(
            self.supervision,
            shutdown=lambda _kind: self.shutdown.set(),
            poll_seconds=0.01,
        )
        self.addCleanup(successor.close)

        self.assertEqual(1, successor.recover_restart_intents())
        snapshot = self.supervision.snapshot()
        self.assertEqual("draining", snapshot.state.value)
        self.assertTrue(snapshot.maintenance_hold)

    def test_second_reversible_lifecycle_is_rejected_before_resume(self) -> None:
        self.maintenance_active.set()
        stopping = self._confirm(ActionKind.SERVICE_STOP)

        with self.assertRaises(CoordinatorError) as rejected:
            self._confirm(ActionKind.SERVICE_RESTART)

        self.assertEqual("lifecycle_already_active", rejected.exception.code)
        with self.assertRaises(CoordinatorError) as resumed:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("lifecycle_restart_draining", resumed.exception.code)
        self.actions.cancel(
            self.context,
            stopping.action_id,
            reason="test cleanup of reversible stop",
        )
        self.maintenance_active.clear()
        self.assertEqual(
            "cancelled", self.actions.get(self.context, stopping.action_id).status.value
        )
        self.assertFalse(self.shutdown.wait(0.1))

    def test_worker_start_failure_atomically_releases_reversible_lifecycle(self) -> None:
        preview = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Thread.start",
            side_effect=RuntimeError("thread resources exhausted"),
        ):
            with self.assertRaises(CoordinatorError) as failed:
                self.actions.confirm(
                    self.context,
                    preview.action_id,
                    phrase=preview.confirmation_phrase or "",
                    reason="exercise lifecycle worker compensation",
                )

        self.assertEqual("action_execution_failed", failed.exception.code)
        self.assertEqual("healthy", self.supervision.snapshot().state.value)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code, completed_at FROM service_lifecycle_intents "
                "WHERE action_id=?",
                (preview.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code, completed_at FROM action_requests WHERE action_id=?",
                (preview.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_request_failed", True),
                         (intent["status"], intent["error_code"], intent["completed_at"] is not None))
        self.assertEqual(("failed", "lifecycle_request_failed", True),
                         (action["status"], action["error_code"], action["completed_at"] is not None))

        self.maintenance_active.set()
        replacement = self._confirm(ActionKind.SERVICE_STOP)
        with self.assertRaises(CoordinatorError) as resumed:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("executing", replacement.status.value)
        self.assertEqual("lifecycle_restart_draining", resumed.exception.code)
        self.actions.cancel(
            self.context,
            replacement.action_id,
            reason="test cleanup of reversible stop",
        )
        self.maintenance_active.clear()

    def test_close_stops_a_local_stop_worker_before_database_cleanup(self) -> None:
        """Daemon shutdown must not leave a worker holding the local SQLite file."""
        self.maintenance_active.set()
        self._confirm(ActionKind.SERVICE_STOP, timeout=5)
        with self.lifecycle._lock:
            workers = tuple(self.lifecycle._workers.values())
        self.assertEqual(1, len(workers))

        self.lifecycle.close()

        self.assertFalse(workers[0].is_alive())
        self.maintenance_active.clear()

    def test_close_cancels_force_stop_fallback_before_database_cleanup(self) -> None:
        """A closed daemon must not retain a delayed force-stop callback."""
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if self.supervision.snapshot().state is SupervisionState.OFFLINE:
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")
        with self.lifecycle._lock:
            fallback = self.lifecycle._force_stop_timers[forcing.action_id]
        original_cancel = fallback.cancel
        try:
            with patch.object(fallback, "cancel", wraps=original_cancel) as cancelled:
                self.lifecycle.close()
                cancelled.assert_called_once_with()
            fallback.join(1)
            self.assertFalse(fallback.is_alive())
        finally:
            original_cancel()

    def test_resume_reconciles_active_intent_whose_action_is_already_terminal(self) -> None:
        orphan = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='failed', error_code='injected', "
                "completed_at='now' WHERE action_id=?",
                (orphan.action_id,),
            )
        self.supervision.create_intent(
            LifecycleKind.STOP,
            action_id=orphan.action_id,
            actor="test",
            deadline_at="later",
        )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.orphan",
            actor="test",
            action_id=orphan.action_id,
        )

        with self.assertRaises(CoordinatorError) as resumed:
            self._confirm(ActionKind.SERVICE_RESUME)
        self.assertEqual("lifecycle_restart_draining", resumed.exception.code)
        self.supervision.fail_lifecycle(
            orphan.action_id,
            actor="test",
            error_code="lifecycle_orphan_reconciled",
        )
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_orphan_reconciled"), tuple(status))

        self.maintenance_active.set()
        replacement = self._confirm(ActionKind.SERVICE_STOP)
        self.actions.cancel(
            self.context, replacement.action_id, reason="cleanup replacement lifecycle"
        )
        self.maintenance_active.clear()

    def test_successor_startup_reconciles_old_active_lifecycle(self) -> None:
        orphan = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='executing' WHERE action_id=?",
                (orphan.action_id,),
            )
        self.supervision.create_intent(
            LifecycleKind.STOP,
            action_id=orphan.action_id,
            actor="test",
            deadline_at="later",
        )

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize(start_reason="recovery.startup")
        successor.mark_healthy()
        recovered = LifecycleService(successor).recover_restart_intents()

        self.assertEqual(1, recovered)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(intent))
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(action))

        fresh = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        fresh_intent = successor.create_intent(
            LifecycleKind.STOP,
            action_id=fresh.action_id,
            actor="test",
            deadline_at="later",
        )
        self.assertTrue(fresh_intent)

    def test_successor_recovers_orphan_under_hold_without_reopening_mutations(self) -> None:
        orphan = self.actions.preview(
            self.context, ActionKind.SERVICE_STOP.value, {"timeoutSeconds": 5}
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE action_requests SET status='executing' WHERE action_id=?",
                (orphan.action_id,),
            )
        self.supervision.create_intent(
            LifecycleKind.STOP,
            action_id=orphan.action_id,
            actor="test",
            deadline_at="later",
        )
        self.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.orphan_under_hold",
            actor="test",
            action_id=orphan.action_id,
            updates={"maintenance_hold": 1},
        )

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize(start_reason="recovery.maintenance_hold")
        successor.mark_healthy()
        recovered_lifecycle = LifecycleService(successor)
        self.addCleanup(recovered_lifecycle.close)

        self.assertEqual(1, recovered_lifecycle.recover_restart_intents())
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (orphan.action_id,),
            ).fetchone()
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(intent))
        self.assertEqual(("failed", "lifecycle_orphan_recovered"), tuple(action))
        self.assertEqual("healthy", successor.snapshot().state.value)
        self.assertTrue(successor.snapshot().maintenance_hold)
        with self.assertRaises(CoordinatorError) as rejected:
            successor.require_mutation_allowed("session.register@ordinary-session")
        self.assertEqual("maintenance_hold_active", rejected.exception.code)

    def test_restart_is_completed_only_by_successor_daemon(self) -> None:
        restarting = self._confirm(ActionKind.SERVICE_RESTART)
        self.assertTrue(self.shutdown.wait(2))
        self.assertEqual("executing", self.actions.get(self.context, restarting.action_id).status.value)

        successor = SupervisionService(
            self.database,
            repository_key="repo",
            daemon_instance_id="daemon-b",
            process_creation_time="created-b",
        )
        successor.initialize(start_reason="recovery.restart")
        successor.mark_healthy()
        recovered = LifecycleService(successor).recover_restart_intents()

        self.assertEqual(1, recovered)
        with self.database.connect() as connection:
            action_status = connection.execute(
                "SELECT status FROM action_requests WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()[0]
            successor_id = connection.execute(
                "SELECT successor_daemon_instance_id FROM service_lifecycle_intents "
                "WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()[0]
        self.assertEqual("succeeded", action_status)
        self.assertEqual("daemon-b", successor_id)

    def test_force_stop_keeps_transport_available_until_terminal_acknowledgement(self) -> None:
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            record = self.actions.get(self.context, forcing.action_id)
            state = self.supervision.snapshot().state.value
            if record.status.value == "succeeded" and state == "offline":
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach durable succeeded and offline")

        self.assertEqual("offline", self.supervision.snapshot().state.value)
        self.assertFalse(self.shutdown.wait(0.1))
        acknowledged = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(acknowledged["acknowledged"])
        self.assertTrue(self.shutdown.wait(1))

    def test_force_stop_timer_start_failure_keeps_terminal_proof_and_closes(self) -> None:
        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Timer.start",
            side_effect=RuntimeError("timer resources exhausted"),
        ):
            forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
            self.assertTrue(self.shutdown.wait(2))

        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
        self.assertEqual(("succeeded", None), tuple(intent))
        self.assertEqual(("succeeded", None), tuple(action))
        self.assertEqual("offline", self.supervision.snapshot().state.value)

    def test_force_stop_ack_schedule_failure_preserves_original_fallback(self) -> None:
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        with patch(
            "tools.session_coordinator.supervision.lifecycle.threading.Timer.start",
            side_effect=RuntimeError("timer resources exhausted"),
        ):
            with self.assertRaises(CoordinatorError) as failed:
                self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertEqual("force_stop_ack_schedule_failed", failed.exception.code)
        fallback = self.lifecycle._force_stop_timers.pop(forcing.action_id)
        fallback.cancel()
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()[0]
        self.assertEqual("succeeded", intent)

    def test_force_stop_ack_callback_retries_transient_shutdown_failure(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def transient_shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            if len(calls) == 1:
                raise RuntimeError("injected ack callback failure")
            closed.set()

        self.lifecycle.set_shutdown(transient_shutdown)
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        acknowledged = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(acknowledged["acknowledged"])
        self.assertTrue(closed.wait(2))
        self.assertEqual([LifecycleKind.FORCE_STOP, LifecycleKind.FORCE_STOP], calls)
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (forcing.action_id,),
            ).fetchone()
        self.assertEqual(("succeeded", None), tuple(intent))

    def test_repeated_force_stop_ack_is_single_flight(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            closed.set()

        self.lifecycle.set_shutdown(shutdown)
        forcing = self._confirm(ActionKind.SERVICE_FORCE_STOP)
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if (
                self.actions.get(self.context, forcing.action_id).status.value
                == "succeeded"
                and self.supervision.snapshot().state.value == "offline"
            ):
                break
            time.sleep(0.01)
        else:
            self.fail("force-stop action did not reach terminal offline proof")

        first = self.lifecycle.acknowledge_force_stop(forcing.action_id)
        second = self.lifecycle.acknowledge_force_stop(forcing.action_id)

        self.assertTrue(first["acknowledged"])
        self.assertTrue(second["acknowledged"])
        self.assertTrue(second["alreadyAcknowledged"])
        self.assertTrue(closed.wait(2))
        time.sleep(0.3)
        self.assertEqual([LifecycleKind.FORCE_STOP], calls)

    def test_restart_shutdown_transient_failure_retries_without_rewriting_terminal_state(self) -> None:
        calls: list[LifecycleKind] = []
        closed = threading.Event()

        def transient_shutdown(kind: LifecycleKind) -> None:
            calls.append(kind)
            if len(calls) == 1:
                raise RuntimeError("injected shutdown failure")
            closed.set()

        self.lifecycle.set_shutdown(transient_shutdown)
        restarting = self._confirm(ActionKind.SERVICE_RESTART)

        self.assertTrue(closed.wait(2))
        with self.database.connect() as connection:
            intent = connection.execute(
                "SELECT status, error_code FROM service_lifecycle_intents WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()
            action = connection.execute(
                "SELECT status, error_code FROM action_requests WHERE action_id=?",
                (restarting.action_id,),
            ).fetchone()
        self.assertEqual([LifecycleKind.RESTART, LifecycleKind.RESTART], calls)
        self.assertEqual(("awaiting_restart", None), tuple(intent))
        self.assertEqual(("executing", None), tuple(action))
        self.assertEqual("offline", self.supervision.snapshot().state.value)


if __name__ == "__main__":
    unittest.main()
