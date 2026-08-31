from __future__ import annotations

import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from collections.abc import Mapping
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator import cli
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.server import RunningCoordinator
from tools.session_coordinator.snapshots import ObjectStore
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
        self.ownership_checks: list[tuple[str, tuple[str, ...]]] = []
        self.cleanup_calls: list[tuple[str, str]] = []
        self.unowned_overlays: set[str] = set()
        self.sealed_manifests: list[dict[str, str | None]] = []
        self.baseline_commits: list[str | None] = []
        self.start_status = "running"

    def require_overlay_ownership(
        self, session_id: str, overlay_paths: tuple[str, ...]
    ) -> tuple[str, ...]:
        normalized = tuple(path.replace("\\", "/") for path in overlay_paths)
        self.ownership_checks.append((session_id, normalized))
        unowned = sorted(
            path for path in normalized if path.casefold() in self.unowned_overlays
        )
        if unowned:
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Validation overlay paths require current Session attribution",
                details={"paths": unowned},
            )
        return normalized

    def materialize_validation_async(
        self,
        session_id: str,
        *,
        dependency_roots: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> SimpleNamespace:
        job_id = f"copy-{len(self.records) + 1}"
        source_root = self.root / job_id / "source"
        record = SimpleNamespace(
            job_id=job_id,
            source_root=source_root,
            status="materializing",
            materialization_kind=None,
            error_code=None,
            error_stage=None,
            error_path=None,
            error_details=None,
        )
        self.records[job_id] = record
        self.generic_materializations.append(
            (session_id, dependency_roots, overlay_paths)
        )
        if sealed_overlay_manifest is not None:
            self.sealed_manifests.append(dict(sealed_overlay_manifest))
        self.baseline_commits.append(baseline_commit)
        return record

    def materialize_cargo_async(
        self,
        session_id: str,
        *,
        command: tuple[str, ...],
        overlay_paths: tuple[str, ...],
        discover_external_sources: bool,
        external_sources: tuple[Mapping[str, object], ...] = (),
        sealed_overlay_manifest: Mapping[str, str | None] | None = None,
        baseline_commit: str | None = None,
    ) -> SimpleNamespace:
        del discover_external_sources, external_sources
        job_id = f"copy-{len(self.records) + 1}"
        source_root = self.root / job_id / "source"
        record = SimpleNamespace(
            job_id=job_id,
            source_root=source_root,
            status="materializing",
            materialization_kind="cargo",
            error_code=None,
            error_stage=None,
            error_path=None,
            error_details=None,
        )
        self.records[job_id] = record
        self.materializations.append((session_id, command, overlay_paths))
        if sealed_overlay_manifest is not None:
            self.sealed_manifests.append(dict(sealed_overlay_manifest))
        self.baseline_commits.append(baseline_commit)
        return record

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
        if self.start_status == "running":
            self.records[job_id].status = "running"
        return {"jobId": job_id, "runId": run_id, "status": self.start_status}

    def run_result(self, run_id: str) -> dict[str, object] | None:
        return self.run_results.get(run_id)

    def cleanup_terminal_ticket_copy(self, ticket_id: str, job_id: str) -> bool:
        self.cleanup_calls.append((ticket_id, job_id))
        return True


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

    def test_powershell_wrappers_forward_large_manifest_from_terminal_pipeline(
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
$payload = [IO.File]::ReadAllText(
    $env:ZIRCON_TEST_MANIFEST_PATH,
    [Text.Encoding]::UTF8
)
if ($env:ZIRCON_TEST_PREFIX_BOM -eq '1') {
    $payload = [char]0xFEFF + $payload
}
$payload | & $env:ZIRCON_TEST_WRAPPER status --source-manifest-stdin -Json
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
            connection.execute(
                "UPDATE sessions SET base_head=? WHERE session_id='primary'",
                ("a" * 40,),
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

    def _sealed_service(self) -> tuple[ValidationTicketService, ObjectStore]:
        objects = ObjectStore(
            self.database, Path(self.temporary.name) / "validation-source-objects"
        )
        return (
            ValidationTicketService(
                self.database,
                repo_root=self.repo,
                object_store=objects,
            ),
            objects,
        )

    def test_submit_seals_source_bytes_before_queued_work_can_drift(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        original = b"value = 'submitted'\n"
        source.write_bytes(original)
        digest = hashlib.sha256(original).hexdigest()
        with self.database.transaction() as connection:
            cursor = connection.execute(
                """
                INSERT INTO baseline_epochs(
                    head_commit, index_tree, health, manifest_json, created_at
                ) VALUES (?, 'tree', 'healthy', '{}', '2026-07-31T00:00:00+00:00')
                """,
                ("a" * 40,),
            )
            baseline_epoch = int(cursor.lastrowid)
            connection.execute(
                "UPDATE sessions SET baseline_epoch=? WHERE session_id='primary'",
                (baseline_epoch,),
            )
        service, objects = self._sealed_service()
        arguments = {
            "session_id": "primary",
            "request_id": "sealed-source-request",
            "source_manifest": {"tools/owned.py": digest},
            "command": ("cargo", "check", "-p", "zircon_runtime", "--locked"),
            "toolchain": {"rust": "1.94.1"},
            "coverage": {"kind": "compile"},
        }

        receipt = service.submit(**arguments)
        source.write_bytes(b"value = 'new task'\n")
        replay = service.submit(**arguments)

        self.assertEqual(receipt, replay)
        self.assertTrue(service.source_is_sealed(receipt.ticket.ticket_id))
        self.assertEqual(original, objects.get(digest))
        with self.database.connect() as connection:
            pin = connection.execute(
                "SELECT baseline_epoch, manifest_json FROM snapshots WHERE purpose=?",
                (f"validation-ticket-source:{receipt.ticket.ticket_id}",),
            ).fetchone()
        self.assertEqual(baseline_epoch, pin["baseline_epoch"])
        self.assertEqual({"tools/owned.py": digest}, json.loads(pin["manifest_json"]))

    def test_submit_binds_session_baseline_and_does_not_reuse_across_baselines(
        self,
    ) -> None:
        arguments = {
            "source_manifest": {"tools/owned.py": "a" * 64},
            "command": ("cargo", "check", "-p", "zircon_runtime", "--locked"),
            "toolchain": {"rust": "1.94.1"},
            "coverage": {"kind": "compile"},
        }
        first = self.service.submit(
            session_id="primary", request_id="baseline-a", **arguments
        )
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE sessions SET base_head=? WHERE session_id='primary'",
                ("b" * 40,),
            )
        second = self.service.submit(
            session_id="primary", request_id="baseline-b", **arguments
        )

        self.assertEqual("a" * 40, first.ticket.base_head)
        self.assertEqual("b" * 40, second.ticket.base_head)
        self.assertNotEqual(first.ticket.ticket_id, second.ticket.ticket_id)
        self.assertFalse(second.reused)
        self.assertEqual("a" * 40, self.service.get(first.ticket.ticket_id).base_head)

    def test_submit_rejects_a_manifest_that_no_longer_matches_source_bytes(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        service, _objects = self._sealed_service()

        with self.assertRaises(CoordinatorError) as rejected:
            service.submit(
                session_id="primary",
                request_id="stale-at-submit",
                source_manifest={"tools/owned.py": "a" * 64},
                command=("cargo", "check", "-p", "zircon_runtime"),
                toolchain={"rust": "1.94.1"},
                coverage={"kind": "compile"},
            )

        self.assertEqual(
            "validation_ticket_source_snapshot_stale", rejected.exception.code
        )
        self.assertEqual({"path": "tools/owned.py"}, rejected.exception.details)
        with self.database.connect() as connection:
            counts = tuple(
                connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                for table in (
                    "validation_tickets",
                    "validation_ticket_requests",
                    "snapshots",
                    "objects",
                )
            )
        self.assertEqual((0, 0, 0, 0), counts)

    def test_submit_hashes_every_source_before_writing_any_object(self) -> None:
        first = self.repo / "tools" / "first.py"
        second = self.repo / "tools" / "second.py"
        first.parent.mkdir(parents=True)
        first.write_text("first = True\n", encoding="utf-8")
        second.write_text("second = True\n", encoding="utf-8")
        first_hash = hashlib.sha256(first.read_bytes()).hexdigest()
        service, objects = self._sealed_service()

        with self.assertRaises(CoordinatorError) as rejected:
            service.submit(
                session_id="primary",
                request_id="partially-stale-at-submit",
                source_manifest={
                    "tools/first.py": first_hash,
                    "tools/second.py": "a" * 64,
                },
                command=("cargo", "check", "-p", "zircon_runtime"),
                toolchain={"rust": "1.94.1"},
                coverage={"kind": "compile"},
            )

        self.assertEqual(
            "validation_ticket_source_snapshot_stale", rejected.exception.code
        )
        with self.database.connect() as connection:
            self.assertEqual(0, connection.execute("SELECT COUNT(*) FROM objects").fetchone()[0])
        self.assertEqual([], [path for path in objects.root.rglob("*") if path.is_file()])

    def test_submit_rejects_paths_that_cannot_enter_a_windows_copy(self) -> None:
        invalid_manifests = (
            {"tools/file.py:stream": "a" * 64},
            {"tools/\0file.py": "a" * 64},
            {"tools\\owned.py": "a" * 64, "tools/owned.py": "a" * 64},
            {"Tools/owned.py": "a" * 64, "tools/owned.py": "a" * 64},
            {"CON": None},
            {"tools/aux.rs": "a" * 64},
            {"tools/file.": "a" * 64},
            {"tools/file ": "a" * 64},
            {".git/config": "a" * 64},
            {"target/output.txt": "a" * 64},
            {".codex/state/private.json": "a" * 64},
        )

        for index, manifest in enumerate(invalid_manifests):
            with self.subTest(index=index, paths=tuple(manifest)):
                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.submit(
                        session_id="primary",
                        request_id=f"unsafe-path-{index}",
                        source_manifest=manifest,
                        command=("cargo", "check", "-p", "zircon_runtime"),
                        toolchain={"rust": "1.94.1"},
                        coverage={"kind": "compile"},
                    )
                self.assertEqual(
                    "validation_ticket_manifest_invalid", rejected.exception.code
                )

        with self.database.connect() as connection:
            self.assertEqual(
                0,
                connection.execute("SELECT COUNT(*) FROM validation_tickets").fetchone()[0],
            )

    def test_submit_rejects_cargo_output_and_compiler_overrides(self) -> None:
        commands = (
            ("cargo", "test", "--target-dir", "D:/unmanaged"),
            ("cargo", "test", "--target-dir=D:/unmanaged"),
            ("cargo", "test", "--config", "build.target-dir='D:/unmanaged'"),
            ("cargo", "test", "--config=build.rustc-wrapper='custom.exe'"),
        )

        for index, command in enumerate(commands):
            with self.subTest(command=command):
                with self.assertRaises(CoordinatorError) as rejected:
                    self.service.submit(
                        session_id="primary",
                        request_id=f"cargo-output-override-{index}",
                        source_manifest={"tools/owned.py": "a" * 64},
                        command=command,
                        toolchain={"rust": "1.94.1"},
                        coverage={"kind": "compile"},
                    )
                self.assertEqual(
                    "validation_ticket_cargo_storage_override",
                    rejected.exception.code,
                )

    def test_submit_rejects_opaque_cargo_wrapper(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="primary",
                request_id="opaque-cargo-wrapper",
                source_manifest={"tools/owned.py": "a" * 64},
                command=("pwsh.exe", "-Command", "cargo test --target-dir D:/unmanaged"),
                toolchain={"cargo": "1.94.1"},
                coverage={"kind": "compile"},
            )

        self.assertEqual(
            "validation_ticket_cargo_command_opaque", rejected.exception.code
        )

    def test_submit_removes_new_object_files_when_publication_fails_mid_batch(self) -> None:
        first = self.repo / "tools/first.py"
        second = self.repo / "tools/second.py"
        first.parent.mkdir(parents=True)
        first.write_text("first = True\n", encoding="utf-8")
        second.write_text("second = True\n", encoding="utf-8")
        service, objects = self._sealed_service()
        original_put = objects.put
        calls = 0

        def fail_second_put(content, *, connection=None):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected publication failure")
            return original_put(content, connection=connection)

        with mock.patch.object(objects, "put", side_effect=fail_second_put):
            with self.assertRaises(OSError):
                service.submit(
                    session_id="primary",
                    request_id="object-publication-failure",
                    source_manifest={
                        "tools/first.py": hashlib.sha256(first.read_bytes()).hexdigest(),
                        "tools/second.py": hashlib.sha256(second.read_bytes()).hexdigest(),
                    },
                    command=("cargo", "+1.94.1", "check", "-p", "zircon_runtime"),
                    toolchain={"rust": "1.94.1"},
                    coverage={"kind": "compile"},
                )

        with self.database.connect() as connection:
            self.assertEqual(0, connection.execute("SELECT COUNT(*) FROM objects").fetchone()[0])
            self.assertEqual(0, connection.execute("SELECT COUNT(*) FROM validation_tickets").fetchone()[0])
        self.assertEqual([], [path for path in objects.root.rglob("*") if path.is_file()])

    def test_submit_does_not_hold_the_writer_lock_while_reading_source_bytes(self) -> None:
        source = self.repo / "tools" / "slow.py"
        source.parent.mkdir(parents=True)
        source.write_text("slow = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        service, _objects = self._sealed_service()
        read_started = threading.Event()
        allow_read = threading.Event()
        mutation_finished = threading.Event()
        errors: list[BaseException] = []
        original_read_bytes = Path.read_bytes

        def delayed_read(path: Path) -> bytes:
            if path == source:
                read_started.set()
                if not allow_read.wait(5):
                    raise TimeoutError("test did not release source read")
            return original_read_bytes(path)

        def submit() -> None:
            try:
                service.submit(
                    session_id="primary",
                    request_id="slow-source-submit",
                    source_manifest={"tools/slow.py": digest},
                    command=("cargo", "check", "-p", "zircon_runtime"),
                    toolchain={"rust": "1.94.1"},
                    coverage={"kind": "compile"},
                )
            except BaseException as error:
                errors.append(error)

        def mutate_database() -> None:
            try:
                with self.database.transaction() as connection:
                    connection.execute(
                        "INSERT INTO events(event_type, payload_json, created_at) "
                        "VALUES ('test.concurrent_write', '{}', datetime('now'))"
                    )
            except BaseException as error:
                errors.append(error)
            finally:
                mutation_finished.set()

        with mock.patch.object(Path, "read_bytes", delayed_read):
            submit_worker = threading.Thread(target=submit)
            submit_worker.start()
            self.assertTrue(read_started.wait(2))
            mutation_worker = threading.Thread(target=mutate_database)
            mutation_worker.start()
            try:
                self.assertTrue(
                    mutation_finished.wait(0.5),
                    "source reads must not occupy the coordinator writer transaction",
                )
            finally:
                allow_read.set()
                submit_worker.join(5)
                mutation_worker.join(5)

        self.assertFalse(submit_worker.is_alive())
        self.assertFalse(mutation_worker.is_alive())
        self.assertEqual([], errors)

    def test_submit_rejects_session_baseline_change_during_source_sealing(self) -> None:
        source = self.repo / "tools" / "baseline-race.py"
        source.parent.mkdir(parents=True)
        source.write_text("value = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        service, objects = self._sealed_service()
        original_read_bytes = Path.read_bytes

        def advance_baseline(path: Path) -> bytes:
            if path == source:
                with self.database.transaction() as connection:
                    connection.execute(
                        "UPDATE sessions SET base_head=? WHERE session_id='primary'",
                        ("b" * 40,),
                    )
            return original_read_bytes(path)

        with mock.patch.object(Path, "read_bytes", advance_baseline):
            with self.assertRaises(CoordinatorError) as rejected:
                service.submit(
                    session_id="primary",
                    request_id="baseline-changed-during-seal",
                    source_manifest={"tools/baseline-race.py": digest},
                    command=("cargo", "check", "-p", "zircon_runtime"),
                    toolchain={"rust": "1.94.1"},
                    coverage={"kind": "compile"},
                )

        self.assertEqual("validation_ticket_baseline_changed", rejected.exception.code)
        with self.database.connect() as connection:
            counts = tuple(
                connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                for table in (
                    "validation_tickets",
                    "validation_ticket_requests",
                    "snapshots",
                    "objects",
                )
            )
        self.assertEqual((0, 0, 0, 0), counts)
        self.assertEqual([], [path for path in objects.root.rglob("*") if path.is_file()])

    def test_worker_uses_sealed_source_after_live_ownership_and_bytes_change(self) -> None:
        relative_path = "tools/owned.py"
        source = self.repo / relative_path
        source.parent.mkdir(parents=True)
        source.write_text("submitted = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        service, _objects = self._sealed_service()
        receipt = service.submit(
            session_id="primary",
            request_id="sealed-worker-request",
            source_manifest={relative_path: digest},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        source.write_text("edited_by_another_task = True\n", encoding="utf-8")
        self.workspace_copy.unowned_overlays.add(relative_path.casefold())
        worker = ValidationTicketWorker(
            self.database,
            self.repo,
            service,
            self.workspace_copy,
            run_result_lookup=self.workspace_copy.run_result,
        )

        result = worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.ownership_checks)
        self.assertEqual(
            [{relative_path: digest}], self.workspace_copy.sealed_manifests
        )
        self.assertEqual(["a" * 40], self.workspace_copy.baseline_commits)
        service.record_result(receipt.ticket.ticket_id, "failed")
        with self.database.connect() as connection:
            pin_count = connection.execute(
                "SELECT COUNT(*) FROM snapshots WHERE purpose=?",
                (f"validation-ticket-source:{receipt.ticket.ticket_id}",),
            ).fetchone()[0]
        self.assertEqual(0, pin_count)
        self.assertFalse(service.source_is_sealed(receipt.ticket.ticket_id))

    def test_sealed_deletion_remains_a_tombstone_if_live_source_reappears(self) -> None:
        relative_path = "zircon_runtime/src/core/framework/error.rs"
        service, _objects = self._sealed_service()
        receipt = service.submit(
            session_id="primary",
            request_id="sealed-deletion-request",
            source_manifest={relative_path: None},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        source = self.repo / relative_path
        source.parent.mkdir(parents=True)
        source.write_text("reappeared in another task\n", encoding="utf-8")
        worker = ValidationTicketWorker(
            self.database,
            self.repo,
            service,
            self.workspace_copy,
            run_result_lookup=self.workspace_copy.run_result,
        )

        result = worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.ownership_checks)
        self.assertEqual(
            [{relative_path: None}], self.workspace_copy.sealed_manifests
        )

    def test_submit_returns_a_durable_receipt_without_blocking_the_owner_session(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "scope": ["governance"],
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        replay = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "scope": ["governance"],
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        merged = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "scope": ["governance"],
                "dependencyRoots": ["tools/session_coordinator"],
            },
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

    def test_submit_preflight_rejects_unowned_overlay_without_persisting(self) -> None:
        def reject_unowned(
            session_id: str, overlay_paths: tuple[str, ...]
        ) -> tuple[str, ...]:
            self.assertEqual("primary", session_id)
            self.assertEqual(
                ("tools/session_coordinator/governance.py",), overlay_paths
            )
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Validation overlay paths require current Session attribution",
                details={"paths": list(overlay_paths)},
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="primary",
                request_id="unowned-submit",
                source_manifest={
                    "tools/session_coordinator/governance.py": "a" * 64
                },
                command=("cargo", "check", "-p", "zircon_runtime"),
                toolchain={"rust": "1.94.1"},
                coverage={"kind": "compile"},
                overlay_ownership_preflight=reject_unowned,
            )

        self.assertEqual("validation_copy_overlay_not_owned", rejected.exception.code)
        self.assertEqual(
            {"paths": ["tools/session_coordinator/governance.py"]},
            rejected.exception.details,
        )
        with self.database.connect() as connection:
            self.assertEqual(
                0,
                connection.execute("SELECT COUNT(*) FROM validation_tickets").fetchone()[0],
            )
            self.assertEqual(
                0,
                connection.execute(
                    "SELECT COUNT(*) FROM validation_ticket_requests"
                ).fetchone()[0],
            )

    def test_submit_replays_persisted_request_before_ownership_preflight(self) -> None:
        ownership_checks = 0
        ownership_current = True

        def require_current_ownership(
            _session_id: str, overlay_paths: tuple[str, ...]
        ) -> tuple[str, ...]:
            nonlocal ownership_checks
            ownership_checks += 1
            if not ownership_current:
                raise CoordinatorError(
                    "validation_copy_overlay_not_owned",
                    "Validation overlay paths require current Session attribution",
                    details={"paths": list(overlay_paths)},
                )
            return overlay_paths

        arguments = {
            "session_id": "primary",
            "request_id": "persisted-before-lease-expiry",
            "source_manifest": {
                "tools/session_coordinator/governance.py": "a" * 64
            },
            "command": ("cargo", "check", "-p", "zircon_runtime", "--locked"),
            "toolchain": {"rust": "1.94.1"},
            "coverage": {"kind": "compile"},
            "overlay_ownership_preflight": require_current_ownership,
        }
        first = self.service.submit(**arguments)
        ownership_current = False

        replay = self.service.submit(**arguments)

        self.assertEqual(first, replay)
        self.assertEqual(1, ownership_checks)
        with self.database.connect() as connection:
            self.assertEqual(
                1,
                connection.execute("SELECT COUNT(*) FROM validation_tickets").fetchone()[0],
            )
            self.assertEqual(
                1,
                connection.execute(
                    "SELECT COUNT(*) FROM validation_ticket_requests"
                ).fetchone()[0],
            )

    def test_submit_replays_persisted_request_before_new_body_policy(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="persisted-before-policy-upgrade",
            source_manifest={
                "tools/session_coordinator/governance.py": "a" * 64
            },
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )

        replay = self.service.submit(
            session_id="different-replay-body",
            request_id="persisted-before-policy-upgrade",
            source_manifest={},
            command=("cargo", "test"),
            toolchain={},
            coverage={},
        )

        self.assertEqual(first, replay)

    def test_submit_preflight_rejects_non_cargo_without_dependency_roots(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="primary",
                request_id="python-without-submit-dependencies",
                source_manifest={
                    "tools/session_coordinator/governance.py": "a" * 64
                },
                command=("python", "-m", "unittest", "focused"),
                toolchain={"python": "3.14"},
                coverage={"kind": "focused"},
                overlay_ownership_preflight=lambda _session, paths: paths,
            )

        self.assertEqual(
            "validation_ticket_dependency_roots_missing", rejected.exception.code
        )
        with self.database.connect() as connection:
            self.assertEqual(
                (0, 0),
                (
                    connection.execute(
                        "SELECT COUNT(*) FROM validation_tickets"
                    ).fetchone()[0],
                    connection.execute(
                        "SELECT COUNT(*) FROM validation_ticket_requests"
                    ).fetchone()[0],
                ),
            )

    def test_submit_rejects_non_cargo_without_dependency_roots_without_callback(
        self,
    ) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="primary",
                request_id="python-without-submit-dependencies-or-callback",
                source_manifest={
                    "tools/session_coordinator/governance.py": "a" * 64
                },
                command=("python", "-m", "unittest", "focused"),
                toolchain={"python": "3.14"},
                coverage={"kind": "focused"},
            )

        self.assertEqual(
            "validation_ticket_dependency_roots_missing", rejected.exception.code
        )
        with self.database.connect() as connection:
            counts = (
                connection.execute(
                    "SELECT COUNT(*) FROM validation_tickets"
                ).fetchone()[0],
                connection.execute(
                    "SELECT COUNT(*) FROM validation_ticket_requests"
                ).fetchone()[0],
            )
        self.assertEqual((0, 0), counts)

    def test_submit_preserves_unknown_session_error_before_overlay_preflight(
        self,
    ) -> None:
        ownership_checks: list[tuple[str, tuple[str, ...]]] = []

        def reject_overlay(session_id: str, paths: tuple[str, ...]) -> None:
            ownership_checks.append((session_id, paths))
            raise CoordinatorError(
                "validation_copy_overlay_not_owned",
                "Validation overlay paths require current Session attribution",
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="unknown-session",
                request_id="unknown-session-submit",
                source_manifest={
                    "tools/session_coordinator/governance.py": "a" * 64
                },
                command=("python", "-m", "unittest", "focused"),
                toolchain={"python": "3.14"},
                coverage={
                    "kind": "focused",
                    "dependencyRoots": ["tools/session_coordinator"],
                },
                overlay_ownership_preflight=reject_overlay,
            )

        self.assertEqual("session_not_found", rejected.exception.code)
        self.assertEqual([], ownership_checks)
        with self.database.connect() as connection:
            counts = (
                connection.execute(
                    "SELECT COUNT(*) FROM validation_tickets"
                ).fetchone()[0],
                connection.execute(
                    "SELECT COUNT(*) FROM validation_ticket_requests"
                ).fetchone()[0],
            )
        self.assertEqual((0, 0), counts)

    def test_submit_preflight_accepts_both_dependency_root_spellings(self) -> None:
        for index, key in enumerate(("dependencyRoots", "dependency_roots")):
            with self.subTest(key=key):
                receipt = self.service.submit(
                    session_id="primary",
                    request_id=f"python-submit-dependencies-{index}",
                    source_manifest={
                        f"tools/session_coordinator/owned-{index}.py": "a" * 64
                    },
                    command=("python", "-m", "unittest", "focused"),
                    toolchain={"python": "3.14"},
                    coverage={"kind": "focused", key: ["tools/session_coordinator"]},
                    overlay_ownership_preflight=lambda _session, paths: paths,
                )

                self.assertEqual("queued", receipt.ticket.status)

    def test_submit_preflight_allows_cargo_without_dependency_roots(self) -> None:
        receipt = self.service.submit(
            session_id="primary",
            request_id="cargo-submit-without-dependencies",
            source_manifest={
                "tools/session_coordinator/governance.py": "a" * 64
            },
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
            overlay_ownership_preflight=lambda _session, paths: paths,
        )

        self.assertEqual("queued", receipt.ticket.status)

    def test_different_source_manifest_is_not_merged(self) -> None:
        first = self.service.submit(
            session_id="primary",
            request_id="request-a",
            source_manifest={"tools/session_coordinator/governance.py": "a" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        second = self.service.submit(
            session_id="primary",
            request_id="request-b",
            source_manifest={"tools/session_coordinator/governance.py": "b" * 64},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
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
                        coverage: dict[object, object] = {
                            "kind": "focused",
                            "dependencyRoots": ["tools/session_coordinator"],
                        }
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
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
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
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
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
                    coverage={
                        "kind": "focused",
                        "dependencyRoots": ["tools/session_coordinator"],
                    },
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

    def test_worker_rejects_unowned_overlay_at_claim_before_materialization(self) -> None:
        relative_path = "tools/unowned.py"
        source = self.repo / relative_path
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        self.workspace_copy.unowned_overlays.add(relative_path.casefold())
        receipt = self.service.submit(
            session_id="primary",
            request_id="unowned-claim-request",
            source_manifest={relative_path: digest},
            command=("cargo", "check", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )

        result = self.worker.tick()

        self.assertEqual(1, result["failed"])
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)
        self.assertEqual([("primary", (relative_path,))], self.workspace_copy.ownership_checks)
        self.assertEqual([], self.workspace_copy.materializations)
        with self.database.connect() as connection:
            event = connection.execute(
                """SELECT payload_json FROM validation_ticket_events
                   WHERE ticket_id=? AND event_type='validation.ticket_status_changed'
                   ORDER BY event_id DESC LIMIT 1""",
                (receipt.ticket.ticket_id,),
            ).fetchone()
        evidence = json.loads(event["payload_json"])["evidence"]
        self.assertEqual("queue_claim", evidence["phase"])
        self.assertEqual("validation_copy_overlay_not_owned", evidence["errorCode"])
        self.assertEqual({"paths": [relative_path]}, evidence["errorDetails"])

    def test_worker_runs_two_independent_snapshots_and_backfills_the_fifo_slot(self) -> None:
        sources: dict[str, str] = {}
        ticket_ids: list[str] = []
        for index in range(3):
            relative_path = f"tools/owned-{index}.py"
            source = self.repo / relative_path
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(f"current = {index}\n", encoding="utf-8")
            sources[relative_path] = hashlib.sha256(source.read_bytes()).hexdigest()
            receipt = self.service.submit(
                session_id="primary",
                request_id=f"parallel-request-{index}",
                source_manifest={relative_path: sources[relative_path]},
                command=("cargo", "check", "-p", f"zircon_parallel_{index}"),
                toolchain={"rust": "1.94.1"},
                coverage={"kind": "compile"},
            )
            ticket_ids.append(receipt.ticket.ticket_id)

        claimed = self.worker.tick()

        self.assertEqual(2, claimed["materializing"])
        self.assertEqual(2, len(self.workspace_copy.materializations))
        self.assertEqual(
            ["materializing", "materializing", "queued"],
            [self.service.get(ticket_id).status for ticket_id in ticket_ids],
        )
        self.assertEqual(ticket_ids[:2], [
            ticket.ticket_id for ticket in self.service.active_tickets()
        ])

        self.workspace_copy.records["copy-1"].status = "failed"
        backfilled = self.worker.tick()

        self.assertEqual(1, backfilled["failed"])
        self.assertEqual(2, backfilled["materializing"])
        self.assertEqual(3, len(self.workspace_copy.materializations))
        self.assertEqual(
            ["failed", "materializing", "materializing"],
            [self.service.get(ticket_id).status for ticket_id in ticket_ids],
        )

    def test_worker_rejects_a_nonpositive_parallel_slot_limit(self) -> None:
        with self.assertRaises(ValueError):
            ValidationTicketWorker(
                self.database,
                self.repo,
                self.service,
                self.workspace_copy,
                max_active=0,
            )

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
        self.assertEqual([(ticket_id, copy.job_id)], self.workspace_copy.cleanup_calls)

    def test_worker_keeps_a_materialized_cargo_copy_waiting_until_its_run_starts(
        self,
    ) -> None:
        source = self.repo / "tools" / "waiting.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="waiting-reservation-request",
            source_manifest={"tools/waiting.py": digest},
            command=("cargo", "test", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copied_source = copy.source_root / "tools" / "waiting.py"
        copied_source.parent.mkdir(parents=True)
        copied_source.write_bytes(source.read_bytes())
        copy.status = "materialized"
        self.workspace_copy.start_status = "waiting"

        waiting = self.worker.tick()

        self.assertEqual(1, waiting["materializing"])
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)
        self.assertIsNone(
            self.service.latest_worker_event(
                receipt.ticket.ticket_id, "validation.ticket_run_linked"
            )
        )

    def test_worker_preserves_legacy_running_projection_while_restart_waits(
        self,
    ) -> None:
        source = self.repo / "tools" / "legacy-waiting.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="legacy-waiting-reservation-request",
            source_manifest={"tools/legacy-waiting.py": digest},
            command=("cargo", "test", "-p", "zircon_runtime"),
            toolchain={"rust": "1.94.1"},
            coverage={"kind": "compile"},
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copy.status = "materialized"
        ticket_id = receipt.ticket.ticket_id
        self.service.record_worker_event(
            ticket_id,
            "validation.ticket_run_linked",
            {"jobId": copy.job_id, "runId": ticket_id},
        )
        self.service.transition(
            ticket_id,
            "running",
            evidence={"jobId": copy.job_id, "runId": ticket_id},
        )
        self.workspace_copy.start_status = "waiting"

        recovered = self.worker.tick()

        self.assertEqual(1, recovered["running"])
        self.assertEqual("running", self.service.get(ticket_id).status)
        self.assertEqual(1, len(self.workspace_copy.starts))

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

    def test_worker_accepts_snake_case_dependency_roots(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="python-snake-case-roots",
            source_manifest={"tools/owned.py": digest},
            command=("python", "-m", "unittest", "focused"),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependency_roots": ["tools/session_coordinator"],
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

    def test_worker_routes_cargo_toolchain_wrappers_through_a_cargo_workspace_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="powershell-cargo-request",
            source_manifest={"tools/owned.py": digest},
            command=(
                "pwsh.exe",
                "-NoProfile",
                "-Command",
                "& cargo +1.94.1 test -p zircon_runtime --lib",
            ),
            toolchain={"cargo": "1.94.1", "rustc": "1.94.1"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )

        result = self.worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.generic_materializations)
        self.assertEqual(
            [("primary", receipt.ticket.command, ("tools/owned.py",))],
            self.workspace_copy.materializations,
        )
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)

    def test_worker_routes_declared_cargo_jobs_wrapper_through_cargo_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="powershell-cargo-jobs-wrapper",
            source_manifest={"tools/owned.py": digest},
            command=(
                "pwsh.exe",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                ".codex/state/session-coordinator/cargo-runs/validation.ps1",
            ),
            toolchain={"cargo_jobs": 1, "rust": "1.94.1"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )

        result = self.worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.generic_materializations)
        self.assertEqual(
            [("primary", receipt.ticket.command, ("tools/owned.py",))],
            self.workspace_copy.materializations,
        )
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)

    def test_worker_does_not_route_a_not_required_cargo_marker_to_cargo(self) -> None:
        source = self.repo / "docs" / "owned.md"
        source.parent.mkdir(parents=True)
        source.write_text("current\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="git-diff-with-cargo-not-required",
            source_manifest={"docs/owned.md": digest},
            command=("git", "diff", "--check", "--", "docs/owned.md"),
            toolchain={"cargo": "not_required", "rust": "not_required"},
            coverage={"kind": "focused", "dependencyRoots": ["docs"]},
        )

        result = self.worker.tick()

        self.assertEqual(1, result["materializing"])
        self.assertEqual([], self.workspace_copy.materializations)
        self.assertEqual(
            [("primary", ("docs",), ("docs/owned.md",))],
            self.workspace_copy.generic_materializations,
        )
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)

    def test_worker_restarts_a_removed_pre_fix_wrapper_as_a_cargo_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="removed-powershell-cargo-request",
            source_manifest={"tools/owned.py": digest},
            command=(
                "pwsh.exe",
                "-NoProfile",
                "-Command",
                "& cargo +1.94.1 test -p zircon_runtime --lib",
            ),
            toolchain={"cargo": "1.94.1", "rustc": "1.94.1"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copy.status = "removed"
        copy.materialization_kind = None

        restarted = self.worker.tick()

        self.assertEqual(1, restarted["materializing"])
        self.assertEqual("materializing", self.service.get(receipt.ticket.ticket_id).status)
        self.assertEqual([], self.workspace_copy.generic_materializations)
        self.assertEqual(2, len(self.workspace_copy.materializations))
        link = self.service.latest_worker_event(
            receipt.ticket.ticket_id, "validation.ticket_copy_linked"
        )
        self.assertEqual(copy.job_id, link["recoveredFromJobId"])

    def test_worker_does_not_rematerialize_a_removed_cargo_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="removed-cargo-request",
            source_manifest={"tools/owned.py": digest},
            command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            toolchain={"cargo": "1.94.1", "rustc": "1.94.1"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        copy.status = "removed"

        terminal = self.worker.tick()

        self.assertEqual(1, terminal["failed"])
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)
        self.assertEqual(1, len(self.workspace_copy.materializations))
        self.assertEqual([], self.workspace_copy.generic_materializations)

    def test_worker_preserves_failure_evidence_from_a_removed_cargo_copy(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="removed-cargo-failure-evidence",
            source_manifest={"tools/owned.py": digest},
            command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            toolchain={"cargo": "1.94.1", "rustc": "1.94.1"},
            coverage={"kind": "focused"},
        )
        self.assertEqual(1, self.worker.tick()["materializing"])
        copy = next(iter(self.workspace_copy.records.values()))
        expected = {
            "sourcePath": str(self.repo / "zircon_runtime/src/tests/host_adapter.rs"),
            "resourcePath": str(
                self.repo / "zircon_runtime_interface/src/runtime_api/host_requests.rs"
            ),
        }
        copy.status = "removed"
        copy.materialization_phase = "failed"
        copy.error_code = "validation_copy_compile_time_resource_missing"
        copy.error_stage = "closure_planning"
        copy.error_path = expected["resourcePath"]
        copy.error_details = expected

        terminal = self.worker.tick()

        self.assertEqual(1, terminal["failed"])
        self.assertEqual("failed", self.service.get(receipt.ticket.ticket_id).status)
        self.assertEqual(1, len(self.workspace_copy.materializations))
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
        self.assertEqual(
            "validation_copy_compile_time_resource_missing", evidence["errorCode"]
        )
        self.assertEqual("closure_planning", evidence["errorStage"])
        self.assertEqual(expected["resourcePath"], evidence["errorPath"])
        self.assertEqual(expected, evidence["errorDetails"])

    def test_worker_restarts_a_removed_generic_copy_before_run(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        receipt = self.service.submit(
            session_id="primary",
            request_id="generic-copy-restart-request",
            source_manifest={"tools/owned.py": digest},
            command=(
                "pwsh.exe",
                "-NoProfile",
                "-Command",
                "python -m unittest focused",
            ),
            toolchain={"python": "3.14"},
            coverage={
                "kind": "focused",
                "dependencyRoots": ["tools/session_coordinator"],
            },
        )
        ticket_id = receipt.ticket.ticket_id

        self.assertEqual(1, self.worker.tick()["materializing"])
        self.workspace_copy.records["copy-1"].status = "removed"

        recovered = self.worker.tick()

        self.assertEqual(1, recovered["materializing"])
        self.assertEqual("materializing", self.service.get(ticket_id).status)
        self.assertEqual(2, len(self.workspace_copy.generic_materializations))
        self.assertEqual([], self.workspace_copy.starts)
        self.assertEqual(
            {"jobId": "copy-2", "recoveredFromJobId": "copy-1"},
            self.service.latest_worker_event(ticket_id, "validation.ticket_copy_linked"),
        )

    def test_worker_uses_durable_result_from_removed_generic_copy_without_run_link(
        self,
    ) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()

        for index, (exit_code, expected_status) in enumerate(
            ((0, "passed"), (1, "failed")), start=1
        ):
            with self.subTest(exit_code=exit_code):
                receipt = self.service.submit(
                    session_id="primary",
                    request_id=f"generic-copy-durable-result-{exit_code}",
                    source_manifest={"tools/owned.py": digest},
                    command=("python", "-m", "unittest", "focused"),
                    toolchain={"python": "3.14"},
                    coverage={
                        "kind": "focused",
                        "dependencyRoots": ["tools/session_coordinator"],
                    },
                )
                ticket_id = receipt.ticket.ticket_id

                self.assertEqual(1, self.worker.tick()["materializing"])
                copy = self.workspace_copy.records[f"copy-{index}"]
                copy.status = "removed"
                self.workspace_copy.run_results[ticket_id] = {
                    "runId": ticket_id,
                    "jobId": copy.job_id,
                    "exitCode": exit_code,
                    "stdout": "ok",
                    "stderr": "failed" if exit_code else "",
                }

                completed = self.worker.tick()

                self.assertEqual(1, completed[expected_status])
                self.assertEqual(expected_status, self.service.get(ticket_id).status)
                self.assertEqual(index, len(self.workspace_copy.generic_materializations))
                self.assertEqual([], self.workspace_copy.starts)

    def test_non_cargo_missing_dependency_roots_never_reaches_worker(self) -> None:
        source = self.repo / "tools" / "owned.py"
        source.parent.mkdir(parents=True)
        source.write_text("current = True\n", encoding="utf-8")
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        with self.assertRaises(CoordinatorError) as rejected:
            self.service.submit(
                session_id="primary",
                request_id="python-without-dependencies",
                source_manifest={"tools/owned.py": digest},
                command=("python", "-m", "unittest", "focused"),
                toolchain={"python": "3.14"},
                coverage={"kind": "focused"},
            )

        self.assertEqual(
            "validation_ticket_dependency_roots_missing", rejected.exception.code
        )
        self.assertEqual([], self.workspace_copy.generic_materializations)
        with self.database.connect() as connection:
            counts = (
                connection.execute(
                    "SELECT COUNT(*) FROM validation_tickets"
                ).fetchone()[0],
                connection.execute(
                    "SELECT COUNT(*) FROM validation_ticket_requests"
                ).fetchone()[0],
            )
        self.assertEqual((0, 0), counts)

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
            source = repo / "tools" / "session_coordinator" / "cli.py"
            source.parent.mkdir(parents=True)
            source.write_text("BOUNDARY = True\n", encoding="utf-8")
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
                    source_manifest={
                        "tools/session_coordinator/cli.py": hashlib.sha256(
                            source.read_bytes()
                        ).hexdigest()
                    },
                    command=("python", "-m", "unittest", "focused"),
                    toolchain={"python": "3.14"},
                    coverage={
                        "kind": "focused",
                        "dependencyRoots": ["tools/session_coordinator"],
                    },
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
