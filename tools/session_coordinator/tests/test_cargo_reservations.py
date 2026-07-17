from __future__ import annotations

import hashlib
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
    MAX_SOURCE_MANIFEST_ENTRIES,
    TargetPathPolicy,
)
from tools.session_coordinator.cargo_reservations import (
    reconcile_terminal_finished_cpu_reservations,
)
from tools.session_coordinator.resource_budget import (
    BURST_MIN_FREE_BYTES,
    BURST_MIN_FREE_MEMORY_BYTES,
    BURST_SAMPLE_COUNT,
    ResourceSample,
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
        self.sessions.register(session_id="session-c")
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

    def test_cpu_burst_eligibility_is_opt_in_and_check_only(self) -> None:
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "check", "-p", "zircon_runtime"),
            burst_eligible=True,
        )

        self.assertTrue(reservation["burstEligible"])
        self.assertEqual("warm", reservation["executionMode"])
        with self.database.connect() as connection:
            persisted = connection.execute(
                "SELECT burst_eligible, execution_mode FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual(1, persisted["burst_eligible"])
        self.assertEqual("warm", persisted["execution_mode"])

        self.service.release_cpu_reservation(
            reservation["reservationId"], session_id="session-a"
        )
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.reserve_cpu(
                "session-a",
                compatibility=self.compatibility(),
                command=("cargo", "test", "-p", "zircon_runtime"),
                burst_eligible=True,
            )
        self.assertEqual("cargo_cpu_burst_eligibility_invalid", rejected.exception.code)

    def test_eligible_check_uses_one_isolated_burst_target_behind_warm_work(self) -> None:
        burst_root = self.target_root / "zircon-engine/burst"
        samples = tuple(
            ResourceSample(cpu_percent=25.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)
            for _ in range(BURST_SAMPLE_COUNT)
        )
        service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: BURST_MIN_FREE_BYTES,
            process_alive=lambda pid: pid == 4242,
            burst_target_root=burst_root,
            burst_samples=lambda: samples,
        )
        service.process_creation_time = lambda pid: f"stable:{pid}"
        warm = service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        warm_job = service.consume_cpu_reservation(
            warm["reservationId"], session_id="session-a", lane_kind=CargoLaneKind.TEST
        )
        burst = service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=dev"),
            command=("cargo", "check", "-p", "zircon_runtime"),
            burst_eligible=True,
        )

        burst_job = service.consume_cpu_reservation(
            burst["reservationId"], session_id="session-b", lane_kind=CargoLaneKind.CHECK
        )

        self.assertEqual(str(burst_root / burst["reservationId"]), burst_job.target_dir)
        self.assertEqual("delete_on_release", burst_job.cleanup_policy.value)
        with self.database.connect() as connection:
            reservation = connection.execute(
                "SELECT execution_mode, status FROM cargo_lane_reservations WHERE reservation_id=?",
                (burst["reservationId"],),
            ).fetchone()
        self.assertEqual("burst", reservation["execution_mode"])
        self.assertEqual("leased", reservation["status"])
        started = service.start(
            burst_job.job_id,
            session_id="session-b",
            pid=4242,
            command=["cargo", "check", "-p", "zircon_runtime"],
        )
        self.assertEqual(CargoJobStatus.RUNNING, started.status)
        self.assertEqual(CargoJobStatus.RUNNING, service.heartbeat(
            burst_job.job_id, session_id="session-b"
        ).status)
        with self.database.connect() as connection:
            running = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (burst["reservationId"],),
            ).fetchone()
        self.assertEqual("running", running["status"])
        service.process_tree_pids = lambda _pid: ()
        finished = service.finish(burst_job.job_id, session_id="session-b", exit_code=0)
        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        released = service.release(burst_job.job_id, session_id="session-b")
        self.assertEqual(CargoJobStatus.RELEASED, released.status)
        self.assertEqual("pending", released.cleanup_status.value)
        with self.database.connect() as connection:
            released_reservation = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (burst["reservationId"],),
            ).fetchone()
        self.assertEqual("released", released_reservation["status"])
        self.assertEqual(CargoJobStatus.LEASED, service.get(warm_job.job_id).status)

    def test_resource_denied_burst_keeps_the_reservation_in_warm_fifo(self) -> None:
        burst_root = self.target_root / "zircon-engine/burst"
        samples = tuple(
            ResourceSample(cpu_percent=25.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)
            for _ in range(BURST_SAMPLE_COUNT)
        )
        service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: BURST_MIN_FREE_BYTES - 1,
            process_alive=lambda pid: pid == 4242,
            burst_target_root=burst_root,
            burst_samples=lambda: samples,
        )
        warm = service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        service.consume_cpu_reservation(
            warm["reservationId"], session_id="session-a", lane_kind=CargoLaneKind.TEST
        )
        burst = service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=dev"),
            command=("cargo", "check", "-p", "zircon_runtime"),
            burst_eligible=True,
        )

        with self.assertRaises(CoordinatorError) as queued:
            service.consume_cpu_reservation(
                burst["reservationId"], session_id="session-b", lane_kind=CargoLaneKind.CHECK
            )

        self.assertEqual("cargo_cpu_reservation_not_fifo_head", queued.exception.code)
        with self.database.connect() as connection:
            reservation = connection.execute(
                "SELECT execution_mode, status FROM cargo_lane_reservations WHERE reservation_id=?",
                (burst["reservationId"],),
            ).fetchone()
        self.assertEqual("warm", reservation["execution_mode"])
        self.assertEqual("pending", reservation["status"])

    def test_burst_reservation_rechecks_its_source_manifest_before_start(self) -> None:
        relative_path = "zircon_runtime/src/input/runtime/event_buffer/frame.rs"
        source = self.repo / relative_path
        source.parent.mkdir(parents=True)
        source.write_text("pub(crate) struct FrameEventBuffer;\n")
        compatibility = self.compatibility(
            build_config=json.dumps(
                {
                    "profile": "dev",
                    "source_manifest": {
                        relative_path: hashlib.sha256(source.read_bytes()).hexdigest().upper()
                    },
                },
                sort_keys=True,
            )
        )
        burst_root = self.target_root / "zircon-engine/burst"
        samples = tuple(
            ResourceSample(cpu_percent=25.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)
            for _ in range(BURST_SAMPLE_COUNT)
        )
        service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: BURST_MIN_FREE_BYTES,
            process_alive=lambda pid: pid == 4242,
            burst_target_root=burst_root,
            burst_samples=lambda: samples,
        )
        warm = service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        service.consume_cpu_reservation(
            warm["reservationId"], session_id="session-a", lane_kind=CargoLaneKind.TEST
        )
        command = ("cargo", "check", "-p", "zircon_runtime")
        burst = service.reserve_cpu(
            "session-b", compatibility=compatibility, command=command, burst_eligible=True
        )
        job = service.consume_cpu_reservation(
            burst["reservationId"], session_id="session-b", lane_kind=CargoLaneKind.CHECK
        )

        source.write_text("pub(crate) struct ChangedFrameEventBuffer;\n")
        with self.assertRaises(CoordinatorError) as stale:
            service.reserved_run_environment(
                burst["reservationId"],
                session_id="session-b",
                job_id=job.job_id,
                command=command,
            )

        self.assertEqual("cargo_cpu_reservation_source_manifest_stale", stale.exception.code)

    def test_source_manifest_rejects_entries_past_explicit_hard_cutover_limit(self) -> None:
        """Large complete scopes are valid, but the payload limit remains bounded."""
        manifest = {
            f"zircon_plugins/sound/runtime/src/generated/owned_{index:04d}.rs": "A" * 64
            for index in range(MAX_SOURCE_MANIFEST_ENTRIES + 1)
        }

        with self.assertRaises(CoordinatorError) as rejected:
            self.compatibility(source_manifest=manifest).canonical()

        self.assertEqual("invalid_cargo_source_manifest", rejected.exception.code)
        self.assertIn(str(MAX_SOURCE_MANIFEST_ENTRIES), rejected.exception.message)

    def test_burst_target_creation_failure_returns_its_reservation_to_warm_fifo(self) -> None:
        blocked_root = self.target_root / "blocked-burst-root"
        blocked_root.write_text("not a directory", encoding="utf-8")
        samples = tuple(
            ResourceSample(cpu_percent=25.0, free_memory_bytes=BURST_MIN_FREE_MEMORY_BYTES)
            for _ in range(BURST_SAMPLE_COUNT)
        )
        service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: BURST_MIN_FREE_BYTES,
            process_alive=lambda pid: pid == 4242,
            burst_target_root=blocked_root,
            burst_samples=lambda: samples,
        )
        warm = service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        service.consume_cpu_reservation(
            warm["reservationId"], session_id="session-a", lane_kind=CargoLaneKind.TEST
        )
        burst = service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=dev"),
            command=("cargo", "check", "-p", "zircon_runtime"),
            burst_eligible=True,
        )

        with self.assertRaises(OSError):
            service.consume_cpu_reservation(
                burst["reservationId"], session_id="session-b", lane_kind=CargoLaneKind.CHECK
            )

        with self.database.connect() as connection:
            reservation = connection.execute(
                "SELECT execution_mode, status, job_id FROM cargo_lane_reservations "
                "WHERE reservation_id=?",
                (burst["reservationId"],),
            ).fetchone()
        self.assertEqual("warm", reservation["execution_mode"])
        self.assertEqual("pending", reservation["status"])
        self.assertIsNone(reservation["job_id"])

    def test_cpu_reservation_preserves_explicit_approved_target_when_consumed(
        self,
    ) -> None:
        target = self.target_root / "zircon-engine/pool/render01-parity"
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            target_dir=target,
            command=("cargo", "test", "-p", "zircon_runtime"),
        )

        job = self.service.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )

        self.assertEqual(str(target), reservation["targetDir"])
        self.assertEqual(str(target), job.target_dir)

    def test_same_owner_can_correct_pending_reservation_command_without_losing_fifo(
        self,
    ) -> None:
        """An unstarted reservation stays in place when its exact command is repaired."""
        target = self.target_root / "zircon-engine/pool/runtime12-event-buffer"
        compatibility = self.compatibility()
        original = self.service.reserve_cpu(
            "session-a",
            compatibility=compatibility,
            target_dir=target,
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "input"),
            ttl_seconds=900,
        )

        corrected_command = (
            "cargo",
            "test",
            "-p",
            "zircon_runtime",
            "--lib",
            "input::tests::input_manager::event_buffer::begin_frame_discards_undrained_transient_events",
            "--locked",
            "--jobs",
            "1",
            "--",
            "--exact",
            "--test-threads=1",
        )
        corrected = self.service.reserve_cpu(
            "session-a",
            compatibility=compatibility,
            target_dir=target,
            command=corrected_command,
            ttl_seconds=900,
        )

        self.assertEqual(original["reservationId"], corrected["reservationId"])
        self.assertEqual(original["createdAt"], corrected["createdAt"])
        self.assertEqual("pending", corrected["status"])
        self.assertEqual(str(target), corrected["targetDir"])
        self.assertNotEqual(original["commandFingerprint"], corrected["commandFingerprint"])
        self.assertEqual(
            self.service._command_fingerprint(corrected_command),
            corrected["commandFingerprint"],
        )

        queued = self.service.reserve_cpu(
            "session-b",
            compatibility=compatibility,
            target_dir=target,
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "other"),
        )
        self.assertEqual("pending", queued["status"])
        with self.assertRaises(CoordinatorError) as out_of_order:
            self.service.consume_cpu_reservation(
                queued["reservationId"],
                session_id="session-b",
                lane_kind=CargoLaneKind.TEST,
            )
        self.assertEqual("cargo_cpu_reservation_not_fifo_head", out_of_order.exception.code)

    def test_cpu_reservations_queue_multiple_exact_successors_in_fifo_order(self) -> None:
        first = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "first"),
        )
        second = self.service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=test;package=second"),
            command=("cargo", "test", "-p", "zircon_runtime", "--lib", "second"),
        )

        with self.assertRaises(CoordinatorError) as out_of_order:
            self.service.consume_cpu_reservation(
                second["reservationId"],
                session_id="session-b",
                lane_kind=CargoLaneKind.TEST,
            )
        self.assertEqual("cargo_cpu_reservation_not_fifo_head", out_of_order.exception.code)

        first_job = self.service.consume_cpu_reservation(
            first["reservationId"], session_id="session-a", lane_kind=CargoLaneKind.TEST
        )
        self.service.release(first_job.job_id, session_id="session-a")

        second_job = self.service.consume_cpu_reservation(
            second["reservationId"], session_id="session-b", lane_kind=CargoLaneKind.TEST
        )
        self.assertEqual(CargoJobStatus.LEASED, second_job.status)

    def test_unreserved_cpu_lease_cannot_start_ahead_of_consumed_priority_reservation(
        self,
    ) -> None:
        generic = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(build_config="profile=dev;features=generic"),
        )
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        priority_job = self.service.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )

        self.assertEqual(CargoJobStatus.LEASED, priority_job.status)
        with self.assertRaises(CoordinatorError) as blocked:
            self.service.start(
                generic.job_id,
                session_id="session-b",
                pid=4242,
                command=["cargo", "check", "-p", "zircon_editor"],
            )
        self.assertEqual("cargo_cpu_lane_reserved", blocked.exception.code)

    def test_unreserved_cpu_lease_can_start_after_stale_priority_reservation_expires(
        self,
    ) -> None:
        generic = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(build_config="profile=dev;features=generic"),
        )
        reservation = self.service.reserve_cpu(
            "session-a",
            compatibility=self.compatibility(),
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE cargo_lane_reservations SET expires_at=? WHERE reservation_id=?",
                ("2000-01-01T00:00:00+00:00", reservation["reservationId"]),
            )

        started = self.service.start(
            generic.job_id,
            session_id="session-b",
            pid=4242,
            command=["cargo", "check", "-p", "zircon_editor"],
        )
        self.assertEqual(CargoJobStatus.RUNNING, started.status)

    def test_start_persists_initial_managed_process_tree_observation(self) -> None:
        self.service.supervisor_cargo_pids = lambda _pid: (7101, 7102)
        job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=self.compatibility()
        )

        started = self.service.start(
            job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
            root_is_supervisor=True,
        )

        self.assertEqual((7101, 7102), started.live_process_pids)
        self.assertIsNotNone(started.process_tree_observed_at)

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

    def test_cpu_reservation_rejects_start_when_bound_source_manifest_drifts(self) -> None:
        """A reserved command must not run after its attributed source bytes change."""
        relative_path = "zircon_runtime/src/input/runtime/event_buffer/frame.rs"
        source = self.repo / relative_path
        source.parent.mkdir(parents=True)
        source.write_text("pub(in crate::input::runtime) struct FrameEventBuffer;\n")
        expected_hash = hashlib.sha256(source.read_bytes()).hexdigest().upper()
        compatibility = self.compatibility(
            build_config=json.dumps(
                {
                    "profile": "test",
                    "source_manifest": {relative_path: expected_hash},
                },
                sort_keys=True,
            )
        )
        command = ("cargo", "test", "-p", "zircon_runtime", "--lib", "input")

        reservation = self.service.reserve_cpu(
            "session-a", compatibility=compatibility, command=command
        )
        self.assertEqual({relative_path: expected_hash}, reservation["sourceManifest"])
        job = self.service.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )

        source.write_text("pub(crate) struct FrameEventBuffer;\n")
        with self.assertRaises(CoordinatorError) as stale:
            self.service.reserved_run_environment(
                reservation["reservationId"],
                session_id="session-a",
                job_id=job.job_id,
                command=command,
            )

        self.assertEqual("cargo_cpu_reservation_source_manifest_stale", stale.exception.code)

    def test_cpu_reservation_supports_first_class_large_source_manifest_and_rechecks_all_entries(
        self,
    ) -> None:
        """A full expanded directory scope cannot be squeezed into build_config."""
        manifest: dict[str, str] = {}
        # The Sound hard-cutover owns 1,275 source files. Its complete
        # current-source manifest must remain reserve/start-bound rather than
        # being truncated to fit the former Render01-sized limit.
        for index in range(1275):
            relative_path = (
                "zircon_runtime/src/graphics/pipeline/declarations/"
                f"compiled_render_pipeline/owned_{index:02d}.rs"
            )
            source = self.repo / relative_path
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(f"pub const OWNED_{index}: u8 = {index};\n")
            manifest[relative_path] = hashlib.sha256(source.read_bytes()).hexdigest().upper()

        serialized_manifest = json.dumps(manifest, sort_keys=True, separators=(",", ":"))
        self.assertGreater(len(serialized_manifest), 4096)
        ordinal_manifest = "\n".join(
            f"{path.casefold()}={digest.casefold()}"
            for path, digest in sorted(manifest.items())
        )
        manifest_fingerprint = hashlib.sha256(ordinal_manifest.encode("utf-8")).hexdigest()
        compatibility = CargoCompatibility(
            platform="windows",
            toolchain="stable-x86_64-pc-windows-msvc",
            target_architecture="x86_64-pc-windows-msvc",
            workspace="Cargo.toml",
            build_config="profile=test;features=default;rustflags=;incremental=0;debug=0",
            source_manifest=manifest,
        )
        command = ("cargo", "test", "-p", "zircon_runtime", "--lib", "render01_compiled_pipeline_")

        reservation = self.service.reserve_cpu(
            "session-a", compatibility=compatibility, command=command
        )
        self.assertEqual(manifest, reservation["sourceManifest"])
        self.assertEqual(manifest_fingerprint, reservation["sourceManifestFingerprint"])
        self.assertNotIn("source_manifest", reservation["compatibility"]["build_config"])

        job = self.service.consume_cpu_reservation(
            reservation["reservationId"],
            session_id="session-a",
            lane_kind=CargoLaneKind.TEST,
        )
        changed = self.repo / "zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/owned_1274.rs"
        changed.write_text("pub const OWNED_1274: u16 = 65535;\n")

        with self.assertRaises(CoordinatorError) as stale:
            self.service.reserved_run_environment(
                reservation["reservationId"],
                session_id="session-a",
                job_id=job.job_id,
                command=command,
            )

        self.assertEqual("cargo_cpu_reservation_source_manifest_stale", stale.exception.code)

    def test_reservation_rejects_coordinator_json_output_flag_in_cargo_command(self) -> None:
        """Coordinator formatting flags must never reach Cargo's test runner."""
        for flag in ("-Json", "--json"):
            with self.subTest(flag=flag), self.assertRaises(CoordinatorError) as rejected:
                self.service.reserve_cpu(
                    "session-a",
                    compatibility=self.compatibility(),
                    command=(
                        "cargo",
                        "test",
                        "-p",
                        "zircon_runtime",
                        "--",
                        "--nocapture",
                        flag,
                    ),
                )

            self.assertEqual("cargo_command_contains_coordinator_flag", rejected.exception.code)

    def test_reserved_start_rechecks_coordinator_json_output_flag(self) -> None:
        """An older persisted reservation cannot start a malformed command."""
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.reserved_run_environment(
                "legacy-reservation",
                session_id="session-a",
                job_id="legacy-job",
                command=("cargo", "test", "--", "-Json"),
            )

        self.assertEqual("cargo_command_contains_coordinator_flag", rejected.exception.code)

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

    def test_legacy_finished_reconciliation_preserves_each_safety_predicate(self) -> None:
        cases = (
            ("executable-owner", SessionStatus.ACTIVE, CargoJobStatus.RELEASED, ()),
            ("job-not-released", SessionStatus.STALE, CargoJobStatus.SUCCEEDED, ()),
            ("recorded-process-live", SessionStatus.STALE, CargoJobStatus.RELEASED, (4242,)),
        )
        for name, owner_status, job_status, recorded_pids in cases:
            with self.subTest(name=name):
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET status='active' WHERE session_id='session-a'"
                    )
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

                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET status=? WHERE session_id='session-a'",
                        (owner_status.value,),
                    )
                    connection.execute(
                        """
                        UPDATE cargo_jobs
                        SET status=?, process_tree_live_pids_json=?,
                            released_at=CASE WHEN ?='released' THEN released_at ELSE NULL END
                        WHERE job_id=?
                        """,
                        (
                            job_status.value,
                            json.dumps(recorded_pids),
                            job_status.value,
                            job.job_id,
                        ),
                    )
                    connection.execute(
                        "UPDATE cargo_lane_reservations SET status='finished' WHERE reservation_id=?",
                        (reservation["reservationId"],),
                    )
                    reconcile_terminal_finished_cpu_reservations(
                        connection, now="2026-07-16T07:30:00+00:00"
                    )
                    row = connection.execute(
                        "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                        (reservation["reservationId"],),
                    ).fetchone()
                    self.assertEqual("finished", row["status"])
                    connection.execute(
                        "UPDATE cargo_lane_reservations SET status='released' WHERE reservation_id=?",
                        (reservation["reservationId"],),
                    )

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

        successor = self.service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
            command=("cargo", "check", "-p", "zircon_editor"),
        )
        self.assertEqual("pending", successor["status"])
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (reservation["reservationId"],),
            ).fetchone()
        self.assertEqual("running", row["status"])

    def test_cpu_reservation_queues_one_pending_successor_behind_running_job(self) -> None:
        running_compatibility = self.compatibility()
        successor_compatibility = self.compatibility(
            build_config="profile=dev;features=graphics"
        )
        running = self.service.reserve_cpu(
            "session-a",
            compatibility=running_compatibility,
            command=("cargo", "test", "-p", "zircon_runtime"),
        )
        running_job = self.service.acquire(
            "session-a", CargoLaneKind.TEST, compatibility=running_compatibility
        )
        self.service.start(
            running_job.job_id,
            session_id="session-a",
            pid=4242,
            command=["cargo", "test", "-p", "zircon_runtime"],
        )

        successor = self.service.reserve_cpu(
            "session-b",
            compatibility=successor_compatibility,
            command=("cargo", "check", "-p", "zircon_editor"),
        )
        self.assertEqual("pending", successor["status"])

        with self.assertRaises(CoordinatorError) as generic_overtake:
            self.service.acquire("session-c", CargoLaneKind.CHECK)
        self.assertEqual("cargo_cpu_lane_reserved", generic_overtake.exception.code)

        self.service.process_tree_pids = lambda _pid: ()
        self.service.finish(running_job.job_id, session_id="session-a", exit_code=0)
        self.service.release(running_job.job_id, session_id="session-a")

        successor_job = self.service.consume_cpu_reservation(
            successor["reservationId"],
            session_id="session-b",
            lane_kind=CargoLaneKind.CHECK,
        )
        self.assertEqual(CargoJobStatus.LEASED, successor_job.status)
        with self.database.connect() as connection:
            running_row = connection.execute(
                "SELECT status FROM cargo_lane_reservations WHERE reservation_id=?",
                (running["reservationId"],),
            ).fetchone()
        self.assertEqual("released", running_row["status"])

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

        successor = self.service.reserve_cpu(
            "session-b",
            compatibility=self.compatibility(build_config="profile=dev;features=graphics"),
            command=("cargo", "check", "-p", "zircon_editor"),
        )
        self.assertEqual("pending", successor["status"])
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
