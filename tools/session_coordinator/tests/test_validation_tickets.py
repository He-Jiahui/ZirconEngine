from __future__ import annotations

import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator import cli
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.tests.failure_fixture import FailureGraphFixture
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_ticket_worker import ValidationTicketWorker
from tools.session_coordinator.validation_tickets import ValidationTicketService


class _FakeWorkspaceCopy:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.records: dict[str, SimpleNamespace] = {}
        self.materializations: list[tuple[str, tuple[str, ...], tuple[str, ...]]] = []
        self.generic_materializations: list[
            tuple[str, tuple[str, ...], tuple[str, ...]]
        ] = []
        self.starts: list[tuple[str, str, tuple[str, ...], str]] = []
        self.run_results: dict[str, dict[str, object]] = {}

    def materialize_validation_async(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
    ) -> SimpleNamespace:
        job_id = f"copy-{len(self.records) + 1}"
        source_root = self.root / job_id / "source"
        record = SimpleNamespace(
            job_id=job_id,
            source_root=source_root,
            status="materializing",
            error_code=None,
            error_stage=None,
            error_path=None,
            error_details=None,
        )
        self.records[job_id] = record
        self.generic_materializations.append(
            (session_id, dependency_roots, overlay_paths)
        )
        return record

    def materialize_cargo_async(
        self,
        session_id: str,
        *,
        command: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        discover_external_sources: bool,
    ) -> SimpleNamespace:
        self.assert_external_discovery(discover_external_sources)
        job_id = f"copy-{len(self.records) + 1}"
        source_root = self.root / job_id / "source"
        record = SimpleNamespace(
            job_id=job_id,
            source_root=source_root,
            status="materializing",
            error_code=None,
            error_stage=None,
            error_path=None,
            error_details=None,
        )
        self.records[job_id] = record
        self.materializations.append((session_id, command, overlay_paths))
        return record

    @staticmethod
    def assert_external_discovery(discover_external_sources: bool) -> None:
        if not discover_external_sources:
            raise AssertionError("validation worker must pin discovered sibling sources")

    def status(self, session_id: str, job_id: str) -> SimpleNamespace:
        del session_id
        return self.records[job_id]

    def start(
        self,
        session_id: str,
        job_id: str,
        *,
        command: tuple[str, ...],
        run_id: str,
    ) -> dict[str, object]:
        self.starts.append((session_id, job_id, command, run_id))
        self.records[job_id].status = "running"
        return {"jobId": job_id, "runId": run_id, "status": "running"}

    def run_result(self, run_id: str) -> dict[str, object] | None:
        return self.run_results.get(run_id)


class ValidationTicketCliTransportTests(unittest.TestCase):
    @staticmethod
    def _submit_arguments(*manifest_arguments: str) -> list[str]:
        return [
            "validation",
            "submit",
            "--session-id",
            "primary",
            "--request-id",
            "transport-request",
            *manifest_arguments,
            "--command-json",
            '["python","-m","unittest","focused"]',
            "--toolchain-json",
            '{"python":"3.14"}',
            "--coverage-json",
            '{"kind":"focused"}',
        ]

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_inline_manifest_transport_preserves_the_existing_payload(
        self, from_runtime
    ) -> None:
        manifest = {"tools/session_coordinator/cli.py": "a" * 64}
        arguments = cli._parser().parse_args(
            self._submit_arguments(
                "--source-manifest-json",
                json.dumps(manifest),
            )
        )
        from_runtime.return_value.command.return_value = {"status": "queued"}

        cli._run(arguments)

        payload = from_runtime.return_value.command.call_args.args[1]
        self.assertEqual(manifest, payload["source_manifest"])

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_stdin_manifest_transport_accepts_more_than_32k_and_1849_tombstones(
        self, from_runtime
    ) -> None:
        manifest = {
            f"zircon_runtime/src/legacy/deleted_{index:04d}.rs": None
            for index in range(1849)
        }
        encoded = json.dumps(manifest, separators=(",", ":"))
        self.assertGreater(len(encoded.encode("utf-8")), 32767)
        arguments = cli._parser().parse_args(
            self._submit_arguments("--source-manifest-stdin")
        )
        from_runtime.return_value.command.return_value = {"status": "queued"}

        with mock.patch.object(cli.sys, "stdin", io.StringIO(encoded)):
            cli._run(arguments)

        payload = from_runtime.return_value.command.call_args.args[1]
        self.assertEqual(manifest, payload["source_manifest"])

    def test_manifest_transports_are_required_and_mutually_exclusive(self) -> None:
        with self.assertRaises(CoordinatorError) as missing:
            cli._parser().parse_args(self._submit_arguments())
        with self.assertRaises(CoordinatorError) as overlapping:
            cli._parser().parse_args(
                self._submit_arguments(
                    "--source-manifest-json",
                    "{}",
                    "--source-manifest-stdin",
                )
            )

        self.assertEqual("cli_arguments_invalid", missing.exception.code)
        self.assertEqual("cli_arguments_invalid", overlapping.exception.code)

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_stdin_manifest_transport_preserves_typed_json_errors(
        self, from_runtime
    ) -> None:
        arguments = cli._parser().parse_args(
            self._submit_arguments("--source-manifest-stdin")
        )

        for payload in ("{not-json}", "[]"):
            with self.subTest(payload=payload), mock.patch.object(
                cli.sys, "stdin", io.StringIO(payload)
            ), self.assertRaises(CoordinatorError) as rejected:
                cli._run(arguments)

            self.assertEqual("validation_ticket_json_invalid", rejected.exception.code)

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_validation_submit_rejects_nonstandard_json_constants(self, from_runtime) -> None:
        for field in ("toolchain_json", "coverage_json"):
            for constant in ("NaN", "Infinity", "-Infinity"):
                with self.subTest(field=field, constant=constant):
                    arguments = cli._parser().parse_args(
                        self._submit_arguments(
                            "--source-manifest-json",
                            '{"tools/session_coordinator/cli.py":"'
                            + "a" * 64
                            + '"}',
                        )
                    )
                    setattr(arguments, field, '{"value":' + constant + "}")
                    with self.assertRaises(CoordinatorError) as rejected:
                        cli._run(arguments)

                    self.assertEqual("validation_ticket_json_invalid", rejected.exception.code)
        from_runtime.return_value.command.assert_not_called()

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_validation_record_result_rejects_nonstandard_json_constants(self, from_runtime) -> None:
        for field in ("evidence_json", "failure_json"):
            for constant in ("NaN", "Infinity", "-Infinity"):
                with self.subTest(field=field, constant=constant):
                    arguments = cli._parser().parse_args(
                        [
                            "validation",
                            "record-result",
                            "--ticket-id",
                            "ticket-a",
                            "--status",
                            "failed",
                            "--evidence-json",
                            '{"exitCode":1}',
                            "--failure-json",
                            '{"summary":"compile failed"}',
                        ]
                    )
                    setattr(arguments, field, '{"value":' + constant + "}")

                    with self.assertRaises(CoordinatorError) as rejected:
                        cli._run(arguments)

                    self.assertEqual("validation_ticket_json_invalid", rejected.exception.code)
        from_runtime.return_value.command.assert_not_called()

    @mock.patch("tools.session_coordinator.cli.CoordinatorClient.from_runtime")
    def test_stdin_manifest_transport_accepts_windows_powershell_utf8_bom(
        self, from_runtime
    ) -> None:
        arguments = cli._parser().parse_args(
            self._submit_arguments("--source-manifest-stdin")
        )
        from_runtime.return_value.command.return_value = {"status": "queued"}

        with mock.patch.object(cli.sys, "stdin", io.StringIO("\ufeff{}")):
            cli._run(arguments)

        payload = from_runtime.return_value.command.call_args.args[1]
        self.assertEqual({}, payload["source_manifest"])

    def test_powershell_wrappers_forward_large_manifest_and_restore_encoding(
        self,
    ) -> None:
        shells = [
            (name, executable)
            for name, executable in (
                ("PowerShell 7", shutil.which("pwsh")),
                ("Windows PowerShell 5.1", shutil.which("powershell.exe")),
            )
            if executable is not None
        ]
        if not shells:
            self.skipTest("PowerShell is unavailable")

        manifest = {
            f"zircon_runtime/src/legacy/deleted_{index:04d}.rs": None
            for index in range(1849)
        }
        encoded = json.dumps(manifest, separators=(",", ":"))
        self.assertGreater(len(encoded.encode("utf-8")), 32767)
        wrapper = Path(__file__).resolve().parents[3] / "tools" / "zircon-session.ps1"
        invocation = r"""
$beforeOutputEncoding = $OutputEncoding.WebName
$beforePythonIoEncoding = $env:PYTHONIOENCODING
$payload = [IO.File]::ReadAllText(
    $env:ZIRCON_TEST_MANIFEST_PATH,
    [Text.Encoding]::UTF8
)
if ($env:ZIRCON_TEST_PREFIX_BOM -eq '1') {
    $payload = [char]0xFEFF + $payload
}
$result = $payload | & $env:ZIRCON_TEST_WRAPPER status --source-manifest-stdin -Json
$nativeExit = $LASTEXITCODE
if ($OutputEncoding.WebName -ne $beforeOutputEncoding) {
    throw 'OutputEncoding was not restored'
}
if ($env:PYTHONIOENCODING -ne $beforePythonIoEncoding) {
    throw 'PYTHONIOENCODING was not restored'
}
$result
exit $nativeExit
"""
        fake_module = """
import json
import os
import sys

source = sys.stdin.read()
manifest = json.loads(source.removeprefix("\\ufeff"))
print(json.dumps({
    "entryCount": len(manifest),
    "firstCodepoint": ord(source[0]),
    "pythonIoEncoding": os.environ.get("PYTHONIOENCODING"),
}, separators=(",", ":")))
"""

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = root / "manifest.json"
            manifest_path.write_text(encoded, encoding="utf-8")
            fake_module_path = root / "fake_python.py"
            fake_module_path.write_text(fake_module, encoding="utf-8")
            (root / "python.cmd").write_text(
                '@echo off\r\n"%ZIRCON_TEST_REAL_PYTHON%" '
                '"%ZIRCON_TEST_FAKE_PYTHON%" %*\r\n',
                encoding="ascii",
            )
            base_environment = {
                **os.environ,
                "PATH": f"{root}{os.pathsep}{os.environ['PATH']}",
                "ZIRCON_TEST_FAKE_PYTHON": str(fake_module_path),
                "ZIRCON_TEST_MANIFEST_PATH": str(manifest_path),
                "ZIRCON_TEST_REAL_PYTHON": sys.executable,
                "ZIRCON_TEST_WRAPPER": str(wrapper),
            }

            for shell_name, executable in shells:
                for original_python_encoding in (None, "cp1252"):
                    with self.subTest(
                        shell=shell_name,
                        original_python_encoding=original_python_encoding,
                    ):
                        environment = dict(base_environment)
                        environment["ZIRCON_TEST_PREFIX_BOM"] = (
                            "1" if shell_name == "PowerShell 7" else "0"
                        )
                        if original_python_encoding is None:
                            environment.pop("PYTHONIOENCODING", None)
                        else:
                            environment["PYTHONIOENCODING"] = (
                                original_python_encoding
                            )
                        completed = subprocess.run(
                            [
                                executable,
                                "-NoProfile",
                                "-ExecutionPolicy",
                                "Bypass",
                                "-Command",
                                invocation,
                            ],
                            cwd=wrapper.parents[1],
                            env=environment,
                            capture_output=True,
                            text=True,
                            encoding="utf-8",
                            timeout=30,
                            check=False,
                        )

                        self.assertEqual(0, completed.returncode, completed.stderr)
                        lines = [
                            line for line in completed.stdout.splitlines() if line.strip()
                        ]
                        self.assertEqual(1, len(lines), completed.stdout)
                        result = json.loads(lines[0])
                        self.assertEqual(1849, result["entryCount"])
                        self.assertIn(
                            result["firstCodepoint"],
                            (ord("{"), 0xFEFF),
                            "native PowerShell pipelines may prepend a UTF-8 BOM",
                        )
                        self.assertEqual("utf-8", result["pythonIoEncoding"])


class ValidationTicketTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.database = Database(Path(self.temporary.name) / "coordinator.sqlite3")
        self.repo = Path(self.temporary.name) / "repo"
        self.repo.mkdir()
        migrate(self.database)
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO sessions(
                    session_id, plan_path, status, created_at, updated_at, last_heartbeat_at
                ) VALUES (
                    'primary', 'docs/plans/tooling/01-tooling.md', 'active',
                    '2026-07-31T00:00:00+00:00', '2026-07-31T00:00:00+00:00',
                    '2026-07-31T00:00:00+00:00'
                )
                """
            )
        self.service = ValidationTicketService(self.database)
        self.workspace_copy = _FakeWorkspaceCopy(Path(self.temporary.name) / "copies")
        self.worker = ValidationTicketWorker(
            self.database,
            self.repo,
            self.service,
            self.workspace_copy,
            run_result_lookup=self.workspace_copy.run_result,
        )

    def test_submit_returns_a_durable_receipt_without_blocking_the_owner_session(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        replay = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )
        merged = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused", "scope": ["governance"]},
        )

        self.assertEqual("queued", first.ticket.status)
        self.assertFalse(first.reused)
        self.assertEqual(first, replay)
        self.assertEqual(first.ticket.ticket_id, merged.ticket.ticket_id)
        self.assertTrue(merged.reused)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
            request_count = connection.execute(
                "SELECT COUNT(*) FROM validation_ticket_requests"
            ).fetchone()[0]
        self.assertEqual("active", owner_status)
        self.assertEqual(2, request_count)

    def test_different_source_manifest_is_not_merged(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )
        second = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "b" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        self.assertNotEqual(first.ticket.ticket_id, second.ticket.ticket_id)
        self.assertFalse(second.reused)

    def test_submit_rejects_nonfinite_toolchain_and_coverage_values(self) -> None:
        for field in ("toolchain", "coverage"):
            for location in ("value", "key"):
                for index, value in enumerate(
                    (float("nan"), float("inf"), float("-inf"))
                ):
                    with self.subTest(field=field, location=location, value=value):
                        toolchain: dict[object, object] = {"python": "3.14"}
                        coverage: dict[object, object] = {"kind": "focused"}
                        target = toolchain if field == "toolchain" else coverage
                        if location == "value":
                            target["value"] = value
                        else:
                            target[value] = "nonfinite-key"

                        with self.assertRaises(CoordinatorError) as rejected:
                            self.service.submit(
                                session_id="primary",
                                request_id=f"nonfinite-{field}-{location}-{index}",
                                source_manifest={
                                    "tools/session_coordinator/governance.py": "a" * 64
                                },
                                command=("python", "-m", "unittest", "focused"),
                                toolchain=toolchain,
                                coverage=coverage,
                            )

                        self.assertEqual(
                            "validation_ticket_input_invalid", rejected.exception.code
                        )

        with self.database.connect() as connection:
            ticket_count = connection.execute(
                "SELECT COUNT(*) FROM validation_tickets"
            ).fetchone()[0]
            request_count = connection.execute(
                "SELECT COUNT(*) FROM validation_ticket_requests"
            ).fetchone()[0]
        self.assertEqual(0, ticket_count)
        self.assertEqual(0, request_count)

    def test_terminal_result_can_be_recorded_without_polling_for_running(self) -> None:
        receipt = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        completed = self.service.record_result(
            receipt.ticket.ticket_id,
            "failed",
            evidence={"exitCode": 1},
        )

        self.assertEqual("failed", completed.status)
        with self.database.connect() as connection:
            owner_status = connection.execute(
                "SELECT status FROM sessions WHERE session_id='primary'"
            ).fetchone()["status"]
        self.assertEqual("active", owner_status)

    def test_result_and_worker_events_reject_nonfinite_payloads_without_side_effects(self) -> None:
        receipt = self.service.submit(
            session_id="primary",
            request_id="reject-result-payload",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )
        ticket_id = receipt.ticket.ticket_id

        with self.assertRaises(CoordinatorError) as rejected_result:
            self.service.record_result(
                ticket_id,
                "passed",
                evidence={"nested": [{"value": float("nan")}]},
            )
        with self.assertRaises(CoordinatorError) as rejected_worker:
            self.service.record_worker_event(
                ticket_id,
                "validation.worker_progress",
                {"nested": {float("inf"): "invalid object key"}},
            )

        self.assertEqual("validation_ticket_input_invalid", rejected_result.exception.code)
        self.assertEqual("validation_ticket_input_invalid", rejected_worker.exception.code)
        self.assertEqual("queued", self.service.get(ticket_id).status)
        self.assertIsNone(self.service.latest_worker_event(ticket_id, "validation.worker_progress"))
        with self.database.connect() as connection:
            status_change_count = connection.execute(
                """SELECT COUNT(*) FROM validation_ticket_events
                   WHERE ticket_id=? AND event_type='validation.ticket_status_changed'""",
                (ticket_id,),
            ).fetchone()[0]
        self.assertEqual(0, status_change_count)

    def test_result_rejects_falsy_nonobject_evidence_without_status_side_effects(self) -> None:
        for index, evidence in enumerate(([], False, 0, "")):
            with self.subTest(evidence=evidence):
                receipt = self.service.submit(
                    session_id="primary",
                    request_id=f"reject-falsy-evidence-{index}",
                    source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
                    command=("python", "-m", "unittest", "focused"),
                    toolchain={"python": "3.14"},
                    coverage={"kind": "focused"},
                )
                ticket_id = receipt.ticket.ticket_id

                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.record_result(  # type: ignore[arg-type]
                        ticket_id,
                        "passed",
                        evidence=evidence,
                    )

                self.assertEqual("validation_ticket_input_invalid", rejected.exception.code)
                self.assertEqual("queued", self.service.get(ticket_id).status)
                with self.database.connect() as connection:
                    status_change_count = connection.execute(
                        """SELECT COUNT(*) FROM validation_ticket_events
                           WHERE ticket_id=? AND event_type='validation.ticket_status_changed'""",
                        (ticket_id,),
                    ).fetchone()[0]
                self.assertEqual(0, status_change_count)

    def test_worker_terminalizes_a_stale_snapshot_without_materializing_it(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        receipt = self.service.submit(
            session_id="primary",
            request_id="stale-request",
            source_manifest={"tools/owned.py": "a" * 64},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )
        self.assertEqual([], self.workspace_copy.materializations)

    def test_worker_projects_structured_copy_failure_details(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="compile-resource-error-details",
            source_manifest={"tools/owned.py": digest},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        expected = {
            "sourcePath": str(self.repo / "zircon_runtime/src/tests/host_adapter.rs"),
            "resourcePath": str(
                self.repo / "zircon_runtime/src/plugin/native_plugin_loader/tests.rs"
            ),
        }
        copy.status = "failed"
        copy.error_code = "validation_copy_compile_time_resource_missing"
        copy.error_stage = "closure_planning"
        copy.error_path = expected["resourcePath"]
        copy.error_details = expected

        self.assertEqual(1, self.worker.tick()["failed"])

        with self.database.connect() as connection:
            event = connection.execute(
                """
                SELECT payload_json FROM validation_ticket_events
                WHERE ticket_id=? AND event_type='validation.ticket_status_changed'
                ORDER BY event_id DESC LIMIT 1
                """,
                (receipt.ticket.ticket_id,),
            ).fetchone()
        evidence = json.loads(event[0])["evidence"]
        self.assertEqual(expected["resourcePath"], evidence["errorPath"])
        self.assertEqual(expected, evidence["errorDetails"])

    def test_worker_materializes_a_source_deletion_tombstone(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self.service.submit(
            session_id="primary",
            request_id="deleted-source-request",
            source_manifest={deleted_path: None},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )

        claimed = self.worker.tick()

        self.assertEqual(1, claimed["materializing"])
        self.assertEqual({deleted_path: None}, receipt.ticket.source_manifest)
        self.assertEqual(
            [("primary", receipt.ticket.command, (deleted_path,))],
            self.workspace_copy.materializations,
        )
        copy = next(iter(self.workspace_copy.records.values()))
        copy.source_root.mkdir(parents=True)
        copy.status = "materialized"

        started = self.worker.tick()

        self.assertEqual(1, started["running"])
        self.assertEqual("running", self.service.get(receipt.ticket.ticket_id).status)

    def test_worker_rejects_a_deleted_source_that_reappears_before_claim(self) -> None:
        deleted_path = "zircon_runtime/src/core/framework/error.rs"
        receipt = self.service.submit(
            session_id="primary",
            request_id="reappeared-source-request",
            source_manifest={deleted_path: None},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        source = self.repo / deleted_path
        source.parent.mkdir(parents=True)
        source.write_text("legacy shim\n", encoding="utf-8")

        result = self.worker.tick()

        self.assertEqual(1, result["snapshot_stale"])
        self.assertEqual(
            "snapshot_stale", self.service.get(receipt.ticket.ticket_id).status
        )
        self.assertEqual([], self.workspace_copy.materializations)

    def test_worker_consumes_an_exact_snapshot_through_copy_and_run_links(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="exact-request",
            source_manifest={"tools/owned.py": digest},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        ticket_id = receipt.ticket.ticket_id

        claimed = self.worker.tick()
        self.assertEqual(1, claimed["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copied_source = copy.source_root / "tools" / "owned.py"
        copied_source.parent.mkdir(parents=True)
        copied_source.write_bytes(source.read_bytes())
        copy.status = "materialized"

        started = self.worker.tick()
        self.assertEqual(1, started["running"])
        self.assertEqual("running", self.service.get(ticket_id).status)
        self.workspace_copy.run_results[ticket_id] = {
            "runId": ticket_id,
            "jobId": copy.job_id,
            "exitCode": 0,
            "stdout": "ok",
            "stderr": "",
        }

        completed = self.worker.tick()

        self.assertEqual(1, completed["passed"])
        self.assertEqual("passed", self.service.get(ticket_id).status)
        self.assertEqual(ticket_id, self.workspace_copy.starts[0][3])

    def test_worker_routes_non_cargo_commands_through_a_generic_workspace_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="python-request",
            source_manifest={"tools/owned.py": digest},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )

        result = self.worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.materializations)
        self.assertEqual(
            [
                (
                    "primary",
                    ("tools/session_coordinator",),
                    ("tools/owned.py",),
                )
            ],
            self.workspace_copy.generic_materializations,
        )
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)

    def test_worker_rejects_non_cargo_commands_without_dependency_roots(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="python-without-dependencies",
            source_manifest={"tools/owned.py": digest},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={"kind": "focused"},
        )

        result = self.worker.tick()

        self.assertEqual(1, result["failed"])
        self.assertEqual([], self.workspace_copy.generic_materializations)
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)
        with self.database.connect() as connection:
            event = connection.execute(
                """
                SELECT payload_json FROM validation_ticket_events
                WHERE ticket_id=? AND event_type='validation.ticket_status_changed'
                ORDER BY event_id DESC LIMIT 1
                """,
                (receipt.ticket.ticket_id,),
            ).fetchone()
        self.assertIn("validation_ticket_dependency_roots_missing", event[0])

    def test_worker_terminalizes_an_interrupted_claim_without_a_copy_link(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        receipt = self.service.submit(
            session_id="primary",
            request_id="interrupted-request",
            source_manifest={
                "tools/owned.py": hashlib.sha256(source.read_bytes()).hexdigest()
            },
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        self.service.claim_next()

        result = self.worker.tick()

        self.assertEqual(1, result["failed"])
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)

class ValidationTicketServerBoundaryTests(unittest.TestCase):
    def test_record_result_is_worker_only_before_payload_or_failure_processing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            FailureGraphFixture(repo).add_plan("docs/plans/tooling/01-tooling.md")
            config = CoordinatorConfig.for_repo(
                repo, state_root=root / "state", port=0
            )
            with RunningCoordinator.start(config) as running:
                application = running.httpd.application
                application.sessions.register(
                    session_id="primary",
                    plan_path="docs/plans/tooling/01-tooling.md",
                )
                receipt = application.validation_tickets.submit(
                    session_id="primary",
                    request_id="server-boundary-ticket",
                    source_manifest={"tools/session_coordinator/cli.py": "a" * 64},
                    command=("python", "-m", "unittest", "focused"),
                    toolchain={"python": "3.14"},
                    coverage={"kind": "focused"},
                )

                with self.assertRaises(CoordinatorError) as rejected_evidence:
                    application.command(
                        "validation.record_result",
                        {
                            "ticket_id": receipt.ticket.ticket_id,
                            "status": "passed",
                            "evidence": [],
                        },
                    )
                with self.assertRaises(CoordinatorError) as rejected_failure:
                    application.command(
                        "validation.record_result",
                        {
                            "ticket_id": receipt.ticket.ticket_id,
                            "status": "failed",
                            "evidence": {},
                            "failure": {"created_at": float("nan")},
                        },
                    )
                with mock.patch.object(
                    application.failures,
                    "materialize_local_validation_failure",
                    wraps=application.failures.materialize_local_validation_failure,
                ) as materialize:
                    with self.assertRaises(CoordinatorError) as rejected_failed_evidence:
                        application.command(
                            "validation.record_result",
                            {
                                "ticket_id": receipt.ticket.ticket_id,
                                "status": "failed",
                                "evidence": {"nested": [float("nan")]},
                                "failure": {
                                    "created_at": "2026-08-03",
                                    "summary_slug": "server-evidence-rejection",
                                    "source_slice": "server boundary regression",
                                    "reproduction": "python -m unittest focused",
                                    "lowest_known_cause": "invalid evidence payload",
                                    "acceptance_criteria": [
                                        "The invalid evidence must be rejected before materialization."
                                    ],
                                    "related_code": ["tools/session_coordinator/server.py"],
                                },
                            },
                        )

                ticket = application.validation_tickets.get(receipt.ticket.ticket_id)

        self.assertEqual("validation_ticket_result_worker_only", rejected_evidence.exception.code)
        self.assertEqual("validation_ticket_result_worker_only", rejected_failure.exception.code)
        self.assertEqual(
            "validation_ticket_result_worker_only", rejected_failed_evidence.exception.code
        )
        materialize.assert_not_called()
        self.assertEqual("queued", ticket.status)


if __name__ == "__main__":
    unittest.main()
