from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import WorkflowArtifactKind, WorkflowNodeState
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.artifacts import WorkflowArtifactStore
from tools.session_coordinator.workflows.attempts import WorkflowAttemptService
from tools.session_coordinator.workflows.store import WorkflowStore


class WorkflowAttemptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.database = Database(root / "state.sqlite3")
        migrate(self.database)
        SessionService(self.database, self.repo).register(session_id="session-a")
        self.store = WorkflowStore(self.database)
        self.run = self.store.ensure_session_run("session-a", None)
        self.goal = self.store.nodes(self.run.run_id)[0]

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_attempt_replacement_preserves_history_and_current_selection(self) -> None:
        attempts = WorkflowAttemptService(self.store)
        failed = attempts.record(
            self.goal.node_id, WorkflowNodeState.FAILED, {"exitCode": 1}
        )
        succeeded = attempts.record(
            self.goal.node_id, WorkflowNodeState.SUCCEEDED, {"exitCode": 0}
        )

        self.assertEqual(1, failed.attempt_number)
        self.assertEqual(2, succeeded.attempt_number)
        self.assertEqual(
            succeeded.attempt_id,
            attempts.current(self.run.run_id)[self.goal.node_id].attempt_id,
        )
        self.assertEqual(2, len(attempts.history(self.goal.node_id)))

    def test_artifact_binds_to_attempt_and_hashes_content(self) -> None:
        attempt = WorkflowAttemptService(self.store).record(
            self.goal.node_id, WorkflowNodeState.SUCCEEDED, {"ok": True}
        )

        artifact = WorkflowArtifactStore(self.database).record_bytes(
            run_id=self.run.run_id,
            node_id=self.goal.node_id,
            attempt_id=attempt.attempt_id,
            kind=WorkflowArtifactKind.REPORT,
            display_name="validation.txt",
            content=b"accepted\n",
        )

        self.assertEqual(9, artifact.byte_count)
        self.assertEqual(64, len(artifact.content_hash))
        with self.database.connect() as connection:
            stored = connection.execute(
                "SELECT * FROM workflow_artifacts WHERE artifact_id=?",
                (artifact.artifact_id,),
            ).fetchone()
        self.assertEqual(attempt.attempt_id, stored["attempt_id"])


if __name__ == "__main__":
    unittest.main()
