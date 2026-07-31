from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from tools.session_coordinator.control_plane.actions.catalog import action_spec
from tools.session_coordinator.control_plane.actions.executor import ActionExecutor
from tools.session_coordinator.control_plane.actions.models import ActionKind
from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.plan_import import TopologyImporter


class TopologyPrepareIdentityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.plan = self.repo / "docs/plans/runtime/01-control.md"
        self.plan.parent.mkdir(parents=True)
        self.body = (
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n\nOperator note A.\n"
        )
        self.plan.write_text(self.body, encoding="utf-8")
        self.database = Database(root / "state.sqlite3")
        migrate(self.database)
        SessionService(self.database, self.repo).register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_content_only_refresh_reuses_the_active_topology_identity(self) -> None:
        importer = TopologyImporter(self.database, self.repo)
        first = importer.import_plan("session-a", "docs/plans/runtime/01-control.md")
        self.plan.write_text(
            self.body.replace("Operator note A.", "Operator note B."), encoding="utf-8"
        )

        refreshed = importer.import_plan(
            "session-a", "docs/plans/runtime/01-control.md", activate_candidate=True
        )

        self.assertEqual(first.topology_version_id, refreshed.topology_version_id)
        self.assertEqual(1, refreshed.version_number)
        self.assertTrue(refreshed.activated)
        self.assertNotEqual(first.content_hash, refreshed.content_hash)
        with self.database.connect() as connection:
            self.assertEqual(
                1,
                connection.execute("SELECT COUNT(*) FROM workflow_topology_versions").fetchone()[0],
            )

    def test_prepare_action_binds_the_requested_node_after_refresh(self) -> None:
        importer = mock.Mock()
        importer.import_plan.return_value = SimpleNamespace(
            run_id="run-1",
            topology_version_id="version-1",
            version_number=1,
            activated=True,
        )
        milestones = mock.Mock()
        milestones.prepare_milestone.return_value = {
            "milestoneId": "M1",
            "nodeId": "run-1:M1",
            "topologyVersionId": "version-1",
            "manifestId": "manifest-1",
            "manifestHash": "abc",
        }
        sessions = mock.Mock()
        sessions.get.return_value = SimpleNamespace(
            plan_path="docs/plans/runtime/01-control.md"
        )
        executor = ActionExecutor(
            sessions=sessions,
            leases=mock.Mock(),
            patches=mock.Mock(),
            failures=mock.Mock(),
            workspace_copy=mock.Mock(),
            workflows=mock.Mock(),
            topology_importer=importer,
            milestones=milestones,
        )
        spec = action_spec(ActionKind.TOPOLOGY_REFRESH.value)
        parameters = spec.parse_parameters({"sessionId": "session-a", "milestoneId": "M1"})

        result = executor.execute(spec, parameters, resource_snapshot={}, action_id="action-1")

        importer.import_plan.assert_called_once_with(
            "session-a", "docs/plans/runtime/01-control.md", activate_candidate=True
        )
        milestones.prepare_milestone.assert_called_once_with(
            session_id="session-a",
            run_id="run-1",
            milestone_key="M1",
            actor="controlled-action",
            action_id="action-1",
        )
        self.assertEqual("M1", result["prepared"]["milestoneId"])
        self.assertEqual("version-1", result["prepared"]["topologyVersionId"])


if __name__ == "__main__":
    unittest.main()
