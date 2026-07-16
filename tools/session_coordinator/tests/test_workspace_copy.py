from __future__ import annotations

import sys
import tempfile
import threading
import unittest
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
            if status != "running":
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
