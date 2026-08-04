from __future__ import annotations

import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

from tools.session_coordinator.baselines import BaselineService
from tools.session_coordinator.config import CoordinatorConfig
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workspace_copy import WorkspaceCopyService


class WorkspaceCopyTerminalStatusTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        root = Path(self.temporary_directory.name)
        self.repo = init_repo(root / "repo")
        self.target_root = root / "drive/targets/zircon-engine"
        self.target_root.mkdir(parents=True)
        config = CoordinatorConfig.for_repo(self.repo, state_root=root / "state")
        self.database = Database(config.database_path)
        migrate(self.database)
        sessions = SessionService(self.database, self.repo)
        sessions.register(session_id="session-a")
        sessions.register(session_id="session-b")
        BaselineService(self.database, self.repo).initialize()
        with mock.patch(
            "tools.session_coordinator.workspace_copy._is_managed_validation_root",
            return_value=True,
        ):
            self.service = WorkspaceCopyService(
                self.database, self.repo, (self.target_root,)
            )

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def _run_nonzero_copy(self):
        copy = self.service.materialize(
            "session-a", include_paths=("README.md",)
        )
        evidence = self.service.run(
            "session-a",
            copy.job_id,
            command=(
                sys.executable,
                "-c",
                "import sys; print('cargo stdout'); "
                "print('cargo stderr', file=sys.stderr); raise SystemExit(101)",
            ),
        )
        return copy, evidence

    @staticmethod
    def _invalid_utf8_command() -> tuple[str, ...]:
        return (
            sys.executable,
            "-c",
            "import sys; "
            "sys.stdout.buffer.write(b'cargo stdout \\xff tail\\n'); "
            "sys.stdout.buffer.flush(); "
            "sys.stderr.buffer.write(b'cargo stderr \\xfe tail\\n'); "
            "sys.stderr.buffer.flush(); "
            "raise SystemExit(101)",
        )

    def test_status_returns_nonzero_terminal_evidence_after_copy_cleanup(self) -> None:
        copy, evidence = self._run_nonzero_copy()

        self.assertFalse(copy.job_root.exists())
        status = self.service.status("session-a", copy.job_id).to_dict()
        terminal = status["terminalEvidence"]
        self.assertEqual("removed", status["status"])
        self.assertEqual(evidence.run_id, terminal["run_id"])
        self.assertEqual(101, terminal["exit_code"])
        self.assertIn("cargo stdout", terminal["stdout"])
        self.assertIn("cargo stderr", terminal["stderr"])

    def test_run_boundedly_drains_large_dual_streams_before_cleanup(self) -> None:
        copy = self.service.materialize("session-a", include_paths=("README.md",))
        stdout = "stdout-prefix\n" + ("O" * 70_000) + "\nstdout-tail\n"
        stderr = "stderr-prefix\n" + ("E" * 70_000) + "\nstderr-tail\n"
        real_popen = subprocess.Popen

        def popen_without_communicate(*args, **kwargs):
            process = real_popen(*args, **kwargs)
            process.communicate = mock.Mock(  # type: ignore[method-assign]
                side_effect=AssertionError(
                    "validation-copy terminal capture must drain bounded streams"
                )
            )
            return process

        with mock.patch(
            "tools.session_coordinator.workspace_copy.subprocess.Popen",
            side_effect=popen_without_communicate,
        ):
            evidence = self.service.run(
                "session-a",
                copy.job_id,
                command=(
                    sys.executable,
                    "-c",
                    "import sys; "
                    "sys.stdout.write('stdout-prefix\\n' + 'O' * 70000 + "
                    "'\\nstdout-tail\\n'); sys.stdout.flush(); "
                    "sys.stderr.write('stderr-prefix\\n' + 'E' * 70000 + "
                    "'\\nstderr-tail\\n'); sys.stderr.flush(); "
                    "raise SystemExit(101)",
                ),
            )

        self.assertEqual(101, evidence.exit_code)
        self.assertEqual(stdout[-65_536:], evidence.stdout)
        self.assertEqual(stderr[-65_536:], evidence.stderr)
        self.assertFalse(copy.job_root.exists())
        terminal = self.service.status("session-a", copy.job_id).to_dict()[
            "terminalEvidence"
        ]
        self.assertEqual(evidence.stdout, terminal["stdout"])
        self.assertEqual(evidence.stderr, terminal["stderr"])

    def test_run_replaces_invalid_utf8_without_losing_terminal_evidence(self) -> None:
        copy = self.service.materialize("session-a", include_paths=("README.md",))

        evidence = self.service.run(
            "session-a", copy.job_id, command=self._invalid_utf8_command()
        )

        self.assertEqual(101, evidence.exit_code)
        self.assertIn("cargo stdout \ufffd tail", evidence.stdout)
        self.assertIn("cargo stderr \ufffd tail", evidence.stderr)
        self.assertFalse(copy.job_root.exists())

    def test_started_run_replaces_invalid_utf8_before_cleanup(self) -> None:
        copy = self.service.materialize("session-a", include_paths=("README.md",))

        self.service.start(
            "session-a",
            copy.job_id,
            command=self._invalid_utf8_command(),
            run_id="invalid-utf8-async",
        )
        deadline = time.monotonic() + 5
        status = self.service.status("session-a", copy.job_id).to_dict()
        while status["status"] != "removed" and time.monotonic() < deadline:
            time.sleep(0.01)
            status = self.service.status("session-a", copy.job_id).to_dict()

        terminal = status["terminalEvidence"]
        self.assertEqual("removed", status["status"])
        self.assertEqual(101, terminal["exit_code"])
        self.assertIn("cargo stdout \ufffd tail", terminal["stdout"])
        self.assertIn("cargo stderr \ufffd tail", terminal["stderr"])

    def test_status_does_not_expose_terminal_evidence_to_foreign_session(self) -> None:
        copy, _evidence = self._run_nonzero_copy()

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.status("session-b", copy.job_id)

        self.assertEqual("validation_copy_foreign_session", rejected.exception.code)

    def test_status_selects_latest_terminal_evidence_deterministically(self) -> None:
        copy, _evidence = self._run_nonzero_copy()
        with self.database.transaction() as connection:
            for run_id in ("latest-a", "latest-z"):
                connection.execute(
                    """INSERT INTO validation_copy_runs(
                           run_id, job_id, session_id, command_json, exit_code,
                           stdout_text, stderr_text, started_at, completed_at
                       ) VALUES (?, ?, 'session-a', '[\"python\", \"latest\"]', 2,
                                 ?, 'latest stderr', '9999-12-30T00:00:00Z',
                                 '9999-12-31T00:00:00Z')""",
                    (run_id, copy.job_id, run_id),
                )

        terminal = self.service.status(
            "session-a", copy.job_id
        ).to_dict()["terminalEvidence"]

        self.assertEqual("latest-z", terminal["run_id"])
        self.assertEqual("latest-z", terminal["stdout"])

    def test_status_rejects_malformed_terminal_command_json(self) -> None:
        copy, evidence = self._run_nonzero_copy()
        with self.database.transaction() as connection:
            connection.execute(
                "UPDATE validation_copy_runs SET command_json = '{' WHERE run_id = ?",
                (evidence.run_id,),
            )

        with self.assertRaises(CoordinatorError) as rejected:
            self.service.status("session-a", copy.job_id)

        self.assertEqual(
            "validation_copy_terminal_evidence_invalid", rejected.exception.code
        )


if __name__ == "__main__":
    unittest.main()
