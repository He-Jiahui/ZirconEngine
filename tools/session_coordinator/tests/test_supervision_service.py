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
