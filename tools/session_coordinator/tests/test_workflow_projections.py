from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import WorkflowNodeState
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class WorkflowProjectionTests(unittest.TestCase):
    def test_detail_uses_latest_attempt_and_keeps_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="session-a")
            store = WorkflowStore(database)
            run = store.ensure_session_run("session-a", None)
            goal = store.nodes(run.run_id)[0]
            store.append_attempt(goal.node_id, WorkflowNodeState.FAILED, {"reason": "one"})
            store.append_attempt(goal.node_id, WorkflowNodeState.FAILED, {"reason": "two"})
            store.append_attempt(goal.node_id, WorkflowNodeState.SUCCEEDED, {"reason": "fixed"})

            with database.connect() as connection:
                detail = WorkflowProjectionService().workflow_detail(connection, run.run_id)

            node = detail["nodes"][0]
            self.assertEqual("succeeded", node["currentAttempt"]["state"])
            self.assertEqual(3, len(node["attemptHistory"]))

    def test_summary_and_detail_use_same_latest_accepted_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="session-a")
            store = WorkflowStore(database)
            run = store.synchronize_session(sessions.get("session-a"))
            goal = store.nodes(run.run_id)[0]
            store.append_attempt(goal.node_id, WorkflowNodeState.SUCCEEDED, {})
            store.synchronize_session(sessions.get("session-a"))

            with database.connect() as connection:
                projections = WorkflowProjectionService()
                summary = projections.workflow_summaries(connection)[0]
                detail = projections.workflow_detail(connection, run.run_id)

            self.assertEqual(1, summary["succeededCount"])
            self.assertEqual("succeeded", detail["nodes"][0]["state"])
            self.assertEqual(
                WorkflowNodeState.SUCCEEDED, store.nodes(run.run_id)[0].state
            )
            self.assertEqual(
                WorkflowNodeState.SUCCEEDED,
                store.current_attempts(run.run_id)[goal.node_id].state,
            )


if __name__ == "__main__":
    unittest.main()
