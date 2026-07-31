from __future__ import annotations

import tempfile
import threading
import unittest
from contextlib import contextmanager
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator.cargo_runner import CargoJobRunner
from tools.session_coordinator.models import CoordinatorError


class CargoRunnerSourceRootTests(unittest.TestCase):
    def test_runner_uses_the_coordinator_selected_immutable_working_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            source = root / "verify/job/source"
            logs = root / "logs"
            repo.mkdir()
            source.mkdir(parents=True)
            process = mock.Mock()
            process.pid = 4242
            process.poll.return_value = None
            release = threading.Event()
            process.wait.side_effect = lambda *args, **kwargs: release.wait(timeout=2) or 0
            jobs = mock.Mock()
            jobs.get.return_value = SimpleNamespace(
                session_id="session-a",
                status=SimpleNamespace(value="leased"),
                target_dir=str(root / "target"),
            )
            connection = mock.Mock()

            @contextmanager
            def transaction():
                yield connection

            database = SimpleNamespace(transaction=transaction)
            runner = CargoJobRunner(
                database,
                jobs,
                repo_root=repo,
                log_root=logs,
                popen=mock.Mock(return_value=process),
            )

            runner.start(
                session_id="session-a",
                job_id="job-a",
                command=("cargo", "test"),
                working_directory=source,
            )

            self.assertEqual(source, runner.popen.call_args.kwargs["cwd"])
            release.set()

    def test_runner_rejects_a_missing_source_root_before_spawn(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = root / "repo"
            repo.mkdir()
            jobs = mock.Mock()
            jobs.get.return_value = SimpleNamespace(
                session_id="session-a",
                status=SimpleNamespace(value="leased"),
                target_dir=str(root / "target"),
            )
            runner = CargoJobRunner(
                mock.Mock(),
                jobs,
                repo_root=repo,
                log_root=root / "logs",
                popen=mock.Mock(),
            )

            with self.assertRaises(CoordinatorError) as rejected:
                runner.start(
                    session_id="session-a",
                    job_id="job-a",
                    command=("cargo", "test"),
                    working_directory=root / "missing",
                )

        self.assertEqual("cargo_run_source_root_invalid", rejected.exception.code)
        runner.popen.assert_not_called()


if __name__ == "__main__":
    unittest.main()
