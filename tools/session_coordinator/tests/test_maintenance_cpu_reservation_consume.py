from __future__ import annotations

import json
import tempfile
import unittest
from datetime import timedelta
from pathlib import Path

from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoJobStatus,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, SupervisionState
from tools.session_coordinator.server import CoordinatorApplication
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.supervision.service import SupervisionService
from tools.session_coordinator.tests.helpers import init_repo


class MaintenanceCpuReservationConsumeTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary_directory.cleanup)
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "D" / "cargo-targets"
        self.target_root.mkdir(parents=True)
        self.config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state", port=0)
        self.database = Database(self.config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.sessions.register(session_id="session-b")
        self.jobs = CargoJobService(
            self.database,
            TargetPathPolicy((self.target_root,)),
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
        )

    @staticmethod
    def compatibility() -> CargoCompatibility:
        return CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="Cargo.toml",
            build_config=json.dumps(
                {"rustflags": "-C debuginfo=0", "cargo_incremental": "0"},
                sort_keys=True,
            ),
        )

    def test_scoped_hold_allows_only_the_bound_reservation_consume(self) -> None:
        supervision = SupervisionService(
            self.database,
            repository_key="repo-key",
            daemon_instance_id="daemon-a",
            process_creation_time="creation-a",
            maintenance_session_ids=("session-a",),
        )
        supervision.initialize()
        supervision.mark_healthy()
        supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )

        supervision.require_mutation_allowed(
            "cargo.consume_cpu_reservation@session-a"
        )
        with self.assertRaises(CoordinatorError) as foreign:
            supervision.require_mutation_allowed(
                "cargo.consume_cpu_reservation@session-b"
            )
        self.assertEqual("maintenance_hold_active", foreign.exception.code)
        with self.assertRaises(CoordinatorError) as generic:
            supervision.require_mutation_allowed("cargo.acquire@session-a")
        self.assertEqual("maintenance_hold_active", generic.exception.code)
        supervision.require_mutation_allowed("cargo.run_reserved@session-a")
        supervision.require_mutation_allowed("cargo.recover_expired_reservation@session-a")
        with self.assertRaises(CoordinatorError) as generic_run:
            supervision.require_mutation_allowed("cargo.run@session-a")
        self.assertEqual("maintenance_hold_active", generic_run.exception.code)

    def test_consume_pending_cpu_reservation_creates_one_unstarted_job(self) -> None:
        reservation = self.jobs.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime", "render_volumetric"),
        )

        job = self.jobs.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )
        repeated = self.jobs.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )

        self.assertEqual(CargoJobStatus.LEASED, job.status)
        self.assertIsNone(job.pid)
        self.assertEqual(job.job_id, repeated.job_id)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("leased", row["status"])
        self.assertEqual(job.job_id, row["job_id"])

    def test_held_scope_exposes_a_typed_gpu_reservation_consume_api(self) -> None:
        """A held owner needs one exact GPU lease without generic acquisition."""
        self.assertTrue(hasattr(self.jobs, "reserve_gpu"))
        self.assertTrue(hasattr(self.jobs, "consume_gpu_reservation"))

        supervision = SupervisionService(
            self.database,
            repository_key="repo-key",
            daemon_instance_id="daemon-a",
            process_creation_time="creation-a",
            maintenance_session_ids=("session-a",),
        )
        supervision.initialize()
        supervision.mark_healthy()
        supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"explicit_stop": 1, "maintenance_hold": 1},
        )
        supervision.require_mutation_allowed("cargo.reserve_gpu@session-a")
        supervision.require_mutation_allowed("cargo.consume_gpu_reservation@session-a")
        supervision.require_mutation_allowed("cargo.run_reserved@session-a")
        with self.assertRaises(CoordinatorError) as generic:
            supervision.require_mutation_allowed("cargo.acquire@session-a")
        self.assertEqual("service_explicit_stop_active", generic.exception.code)

        command = (
            "cargo",
            "test",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
            "-p",
            "zircon_plugin_rendering_volumetric_fog_runtime",
            "--locked",
        )
        target = self.target_root / "zircon-engine" / "render18-af-m3-plugin"
        reservation = self.jobs.reserve_gpu(
            "session-a",
            compatibility=self.compatibility(),
            target_dir=target,
            command=command,
        )
        job = self.jobs.consume_gpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )
        self.assertEqual("gpu", reservation["laneScope"])
        self.assertEqual(str(target), reservation["targetDir"])
        self.assertEqual(CargoLaneKind.GPU, job.lane_kind)
        self.assertEqual(str(target), str(job.target_dir))
        self.assertEqual(
            {"RUSTFLAGS": "-C debuginfo=0", "CARGO_INCREMENTAL": "0"},
            self.jobs.reserved_run_environment(
                reservation["reservationId"],
                session_id="session-a",
                job_id=job.job_id,
                command=command,
            ),
        )

    def test_gpu_reserved_run_accepts_existing_semicolon_compatibility(self) -> None:
        command = ("pwsh", "-NoProfile", "-Command", "render18 gpu sequence")
        compatibility = CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="zircon_plugins/Cargo.toml",
            build_config=(
                "profile=test;package=zircon_plugin_rendering_volumetric_fog_runtime;"
                "features=default;locked=true;render18-af-m3-product;current-source-rebuild"
            ),
        )
        reservation = self.jobs.reserve_gpu(
            "session-a",
            compatibility=compatibility,
            target_dir=self.target_root / "zircon-engine" / "render18-af-m3-plugin",
            command=command,
        )
        job = self.jobs.consume_gpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )

        self.assertEqual(
            {},
            self.jobs.reserved_run_environment(
                reservation["reservationId"],
                session_id="session-a",
                job_id=job.job_id,
                command=command,
            ),
        )

    def test_gpu_recovery_restores_only_its_expired_unstarted_binding(self) -> None:
        self.assertTrue(hasattr(self.jobs, "recover_expired_reservation"))
        command = ("pwsh", "-NoProfile", "-Command", "render18 gpu sequence")
        reservation = self.jobs.reserve_gpu(
            "session-a",
            compatibility=self.compatibility(),
            target_dir=self.target_root / "zircon-engine" / "render18-af-m3-plugin",
            command=command,
        )
        job = self.jobs.consume_gpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )
        self.jobs.reconcile_orphans(
            now=job.last_heartbeat_at + timedelta(minutes=10)
        )

        recovered = self.jobs.recover_expired_reservation(
            reservation["reservationId"],
            job_id=job.job_id,
            session_id="session-a",
        )

        self.assertEqual(CargoJobStatus.LEASED, recovered.status)
        self.assertIsNone(recovered.pid)
        self.assertIsNone(recovered.started_at)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id, completed_at FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("leased", row["status"])
        self.assertEqual(job.job_id, row["job_id"])
        self.assertIsNone(row["completed_at"])

    def test_reserved_run_derives_environment_and_rechecks_exact_command(self) -> None:
        command = ("cargo", "test", "-p", "zircon_runtime", "render_volumetric")
        reservation = self.jobs.reserve_cpu(
            "session-a", compatibility=self.compatibility(), command=command
        )
        job = self.jobs.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )

        environment = self.jobs.reserved_run_environment(
            reservation["reservationId"],
            session_id="session-a",
            job_id=job.job_id,
            command=command,
        )

        self.assertEqual(
            {"RUSTFLAGS": "-C debuginfo=0", "CARGO_INCREMENTAL": "0"},
            environment,
        )
        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.reserved_run_environment(
                reservation["reservationId"],
                session_id="session-a",
                job_id=job.job_id,
                command=("cargo", "test", "-p", "zircon_runtime", "other"),
            )
        self.assertEqual("cargo_cpu_reservation_command_mismatch", rejected.exception.code)

    def test_consume_rejects_foreign_owner_without_consuming_fifo(self) -> None:
        reservation = self.jobs.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.consume_cpu_reservation(
                reservation["reservationId"],
                session_id="session-b",
                lane_kind=CargoLaneKind.TEST,
            )

        self.assertEqual("cargo_cpu_reservation_owner_mismatch", rejected.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("pending", row["status"])
        self.assertIsNone(row["job_id"])

    def test_consume_rejects_released_reservation_without_rebinding(self) -> None:
        reservation = self.jobs.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        self.jobs.release_cpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.jobs.consume_cpu_reservation(
                reservation["reservationId"],
                session_id="session-a",
                lane_kind=CargoLaneKind.TEST,
            )

        self.assertEqual("cargo_cpu_reservation_consumed", rejected.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("released", row["status"])
        self.assertIsNone(row["job_id"])

    def test_server_rejects_client_target_or_compatibility_overrides(self) -> None:
        application = CoordinatorApplication(self.config)

        with self.assertRaises(CoordinatorError) as rejected:
            application._command_unlocked(
                "cargo.consume_cpu_reservation",
                {
                    "session_id": "session-a",
                    "reservation_id": "reservation-a",
                    "lane_kind": "test",
                    "target_dir": "D:/cargo-targets/foreign",
                },
            )

        self.assertEqual("cargo_reservation_consume_arguments_invalid", rejected.exception.code)

    def test_server_rejects_reserved_run_environment_or_target_override(self) -> None:
        application = CoordinatorApplication(self.config)

        with self.assertRaises(CoordinatorError) as rejected:
            application._command_unlocked(
                "cargo.run_reserved",
                {
                    "session_id": "session-a",
                    "reservation_id": "reservation-a",
                    "job_id": "job-a",
                    "command": ["cargo", "test"],
                    "environment": {"RUSTFLAGS": "-C target-cpu=native"},
                },
            )

        self.assertEqual("cargo_reservation_run_arguments_invalid", rejected.exception.code)

    def test_server_rejects_gpu_consume_target_override(self) -> None:
        application = CoordinatorApplication(self.config)

        with self.assertRaises(CoordinatorError) as rejected:
            application._command_unlocked(
                "cargo.consume_gpu_reservation",
                {
                    "session_id": "session-a",
                    "reservation_id": "reservation-a",
                    "target_dir": "D:/cargo-targets/foreign",
                },
            )

        self.assertEqual("cargo_reservation_consume_arguments_invalid", rejected.exception.code)

    def test_server_rejects_recovery_target_override(self) -> None:
        application = CoordinatorApplication(self.config)

        with self.assertRaises(CoordinatorError) as rejected:
            application._command_unlocked(
                "cargo.recover_expired_reservation",
                {
                    "session_id": "session-a",
                    "reservation_id": "reservation-a",
                    "job_id": "job-a",
                    "target_dir": "D:/cargo-targets/foreign",
                },
            )

        self.assertEqual("cargo_reservation_recovery_arguments_invalid", rejected.exception.code)

    def test_startup_scope_uses_latest_succeeded_drain_when_event_is_coalesced(self) -> None:
        application = CoordinatorApplication(self.config)
        application.supervision.mark_healthy()
        application.supervision.transition(
            SupervisionState.DRAINING,
            reason_code="test.maintenance_hold",
            actor="test",
            updates={"maintenance_hold": 1},
        )
        drains = (
            ("drain-old", ("session-a",), "2026-07-16T00:00:00+00:00"),
            (
                "drain-latest",
                ("session-a", "session-b"),
                "2026-07-16T00:01:00+00:00",
            ),
        )
        with self.database.transaction() as connection:
            for action_id, session_ids, completed_at in drains:
                connection.execute(
                    """INSERT INTO action_requests(
                           action_id, action_kind, risk, required_role, actor,
                           daemon_instance_id, parameters_json, impact_json, warnings_json,
                           state_fingerprint, confirmation_phrase_hash, status, created_at,
                           expires_at, completed_at
                       ) VALUES (?, 'service.drain', 'yellow', 'operator', 'test',
                                 'daemon', ?, '[]', '[]', 'fingerprint', 'phrase', 'succeeded',
                                 ?, '2099-01-01T00:00:00+00:00', ?)""",
                    (
                        action_id,
                        json.dumps({"maintenanceSessionIds": list(session_ids), "timeoutSeconds": 60}),
                        completed_at,
                        completed_at,
                    ),
                )

        self.assertEqual(
            ("session-a", "session-b"),
            application._maintenance_session_ids_for_startup(),
        )


if __name__ == "__main__":
    unittest.main()
