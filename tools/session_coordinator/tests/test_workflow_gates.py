from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.session_coordinator.database import Database
from tools.session_coordinator.migrations import migrate
from tools.session_coordinator.models import CoordinatorError, WorkflowNodeState
from tools.session_coordinator.sessions import SessionService
from tools.session_coordinator.tests.helpers import init_repo
from tools.session_coordinator.workflows.gates import (
    GateContext,
    GateEvidenceStore,
    MilestoneGateEvaluator,
)
from tools.session_coordinator.workflows.plan_import import TopologyImporter
from tools.session_coordinator.workflows.store import WorkflowStore


class WorkflowGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        root = Path(self.temporary.name)
        self.repo = init_repo(root / "repo")
        plan = self.repo / "docs/plans/runtime/01-control.md"
        plan.parent.mkdir(parents=True)
        plan.write_text(
            "```zircon-workflow\n"
            '{"schema":1,"workflow_id":"runtime-control","goal":"Runtime",'
            '"milestones":[{"id":"M1","title":"Base","depends_on":[]}]}\n'
            "```\n\n"
            "## Milestone M1: Base\n\n"
            "- [ ] **M1.1 Add storage.** details\n",
            encoding="utf-8",
        )
        self.database = Database(root / "state.sqlite3")
        migrate(self.database)
        sessions = SessionService(self.database, self.repo)
        sessions.register(
            session_id="session-a", plan_path="docs/plans/runtime/01-control.md"
        )
        sessions.register(session_id="reviewer-b")
        imported = TopologyImporter(self.database, self.repo).import_plan(
            "session-a", "docs/plans/runtime/01-control.md"
        )
        self.run_id = imported.run_id
        self.topology_version_id = imported.topology_version_id
        self.store = WorkflowStore(self.database)
        self.nodes = {node.node_key: node for node in self.store.nodes(self.run_id)}
        self.evidence = GateEvidenceStore(self.database)
        self.evaluator = MilestoneGateEvaluator(self.database)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _context(self) -> GateContext:
        return GateContext(
            topology_version_id=self.topology_version_id,
            head_commit="a" * 40,
            baseline_epoch=7,
            manifest_hash="b" * 64,
            failure_revision="failure-r1",
            plan_content_hash="c" * 64,
        )

    def test_all_current_evidence_allows_milestone(self) -> None:
        self.store.append_attempt(
            self.nodes["M1.1"].node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0}
        )
        context = self._context()
        fingerprint = self.evaluator.input_fingerprint(self.run_id, "M1", context)
        milestone_node_id = self.nodes["M1"].node_id
        self.evidence.record_review(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            reviewer="reviewer-b",
            executor="session-a",
            critical_count=0,
            important_count=0,
            summary="clean",
            input_fingerprint=fingerprint,
            node_id=milestone_node_id,
        )
        for kind in MilestoneGateEvaluator.REQUIRED_GATE_KINDS:
            self.evidence.record_gate(
                run_id=self.run_id,
                topology_version_id=self.topology_version_id,
                gate_kind=kind,
                decision="accepted",
                decision_code="accepted",
                input_fingerprint=fingerprint,
                actor="test",
                node_id=milestone_node_id,
            )

        decision = self.evaluator.evaluate(self.run_id, "M1", context)

        self.assertTrue(decision.allowed)
        self.assertEqual("milestone_gate_allowed", decision.code)
        self.assertEqual(1, len(decision.current_attempt_ids))

    def test_retry_makes_previous_gate_evidence_stale(self) -> None:
        self.store.append_attempt(
            self.nodes["M1.1"].node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0}
        )
        context = self._context()
        fingerprint = self.evaluator.input_fingerprint(self.run_id, "M1", context)
        milestone_node_id = self.nodes["M1"].node_id
        self.evidence.record_review(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            reviewer="reviewer-b",
            executor="session-a",
            critical_count=0,
            important_count=0,
            summary="clean",
            input_fingerprint=fingerprint,
            node_id=milestone_node_id,
        )
        for kind in MilestoneGateEvaluator.REQUIRED_GATE_KINDS:
            self.evidence.record_gate(
                run_id=self.run_id,
                topology_version_id=self.topology_version_id,
                gate_kind=kind,
                decision="accepted",
                decision_code="accepted",
                input_fingerprint=fingerprint,
                actor="test",
                node_id=milestone_node_id,
            )
        self.store.append_attempt(
            self.nodes["M1.1"].node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0, "retry": True}
        )

        decision = self.evaluator.evaluate(self.run_id, "M1", context)

        self.assertFalse(decision.allowed)
        self.assertEqual("milestone_gate_stale_evidence", decision.code)

    def test_explicitly_skipped_slice_is_not_a_blocker(self) -> None:
        self.store.append_attempt(
            self.nodes["M1.1"].node_id,
            WorkflowNodeState.SKIPPED,
            {"reason": "not applicable"},
        )
        decision = self.evaluator.evaluate(self.run_id, "M1", self._context())
        self.assertNotIn(self.nodes["M1.1"].node_id, decision.blocking_node_ids)

    def test_review_requires_independence_and_zero_high_findings(self) -> None:
        with self.assertRaises(CoordinatorError) as same_actor:
            self.evidence.record_review(
                run_id=self.run_id,
                topology_version_id=self.topology_version_id,
                reviewer="session-a",
                executor="session-a",
                critical_count=0,
                important_count=0,
                summary="clean",
                input_fingerprint="fingerprint",
            )
        self.assertEqual("workflow_review_not_independent", same_actor.exception.code)

        review = self.evidence.record_review(
            run_id=self.run_id,
            topology_version_id=self.topology_version_id,
            reviewer="reviewer-b",
            executor="session-a",
            critical_count=0,
            important_count=1,
            summary="one important finding",
            input_fingerprint="fingerprint",
        )
        self.assertEqual("rejected", review.verdict)

    def test_review_gate_requires_matching_independent_review_record(self) -> None:
        self.store.append_attempt(
            self.nodes["M1.1"].node_id, WorkflowNodeState.SUCCEEDED, {"exit": 0}
        )
        context = self._context()
        fingerprint = self.evaluator.input_fingerprint(self.run_id, "M1", context)
        milestone_node_id = self.nodes["M1"].node_id
        for kind in MilestoneGateEvaluator.REQUIRED_GATE_KINDS:
            self.evidence.record_gate(
                run_id=self.run_id,
                topology_version_id=self.topology_version_id,
                node_id=milestone_node_id,
                gate_kind=kind,
                decision="accepted",
                decision_code="accepted",
                input_fingerprint=fingerprint,
                actor="test",
            )

        decision = self.evaluator.evaluate(self.run_id, "M1", context)

        self.assertFalse(decision.allowed)
        self.assertEqual("milestone_gate_review_missing", decision.code)


if __name__ == "__main__":
    unittest.main()
