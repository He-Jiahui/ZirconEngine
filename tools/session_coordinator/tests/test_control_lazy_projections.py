from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.snapshot import ControlSnapshotService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService


class ControlLazyProjectionTests(unittest.TestCase):
    def test_summary_defers_continuations_and_validation_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            plan = repo / "docs" / "plans" / "tooling" / "01-workflow.md"
            plan.parent.mkdir(parents=True)
            plan.write_text(
                "# Workflow\n\n## M1\n\n### Implementation\n\n- [ ] Continue this slice.\n",
                encoding="utf-8",
            )
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(
                session_id="waiting-owner",
                plan_path="docs/plans/tooling/01-workflow.md",
            )
            sessions.set_status("waiting-owner", SessionStatus.ACTIVE)
            sessions.set_status("waiting-owner", SessionStatus.WAITING_VALIDATION)
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda _connection: {"status": "ok"},
                repo_root=repo,
            )

            summary = service.build()
            continuations = service.continuation_projection()
            validation = service.validation_projection()

        self.assertEqual([], summary["experience"]["continuations"])
        self.assertEqual([], summary["validation"]["cargoJobs"])
        self.assertEqual([], summary["validation"]["validationCopies"])
        self.assertEqual("waiting-owner", continuations["continuations"][0]["sessionId"])
        self.assertIn("cargoJobs", validation)
        self.assertIn("validationCopies", validation)


if __name__ == "__main__":
    unittest.main()
