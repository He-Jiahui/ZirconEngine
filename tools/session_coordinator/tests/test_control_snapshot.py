from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.control_plane.snapshot import ControlSnapshotService
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.projections import WorkflowProjectionService
from tools.session_coordinator.workflows.store import WorkflowStore


class ControlSnapshotTests(unittest.TestCase):
    def test_snapshot_contains_consistent_cursor_and_domain_sections(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repo = init_repo(root / "repo")
            database = Database(root / "state.sqlite3")
            migrate(database)
            sessions = SessionService(database, repo)
            session = sessions.register(session_id="session-a")
            WorkflowStore(database).synchronize_session(session)
            service = ControlSnapshotService(
                database,
                WorkflowProjectionService(),
                lambda connection: {
                    "status": "ok",
                    "instanceId": "instance-a",
                    "eventCount": connection.execute(
                        "SELECT COUNT(*) FROM events"
                    ).fetchone()[0],
                },
            )

            snapshot = service.build()

            self.assertEqual(1, snapshot["projectionVersion"])
            self.assertGreaterEqual(snapshot["eventCursor"], 1)
            self.assertEqual("session-a", snapshot["sessions"][0]["sessionId"])
            self.assertEqual(1, len(snapshot["workflows"]))
            self.assertIsNone(snapshot["workflows"][0]["topologyHash"])
            with database.connect() as detail_connection:
                detail = WorkflowProjectionService().workflow_detail(
                    detail_connection, snapshot["workflows"][0]["runId"]
                )
            self.assertIsNone(detail["topologyHash"])
            self.assertEqual("goal", detail["nodes"][0]["stage"])
            self.assertEqual(snapshot["eventCursor"], snapshot["service"]["eventCount"])
            self.assertEqual(
                {
                    "service",
                    "workflows",
                    "sessions",
                    "failures",
                    "collaboration",
                    "validation",
                    "git",
                    "audit",
                },
                set(snapshot) - {"projectionVersion", "eventCursor"},
            )


if __name__ == "__main__":
    unittest.main()
