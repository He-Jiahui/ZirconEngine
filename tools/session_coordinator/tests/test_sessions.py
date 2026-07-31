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
    CoordinatorError,
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

    def _insert_validation_copy(self, session_id: str, *, status: str = "planned") -> None:
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root,
                    head_commit, manifest_json, status, created_at
                ) VALUES (?, ?, 'job-root', 'source-root', 'target-root',
                          'head', '[]', ?, '2026-07-30T00:00:00+00:00')
                """,
                (f"copy-{session_id}", session_id, status),
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

    def test_reregister_rejects_a_different_existing_write_scope(self) -> None:
        self.service.register(
            session_id="session-a",
            write_scope=["owned/source.rs"],
        )

        with self.assertRaisesRegex(CoordinatorError, "scope is immutable"):
            self.service.register(
                session_id="session-a",
                write_scope=["foreign/source.rs"],
            )

        self.assertEqual(("owned/source.rs",), self.service.get("session-a").write_scope)

    def test_reregister_moves_stale_session_to_failure_resolution_atomically(self) -> None:
        self.service.register(session_id="session-a", display_name="before")
        self.service.set_status("session-a", SessionStatus.ACTIVE)
        self.service.set_status("session-a", SessionStatus.STALE)

        resumed = self.service.register(
            session_id="session-a",
            display_name="after",
            plan_path="docs/plans/tooling/01-tooling.md",
            requested_status=SessionStatus.RESOLVING_FAILURE,
            status_reason="open failure handoff requires priority",
        )

        self.assertEqual(SessionStatus.RESOLVING_FAILURE, resumed.status)
        self.assertEqual("after", resumed.display_name)
        self.assertEqual("docs/plans/tooling/01-tooling.md", resumed.plan_path)

    def test_reregister_rolls_back_metadata_when_failure_status_projection_fails(self) -> None:
        self.service.register(session_id="session-a", display_name="before")
        self.service.set_status("session-a", SessionStatus.ACTIVE)
        self.service.set_status("session-a", SessionStatus.STALE)
        failing = SessionService(
            self.database,
            self.repo,
            session_change_hook=lambda _connection, session: (
                (_ for _ in ()).throw(RuntimeError("injected projection failure"))
                if session.status is SessionStatus.RESOLVING_FAILURE
                else None
            ),
        )

        with self.assertRaisesRegex(RuntimeError, "injected projection failure"):
            failing.register(
                session_id="session-a",
                display_name="after",
                plan_path="docs/plans/tooling/01-tooling.md",
                requested_status=SessionStatus.RESOLVING_FAILURE,
                status_reason="open failure handoff requires priority",
            )

        unchanged = self.service.get("session-a")
        self.assertEqual(SessionStatus.STALE, unchanged.status)
        self.assertEqual("before", unchanged.display_name)
        self.assertIsNone(unchanged.plan_path)

    def test_default_session_liveness_is_relaxed_without_extending_resource_leases(self) -> None:
        config = CoordinatorConfig.for_repo(self.repo, state_root=self.repo.parent / "state-default")

        self.assertEqual(24 * 60 * 60, config.session_ttl_seconds)
        self.assertEqual(300, config.lease_ttl_seconds)
        self.assertEqual(120, config.lease_grace_seconds)

    def test_legal_transitions_and_heartbeat_are_persisted(self) -> None:
        self.service.register(session_id="session-a")
        active = self.service.set_status("session-a", SessionStatus.ACTIVE)
        before = active.last_heartbeat_at
        heartbeat = self.service.heartbeat("session-a")
        completed = self.service.set_status("session-a", SessionStatus.COMPLETED)

        self.assertGreaterEqual(heartbeat.last_heartbeat_at, before)
        self.assertEqual(SessionStatus.COMPLETED, completed.status)

    def test_heartbeat_reactivates_a_stale_session(self) -> None:
        self.service.register(session_id="resumed")
        self.service.set_status("resumed", SessionStatus.ACTIVE)
        self.service.set_status("resumed", SessionStatus.STALE, reason="heartbeat expired")

        resumed = self.service.heartbeat("resumed")

        self.assertEqual(SessionStatus.ACTIVE, resumed.status)
        self.assertEqual("heartbeat resumed active work", resumed.status_reason)
        with self.database.connect() as connection:
            event = connection.execute(
                "SELECT event_type, payload_json FROM events "
                "WHERE session_id = ? ORDER BY event_id DESC LIMIT 1",
                ("resumed",),
            ).fetchone()
        self.assertEqual("session.status_changed", event["event_type"])
        self.assertIn('"from": "stale"', event["payload_json"])
        self.assertIn('"to": "active"', event["payload_json"])

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

    def test_mark_stale_preserves_an_unexpired_pending_cpu_reservation(self) -> None:
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

        self.assertEqual([], marked)
        self.assertEqual(SessionStatus.ACTIVE, self.service.get("expired").status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, completed_at FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("pending", row["status"])
        self.assertIsNone(row["completed_at"])

    def test_mark_stale_preserves_reservation_bound_to_running_job(self) -> None:
        self.service.register(session_id="running")
        self.service.set_status("running", SessionStatus.ACTIVE)
        process_alive = True
        cargo_jobs = self.cargo_jobs(
            process_alive=lambda pid: process_alive and pid == 4242
        )
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

        self.assertEqual([], marked)
        self.assertEqual(SessionStatus.ACTIVE, self.service.get("running").status)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, job_id FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("running", row["status"])
        self.assertEqual(job.job_id, row["job_id"])

        process_alive = False
        cargo_jobs.finish(job.job_id, session_id="running", exit_code=0)
        cargo_jobs.release(job.job_id, session_id="running")

        marked = self.service.mark_stale(older_than_seconds=60)

        self.assertEqual(["running"], marked)
        self.assertEqual(SessionStatus.STALE, self.service.get("running").status)

    def test_mark_stale_preserves_an_unexpired_pending_cpu_reservation(self) -> None:
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
        self.assertEqual([], self.service.mark_stale(older_than_seconds=60))

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

    def test_stale_and_archive_preserve_a_session_with_a_nonterminal_validation_copy(self) -> None:
        self.service.register(session_id="copy-owner")
        self.service.set_status("copy-owner", SessionStatus.ACTIVE)
        self._insert_validation_copy("copy-owner")
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET last_heartbeat_at='2000-01-01T00:00:00+00:00' "
                "WHERE session_id='copy-owner'"
            )

        self.assertEqual([], self.service.mark_stale(older_than_seconds=1))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET status='stale', updated_at='2000-01-01T00:00:00+00:00' "
                "WHERE session_id='copy-owner'"
            )

        self.assertEqual([], self.service.archive_stale(older_than_seconds=1))
        self.assertEqual(SessionStatus.STALE, self.service.get("copy-owner").status)

    def test_archive_stale_does_not_keep_a_session_for_an_open_failure(self) -> None:
        plan_path = "docs/plans/runtime/01-runtime.md"
        self.service.register(session_id="failure-owner", plan_path=plan_path)
        self.service.set_status("failure-owner", SessionStatus.ACTIVE)
        self.service.set_status("failure-owner", SessionStatus.STALE)
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET updated_at='2000-01-01T00:00:00+00:00' "
                "WHERE session_id='failure-owner'"
            )
            connection.execute(
                """
                INSERT INTO failure_nodes(
                    lifecycle_key, artifact_path, kind, status, created_at, resolved_at,
                    summary_slug, origin_plan, fixing_plan, origin_child_dir,
                    fixing_child_dir, priority, imported_at, origin_workflow_node
                ) VALUES (?, ?, 'failure', 'open', ?, NULL, ?, ?, ?, ?, ?, 100, ?, NULL)
                """,
                (
                    "runtime01-stale-owner",
                    "docs/plans/runtime/01/failure-stale-owner.md",
                    "2000-01-01T00:00:00+00:00",
                    "stale-owner",
                    "docs/plans/runtime/01-runtime.md",
                    plan_path,
                    "docs/plans/runtime/01",
                    "docs/plans/runtime/01",
                    "2000-01-01T00:00:00+00:00",
                ),
            )

        archived = self.service.archive_stale(older_than_seconds=1)

        self.assertEqual(["failure-owner"], archived)
        self.assertEqual(SessionStatus.ARCHIVED, self.service.get("failure-owner").status)
        with self.database.connect() as connection:
            failure = connection.execute(
                "SELECT status FROM failure_nodes WHERE lifecycle_key='runtime01-stale-owner'"
            ).fetchone()
        self.assertEqual("open", failure["status"])


if __name__ == "__main__":
    unittest.main()
