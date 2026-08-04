from __future__ import annotations

import sys
import tempfile
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
