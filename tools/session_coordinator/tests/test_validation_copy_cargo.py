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
        cargo_jobs.acquire.side_effect = CoordinatorError(
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
