from __future__ import annotations

import inspect
import sqlite3
import unittest

from tools.session_coordinator.workflows.projections import WorkflowProjectionService


class WorkflowDetailBatchedProjectionPerformanceContractTests(unittest.TestCase):
    def test_detail_batches_attempt_and_eligibility_queries(self) -> None:
        source = inspect.getsource(WorkflowProjectionService.workflow_detail)

        self.assertIn("attempts_by_node", source)
        self.assertEqual(source.count("FROM workflow_attempts"), 1)
        self.assertNotIn("WHERE node_id = ? ORDER BY attempt_number", source)
        self.assertIn(
            "ORDER BY attempt.node_id, attempt.attempt_number", source
        )
        self.assertNotIn("ORDER BY node.stage, node.node_key", source)
        self.assertIn(
            "commit_eligibilities = self._commit_eligibilities(", source
        )
        self.assertNotIn("self._commit_eligibility(", source)

    def test_batched_eligibility_ranks_latest_evidence_once(self) -> None:
        source = inspect.getsource(WorkflowProjectionService._commit_eligibilities)

        self.assertIn("ROW_NUMBER() OVER", source)
        self.assertIn(
            "PARTITION BY source.node_id, source.gate_kind", source
        )
        self.assertNotIn("SELECT latest.rowid", source)

    def test_batched_eligibility_preserves_per_node_gate_results(self) -> None:
        connection = sqlite3.connect(":memory:")
        self.addCleanup(connection.close)
        connection.row_factory = sqlite3.Row
        connection.executescript(
            """
            CREATE TABLE workflow_nodes(
                node_id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL,
                kind TEXT NOT NULL
            );
            CREATE TABLE workflow_gate_evidence(
                run_id TEXT NOT NULL,
                topology_version_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                gate_kind TEXT NOT NULL,
                decision TEXT NOT NULL,
                input_fingerprint TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE workflow_review_evidence(
                run_id TEXT NOT NULL,
                topology_version_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                input_fingerprint TEXT NOT NULL,
                verdict TEXT NOT NULL
            );
            """
        )
        connection.executemany(
            "INSERT INTO workflow_nodes VALUES (?, 'run-a', 'milestone')",
            (("milestone-ready",), ("milestone-missing",)),
        )
        required = (
            "validation",
            "review",
            "failure_audit",
            "plan_output",
            "commit_manifest",
        )
        connection.executemany(
            "INSERT INTO workflow_gate_evidence VALUES ('run-a', 'topology-a', ?, ?, 'accepted', 'fingerprint-a', ?)",
            (
                ("milestone-ready", gate_kind, f"2026-08-31T00:00:0{index}Z")
                for index, gate_kind in enumerate(required)
            ),
        )
        connection.execute(
            "INSERT INTO workflow_gate_evidence VALUES ('run-a', 'topology-a', 'milestone-missing', 'validation', 'accepted', 'fingerprint-b', '2026-08-31T00:00:00Z')"
        )
        connection.execute(
            "INSERT INTO workflow_review_evidence VALUES ('run-a', 'topology-a', 'milestone-ready', 'fingerprint-a', 'accepted')"
        )
        statements: list[str] = []
        connection.set_trace_callback(statements.append)

        results = WorkflowProjectionService._commit_eligibilities(
            connection,
            "run-a",
            ("milestone-ready", "milestone-missing"),
            "topology-a",
        )

        self.assertTrue(results["milestone-ready"]["eligible"])
        self.assertFalse(results["milestone-missing"]["eligible"])
        self.assertEqual(
            ["commit_manifest", "failure_audit", "plan_output", "review"],
            results["milestone-missing"]["missing"],
        )
        selects = [statement for statement in statements if statement.lstrip().startswith("SELECT")]
        self.assertEqual(2, len(selects))


if __name__ == "__main__":
    unittest.main()
