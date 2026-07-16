from __future__ import annotations

import tempfile
import unittest
from datetime import timedelta
from pathlib import Path

from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoLaneKind,
    TargetPathPolicy,
)
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import (
    InvalidStatusTransition,
    SessionStatus,
    utc_now,
    utc_text,
)
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class SessionServiceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        self.service = SessionService(self.database, self.repo)

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    @staticmethod
    def compatibility() -> CargoCompatibility:
        return CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="Cargo.toml",
            build_config="profile=test;features=default;rustflags=;incremental=0;debug=0",
        )

    def cargo_jobs(self, *, process_alive=None) -> CargoJobService:
        target_root = self.repo.parent / "cargo-targets"
        target_root.mkdir(exist_ok=True)
        return CargoJobService(
            self.database,
            TargetPathPolicy((target_root,)),
            repo_root=self.repo,
            process_alive=process_alive,
            process_creation_time=lambda pid: f"stable:{pid}",
        )

    def test_register_uses_enum_status_and_current_head(self) -> None:
        session = self.service.register(
            session_id="session-a",
            display_name="M1 test",
            plan_path="docs/superpowers/plans/coordinator.md",
            write_scope=["tools/session_coordinator"],
        )

        self.assertEqual(SessionStatus.REGISTERED, session.status)
        self.assertEqual(40, len(session.base_head))
        self.assertEqual(("tools/session_coordinator",), session.write_scope)

    def test_legal_transitions_and_heartbeat_are_persisted(self) -> None:
        self.service.register(session_id="session-a")
        active = self.service.set_status("session-a", SessionStatus.ACTIVE)
        before = active.last_heartbeat_at
        heartbeat = self.service.heartbeat("session-a")
        completed = self.service.set_status("session-a", SessionStatus.COMPLETED)

        self.assertGreaterEqual(heartbeat.last_heartbeat_at, before)
        self.assertEqual(SessionStatus.COMPLETED, completed.status)

    def test_invalid_transition_is_rejected_without_mutation(self) -> None:
        self.service.register(session_id="session-a")

        with self.assertRaises(InvalidStatusTransition):
            self.service.set_status("session-a", SessionStatus.ARCHIVED)

        self.assertEqual(SessionStatus.REGISTERED, self.service.get("session-a").status)

    def test_free_form_status_is_not_accepted(self) -> None:
        self.service.register(session_id="session-a")

        with self.assertRaises(ValueError):
            self.service.set_status("session-a", "almost done")  # type: ignore[arg-type]

    def test_mark_stale_is_atomic_and_preserves_last_heartbeat(self) -> None:
        self.service.register(session_id="expired")
        self.service.register(session_id="live")
        self.service.set_status("expired", SessionStatus.ACTIVE)
        self.service.set_status("live", SessionStatus.ACTIVE)
        expired_heartbeat = utc_text(utc_now() - timedelta(hours=2))
        live_heartbeat = utc_text(utc_now())
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at = ? WHERE session_id = 'expired'",
                (expired_heartbeat,),
            )
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at = ? WHERE session_id = 'live'",
                (live_heartbeat,),
            )

        marked = self.service.mark_stale(older_than_seconds=60)

        self.assertEqual(["expired"], marked)
        expired = self.service.get("expired")
        live = self.service.get("live")
        self.assertEqual(SessionStatus.STALE, expired.status)
        self.assertEqual(expired_heartbeat, utc_text(expired.last_heartbeat_at))
        self.assertEqual(SessionStatus.ACTIVE, live.status)
        self.assertEqual(live_heartbeat, utc_text(live.last_heartbeat_at))

    def test_mark_stale_terminals_pending_cpu_reservation_in_same_transaction(self) -> None:
        self.service.register(session_id="expired")
        self.service.set_status("expired", SessionStatus.ACTIVE)
        reservation = self.cargo_jobs().reserve_cpu(
            "expired",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        expired_heartbeat = utc_text(utc_now() - timedelta(hours=2))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at=? WHERE session_id='expired'",
                (expired_heartbeat,),
            )

        marked = self.service.mark_stale(older_than_seconds=60)

        self.assertEqual(["expired"], marked)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, completed_at FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", row["status"])
        self.assertIsNotNone(row["completed_at"])

    def test_mark_stale_preserves_reservation_bound_to_running_job(self) -> None:
        self.service.register(session_id="running")
        self.service.set_status("running", SessionStatus.ACTIVE)
        cargo_jobs = self.cargo_jobs(process_alive=lambda pid: pid == 4242)
        reservation = cargo_jobs.reserve_cpu(
            "running",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = cargo_jobs.acquire(
            "running", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        cargo_jobs.start(
            job.job_id,
            session_id="running",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at=? WHERE session_id='running'",
                (utc_text(utc_now() - timedelta(hours=2)),),
            )

        marked = self.service.mark_stale(older_than_seconds=60)

        self.assertEqual(["running"], marked)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("running", row["status"])
        self.assertEqual(job.job_id, row["job_id"])

    def test_mark_stale_rolls_back_session_and_reservation_when_hook_fails(self) -> None:
        self.service.register(session_id="expired")
        self.service.set_status("expired", SessionStatus.ACTIVE)
        reservation = self.cargo_jobs().reserve_cpu(
            "expired",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at=? WHERE session_id='expired'",
                (utc_text(utc_now() - timedelta(hours=2)),),
            )
        failing = SessionService(
            self.database,
            self.repo,
            session_change_hook=lambda _connection, session: (
                (_ for _ in ()).throw(RuntimeError("hook failed"))
                if session.status is SessionStatus.STALE
                else None
            ),
        )

        with self.assertRaisesRegex(RuntimeError, "hook failed"):
            failing.mark_stale(older_than_seconds=60)

        self.assertEqual(SessionStatus.ACTIVE, self.service.get("expired").status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("pending", row["status"])

    def test_set_status_stale_terminals_pending_cpu_reservation_atomically(self) -> None:
        self.service.register(session_id="owner")
        self.service.set_status("owner", SessionStatus.ACTIVE)
        reservation = self.cargo_jobs().reserve_cpu(
            "owner",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )

        stale = self.service.set_status("owner", SessionStatus.STALE)

        self.assertEqual(SessionStatus.STALE, stale.status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, completed_at FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("expired", row["status"])
        self.assertIsNotNone(row["completed_at"])

    def test_set_status_stale_preserves_reservation_bound_to_running_job(self) -> None:
        self.service.register(session_id="running")
        self.service.set_status("running", SessionStatus.ACTIVE)
        cargo_jobs = self.cargo_jobs(process_alive=lambda pid: pid == 4242)
        reservation = cargo_jobs.reserve_cpu(
            "running",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        job = cargo_jobs.acquire(
            "running", CargoLaneKind.TEST, compatibility=self.compatibility()
        )
        cargo_jobs.start(
            job.job_id,
            session_id="running",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )

        self.service.set_status("running", SessionStatus.STALE)

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("running", row["status"])
        self.assertEqual(job.job_id, row["job_id"])

    def test_set_status_stale_rolls_back_reservation_cleanup_when_hook_fails(self) -> None:
        self.service.register(session_id="owner")
        self.service.set_status("owner", SessionStatus.ACTIVE)
        reservation = self.cargo_jobs().reserve_cpu(
            "owner",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        failing = SessionService(
            self.database,
            self.repo,
            session_change_hook=lambda _connection, session: (
                (_ for _ in ()).throw(RuntimeError("hook failed"))
                if session.status is SessionStatus.STALE
                else None
            ),
        )

        with self.assertRaisesRegex(RuntimeError, "hook failed"):
            failing.set_status("owner", SessionStatus.STALE)

        self.assertEqual(SessionStatus.ACTIVE, self.service.get("owner").status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("pending", row["status"])

    def test_archive_stale_keeps_a_session_with_an_active_cargo_job(self) -> None:
        self.service.register(session_id="cargo-owner")
        self.service.set_status("cargo-owner", SessionStatus.ACTIVE)
        target_root = self.repo.parent / "cargo-targets"
        target_root.mkdir()
        CargoJobService(
            self.database, TargetPathPolicy((target_root,)), repo_root=self.repo
        ).acquire("cargo-owner", CargoLaneKind.TEST)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET status='stale', updated_at='2000-01-01T00:00:00+00:00' "
                "WHERE session_id='cargo-owner'"
            )

        archived = self.service.archive_stale(older_than_seconds=1)

        self.assertEqual([], archived)
        self.assertEqual(SessionStatus.STALE, self.service.get("cargo-owner").status)


if __name__ == "__main__":
    unittest.main()
