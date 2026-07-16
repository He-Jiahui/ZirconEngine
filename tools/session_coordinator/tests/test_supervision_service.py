from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SupervisionState
from tools.session_coordinator.supervision.service import SupervisionService


class SupervisionServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.database = Database(Path(self.temporary.name) / "coordinator.sqlite3")
        migrate(self.database)
        self.service = SupervisionService(
            self.database,
            repository_key="repo-key",
            daemon_instance_id="daemon-a",
            process_creation_time="creation-a",
            maintenance_session_ids=("session-a", "reviewer-session"),
        )

    def test_initialize_and_health_are_durable_enum_transitions(self) -> None:
        starting = self.service.initialize()
        healthy = self.service.mark_healthy()

        self.assertEqual(SupervisionState.STARTING, starting.state)
        self.assertEqual(SupervisionState.HEALTHY, healthy.state)
        with self.database.connect() as connection:
            transitions = [
                (row["from_state"], row["to_state"], row["reason_code"])
                for row in connection.execute(
                    "SELECT * FROM service_supervision_events ORDER BY sequence"
                )
            ]
        self.assertEqual(
            [(None, "starting", "startup.begin"), ("starting", "healthy", "startup.ready")],
            transitions,
        )

    def test_draining_rejects_new_mutation_but_allows_lifecycle_control(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()
        self.service.transition(
            SupervisionState.DRAINING,
            reason_code="test.drain",
            actor="test",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_mutation_allowed("lease.claim")
        self.assertEqual("service_not_accepting_mutations", rejected.exception.code)
        self.service.require_mutation_allowed("service.resume")

    def test_maintenance_hold_allows_only_evidence_reconciliation(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()
        self.service.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )

        self.service.require_mutation_allowed("milestone.reconcile_accepted")
        self.service.require_mutation_allowed("session.activate@session-a")
        self.service.require_mutation_allowed("session.set_status@session-a")
        self.service.require_mutation_allowed("lease.claim@session-a")
        self.service.require_mutation_allowed("milestone.commit@session-a")
        self.service.require_mutation_allowed("milestone.review@reviewer-session")
        self.service.require_mutation_allowed("topology.refresh@reviewer-session")
        self.service.require_mutation_allowed("maintenance.cleanup@session-a")
        self.service.require_mutation_allowed("session.register@session-a")
        self.service.require_mutation_allowed("failure.return@session-a")
        self.service.require_mutation_allowed("finalize.preview@session-a")
        self.service.require_mutation_allowed("finalize.commit@session-a")
        self.service.require_mutation_allowed("cargo.consume_cpu_reservation@session-a")
        self.service.require_mutation_allowed("codex.sessions.reconcile")
        self.service.require_mutation_allowed("cargo.reserve_cpu@session-b")
        with self.assertRaises(CoordinatorError) as resume_rejected:
            self.service.require_mutation_allowed("service.resume")
        self.assertEqual("maintenance_scope_resume_blocked", resume_rejected.exception.code)
        with self.assertRaises(CoordinatorError) as hold_rejected:
            self.service.require_mutation_allowed("lease.claim@session-b")
        self.assertEqual("maintenance_hold_active", hold_rejected.exception.code)
        with self.assertRaises(CoordinatorError) as cleanup_rejected:
            self.service.require_mutation_allowed("maintenance.cleanup@session-b")
        self.assertEqual("maintenance_hold_active", cleanup_rejected.exception.code)
        with self.assertRaises(CoordinatorError) as failure_return_rejected:
            self.service.require_mutation_allowed("failure.return@session-b")
        self.assertEqual("maintenance_hold_active", failure_return_rejected.exception.code)
        self.service.transition(
            SupervisionState.DRAINING,
            reason_code="test.explicit_scope",
            actor="test",
            updates={"explicit_stop": 1},
        )
        self.service.require_mutation_allowed("lease.claim@session-a")
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_mutation_allowed("lease.claim@session-b")
        self.assertEqual("service_explicit_stop_active", rejected.exception.code)
        with self.assertRaises(CoordinatorError) as cargo_rejected:
            self.service.require_mutation_allowed("cargo.acquire@session-a")
        self.assertEqual("service_explicit_stop_active", cargo_rejected.exception.code)

    def test_scoped_hold_allows_only_bound_explicit_release(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()
        self.service.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )

        with self.assertRaises(CoordinatorError) as ordinary_resume:
            self.service.require_mutation_allowed("service.resume")
        self.assertEqual("maintenance_scope_resume_blocked", ordinary_resume.exception.code)

        self.service.require_mutation_allowed("service.resume.release")
        with self.assertRaises(CoordinatorError) as legacy_session_release:
            self.service.require_mutation_allowed("service.resume.release@session-a")
        self.assertEqual("maintenance_release_scope_invalid", legacy_session_release.exception.code)

    def test_explicit_stop_blocks_new_mutations_even_if_timeout_restores_healthy_state(self) -> None:
        """A timed-out stop must not reopen Cargo admission before an explicit start."""
        self.service.initialize()
        self.service.mark_healthy()
        self.service.transition(
            SupervisionState.DRAINING,
            reason_code="test.stop",
            actor="test",
            updates={"explicit_stop": 1},
        )
        self.service.transition(
            SupervisionState.HEALTHY,
            reason_code="test.timeout_reconciled",
            actor="test",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.require_mutation_allowed("cargo.acquire")
        self.assertEqual("service_explicit_stop_active", rejected.exception.code)
        self.service.require_mutation_allowed("session.heartbeat")
        self.service.require_mutation_allowed("service.restart")

    def test_blocker_inventory_distinguishes_critical_work_from_advisory_leases(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(session_id, status, created_at, updated_at, last_heartbeat_at)
                VALUES ('session-a', 'active', 'now', 'now', 'now')
                """
            )
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, target_key,
                    status, created_at, last_heartbeat_at
                ) VALUES ('cargo-a', 'session-a', 'test', 'E:\\cargo-targets\\a',
                          'e:\\cargo-targets\\a', 'running', 'now', 'now')
                """
            )
            connection.execute(
                """
                INSERT INTO leases(
                    path_key, display_path, session_id, acquired_at,
                    last_heartbeat_at, expires_at
                ) VALUES ('file', 'file.txt', 'session-a', 'now', 'now', 'later')
                """
            )

        snapshot = self.service.snapshot()

        self.assertTrue(snapshot.to_dict()["busy"])
        cargo = next(item for item in snapshot.blockers if item.kind.value == "cargo")
        lease = next(item for item in snapshot.blockers if item.kind.value == "lease")
        self.assertTrue(cargo.blocking)
        self.assertFalse(lease.blocking)

    def test_invalid_transition_fails_closed(self) -> None:
        self.service.initialize()
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.transition(
                SupervisionState.OFFLINE,
                reason_code="invalid",
                actor="test",
            )
        self.assertEqual("supervision_transition_invalid", rejected.exception.code)

    def test_automatic_start_respects_persisted_explicit_stop(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()
        self.service.transition(
            SupervisionState.OFFLINE,
            reason_code="test.explicit_stop",
            actor="test",
            updates={"explicit_stop": 1},
        )
        successor = SupervisionService(
            self.database,
            repository_key="repo-key",
            daemon_instance_id="daemon-b",
            process_creation_time="creation-b",
        )

        with self.assertRaises(CoordinatorError) as rejected:
            successor.initialize(automatic_start=True)

        self.assertEqual("explicit_stop_persisted", rejected.exception.code)
        self.assertEqual(SupervisionState.OFFLINE, successor.snapshot().state)

    def test_tray_recovery_state_is_persisted_and_audited_without_state_transition(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()

        snapshot = self.service.record_recovery(
            failure_count=3,
            failure_window_started_at=100,
            next_retry_at=130,
            circuit_open_until=None,
            healthy_since=None,
        )

        self.assertEqual(SupervisionState.HEALTHY, snapshot.state)
        self.assertEqual(3, snapshot.failure_count)
        self.assertIsNotNone(snapshot.next_retry_at)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM service_recovery_state WHERE repository_key='repo-key'"
            ).fetchone()
            event = connection.execute(
                "SELECT * FROM service_supervision_events ORDER BY sequence DESC LIMIT 1"
            ).fetchone()
        self.assertEqual(3, row["failure_count"])
        self.assertEqual("tray.recovery_backoff", event["reason_code"])
        self.assertEqual("zircon-session-tray", event["actor"])

    def test_tray_recovery_state_rejects_incoherent_circuit_payload(self) -> None:
        self.service.initialize()
        self.service.mark_healthy()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.record_recovery(
                failure_count=2,
                failure_window_started_at=100,
                next_retry_at=None,
                circuit_open_until=700,
                healthy_since=None,
            )

        self.assertEqual("recovery_state_invalid", rejected.exception.code)


if __name__ == "__main__":
    unittest.main()
