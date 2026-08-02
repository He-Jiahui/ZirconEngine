from __future__ import annotations

import io
import sqlite3
import sys
import tarfile
import tempfile
import threading
import unittest
from contextlib import contextmanager
from unittest import mock
from pathlib import Path
import subprocess

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.validation_copies import CargoInputClosure
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


class WorkspaceCopyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/targets/zircon-engine"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.baselines = BaselineService(self.database, self.repo)
        self.baselines.initialize()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            self.service = WorkspaceCopyService(
                self.database, self.repo, (self.target_root,)
            )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _run_with_mocked_streams(
        self,
        stdout: str | None,
        stderr: str | None,
        *,
        exit_code: int = 101,
    ):
        result = self.service.materialize("session-a", include_paths=("README.md",))
        process = mock.Mock()
        process.pid = 4242
        process.returncode = exit_code
        process.communicate.return_value = (stdout, stderr)
        process.poll.return_value = exit_code
        with mock.patch(
            "tools.session_coordinator.workspace_copy.subprocess.Popen",
            return_value=process,
        ):
            evidence = self.service.run(
                "session-a", result.job_id, command=("cargo", "test")
            )
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT * FROM validation_copy_runs WHERE run_id = ?",
                (evidence.run_id,),
            ).fetchone()
        return evidence, row

    def test_copy_uses_head_for_foreign_dirty_and_overlay_for_owned_files(self) -> None:
        (self.repo / "README.md").write_text("foreign dirty\n", encoding="utf-8")
        owned = self.repo / "src/owned.txt"
        owned.parent.mkdir()
        owned.write_text("owned change\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["src/owned.txt"])

        result = self.service.materialize(
            "session-a", include_paths=("README.md", "src/owned.txt")
        )

        self.assertEqual("baseline\n", (result.source_root / "README.md").read_text())
        self.assertEqual("owned change\n", (result.source_root / "src/owned.txt").read_text())
        self.assertFalse((result.source_root / ".git").exists())
        self.assertFalse((result.source_root / "target").exists())
        self.assertTrue(result.target_root.is_dir())

    def test_materialize_uses_a_single_baseline_archive_for_large_manifests(self) -> None:
        for index in range(3):
            source = self.repo / "src" / f"baseline-{index}.txt"
            source.parent.mkdir(exist_ok=True)
            source.write_text(f"baseline {index}\n", encoding="utf-8")
        subprocess.run(["git", "add", "src"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-m", "test: add baseline archive fixture"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        )

        original_popen = subprocess.Popen
        archives: list[list[str]] = []

        def record_archive(arguments, *args, **kwargs):
            if len(arguments) > 1 and arguments[1] == "archive":
                archives.append(list(arguments))
            return original_popen(arguments, *args, **kwargs)

        with (
            mock.patch.object(
                self.service,
                "_head_content",
                side_effect=AssertionError("large manifests must not spawn git show per file"),
            ),
            mock.patch("tools.session_coordinator.workspace_copy.subprocess.Popen", side_effect=record_archive),
        ):
            result = self.service.materialize(
                "session-a",
                include_paths=(
                    "README.md",
                    "src/baseline-0.txt",
                    "src/baseline-1.txt",
                    "src/baseline-2.txt",
                ),
            )

        self.assertEqual("baseline 2\n", (result.source_root / "src/baseline-2.txt").read_text())
        self.assertEqual(1, len(archives))
        self.assertEqual("--", archives[0][-5])
        self.assertEqual(
            {
                "README.md",
                "src/baseline-0.txt",
                "src/baseline-1.txt",
                "src/baseline-2.txt",
            },
            set(archives[0][-4:]),
        )

    def test_materialize_drains_baseline_archive_stream_before_waiting(self) -> None:
        archive_buffer = io.BytesIO()
        with tarfile.open(fileobj=archive_buffer, mode="w") as archive:
            content = b"baseline\n"
            member = tarfile.TarInfo("README.md")
            member.size = len(content)
            archive.addfile(member, io.BytesIO(content))

        class DrainAwareStream(io.BytesIO):
            closed_with_unread_bytes = False

            def close(self) -> None:
                self.closed_with_unread_bytes = self.tell() < len(self.getbuffer())
                super().close()

        class ArchiveProcess:
            def __init__(self) -> None:
                # A real git archive can still be writing record padding after
                # tarfile has consumed the end-of-archive markers.
                self.stdout = DrainAwareStream(archive_buffer.getvalue() + b"\0" * 20_480)
                self.stderr = io.BytesIO()
                self.returncode: int | None = None

            def poll(self) -> int | None:
                return self.returncode

            def kill(self) -> None:
                self.returncode = -9

            def communicate(self) -> tuple[bytes, bytes]:
                stdout = self.stdout.read()
                stderr = self.stderr.read()
                self.wait()
                self.stdout.close()
                self.stderr.close()
                return stdout, stderr

            def wait(self) -> int:
                if self.returncode is None:
                    self.returncode = 141 if self.stdout.closed_with_unread_bytes else 0
                return self.returncode

        process = ArchiveProcess()
        original_popen = subprocess.Popen

        def replace_archive(arguments, *args, **kwargs):
            if len(arguments) > 1 and arguments[1] == "archive":
                return process
            return original_popen(arguments, *args, **kwargs)

        with mock.patch(
            "tools.session_coordinator.workspace_copy.subprocess.Popen",
            side_effect=replace_archive,
        ):
            result = self.service.materialize(
                "session-a", include_paths=("README.md",)
            )

        self.assertEqual("baseline\n", (result.source_root / "README.md").read_text())
        self.assertFalse(process.stdout.closed_with_unread_bytes)

    def test_archive_cleanup_failure_does_not_mask_extraction_error(self) -> None:
        class InvalidArchiveProcess:
            def __init__(self) -> None:
                self.stdout = io.BytesIO(b"not a tar archive")
                self.stderr = io.BytesIO()
                self.returncode: int | None = None

            def poll(self) -> int | None:
                return self.returncode

            def kill(self) -> None:
                self.returncode = -9

            def communicate(self) -> tuple[bytes, bytes]:
                raise OSError("injected archive cleanup failure")

        process = InvalidArchiveProcess()
        original_popen = subprocess.Popen

        def replace_archive(arguments, *args, **kwargs):
            if len(arguments) > 1 and arguments[1] == "archive":
                return process
            return original_popen(arguments, *args, **kwargs)

        with (
            mock.patch(
                "tools.session_coordinator.workspace_copy.subprocess.Popen",
                side_effect=replace_archive,
            ),
            self.assertRaises(tarfile.ReadError),
        ):
            self.service.materialize("session-a", include_paths=("README.md",))

    def test_runtime_cargo_closure_materializes_workspace_and_runs_from_copy(self) -> None:
        files = {
            "Cargo.toml": "[workspace]\nmembers=['zircon_runtime','zircon_runtime_interface','zircon_reflect_derive','workspace_tool']\n",
            "Cargo.lock": "# fixture lock\n",
            "rust-toolchain.toml": "[toolchain]\nchannel='1.94.1'\n",
            "zircon_runtime/Cargo.toml": "[package]\nname='zircon_runtime'\nversion='0.1.0'\n",
            "zircon_runtime/src/lib.rs": "pub fn runtime() {}\n",
            "zircon_runtime_interface/Cargo.toml": "[package]\nname='zircon_runtime_interface'\nversion='0.1.0'\n",
            "zircon_runtime_interface/src/lib.rs": "pub fn interface() {}\n",
            "zircon_reflect_derive/Cargo.toml": "[package]\nname='zircon_reflect_derive'\nversion='0.1.0'\n",
            "zircon_reflect_derive/src/lib.rs": "pub fn derive_marker() {}\n",
            "workspace_tool/Cargo.toml": "[package]\nname='workspace_tool'\nversion='0.1.0'\n",
            "workspace_tool/src/lib.rs": "pub fn tool() {}\n",
        }
        for relative, content in files.items():
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "--", *files], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "test: add runtime Cargo closure"],
            cwd=self.repo,
            check=True,
        )
        packages = [
            {
                "id": f"{name}-id",
                "name": name,
                "manifest_path": str(self.repo / name / "Cargo.toml"),
            }
            for name in (
                "zircon_runtime",
                "zircon_runtime_interface",
                "zircon_reflect_derive",
                "workspace_tool",
            )
        ]
        metadata = {
            "packages": packages,
            "workspace_members": [item["id"] for item in packages],
            "resolve": {
                "nodes": [
                    {
                        "id": "zircon_runtime-id",
                        "deps": [
                            {"pkg": "zircon_runtime_interface-id"},
                            {"pkg": "zircon_reflect_derive-id"},
                        ],
                    },
                    {"id": "zircon_runtime_interface-id", "deps": []},
                    {"id": "zircon_reflect_derive-id", "deps": []},
                    {"id": "workspace_tool-id", "deps": []},
                ]
            },
        }
        cargo_command = ("cargo", "+1.94.1", "test", "-p", "zircon_runtime", "--lib")

        record = self.service.materialize_cargo(
            "session-a",
            command=cargo_command,
            metadata_runner=lambda _command: metadata,
        )

        for relative in (
            "Cargo.toml",
            "zircon_runtime/Cargo.toml",
            "zircon_runtime_interface/Cargo.toml",
            "zircon_reflect_derive/Cargo.toml",
            "workspace_tool/Cargo.toml",
        ):
            self.assertTrue((record.source_root / relative).is_file(), relative)
        evidence = self.service.run(
            "session-a",
            record.job_id,
            command=(
                sys.executable,
                "-c",
                "from pathlib import Path; "
                "assert Path.cwd().name == 'source'; "
                "assert Path('zircon_runtime_interface/Cargo.toml').is_file(); "
                "assert Path('zircon_reflect_derive/Cargo.toml').is_file(); "
                "assert Path('workspace_tool/Cargo.toml').is_file(); "
                "print('runtime14 focused test reached')",
            ),
        )

        self.assertEqual(0, evidence.exit_code)
        self.assertIn("runtime14 focused test reached", evidence.stdout)

    def test_async_materialize_returns_before_copy_finishes_and_exposes_status(self) -> None:
        started = threading.Event()
        release = threading.Event()
        original = self.service._materialize_record

        def slow_materialize(record):
            started.set()
            release.wait(timeout=2)
            return original(record)

        with mock.patch.object(self.service, "_materialize_record", side_effect=slow_materialize):
            result = self.service.materialize_async(
                "session-a", include_paths=("README.md",)
            )
            self.assertEqual("materializing", result.status)
            self.assertTrue(started.wait(timeout=1))
            self.assertEqual(
                "materializing",
                self.service.status("session-a", result.job_id).status,
            )
            with self.assertRaises(CoordinatorError) as cleanup:
                self.service.cleanup("session-a", result.job_root)
            self.assertEqual("validation_copy_cleanup_busy", cleanup.exception.code)
            release.set()

        for _ in range(100):
            status = self.service.status("session-a", result.job_id).status
            if status == "materialized":
                break
            threading.Event().wait(0.02)
        self.assertEqual("materialized", status)

    def test_async_cargo_acknowledges_before_closure_planning_and_finishes_off_thread(self) -> None:
        """Cargo closure planning must not hold the command request open."""
        started = threading.Event()
        release = threading.Event()
        metadata = {
            "packages": [
                {
                    "id": "runtime-id",
                    "name": "zircon_runtime",
                    "manifest_path": str(self.repo / "Cargo.toml"),
                }
            ],
            "workspace_members": ["runtime-id"],
            "resolve": {"nodes": [{"id": "runtime-id", "deps": []}]},
        }

        (self.repo / "Cargo.toml").write_text(
            "[package]\nname='zircon_runtime'\nversion='0.1.0'\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "add", "Cargo.toml"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: add cargo async fixture"],
            cwd=self.repo,
            check=True,
        )

        def delayed_metadata(_command):
            started.set()
            release.wait(timeout=5)
            return metadata

        accepted = self.service.materialize_cargo_async(
            "session-a",
            command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            metadata_runner=delayed_metadata,
        )

        self.assertEqual("materializing", accepted.status)
        self.assertEqual((), accepted.manifest)
        self.assertTrue(started.wait(timeout=1))
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT status, materialization_kind, materialization_phase
                   FROM validation_copies WHERE job_id=?""",
                (accepted.job_id,),
            ).fetchone()
        self.assertEqual("planned", row["status"])
        self.assertEqual("cargo", row["materialization_kind"])
        self.assertEqual("closure_planning", row["materialization_phase"])

        release.set()
        for _ in range(100):
            if self.service.status("session-a", accepted.job_id).status == "materialized":
                break
            threading.Event().wait(0.02)
        self.assertEqual("materialized", self.service.status("session-a", accepted.job_id).status)

    def test_async_cargo_ack_is_durable_before_disk_or_git_probes(self) -> None:
        """Accepting a Cargo copy must not perform worker-only host probes inline."""
        with (
            mock.patch.object(self.service, "_spawn_cargo_materialization_worker"),
            mock.patch(
                "tools.session_coordinator.workspace_copy.shutil.disk_usage",
                side_effect=AssertionError("disk probing belongs to the worker"),
            ),
            mock.patch.object(
                self.service,
                "_git_text",
                side_effect=AssertionError("Git pinning belongs to the worker"),
            ),
            mock.patch.object(
                self.service,
                "_normalize",
                side_effect=AssertionError("overlay normalization belongs to the worker"),
            ),
        ):
            accepted = self.service.materialize_cargo_async(
                "session-a",
                command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
                overlay_paths=("README.md",),
            )

        self.assertEqual("materializing", accepted.status)
        self.assertEqual("accepted", accepted.materialization_phase)
        with self.database.connect() as connection:
            row = connection.execute(
                """SELECT status, materialization_phase, head_commit
                   FROM validation_copies WHERE job_id=?""",
                (accepted.job_id,),
            ).fetchone()
        self.assertEqual("planned", row["status"])
        self.assertEqual("accepted", row["materialization_phase"])
        self.assertEqual("pending", row["head_commit"])

    def test_async_cargo_persists_then_rejects_unowned_overlay_and_invalid_external_descriptor(self) -> None:
        """Ownership validation is durable worker work, never pre-ack request work."""
        metadata = {
            "packages": [
                {
                    "id": "runtime-id",
                    "name": "zircon_runtime",
                    "manifest_path": str(self.repo / "Cargo.toml"),
                }
            ],
            "workspace_members": ["runtime-id"],
            "resolve": {"nodes": [{"id": "runtime-id", "deps": []}]},
        }
        (self.repo / "Cargo.toml").write_text(
            "[package]\nname='zircon_runtime'\nversion='0.1.0'\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "add", "Cargo.toml"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "test: add cargo ownership fixture"],
            cwd=self.repo,
            check=True,
        )

        unowned = self.service.materialize_cargo_async(
            "session-a",
            command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            overlay_paths=("unowned.rs",),
            metadata_runner=lambda _command: metadata,
        )
        invalid_external = self.service.materialize_cargo_async(
            "session-a",
            command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            external_sources=({"repoRoot": str(self.repo)},),
            metadata_runner=lambda _command: metadata,
        )

        for _ in range(100):
            statuses = (
                self.service.status("session-a", unowned.job_id),
                self.service.status("session-a", invalid_external.job_id),
            )
            if all(item.status == "failed" for item in statuses):
                break
            threading.Event().wait(0.02)
        self.assertEqual("failed", statuses[0].status)
        self.assertEqual("validation_copy_overlay_not_owned", statuses[0].error_code)
        self.assertEqual("overlay_ownership", statuses[0].error_stage)
        self.assertEqual("failed", statuses[1].status)
        self.assertEqual("validation_copy_external_source_invalid", statuses[1].error_code)
        self.assertEqual("closure_planning", statuses[1].error_stage)

    def test_async_cargo_terminalizes_tracked_path_added_after_pinned_baseline(self) -> None:
        """A closure drift must be typed, not misreported as an unowned overlay."""

        def plan_after_baseline(*_args, **_kwargs):
            drifted = self.repo / "foreign-after-baseline.rs"
            drifted.write_text("pub const DRIFTED: bool = true;\n", encoding="utf-8")
            subprocess.run(["git", "add", drifted.name], cwd=self.repo, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "test: add foreign closure after baseline"],
                cwd=self.repo,
                check=True,
            )
            return CargoInputClosure(("README.md", drifted.name), ())

        with mock.patch.object(self.service, "_spawn_cargo_materialization_worker"):
            accepted = self.service.materialize_cargo_async(
                "session-a",
                command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            )

        with mock.patch(
            "tools.session_coordinator.workspace_copy.CargoInputClosurePlanner.plan",
            side_effect=plan_after_baseline,
        ):
            self.service._materialize_cargo_async_worker(
                accepted.job_id,
                metadata_runner=None,
            )

        status = self.service.status("session-a", accepted.job_id)
        self.assertEqual("failed", status.status)
        self.assertEqual("failed", status.materialization_phase)
        self.assertEqual("validation_copy_baseline_drift", status.error_code)
        self.assertEqual("materialization_prepare", status.error_stage)
        self.assertEqual("foreign-after-baseline.rs", status.error_path)

    def test_startup_recovery_claims_an_accepted_cargo_copy_once(self) -> None:
        """A restart may resume the same durable job, but never create another worker claim."""
        with mock.patch.object(self.service, "_spawn_cargo_materialization_worker") as spawn:
            accepted = self.service.materialize_cargo_async(
                "session-a",
                command=("cargo", "test", "-p", "zircon_runtime", "--lib"),
            )
        spawn.assert_called_once_with(accepted.job_id, metadata_runner=None)

        with mock.patch(
            "tools.session_coordinator.workspace_copy.CargoInputClosurePlanner.plan",
            return_value=CargoInputClosure(("README.md",), ()),
        ) as planned:
            with mock.patch(
                "tools.session_coordinator.workspace_copy._is_managed_validation_root",
                return_value=True,
            ):
                restarted = WorkspaceCopyService(
                    self.database,
                    self.repo,
                    (self.target_root,),
                )
            restarted.recover_interrupted_jobs(startup=True)
            for _ in range(100):
                if restarted.status("session-a", accepted.job_id).status == "materialized":
                    break
                threading.Event().wait(0.02)
            restarted.recover_interrupted_jobs(startup=True)

        self.assertEqual("materialized", restarted.status("session-a", accepted.job_id).status)
        self.assertEqual(1, planned.call_count)

    def test_cargo_worker_terminalizes_a_malformed_durable_request_once(self) -> None:
        """Corrupt persisted request JSON must not remain an endlessly recoverable ACK."""
        job_id = "malformed-cargo-request"
        job_root = self.target_root / "verify" / job_id
        with self.database.transaction() as connection:
            connection.execute(
                """
                INSERT INTO validation_copies(
                    job_id, session_id, job_root, source_root, target_root, head_commit,
                    manifest_json, status, created_at, external_sources_json,
                    materialization_kind, materialization_request_json,
                    materialization_phase, materialization_attempt
                ) VALUES (?, 'session-a', ?, ?, ?, 'pending', '[]', 'planned',
                          '2026-07-26T00:00:00+00:00', '[]', 'cargo',
                          '{malformed', 'accepted', 0)
                """,
                (job_id, str(job_root), str(job_root / "source"), str(job_root / "target")),
            )

        self.service._materialize_cargo_async_worker(job_id, metadata_runner=None)

        status = self.service.status("session-a", job_id)
        self.assertEqual("failed", status.status)
        self.assertEqual("failed", status.materialization_phase)
        self.assertEqual("validation_copy_cargo_request_invalid", status.error_code)
        self.assertEqual("request_decode", status.error_stage)
        self.assertIsNone(status.error_path)

    def test_validation_copy_keeps_baseline_dependencies_outside_milestone_manifest(self) -> None:
        dependency = self.repo / "tools/session_coordinator/probe.py"
        dependency.parent.mkdir(parents=True)
        dependency.write_text("VALUE = 'available'\n", encoding="utf-8")
        subprocess.run(["git", "add", "tools/session_coordinator/probe.py"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-m", "test: add validation dependency"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        )
        milestone = self.repo / "docs/milestone.md"
        milestone.parent.mkdir(parents=True)
        milestone.write_text("owned milestone evidence\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["docs/milestone.md"])

        with mock.patch.object(
            self.service,
            "_head_content",
            side_effect=AssertionError("validation dependencies must use one archive"),
        ):
            result = self.service.materialize_validation(
                "session-a",
                dependency_roots=("tools/session_coordinator",),
                overlay_paths=("docs/milestone.md",),
            )
        completed = subprocess.run(
            [
                sys.executable,
                "-c",
                "from tools.session_coordinator.probe import VALUE; assert VALUE == 'available'",
            ],
            cwd=result.source_root,
            check=True,
            capture_output=True,
            text=True,
        )

        self.assertEqual("", completed.stderr)
        self.assertEqual("owned milestone evidence\n", (result.source_root / "docs/milestone.md").read_text())
        self.assertIn("tools/session_coordinator/probe.py", result.manifest)
        self.assertIn("docs/milestone.md", result.manifest)

    def test_validation_dependency_copy_materializes_off_the_request_thread(self) -> None:
        dependency = self.repo / "tools/session_coordinator/probe.py"
        dependency.parent.mkdir(parents=True)
        dependency.write_text("VALUE = 'available'\n", encoding="utf-8")
        subprocess.run(
            ["git", "add", "tools/session_coordinator/probe.py"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-m", "test: add async validation dependency"],
            cwd=self.repo,
            check=True,
            capture_output=True,
        )
        overlay = self.repo / "docs/milestone.md"
        overlay.parent.mkdir(parents=True)
        overlay.write_text("owned milestone evidence\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["docs/milestone.md"])
        started = threading.Event()
        release = threading.Event()
        original = self.service._extract_baseline_dependencies

        def slow_extract(record, dependency_roots):
            started.set()
            release.wait(timeout=2)
            return original(record, dependency_roots)

        with mock.patch.object(
            self.service,
            "_extract_baseline_dependencies",
            side_effect=slow_extract,
        ):
            result = self.service.materialize_validation_async(
                "session-a",
                dependency_roots=("tools/session_coordinator",),
                overlay_paths=("docs/milestone.md",),
            )
            self.assertEqual("materializing", result.status)
            self.assertTrue(started.wait(timeout=1))
            self.assertEqual(
                "materializing",
                self.service.status("session-a", result.job_id).status,
            )
            release.set()

        for _ in range(100):
            completed = self.service.status("session-a", result.job_id)
            if completed.status == "materialized":
                break
            threading.Event().wait(0.02)

        self.assertEqual("materialized", completed.status)
        self.assertEqual(
            "owned milestone evidence\n",
            (completed.source_root / "docs/milestone.md").read_text(),
        )
        self.assertTrue(
            (completed.source_root / "tools/session_coordinator/probe.py").is_file()
        )

    def test_copy_pins_head_even_if_repository_head_changes_during_materialize(self) -> None:
        original = self.service._head_content
        changed = False

        def advance_head(job_id: str, path: str) -> bytes | None:
            nonlocal changed
            if not changed:
                changed = True
                (self.repo / "README.md").write_text("new head\n", encoding="utf-8")
                import subprocess

                subprocess.run(["git", "add", "README.md"], cwd=self.repo, check=True)
                subprocess.run(
                    ["git", "commit", "-m", "test: advance head"],
                    cwd=self.repo,
                    check=True,
                    capture_output=True,
                )
            return original(job_id, path)

        self.service._head_content = advance_head  # type: ignore[method-assign]
        result = self.service.materialize("session-a", include_paths=("README.md",))

        self.assertEqual("baseline\n", (result.source_root / "README.md").read_text())

    def test_owned_overlay_rejects_content_changed_after_attribution(self) -> None:
        owned = self.repo / "owned.txt"
        owned.write_text("owned\n", encoding="utf-8")
        self.baselines.attribute("session-a", ["owned.txt"])
        owned.write_text("overwritten\n", encoding="utf-8")

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.materialize("session-a", include_paths=("owned.txt",))

        self.assertEqual("validation_copy_attribution_stale", rejected.exception.code)

    def test_run_uses_adjacent_target_and_records_evidence(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        record_path = self.target_root.parent / "target-path.txt"

        evidence = self.service.run(
            "session-a",
            result.job_id,
            command=(
                sys.executable,
                "-c",
                "import os, pathlib; "
                f"pathlib.Path({str(record_path)!r}).write_text(os.environ['CARGO_TARGET_DIR'])",
            ),
        )

        self.assertEqual(0, evidence.exit_code)
        recorded = record_path.read_text(encoding="utf-8")
        self.assertEqual(str(result.target_root), recorded)
        self.assertFalse(result.job_root.exists())
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()["status"]
        self.assertEqual("removed", status)

    def test_run_normalizes_both_missing_streams_before_durable_insert(self) -> None:
        evidence, row = self._run_with_mocked_streams(None, None)

        self.assertEqual(101, evidence.exit_code)
        self.assertEqual("", evidence.stdout)
        self.assertEqual("", evidence.stderr)
        self.assertEqual("", row["stdout_text"])
        self.assertEqual("", row["stderr_text"])

    def test_run_normalizes_missing_stdout_without_losing_stderr(self) -> None:
        evidence, row = self._run_with_mocked_streams(None, "cargo stderr")

        self.assertEqual("", evidence.stdout)
        self.assertEqual("cargo stderr", evidence.stderr)
        self.assertEqual("cargo stderr", row["stderr_text"])

    def test_run_normalizes_missing_stderr_without_losing_stdout(self) -> None:
        evidence, row = self._run_with_mocked_streams("cargo stdout", None)

        self.assertEqual("cargo stdout", evidence.stdout)
        self.assertEqual("", evidence.stderr)
        self.assertEqual("cargo stdout", row["stdout_text"])

    def test_run_persists_real_nonzero_terminal_evidence(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))

        evidence = self.service.run(
            "session-a",
            result.job_id,
            command=(
                sys.executable,
                "-c",
                "import sys; print('cargo stdout'); print('cargo stderr', file=sys.stderr); raise SystemExit(101)",
            ),
        )

        self.assertEqual(101, evidence.exit_code)
        self.assertIn("cargo stdout", evidence.stdout)
        self.assertIn("cargo stderr", evidence.stderr)
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT exit_code, stdout_text, stderr_text FROM validation_copy_runs WHERE run_id = ?",
                (evidence.run_id,),
            ).fetchone()
        self.assertEqual(101, row["exit_code"])
        self.assertIn("cargo stdout", row["stdout_text"])
        self.assertIn("cargo stderr", row["stderr_text"])

    def test_run_preserves_durable_evidence_when_completion_hook_fails(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        self.service.set_completion_hook(
            lambda _run_id: (_ for _ in ()).throw(RuntimeError("hook failed"))
        )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.run(
                "session-a", result.job_id, command=(sys.executable, "-c", "pass")
            )

        self.assertEqual(
            "validation_copy_completion_hook_failed", rejected.exception.code
        )
        with self.database.connect() as connection:
            run_row = connection.execute(
                "SELECT exit_code FROM validation_copy_runs WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()
            copy_status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()["status"]
            event = connection.execute(
                "SELECT event_type, payload_json FROM events "
                "WHERE session_id = ? ORDER BY event_id DESC LIMIT 1",
                ("session-a",),
            ).fetchone()
        self.assertEqual(0, run_row["exit_code"])
        self.assertEqual("failed", copy_status)
        self.assertEqual("validation_copy.completion_hook_failed", event["event_type"])
        self.assertIn("validation_copy_completion_hook_failed", event["payload_json"])
        self.assertTrue(result.job_root.exists())

    def test_started_run_records_observable_completion_hook_failure(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        self.service.set_completion_hook(
            lambda _run_id: (_ for _ in ()).throw(RuntimeError("hook failed"))
        )

        started = self.service.start(
            "session-a",
            result.job_id,
            command=(sys.executable, "-c", "print('async evidence')"),
        )

        for _ in range(100):
            with self.database.connect() as connection:
                copy_status = connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()["status"]
                run_row = connection.execute(
                    "SELECT exit_code FROM validation_copy_runs WHERE run_id = ?",
                    (started["runId"],),
                ).fetchone()
                event = connection.execute(
                    "SELECT event_type, payload_json FROM events "
                    "WHERE session_id = ? ORDER BY event_id DESC LIMIT 1",
                    ("session-a",),
                ).fetchone()
            if copy_status == "failed" and event is not None:
                break
            threading.Event().wait(0.02)

        self.assertEqual("failed", copy_status)
        self.assertEqual(0, run_row["exit_code"])
        self.assertEqual("validation_copy.completion_hook_failed", event["event_type"])
        self.assertIn("validation_copy_completion_hook_failed", event["payload_json"])
        self.assertTrue(result.job_root.exists())

    def test_cleanup_cannot_remove_copy_while_completion_hook_is_running(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        hook_started = threading.Event()
        release_hook = threading.Event()
        outcome: dict[str, object] = {}

        def blocking_hook(_run_id: str) -> None:
            hook_started.set()
            release_hook.wait(timeout=5)

        def run_validation() -> None:
            try:
                outcome["evidence"] = self.service.run(
                    "session-a",
                    result.job_id,
                    command=(sys.executable, "-c", "print('terminal evidence')"),
                )
            except BaseException as error:
                outcome["error"] = error

        self.service.set_completion_hook(blocking_hook)
        worker = threading.Thread(target=run_validation)
        worker.start()
        self.assertTrue(hook_started.wait(5))

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.cleanup("session-a", result.job_root)

        self.assertEqual("validation_copy_cleanup_busy", rejected.exception.code)
        self.assertTrue(result.job_root.exists())
        release_hook.set()
        worker.join(timeout=5)
        self.assertFalse(worker.is_alive())
        self.assertNotIn("error", outcome)
        self.assertEqual("removed", self.service.status("session-a", result.job_id).status)

    def test_periodic_recovery_skips_locally_active_completion_hook(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        hook_started = threading.Event()
        release_hook = threading.Event()
        outcome: dict[str, object] = {}

        def blocking_hook(_run_id: str) -> None:
            hook_started.set()
            release_hook.wait(timeout=5)

        def run_validation() -> None:
            try:
                outcome["evidence"] = self.service.run(
                    "session-a",
                    result.job_id,
                    command=(sys.executable, "-c", "print('terminal evidence')"),
                )
            except BaseException as error:
                outcome["error"] = error

        self.service.set_completion_hook(blocking_hook)
        worker = threading.Thread(target=run_validation)
        worker.start()
        self.assertTrue(hook_started.wait(5))

        recovered = self.service.recover_interrupted_jobs(
            process_alive=lambda _pid: False, startup=False
        )

        self.assertEqual((0, 0), recovered)
        self.assertEqual("running", self.service.status("session-a", result.job_id).status)
        release_hook.set()
        worker.join(timeout=5)
        self.assertFalse(worker.is_alive())
        self.assertNotIn("error", outcome)
        self.assertEqual("removed", self.service.status("session-a", result.job_id).status)

    def test_periodic_recovery_skips_locally_reserved_process_launch(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        launch_started = threading.Event()
        release_launch = threading.Event()
        outcome: dict[str, object] = {}
        process = mock.Mock()
        process.pid = 4444
        process.returncode = 0
        process.communicate.return_value = ("stdout", "")
        process.poll.return_value = 0

        def blocking_popen(*_args, **_kwargs):
            launch_started.set()
            release_launch.wait(timeout=5)
            return process

        def run_validation() -> None:
            try:
                outcome["evidence"] = self.service.run(
                    "session-a", result.job_id, command=("cargo", "test")
                )
            except BaseException as error:
                outcome["error"] = error

        with mock.patch(
            "tools.session_coordinator.workspace_copy.subprocess.Popen",
            side_effect=blocking_popen,
        ):
            worker = threading.Thread(target=run_validation)
            worker.start()
            self.assertTrue(launch_started.wait(5))

            recovered = self.service.recover_interrupted_jobs(
                process_alive=lambda _pid: False, startup=False
            )

            self.assertEqual((0, 0), recovered)
            self.assertEqual(
                "running", self.service.status("session-a", result.job_id).status
            )
            release_launch.set()
            worker.join(timeout=5)

        self.assertFalse(worker.is_alive())
        self.assertNotIn("error", outcome)
        self.assertEqual("removed", self.service.status("session-a", result.job_id).status)

    def test_recovery_running_snapshot_is_atomic_with_run_reservation(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        recovery_transaction_started = threading.Event()
        release_recovery = threading.Event()
        launch_started = threading.Event()
        release_launch = threading.Event()
        outcome: dict[str, object] = {}
        process = mock.Mock()
        process.pid = 4545
        process.returncode = 0
        process.communicate.return_value = ("stdout", "")
        process.poll.return_value = 0
        original_transaction = self.database.transaction

        @contextmanager
        def gated_transaction(*, immediate: bool = True):
            if threading.current_thread().name == "recovery-snapshot":
                recovery_transaction_started.set()
                release_recovery.wait(timeout=5)
            with original_transaction(immediate=immediate) as connection:
                yield connection

        def recover() -> None:
            outcome["recovered"] = self.service.recover_interrupted_jobs(
                process_alive=lambda _pid: False, startup=False
            )

        def blocking_popen(*_args, **_kwargs):
            launch_started.set()
            release_launch.wait(timeout=5)
            return process

        def run_validation() -> None:
            try:
                outcome["evidence"] = self.service.run(
                    "session-a", result.job_id, command=("cargo", "test")
                )
            except BaseException as error:
                outcome["error"] = error

        with (
            mock.patch.object(self.database, "transaction", gated_transaction),
            mock.patch(
                "tools.session_coordinator.workspace_copy.subprocess.Popen",
                side_effect=blocking_popen,
            ),
        ):
            recovery = threading.Thread(target=recover, name="recovery-snapshot")
            recovery.start()
            self.assertTrue(recovery_transaction_started.wait(5))
            worker = threading.Thread(target=run_validation)
            worker.start()
            launched_during_recovery = launch_started.wait(0.2)
            release_recovery.set()
            recovery.join(timeout=5)
            self.assertFalse(recovery.is_alive())
            self.assertTrue(launch_started.wait(5))
            release_launch.set()
            worker.join(timeout=5)

        self.assertFalse(launched_during_recovery)
        self.assertFalse(worker.is_alive())
        self.assertEqual((0, 0), outcome["recovered"])
        self.assertNotIn("error", outcome)
        self.assertEqual("removed", self.service.status("session-a", result.job_id).status)

    def test_run_preserves_copy_when_evidence_insert_fails(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        with self.database.transaction() as connection:
            connection.execute(
                """CREATE TRIGGER reject_validation_copy_run
                   BEFORE INSERT ON validation_copy_runs
                   BEGIN
                     SELECT RAISE(ABORT, 'injected evidence failure');
                   END"""
            )

        with self.assertRaises(sqlite3.IntegrityError):
            self.service.run(
                "session-a", result.job_id, command=(sys.executable, "-c", "pass")
            )

        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, run_pid FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()
            run_count = connection.execute(
                "SELECT COUNT(*) FROM validation_copy_runs WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()[0]
        self.assertEqual("failed", row["status"])
        self.assertIsNone(row["run_pid"])
        self.assertEqual(0, run_count)
        self.assertTrue(result.job_root.exists())

    def test_started_run_normalizes_missing_streams_before_cleanup(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        process = mock.Mock()
        process.pid = 4343
        process.returncode = 101
        process.communicate.return_value = (None, "async cargo stderr")
        process.poll.return_value = 101

        with mock.patch(
            "tools.session_coordinator.workspace_copy.subprocess.Popen",
            return_value=process,
        ):
            started = self.service.start(
                "session-a", result.job_id, command=("cargo", "test")
            )

        for _ in range(100):
            with self.database.connect() as connection:
                run_row = connection.execute(
                    "SELECT exit_code, stdout_text, stderr_text FROM validation_copy_runs WHERE run_id = ?",
                    (started["runId"],),
                ).fetchone()
                copy_status = connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()["status"]
            if run_row is not None and copy_status == "removed":
                break
            threading.Event().wait(0.02)

        self.assertIsNotNone(run_row)
        self.assertEqual(101, run_row["exit_code"])
        self.assertEqual("", run_row["stdout_text"])
        self.assertEqual("async cargo stderr", run_row["stderr_text"])
        self.assertEqual("removed", copy_status)

    def test_start_returns_running_job_that_can_be_cancelled(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))

        started = self.service.start(
            "session-a",
            result.job_id,
            command=(sys.executable, "-c", "import time; time.sleep(2)"),
        )

        self.assertEqual("running", started["status"])
        self.assertGreater(int(started["pid"]), 0)
        cancelled = self.service.cancel("session-a", result.job_id)
        self.assertEqual("cancelling", cancelled["status"])
        for _ in range(100):
            with self.database.connect() as connection:
                status = connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()["status"]
            if status == "removed":
                break
            threading.Event().wait(0.05)
        self.assertEqual("removed", status)

    def test_async_completion_uses_the_shared_mutation_gate(self) -> None:
        gate = threading.Lock()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            service = WorkspaceCopyService(
                self.database,
                self.repo,
                (self.target_root,),
                mutation_gate=lambda: gate,
            )
        result = service.materialize("session-a", include_paths=("README.md",))
        gate.acquire()
        try:
            service.start(
                "session-a", result.job_id, command=(sys.executable, "-c", "pass")
            )
            threading.Event().wait(0.2)
            with self.database.connect() as connection:
                status = connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()["status"]
            self.assertEqual("running", status)
        finally:
            gate.release()
        for _ in range(100):
            with self.database.connect() as connection:
                status = connection.execute(
                    "SELECT status FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()["status"]
            if status == "removed":
                break
            threading.Event().wait(0.05)
        self.assertEqual("removed", status)

    def test_cleanup_rejects_paths_outside_managed_verify_job(self) -> None:
        with self.assertRaises(CoordinatorError):
            self.service.cleanup("session-a", self.repo)

    def test_cleanup_removes_only_materialized_job_root(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))

        removed = self.service.cleanup("session-a", result.job_root)

        self.assertEqual(result.job_root, removed)
        self.assertFalse(result.job_root.exists())
        self.assertTrue(self.target_root.exists())

    def test_materialize_rejects_verify_root_resolving_outside_managed_root(self) -> None:
        outside = self.target_root.parent / "outside"
        outside.mkdir()
        original_resolve = Path.resolve

        def escaped_resolve(path: Path, *args, **kwargs) -> Path:
            if path == self.target_root / "verify":
                return outside
            return original_resolve(path, *args, **kwargs)

        with mock.patch.object(Path, "resolve", escaped_resolve):
            with self.assertRaises(CoordinatorError) as rejected:
                self.service.materialize("session-a", include_paths=("README.md",))

        self.assertEqual("validation_copy_verify_escape", rejected.exception.code)

    def test_foreign_session_cannot_cleanup_copy(self) -> None:
        SessionService(self.database, self.repo).register(session_id="session-b")
        result = self.service.materialize("session-a", include_paths=("README.md",))

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.cleanup("session-b", result.job_root)

        self.assertEqual("validation_copy_foreign_session", rejected.exception.code)
        self.assertTrue(result.job_root.exists())

    def test_running_copy_rejects_second_run_and_cleanup(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        errors: list[BaseException] = []

        def run_first() -> None:
            try:
                self.service.run(
                    "session-a",
                    result.job_id,
                    command=(sys.executable, "-c", "import time; time.sleep(2)"),
                )
            except BaseException as error:
                errors.append(error)

        thread = threading.Thread(target=run_first)
        thread.start()
        for _ in range(50):
            with self.database.connect() as connection:
                running = connection.execute(
                    "SELECT status, run_pid FROM validation_copies WHERE job_id = ?",
                    (result.job_id,),
                ).fetchone()
            if running["status"] == "running" and running["run_pid"]:
                break
            threading.Event().wait(0.05)
        self.assertEqual("running", running["status"])
        self.assertGreater(int(running["run_pid"]), 0)
        with self.assertRaises(CoordinatorError) as second:
            self.service.run("session-a", result.job_id, command=(sys.executable, "-V"))
        self.assertEqual("validation_copy_not_materialized", second.exception.code)
        with self.assertRaises(CoordinatorError) as cleanup:
            self.service.cleanup("session-a", result.job_root)
        self.assertEqual("validation_copy_cleanup_busy", cleanup.exception.code)
        thread.join(timeout=5)
        self.assertFalse(errors)
        self.assertFalse(thread.is_alive())

    def test_restart_recovers_dead_run_and_cleanup_reservations(self) -> None:
        first = self.service.materialize("session-a", include_paths=("README.md",))
        second = self.service.materialize("session-a", include_paths=("README.md",))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'running', run_pid = 999999 WHERE job_id = ?",
                (first.job_id,),
            )
            connection.execute(
                "UPDATE validation_copies SET status = 'cleanup_pending' WHERE job_id = ?",
                (second.job_id,),
            )

        recovered = self.service.recover_interrupted_jobs(process_alive=lambda _pid: False)

        self.assertEqual((1, 1), recovered)
        with self.database.connect() as connection:
            statuses = {
                row["job_id"]: row["status"]
                for row in connection.execute(
                    "SELECT job_id, status FROM validation_copies WHERE job_id IN (?, ?)",
                    (first.job_id, second.job_id),
                )
            }
        self.assertEqual("materialized", statuses[first.job_id])
        self.assertEqual("removed", statuses[second.job_id])

    def test_restart_preserves_copy_with_terminal_evidence_as_failed(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'running', run_pid = 999999 "
                "WHERE job_id = ?",
                (result.job_id,),
            )
            connection.execute(
                """INSERT INTO validation_copy_runs(
                       run_id, job_id, session_id, command_json, exit_code,
                       stdout_text, stderr_text, started_at, completed_at
                   ) VALUES ('terminal-run', ?, 'session-a', '["python"]', 0,
                             'stdout', '', 'started', 'completed')""",
                (result.job_id,),
            )

        recovered = self.service.recover_interrupted_jobs(
            process_alive=lambda _pid: False
        )

        self.assertEqual((1, 0), recovered)
        self.assertEqual("failed", self.service.status("session-a", result.job_id).status)
        self.assertTrue(result.job_root.exists())

    def test_periodic_recovery_retries_cleanup_pending_job(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copies SET status = 'cleanup_pending' WHERE job_id = ?",
                (result.job_id,),
            )

        recovered = self.service.recover_interrupted_jobs(
            process_alive=lambda _pid: False, startup=False
        )

        self.assertEqual((0, 1), recovered)
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()["status"]
        self.assertEqual("removed", status)

    def test_startup_recovery_removes_interrupted_planned_copy(self) -> None:
        planned = self.service.plan("session-a", include_paths=("README.md",))
        planned.source_root.mkdir(parents=True)
        (planned.source_root / "partial.txt").write_text("partial", encoding="utf-8")

        recovered = self.service.recover_interrupted_jobs(
            process_alive=lambda _pid: False, startup=True
        )

        self.assertEqual((0, 1), recovered)
        self.assertFalse(planned.job_root.exists())
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?", (planned.job_id,)
            ).fetchone()["status"]
        self.assertEqual("removed", status)

    def test_cleanup_failure_stays_pending_until_periodic_retry_succeeds(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))

        with mock.patch(
            "tools.session_coordinator.workspace_copy.shutil.rmtree",
            side_effect=OSError("locked by another process"),
        ):
            with self.assertRaises(OSError):
                self.service.cleanup("session-a", result.job_root)

        with self.database.connect() as connection:
            pending = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()["status"]
        self.assertEqual("cleanup_pending", pending)

        recovered = self.service.recover_interrupted_jobs(
            process_alive=lambda _pid: False, startup=False
        )

        self.assertEqual((0, 1), recovered)
        self.assertFalse(result.job_root.exists())
        with self.database.connect() as connection:
            status = connection.execute(
                "SELECT status FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()["status"]
        self.assertEqual("removed", status)

    def test_run_preparation_failure_releases_running_state(self) -> None:
        result = self.service.materialize("session-a", include_paths=("README.md",))
        with mock.patch.object(
            self.service,
            "_validate_job_root",
            side_effect=CoordinatorError("injected", "path validation failed"),
        ):
            with self.assertRaises(CoordinatorError):
                self.service.run(
                    "session-a", result.job_id, command=(sys.executable, "-V")
                )
        with self.database.connect() as connection:
            row = connection.execute(
                "SELECT status, run_pid FROM validation_copies WHERE job_id = ?",
                (result.job_id,),
            ).fetchone()
        self.assertEqual("materialized", row["status"])
        self.assertIsNone(row["run_pid"])


if __name__ == "__main__":
    unittest.main()
