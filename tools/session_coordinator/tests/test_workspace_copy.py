from __future__ import annotations

import sqlite3
import sys
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
