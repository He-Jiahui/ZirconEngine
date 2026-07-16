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
from tools.session_coordinator.models import CoordinatorError, SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CargoReservationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/cargo-targets"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.sessions = SessionService(self.database, self.repo)
        self.sessions.register(session_id="session-a")
        self.sessions.register(session_id="session-b")
        self.policy = TargetPathPolicy([self.target_root])
        self.service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
        )
        self.service.process_creation_time = lambda pid: f"stable:{pid}"

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    @staticmethod
    def compatibility(**overrides: str) -> CargoCompatibility:
        values = {
            "platform": "windows",
            "toolchain": "stable-x86_64-pc-windows-msvc",
            "target_architecture": "x86_64-pc-windows-msvc",
            "workspace": "Cargo.toml",
            "build_config": "profile=test;features=default;rustflags=;incremental=0;debug=0",
        }
        values.update(overrides)
        return CargoCompatibility(**values)

    def test_cpu_reservation_blocks_overtake_and_requires_its_exact_command(self) -> None:
        compatibility = self.compatibility()
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=compatibility,
            command=("cargo", "test", "-p", "zircon_runtime"),
        )

        with self.assertRaises(CoordinatorError) as blocked:
            self.service.acquire("session-b", CargoLaneKind.CHECK, compatibility=compatibility)
        self.assertEqual("cargo_cpu_lane_reserved", blocked.exception.code)

        with self.assertRaises(CoordinatorError) as ephemeral_bypass:
            self.service.acquire("session-a", CargoLaneKind.TEST)
        self.assertEqual(
            "cargo_cpu_reservation_compatibility_mismatch",
            ephemeral_bypass.exception.code,
        )

        with self.assertRaises(CoordinatorError) as incompatible_bypass:
            self.service.acquire(
                "session-a",
                CargoLaneKind.TEST,
                compatibility=self.compatibility(build_config="profile=dev"),
            )
        self.assertEqual(
            "cargo_cpu_reservation_compatibility_mismatch",
            incompatible_bypass.exception.code,
        )

        job = self.service.acquire("session-a", CargoLaneKind.TEST, compatibility=compatibility)
        with self.assertRaises(CoordinatorError) as mismatched:
            self.service.start(
                job.job_id,
                session_id="session-a",
                pid=4242,
                command=["cargo", "test", "-p", "other"],
            )
        self.assertEqual("cargo_cpu_reservation_command_mismatch", mismatched.exception.code)

        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )
        self.service.process_tree_pids = lambda _pid: ()
        self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("finished", row["status"])

        with self.assertRaises(CoordinatorError) as held_for_handoff:
            self.service.acquire(
                "session-b",
                CargoLaneKind.CHECK,
                compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
            )
        self.assertEqual("cargo_cpu_lane_reserved", held_for_handoff.exception.code)

        released = self.service.release_cpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )
        self.assertEqual("released", released["status"])

        following = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
        )
        self.assertEqual(CargoJobStatus.LEASED, following.status)

    def test_pending_cpu_reservation_renews_without_changing_identity(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
            ttl_seconds=30,
        )

        renewed = self.service.renew_cpu_reservation(
            reservation["reservationId"], session_id="session-a", ttl_seconds=3600
        )

        self.assertEqual(reservation["reservationId"], renewed["reservationId"])
        self.assertEqual("pending", renewed["status"])
        self.assertGreater(renewed["expiresAt"], reservation["expiresAt"])

    def test_cpu_reservation_persists_canonical_compatibility_payload(self) -> None:
        compatibility = self.compatibility()

        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=compatibility,
            command=("cargo", "test", "-p", "zircon_runtime"),
        )

        self.assertEqual(compatibility.canonical(), reservation["compatibility"])
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT compatibility_json FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual(
            compatibility.canonical(), json.loads(row["compatibility_json"])
        )

    def test_cpu_reservation_rejects_non_executable_session_states(self) -> None:
        for status in (
            SessionStatus.COMPLETED,
            SessionStatus.STALE,
            SessionStatus.ARCHIVED,
            SessionStatus.CANCELLED,
        ):
            with self.subTest(status=status.value):
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET status=? WHERE session_id='session-a'",
                        (status.value,),
                    )

                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.reserve_cpu(
                        "session-a",
                        compatibility=self.compatibility(),
                        command=("cargo", "test", "-p", "zircon_runtime"),
                    )

                self.assertEqual("cargo_session_not_executable", rejected.exception.code)

    def test_cpu_acquire_rejects_non_executable_session_states(self) -> None:
        for status in (
            SessionStatus.COMPLETED,
            SessionStatus.STALE,
            SessionStatus.ARCHIVED,
            SessionStatus.CANCELLED,
        ):
            with self.subTest(status=status.value):
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET status=? WHERE session_id='session-a'",
                        (status.value,),
                    )

                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.acquire("session-a", CargoLaneKind.TEST)

                self.assertEqual("cargo_session_not_executable", rejected.exception.code)

    def test_pending_cpu_reservation_renewal_rejects_non_executable_owner(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        for status in (
            SessionStatus.COMPLETED,
            SessionStatus.STALE,
            SessionStatus.ARCHIVED,
            SessionStatus.CANCELLED,
        ):
            with self.subTest(status=status.value):
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET status=? WHERE session_id='session-a'",
                        (status.value,),
                    )

                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.renew_cpu_reservation(
                        reservation["reservationId"], session_id="session-a"
                    )

                self.assertEqual("cargo_session_not_executable", rejected.exception.code)

    def test_stale_pending_cpu_reservation_does_not_block_next_acquire(self) -> None:
        stale = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET status='stale' WHERE session_id='session-a'"
            )

        following = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
        )

        with self.database.connect() as connection:
            stale_row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (stale["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", stale_row["status"])
        self.assertEqual(CargoJobStatus.LEASED, following.status)
        self.assertEqual("session-b", following.session_id)

    def test_released_terminal_job_releases_its_cpu_reservation_without_owner_handoff(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )
        self.service.process_tree_pids = lambda _pid: ()
        self.service.finish(job.job_id, session_id="session-a", exit_code=101)
        self.sessions.set_status("session-a", SessionStatus.STALE)
        self.service.release(job.job_id, session_id="session-a")

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("released", row["status"])

        following = self.service.acquire("session-b", CargoLaneKind.CHECK)
        self.assertEqual(CargoJobStatus.LEASED, following.status)

    def test_stale_finished_reservation_from_released_job_is_reconciled_before_next_acquire(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )
        self.service.process_tree_pids = lambda _pid: ()
        self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        self.service.release(job.job_id, session_id="session-a")
        self.sessions.set_status("session-a", SessionStatus.STALE)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET status='finished' WHERE reservation_id=?",
                (reservation["reservationId"],),
            )

        following = self.service.acquire("session-b", CargoLaneKind.CHECK)

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("released", row["status"])
        self.assertEqual(CargoJobStatus.LEASED, following.status)

    def test_expired_pending_cpu_reservation_advances_fifo(self) -> None:
        expired = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                ("2000-01-01T00:00:00+00:00", expired["reservationId"]),
            )

        following = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
        )

        with self.database.connect() as connection:
            expired_row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (expired["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", expired_row["status"])
        self.assertEqual(CargoJobStatus.LEASED, following.status)
        self.assertEqual("session-b", following.session_id)

    def test_recreated_service_preserves_pending_reservation_absolute_expiry(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
            ttl_seconds=3600,
        )
        recreated = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
        )

        recovered = recreated.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
            ttl_seconds=3600,
        )

        self.assertEqual(reservation["reservationId"], recovered["reservationId"])
        self.assertEqual(reservation["expiresAt"], recovered["expiresAt"])

    def test_running_cpu_reservation_is_not_expired_by_pending_ttl(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
            ttl_seconds=30,
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                ("2000-01-01T00:00:00+00:00", reservation["reservationId"]),
            )

        with self.assertRaises(CoordinatorError) as blocked:
            self.service.reserve_cpu(
                "session-b",
                compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
                command=("cargo", "check", "-p", "zircon_editor"),
            )

        self.assertEqual("cargo_cpu_lane_reserved", blocked.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("running", row["status"])

    def test_leased_cpu_reservation_survives_stale_owner_and_pending_ttl(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
            ttl_seconds=30,
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        self.sessions.set_status("session-a", SessionStatus.STALE)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                ("2000-01-01T00:00:00+00:00", reservation["reservationId"]),
            )

        with self.assertRaises(CoordinatorError) as blocked:
            self.service.reserve_cpu(
                "session-b",
                compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
                command=("cargo", "check", "-p", "zircon_editor"),
            )

        self.assertEqual("cargo_cpu_lane_reserved", blocked.exception.code)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("leased", row["status"])
        self.assertEqual(job.job_id, row["job_id"])

    def test_orphaned_leased_job_expires_bound_reservation_and_advances_fifo(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )

        orphaned = self.service.reconcile_orphans(
            now=job.last_heartbeat_at + timedelta(minutes=10),
            leased_timeout_seconds=300,
        )

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", row["status"])
        following = self.service.acquire("session-b", CargoLaneKind.CHECK)
        self.assertEqual(CargoJobStatus.LEASED, following.status)

    def test_orphaned_running_job_expires_bound_reservation_and_advances_fifo(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        self.service.start(
            job.job_id,
            session_id="session-a",
            pid=9999,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )

        orphaned = self.service.reconcile_orphans()

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", row["status"])
        following = self.service.acquire("session-b", CargoLaneKind.CHECK)
        self.assertEqual(CargoJobStatus.LEASED, following.status)


if __name__ == "__main__":
    unittest.main()
