from __future__ import annotations

import json
import unittest
from contextlib import contextmanager
from pathlib import Path

from tools.session_coordinator.cargo_run_registration import (
    SpawnObservation,
    persist_cleanup_unproven_spawn,
)


SCRIPT = (
    Path(__file__).resolve().parents[1]
    / "session_coordinator"
    / "cargo_run_registration.py"
)


class CargoRunCommandSerializationPerformanceContractTests(unittest.TestCase):
    def test_cleanup_unproven_registration_serializes_command_once(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")
        function = source[source.index("def persist_cleanup_unproven_spawn(") :]

        self.assertIn("command_json = json.dumps(tuple(command))", function)
        self.assertEqual(1, function.count("json.dumps(tuple(command))"))
        self.assertGreaterEqual(function.count("command_json,"), 2)

    def test_reuses_identical_command_json_for_job_and_run_rows(self) -> None:
        class Cursor:
            def __init__(self, row: object = None) -> None:
                self.row = row

            def fetchone(self) -> object:
                return self.row

        class Connection:
            def __init__(self) -> None:
                self.calls: list[tuple[str, tuple[object, ...]]] = []

            def execute(
                self, statement: str, parameters: tuple[object, ...] = ()
            ) -> Cursor:
                self.calls.append((statement, parameters))
                if "SELECT * FROM cargo_jobs" in statement:
                    return Cursor(
                        {
                            "session_id": "session-a",
                            "status": "leased",
                            "pid": None,
                        }
                    )
                if "SELECT run_id FROM cargo_job_runs" in statement:
                    return Cursor(None)
                return Cursor()

        class Database:
            def __init__(self) -> None:
                self.connection = Connection()

            @contextmanager
            def transaction(self):
                yield self.connection

        database = Database()
        command = ("cargo", "test", "--package", "zircon_runtime")

        persist_cleanup_unproven_spawn(
            database,  # type: ignore[arg-type]
            run_id="run-a",
            job_id="job-a",
            session_id="session-a",
            command=command,
            environment={"CARGO_TARGET_DIR": "E:/cargo-targets/test"},
            stdout_path=Path("stdout.log"),
            stderr_path=Path("stderr.log"),
            started_at="2026-08-31T00:00:00+00:00",
            observation=SpawnObservation(
                pid=42,
                creation_time="2026-08-31T00:00:00+00:00",
                root_kind="cargo",
                live_pids=(42,),
            ),
            rejection_code="rejected",
        )

        update_parameters = next(
            parameters
            for statement, parameters in database.connection.calls
            if "UPDATE cargo_jobs" in statement
        )
        insert_parameters = next(
            parameters
            for statement, parameters in database.connection.calls
            if "INSERT INTO cargo_job_runs" in statement
        )
        self.assertEqual(update_parameters[3], insert_parameters[3])
        self.assertEqual(list(command), json.loads(str(update_parameters[3])))


if __name__ == "__main__":
    unittest.main()
