from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import SessionStatus, WorkflowNodeState
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class WorkflowStoreTests(unittest.TestCase):
    def test_ensure_session_run_is_stable_and_creates_goal_node(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(
                session_id="session-a", plan_path="docs/plans/tools/01-control.md"
            )
            store = WorkflowStore(database)

            first = store.ensure_session_run(
                "session-a", "docs/plans/tools/01-control.md"
            )
            second = store.ensure_session_run(
                "session-a", "docs/plans/tools/01-control.md"
            )

            self.assertEqual(first.run_id, second.run_id)
            nodes = store.nodes(first.run_id)
            self.assertEqual(["goal"], [node.node_key for node in nodes])
            self.assertEqual(WorkflowNodeState.PENDING, nodes[0].state)

    def test_latest_attempt_supplies_current_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="session-a")
            run = WorkflowStore(database).ensure_session_run("session-a", None)
            store = WorkflowStore(database)
            goal = store.nodes(run.run_id)[0]

            store.append_attempt(goal.node_id, WorkflowNodeState.FAILED, {"exit": 1})
            current = store.append_attempt(
                goal.node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0}
            )

            self.assertEqual(2, current.attempt_number)
            self.assertEqual(
                WorkflowNodeState.SUCCEEDED,
                store.current_attempts(run.run_id)[goal.node_id].state,
            )
            self.assertEqual(2, len(store.attempt_history(goal.node_id)))

    def test_rejected_attempt_preserves_accepted_node_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            SessionService(database, repo).register(session_id="session-a")
            store = WorkflowStore(database)
            run = store.ensure_session_run("session-a", None)
            goal = store.nodes(run.run_id)[0]
            store.append_attempt(
                goal.node_id, WorkflowNodeState.SUCCEEDED, {"accepted": True}
            )

            store.append_attempt(
                goal.node_id,
                WorkflowNodeState.FAILED,
                {"accepted": False},
                accepted=False,
            )

            current = store.current_attempts(run.run_id)[goal.node_id]
            node = store.nodes(run.run_id)[0]
            self.assertEqual(WorkflowNodeState.SUCCEEDED, current.state)
            self.assertEqual(WorkflowNodeState.SUCCEEDED, node.state)
            self.assertEqual(2, node.attempt_count)

    def test_session_status_sync_updates_run_and_goal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            sessions.register(session_id="session-a")
            sessions.set_status("session-a", SessionStatus.ACTIVE)
            store = WorkflowStore(database)

            run = store.synchronize_session(sessions.get("session-a"))

            self.assertEqual("active", run.state.value)
            self.assertEqual(WorkflowNodeState.RUNNING, store.nodes(run.run_id)[0].state)

    def test_session_hook_updates_workflow_atomically_for_maintenance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            store = WorkflowStore(database)
            sessions = SessionService(
                database,
                repo,
                session_change_hook=store.synchronize_session_in_connection,
            )

            sessions.register(session_id="session-a")
            sessions.set_status("session-a", SessionStatus.ACTIVE)
            self.assertEqual(["session-a"], sessions.mark_stale(older_than_seconds=-1))

            with database.connect() as connection:
                summary = WorkflowProjectionService().workflow_summaries(connection)[0]
            self.assertEqual("stale", summary["state"])

    def test_session_hook_failure_rolls_back_authoritative_session(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)

            def reject_change(_connection, _session) -> None:
                raise RuntimeError("projection failed")

            sessions = SessionService(
                database, repo, session_change_hook=reject_change
            )
            with self.assertRaisesRegex(RuntimeError, "projection failed"):
                sessions.register(session_id="session-a")
            with database.connect() as connection:
                self.assertIsNone(
                    connection.execute(
                        "SELECT 1 FROM sessions WHERE session_id = 'session-a'"
                    ).fetchone()
                )


if __name__ == "__main__":
    unittest.main()
