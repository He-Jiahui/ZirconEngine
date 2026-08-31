from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.validation_copy_cargo import (
    ValidationCopyCargoExecution,
)


class ValidationCopyCargoSourceManifestTests(unittest.TestCase):
    def test_loads_source_manifest_from_exact_durable_ticket(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            database = Database(Path(temporary) / "coordinator.sqlite3")
            migrate(database)
            manifest = {"Cargo.toml": "e" * 64}
            with database.transaction() as connection:
                connection.execute(
                    "INSERT INTO sessions(session_id, plan_path, status, created_at, "
                    "updated_at, last_heartbeat_at) VALUES (?, ?, ?, ?, ?, ?)",
                    (
                        "session-a",
                        "docs/plans/tooling/06.md",
                        "active",
                        "2026-08-25T00:00:00+00:00",
                        "2026-08-25T00:00:00+00:00",
                        "2026-08-25T00:00:00+00:00",
                    ),
                )
                connection.execute(
                    "INSERT INTO validation_tickets(ticket_id, session_id, plan_path, "
                    "status, dedupe_key, source_manifest_hash, source_manifest_json, "
                    "command_json, toolchain_json, coverage_json, created_at, updated_at) "
                    "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    (
                        "validation-a",
                        "session-a",
                        "docs/plans/tooling/06.md",
                        "running",
                        "dedupe-a",
                        "f" * 64,
                        json.dumps(manifest),
                        json.dumps(["cargo", "--version"]),
                        "{}",
                        "{}",
                        "2026-08-25T00:00:00+00:00",
                        "2026-08-25T00:00:00+00:00",
                    ),
                )
            execution = ValidationCopyCargoExecution(
                database,
                mock.Mock(),
                mock.Mock(),
                reservation_lookup=lambda _session_id, _copy_job_id: None,
            )

            loaded = execution._validation_source_manifest(
                "session-a", "validation-a"
            )

        self.assertEqual(manifest, loaded)

    def test_binds_ticket_source_manifest_to_copy_reservation(self) -> None:
        source_manifest = {"Cargo.toml": "c" * 64}
        cargo_jobs = mock.Mock()
        cargo_jobs.reserve_cpu.return_value = {
            "reservationId": "reservation-a",
            "status": "pending",
            "jobId": None,
        }
        cargo_jobs.consume_cpu_reservation.side_effect = CoordinatorError(
            "cargo_cpu_reservation_not_fifo_head", "Another reservation is first"
        )
        execution = ValidationCopyCargoExecution(
            mock.Mock(),
            cargo_jobs,
            mock.Mock(),
            reservation_lookup=lambda _session_id, _copy_job_id: None,
            source_manifest_lookup=lambda _session_id, _run_id: source_manifest,
        )

        result = execution.advance(
            session_id="session-a",
            copy_job_id="copy-a",
            source_root=Path("copy-a/source"),
            input_manifest_hash="a" * 64,
            command=("pwsh.exe", "-Command", "cargo --version"),
            validation_run_id="validation-a",
        )

        self.assertEqual("waiting", result["status"])
        compatibility = cargo_jobs.reserve_cpu.call_args.kwargs["compatibility"]
        self.assertEqual(source_manifest, compatibility.source_manifest)
        self.assertEqual("copy-a", compatibility.source_copy_job_id)
        self.assertEqual("a" * 64, compatibility.source_copy_manifest_hash)
        self.assertNotIn("burst_eligible", cargo_jobs.reserve_cpu.call_args.kwargs)
        cargo_jobs.consume_cpu_reservation.assert_called_once_with(
            "reservation-a",
            session_id="session-a",
            lane_kind=mock.ANY,
        )

    def test_resource_denied_burst_keeps_the_validation_waiting(self) -> None:
        cargo_jobs = mock.Mock()
        cargo_jobs.reserve_cpu.return_value = {
            "reservationId": "reservation-a",
            "status": "pending",
            "jobId": None,
        }
        cargo_jobs.consume_cpu_reservation.side_effect = CoordinatorError(
            "cargo_cpu_burst_resource_denied",
            "The warm lane is occupied and burst headroom is unavailable",
        )
        execution = ValidationCopyCargoExecution(
            mock.Mock(),
            cargo_jobs,
            mock.Mock(),
            reservation_lookup=lambda _session_id, _copy_job_id: None,
            source_manifest_lookup=lambda _session_id, _run_id: {
                "Cargo.toml": "b" * 64
            },
        )

        result = execution.advance(
            session_id="session-a",
            copy_job_id="copy-a",
            source_root=Path("copy-a/source"),
            input_manifest_hash="a" * 64,
            command=("cargo", "test", "-p", "zircon_editor"),
            validation_run_id="validation-a",
        )

        self.assertEqual("waiting", result["status"])
        self.assertEqual("reservation-a", result["cargoReservationId"])
        self.assertEqual(
            "cargo_cpu_burst_resource_denied", result["blockerCode"]
        )

    def test_distinct_copy_pending_reservation_keeps_validation_waiting(self) -> None:
        cargo_jobs = mock.Mock()
        cargo_jobs.reserve_cpu.side_effect = CoordinatorError(
            "cargo_cpu_session_reservation_pending",
            "Session already has a pending exact CPU reservation",
            details={"reservationId": "reservation-first-copy"},
        )
        execution = ValidationCopyCargoExecution(
            mock.Mock(),
            cargo_jobs,
            mock.Mock(),
            reservation_lookup=lambda _session_id, _copy_job_id: None,
            source_manifest_lookup=lambda _session_id, _run_id: {
                "Cargo.toml": "b" * 64
            },
        )

        result = execution.advance(
            session_id="session-a",
            copy_job_id="copy-second",
            source_root=Path("copy-second/source"),
            input_manifest_hash="a" * 64,
            command=("cargo", "test", "-p", "zircon_runtime"),
            validation_run_id="validation-second",
        )

        self.assertEqual("waiting", result["status"])
        self.assertEqual("reservation-first-copy", result["cargoReservationId"])
        self.assertEqual(
            "cargo_cpu_session_reservation_pending", result["blockerCode"]
        )
        cargo_jobs.consume_cpu_reservation.assert_not_called()

    def test_rollover_pending_keeps_an_already_leased_validation_waiting(self) -> None:
        cargo_jobs = mock.Mock()
        cargo_jobs.get.return_value.status = "leased"
        cargo_runner = mock.Mock()
        cargo_runner.start.side_effect = CoordinatorError(
            "cargo_start_rollover_pending",
            "The current daemon has committed a rollover handoff",
        )
        execution = ValidationCopyCargoExecution(
            mock.Mock(),
            cargo_jobs,
            cargo_runner,
            reservation_lookup=lambda _session_id, _copy_job_id: {
                "reservationId": "reservation-a",
                "status": "leased",
                "jobId": "job-a",
            },
            source_manifest_lookup=lambda _session_id, _run_id: {
                "Cargo.toml": "b" * 64
            },
        )

        result = execution.advance(
            session_id="session-a",
            copy_job_id="copy-a",
            source_root=Path("copy-a/source"),
            input_manifest_hash="a" * 64,
            command=("cargo", "check"),
            validation_run_id="validation-a",
        )

        self.assertEqual("waiting", result["status"])
        self.assertEqual("reservation-a", result["cargoReservationId"])
        self.assertEqual("job-a", result["cargoJobId"])
        self.assertEqual("cargo_start_rollover_pending", result["blockerCode"])


    def test_compatibility_pool_is_stable_across_validation_ticket_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source_root = Path(temporary)
            first = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-a",
                source_root=source_root,
                input_manifest_hash="a" * 64,
                source_manifest={"Cargo.toml": "b" * 64},
                command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
                validation_run_id="validation-a",
            )
            second = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-b",
                source_root=source_root,
                input_manifest_hash="c" * 64,
                source_manifest={"Cargo.toml": "d" * 64},
                command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
                validation_run_id="validation-b",
            )

        self.assertEqual(first.build_config, second.build_config)
        self.assertNotIn("validation-a", first.build_config)
        self.assertNotIn("validation-b", second.build_config)

    def test_compatibility_separates_explicit_and_workspace_toolchains(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            first_root = root / "first"
            second_root = root / "second"
            first_root.mkdir()
            second_root.mkdir()
            (first_root / "rust-toolchain.toml").write_text(
                "[toolchain]\nchannel='1.94.1'\n", encoding="utf-8"
            )
            (second_root / "rust-toolchain.toml").write_text(
                "[toolchain]\nchannel='1.95.0'\n", encoding="utf-8"
            )

            explicit = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-a",
                source_root=first_root,
                input_manifest_hash="a" * 64,
                source_manifest={"Cargo.toml": "b" * 64},
                command=("cargo", "+1.94.1", "test"),
                validation_run_id="validation-a",
            )
            other_explicit = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-b",
                source_root=first_root,
                input_manifest_hash="b" * 64,
                source_manifest={"Cargo.toml": "b" * 64},
                command=("cargo", "+1.95.0", "test"),
                validation_run_id="validation-b",
            )
            workspace_default = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-c",
                source_root=first_root,
                input_manifest_hash="c" * 64,
                source_manifest={"Cargo.toml": "b" * 64},
                command=("cargo", "test"),
                validation_run_id="validation-c",
            )
            other_workspace_default = ValidationCopyCargoExecution._compatibility(
                copy_job_id="copy-d",
                source_root=second_root,
                input_manifest_hash="d" * 64,
                source_manifest={"Cargo.toml": "b" * 64},
                command=("cargo", "test"),
                validation_run_id="validation-d",
            )

        self.assertNotEqual(explicit.toolchain, other_explicit.toolchain)
        self.assertNotEqual(explicit.toolchain, workspace_default.toolchain)
        self.assertNotEqual(
            workspace_default.toolchain, other_workspace_default.toolchain
        )

    def test_rejects_missing_ticket_source_manifest_before_reservation(self) -> None:
        cargo_jobs = mock.Mock()
        execution = ValidationCopyCargoExecution(
            mock.Mock(),
            cargo_jobs,
            mock.Mock(),
            reservation_lookup=lambda _session_id, _copy_job_id: None,
            source_manifest_lookup=lambda _session_id, _run_id: None,
        )

        with self.assertRaises(CoordinatorError) as rejected:
            execution.advance(
                session_id="session-a",
                copy_job_id="copy-a",
                source_root=Path("copy-a/source"),
                input_manifest_hash="b" * 64,
                command=("pwsh.exe", "-Command", "cargo --version"),
                validation_run_id="validation-a",
            )

        self.assertEqual(
            "validation_copy_cargo_source_manifest_missing",
            rejected.exception.code,
        )
        cargo_jobs.reserve_cpu.assert_not_called()


if __name__ == "__main__":
    unittest.main()
