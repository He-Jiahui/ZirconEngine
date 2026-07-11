from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from datetime import UTC, datetime, timedelta
from pathlib import Path
from unittest.mock import patch

from tools.session_coordinator.cargo_jobs import (
    CargoCompatibility,
    CargoJobService,
    CargoJobStatus,
    CargoLaneKind,
    TargetPathPolicy,
    target_identity,
)
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo


class CargoJobTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/cargo-targets"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        SessionService(self.database, self.repo).register(session_id="session-b")
        self.policy = TargetPathPolicy([self.target_root])
        self.service = CargoJobService(
            self.database,
            self.policy,
            repo_root=self.repo,
            free_space=lambda _path: 200 * 1024**3,
            process_alive=lambda pid: pid == 4242,
        )

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

    def test_allocated_lane_is_unique_and_under_allowlisted_cargo_target_root(self) -> None:
        first = self.service.acquire("session-a", CargoLaneKind.CHECK)
        second = self.service.acquire("session-a", CargoLaneKind.CHECK)

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertTrue(Path(first.target_dir).is_relative_to(self.target_root))
        self.assertTrue(Path(first.target_dir).is_dir())
        self.assertEqual(CargoJobStatus.LEASED, first.status)
        self.assertNotIn(str(self.repo / "target"), first.target_dir)

    def test_explicit_target_outside_allowlist_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.TEST,
                requested_target=self.repo / "target/manual",
            )
        self.assertEqual("cargo_target_not_managed", rejected.exception.code)

    def test_active_explicit_target_cannot_have_two_writers(self) -> None:
        requested = self.target_root / "shared-check"
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        with self.assertRaises(CoordinatorError) as occupied:
            self.service.acquire(
                "session-a", CargoLaneKind.TEST, requested_target=requested
            )

        self.assertEqual(CargoJobStatus.LEASED, first.status)
        self.assertEqual("cargo_lane_occupied", occupied.exception.code)

    def test_target_identity_is_case_and_separator_insensitive(self) -> None:
        self.assertEqual(
            target_identity(r"E:\cargo-targets\Check-A"),
            target_identity("e:/CARGO-TARGETS/check-a"),
        )

    def test_nested_lane_below_cargo_targets_is_accepted(self) -> None:
        requested = self.target_root / "zircon-shared/check"

        job = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, requested_target=requested
        )

        self.assertEqual(requested.resolve(), Path(job.target_dir))

    def test_legacy_targets_zircon_engine_root_is_rejected(self) -> None:
        legacy_root = self.target_root.parent / "targets/zircon-engine"
        legacy_root.mkdir(parents=True)

        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([legacy_root])

        self.assertEqual("invalid_target_root", rejected.exception.code)

    def test_all_three_allowlisted_root_names_are_accepted(self) -> None:
        roots = []
        for name in ("cargo-targets", "targets", "ZirconBuilds"):
            root = self.target_root.parent / name
            root.mkdir(parents=True, exist_ok=True)
            roots.append(root)

        policy = TargetPathPolicy(roots)

        self.assertEqual(
            tuple(root.resolve() for root in roots),
            policy.roots,
        )
        for root in roots:
            self.assertEqual(
                (root / "pool/example").resolve(),
                policy.validate(root / "pool/example"),
            )

    def test_production_config_exposes_three_root_names_per_available_drive(self) -> None:
        config = CoordinatorConfig.for_repo(self.repo)
        with patch("pathlib.Path.exists", return_value=True):
            roots = config.enabled_target_roots

        self.assertEqual(9, len(roots))
        self.assertEqual(
            ["cargo-targets", "targets", "zirconbuilds"] * 3,
            [root.name.casefold() for root in roots],
        )

    def test_arbitrary_root_name_is_rejected(self) -> None:
        arbitrary = self.target_root.parent / "other-builds"
        arbitrary.mkdir()

        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([arbitrary])

        self.assertEqual("invalid_target_root", rejected.exception.code)

    def test_no_configured_target_drive_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            TargetPathPolicy([])
        self.assertEqual("target_root_unavailable", rejected.exception.code)

    def test_symlink_lane_escape_is_rejected_when_supported(self) -> None:
        outside = self.target_root.parent / "outside"
        outside.mkdir()
        link = self.target_root / "escaped-link"
        link.parent.mkdir(parents=True, exist_ok=True)
        try:
            link.symlink_to(outside, target_is_directory=True)
        except OSError as error:
            if os.name != "nt":
                self.skipTest(f"directory symlink is unavailable: {error}")
            junction = subprocess.run(
                ["cmd.exe", "/d", "/c", "mklink", "/J", str(link), str(outside)],
                capture_output=True,
                text=True,
                check=False,
            )
            if junction.returncode != 0:
                self.skipTest(
                    f"directory symlink and junction are unavailable: {junction.stderr}"
                )

        with self.assertRaises(CoordinatorError) as rejected:
            self.policy.validate(link)
        self.assertEqual("cargo_target_not_managed", rejected.exception.code)

    def test_released_explicit_compatible_lane_can_be_reused(self) -> None:
        requested = self.target_root / "manual-check"
        first = self.service.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            requested_target=requested,
            compatibility=self.compatibility(),
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            requested_target=requested,
            compatibility=self.compatibility(),
        )

        self.assertNotEqual(first.job_id, second.job_id)
        self.assertEqual(first.target_dir, second.target_dir)
        self.assertTrue(requested.is_dir())

    def test_released_compatible_lane_is_reused_across_sessions(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertEqual(first.target_dir, second.target_dir)
        self.assertEqual(first.job_id, second.reused_from_job_id)
        self.assertEqual(first.reuse_key, second.reuse_key)

    def test_source_and_lock_changes_reuse_cargo_fingerprint_pool(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        (self.repo / "Cargo.lock").write_text("changed", encoding="utf-8")

        second = self.service.acquire(
            "session-b", CargoLaneKind.CHECK, compatibility=compatibility
        )

        self.assertEqual(first.reuse_key, second.reuse_key)
        self.assertEqual(first.target_dir, second.target_dir)

    def test_missing_primary_is_retired_before_deterministic_replacement(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        Path(first.target_dir).rmdir()

        second = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        self.assertEqual("deleted", self.service.get(first.job_id).cleanup_status.value)
        self.assertEqual(first.reuse_key, second.reuse_key)
        self.assertEqual(first.target_dir, second.target_dir)

    def test_duplicate_retained_pool_is_demoted_to_immediate_cleanup(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )
        self.service.release(first.job_id, session_id="session-a")
        newer_target = self.target_root / "newer-primary"
        newer_target.mkdir()
        newer_job_id = "newer-retained-pool"
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO cargo_jobs(
                    job_id, session_id, lane_kind, target_dir, status, dry_run,
                    created_at, last_heartbeat_at, released_at, target_key,
                    reuse_key, compatibility_json, compatibility_key, reuse_profile,
                    cleanup_policy, cleanup_status
                )
                SELECT ?, session_id, lane_kind, ?, 'released', dry_run,
                       created_at, last_heartbeat_at, ?, ?, reuse_key,
                       compatibility_json, compatibility_key, reuse_profile,
                       'retained', 'retained'
                FROM cargo_jobs WHERE job_id=?
                """,
                (
                    newer_job_id,
                    str(newer_target),
                    (datetime.now(UTC) + timedelta(minutes=1)).isoformat(),
                    target_identity(newer_target),
                    first.job_id,
                ),
            )

        acquired = self.service.acquire(
            "session-b", CargoLaneKind.TEST, compatibility=compatibility
        )

        retired = self.service.get(first.job_id)
        self.assertEqual(str(newer_target.resolve()), acquired.target_dir)
        self.assertEqual(newer_job_id, acquired.reused_from_job_id)
        self.assertEqual("delete_on_release", retired.cleanup_policy.value)
        self.assertEqual("pending", retired.cleanup_status.value)

    def test_incompatible_build_configuration_gets_a_distinct_pool(self) -> None:
        first = self.service.acquire(
            "session-a",
            CargoLaneKind.TEST,
            compatibility=self.compatibility(build_config="profile=test;features=default"),
        )
        self.service.release(first.job_id, session_id="session-a")

        second = self.service.acquire(
            "session-b",
            CargoLaneKind.TEST,
            compatibility=self.compatibility(build_config="profile=dev;features=default"),
        )

        self.assertNotEqual(first.reuse_key, second.reuse_key)
        self.assertNotEqual(first.target_dir, second.target_dir)
        self.assertEqual("retained", self.service.get(first.job_id).cleanup_policy.value)

    def test_windows_and_wsl_never_share_a_pool(self) -> None:
        windows = self.service.acquire(
            "session-a",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(),
        )
        self.service.release(windows.job_id, session_id="session-a")

        wsl = self.service.acquire(
            "session-b",
            CargoLaneKind.CHECK,
            compatibility=self.compatibility(
                platform="wsl",
                toolchain="stable-x86_64-unknown-linux-gnu",
                target_architecture="x86_64-unknown-linux-gnu",
            ),
        )

        self.assertNotEqual(windows.reuse_key, wsl.reuse_key)
        self.assertNotEqual(windows.target_dir, wsl.target_dir)

    def test_compatible_pool_is_busy_instead_of_allocating_a_fallback(self) -> None:
        compatibility = self.compatibility()
        first = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, compatibility=compatibility
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-b", CargoLaneKind.TEST, compatibility=compatibility
            )

        self.assertEqual("cargo_reuse_pool_busy", rejected.exception.code)
        compatible_jobs = [
            job for job in self.service.list() if job.reuse_key == first.reuse_key
        ]
        self.assertEqual([first.target_dir], [job.target_dir for job in compatible_jobs])

    def test_incomplete_compatibility_document_is_rejected(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                compatibility=CargoCompatibility(
                    platform="windows",
                    toolchain="",
                    target_architecture="x86_64-pc-windows-msvc",
                    workspace="Cargo.toml",
                    build_config="profile=dev",
                ),
            )

        self.assertEqual("invalid_cargo_compatibility", rejected.exception.code)

    def test_missing_compatibility_is_ephemeral_by_default(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        self.assertIsNone(job.reuse_key)
        self.assertEqual("delete_on_release", job.cleanup_policy.value)
        self.assertEqual("pending", job.cleanup_status.value)

    def test_ephemeral_and_compatibility_are_mutually_exclusive(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.acquire(
                "session-a",
                CargoLaneKind.CHECK,
                ephemeral=True,
                compatibility=self.compatibility(),
            )

        self.assertEqual("cargo_compatibility_conflict", rejected.exception.code)

    def test_ephemeral_lane_is_marked_for_release_cleanup(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        released = self.service.release(job.job_id, session_id="session-a")

        self.assertEqual("delete_on_release", released.cleanup_policy.value)
        self.assertEqual("pending", released.cleanup_status.value)
        self.assertIsNone(released.reuse_key)

    def test_foreign_session_cannot_mutate_job(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.CHECK)

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.start(
                job.job_id,
                session_id="session-b",
                pid=4242,
                command=["cargo", "check"],
            )
        self.assertEqual("cargo_job_owner_mismatch", rejected.exception.code)

    def test_dead_prestart_owner_is_reconciled_after_leased_timeout(self) -> None:
        job = self.service.acquire(
            "session-a", CargoLaneKind.CHECK, owner_pid=9999
        )

        orphaned = self.service.reconcile_orphans(
            now=datetime.now(UTC) + timedelta(minutes=10),
            leased_timeout_seconds=300,
        )

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, self.service.get(job.job_id).status)

    def test_running_finish_and_release_preserve_job_audit(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.WORKSPACE)
        running = self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )
        finished = self.service.finish(job.job_id, session_id="session-a", exit_code=0)
        released = self.service.release(job.job_id, session_id="session-a")

        self.assertEqual(CargoJobStatus.RUNNING, running.status)
        self.assertEqual(CargoJobStatus.SUCCEEDED, finished.status)
        self.assertEqual(CargoJobStatus.RELEASED, released.status)
        self.assertEqual(0, released.exit_code)
        self.assertEqual(("cargo", "test"), released.command)

    def test_dry_run_allocates_without_creating_target(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.GPU, dry_run=True)

        self.assertFalse(Path(job.target_dir).exists())
        self.assertTrue(job.dry_run)

    def test_reconcile_marks_dead_running_process_as_orphaned(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=9999, command=["cargo", "test"]
        )

        orphaned = self.service.reconcile_orphans()

        self.assertEqual((job.job_id,), tuple(item.job_id for item in orphaned))
        self.assertEqual(CargoJobStatus.ORPHANED, self.service.get(job.job_id).status)

    def test_reconcile_keeps_live_running_process(self) -> None:
        job = self.service.acquire("session-a", CargoLaneKind.TEST)
        self.service.start(
            job.job_id, session_id="session-a", pid=4242, command=["cargo", "test"]
        )

        self.assertEqual((), self.service.reconcile_orphans())
        self.assertEqual(CargoJobStatus.RUNNING, self.service.get(job.job_id).status)


if __name__ == "__main__":
    unittest.main()
