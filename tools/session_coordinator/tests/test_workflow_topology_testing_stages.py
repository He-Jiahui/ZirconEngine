from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.models import CoordinatorError
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.topology import TopologyParser


class WorkflowTopologyTestingStageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        self.plan_dir = self.repo / "docs/plans/runtime"
        self.plan_dir.mkdir(parents=True)
        self.plan = self.plan_dir / "01-control.md"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _parse(self, body: str):
        self.plan.write_text(body, encoding="utf-8")
        return TopologyParser(self.repo).parse("docs/plans/runtime/01-control.md")

    def test_ignores_only_nested_testing_stage_headings(self) -> None:
        topology = self._parse(
            "# Runtime plan\n\n"
            "### M1 Bootstrap\n\n"
            "#### M1 测试阶段（milestone-first）\n\n"
            "### M2 Input stack\n\n"
            "#### M2 Testing stage (milestone-first)\n\n"
            "**Dependencies:** M1 accepted.\n\n"
            "### M3 Testing stage rollout\n"
        )

        self.assertEqual(
            [
                ("M1", "Bootstrap"),
                ("M2", "Input stack"),
                ("M3", "Testing stage rollout"),
            ],
            [(node.node_id, node.title) for node in topology.milestones],
        )
        self.assertEqual(("M1",), topology.milestones[1].depends_on)

    def test_rejects_genuine_duplicate_milestone_headings(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self._parse("### M1 Base\n\n### M1 Duplicate\n")
        self.assertEqual("workflow_topology_duplicate_id", rejected.exception.code)

    def test_does_not_hide_stage_heading_reparented_by_plain_heading(self) -> None:
        with self.assertRaises(CoordinatorError) as rejected:
            self._parse(
                "### M1 Base\n\n"
                "### Other section\n\n"
                "#### M1 Testing stage\n"
            )
        self.assertEqual("workflow_topology_duplicate_id", rejected.exception.code)
