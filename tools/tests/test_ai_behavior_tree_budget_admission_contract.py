import unittest
from pathlib import Path


class AiBehaviorTreeBudgetAdmissionContractTests(unittest.TestCase):
    """Static guard for bounded, deterministic behavior-tree evaluation."""

    def setUp(self) -> None:
        root = Path(__file__).resolve().parents[2]
        self.executor = root / "zircon_plugins/ai/runtime/src/behavior_tree/executor.rs"
        self.support = root / "zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs"
        self.tests = root / "zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs"
        self.plan = root / "docs/plans/astra/optimize/01/2026-09-12-ai-behavior-budget-admission.md"

    def test_executor_has_private_node_and_depth_budgets_with_per_tick_counters(self) -> None:
        source = self.executor.read_text(encoding="utf-8")

        for anchor in (
            "const DEFAULT_BEHAVIOR_TREE_NODE_EVALUATION_BUDGET: usize = 4_096;",
            "const DEFAULT_BEHAVIOR_TREE_EVALUATION_DEPTH_BUDGET: usize = 256;",
            "node_evaluations: usize,",
            "evaluation_depth: usize,",
            "fn admit_node(&mut self)",
            "if self.evaluation_depth >= DEFAULT_BEHAVIOR_TREE_EVALUATION_DEPTH_BUDGET",
            "if self.node_evaluations >= DEFAULT_BEHAVIOR_TREE_NODE_EVALUATION_BUDGET",
            "self.evaluation_depth += 1;",
            "self.node_evaluations += 1;",
            "fn leave_node(&mut self)",
            "self.evaluation_depth -= 1;",
        ):
            self.assertIn(anchor, source)

        self.assertEqual(source.count("node_evaluations: 0,"), 2)
        self.assertEqual(source.count("evaluation_depth: 0,"), 2)
        self.assertLess(
            source.index("if self.evaluation_depth >= DEFAULT_BEHAVIOR_TREE_EVALUATION_DEPTH_BUDGET"),
            source.index("if self.node_evaluations >= DEFAULT_BEHAVIOR_TREE_NODE_EVALUATION_BUDGET"),
        )

    def test_evaluate_node_admits_before_semantics_and_returns_typed_blocked_result(self) -> None:
        source = self.executor.read_text(encoding="utf-8")
        support = self.support.read_text(encoding="utf-8")

        start = source.index("fn evaluate_node(")
        end = source.index("fn evaluate_sequence(", start)
        body = source[start:end]
        for anchor in (
            "let node = tree.node(node_index as usize);",
            "if let Err(admission_error) = context.admit_node()",
            "return blocked(node.id(), admission_error.diagnostic_reason());",
            "let result = match node.semantics()",
            "context.leave_node();",
        ):
            self.assertIn(anchor, body)

        self.assertLess(body.index("context.admit_node()"), body.index("node.semantics()"))
        self.assertLess(body.index("node.semantics()"), body.index("context.leave_node();"))
        for anchor in (
            "pub(super) fn blocked(node_id: &str, reason: &'static str)",
            "status: AiDecisionStatus::Blocked,",
            "active_node: Some(node_id.to_string()),",
            "diagnostic: Some(format!(\"AI behavior node `{node_id}` {reason}\")),",
        ):
            self.assertIn(anchor, support)

    def test_parallel_execution_stops_at_first_blocked_child(self) -> None:
        source = self.executor.read_text(encoding="utf-8")
        start = source.index("fn evaluate_parallel(")
        end = source.index("fn evaluate_decorator(", start)
        body = source[start:end]

        for anchor in (
            "let mut first_blocked_child = None;",
            "AiDecisionStatus::Blocked => {",
            "first_blocked_child.get_or_insert(*child);",
            "break;",
            "if let Some(child) = first_blocked_child",
            ".terminal_children\n            .clear();",
            "return result;",
        ):
            self.assertIn(anchor, body)

        blocked = body.index("if let Some(child) = first_blocked_child")
        return_result = body.index("return result;", blocked)
        self.assertLess(blocked, return_result)
        self.assertLess(body.index("first_blocked_child.get_or_insert(*child);"), blocked)

    def test_regressions_cover_depth_node_and_under_budget_repeatability(self) -> None:
        tests = self.tests.read_text(encoding="utf-8")
        plan = self.plan.read_text(encoding="utf-8")

        for test_name in (
            "fn behavior_tree_depth_budget_returns_deterministic_blocked_result()",
            "fn behavior_tree_node_budget_returns_deterministic_blocked_result()",
            "fn behavior_tree_budget_preserves_wide_tree_below_limit()",
        ):
            self.assertIn(test_name, tests)

        for anchor in (
            "const DEPTH: usize = 300;",
            "const LEAF_COUNT: usize = 4_100;",
            "const LEAF_COUNT: usize = 2_048;",
            "assert_eq!(first.status, AiDecisionStatus::Blocked);",
            "assert_eq!(first.active_node, second.active_node);",
            "assert_eq!(first.diagnostic, second.diagnostic);",
            "evaluation depth budget",
            "node evaluation budget",
        ):
            self.assertIn(anchor, tests)

        for anchor in (
            "DEFAULT_BEHAVIOR_TREE_NODE_EVALUATION_BUDGET = 4_096",
            "DEFAULT_BEHAVIOR_TREE_EVALUATION_DEPTH_BUDGET = 256",
            "300-node sequence regression",
            "4,100-leaf parallel regression",
            "2,048-leaf parallel regression",
            "preserving the first-blocked result",
        ):
            self.assertIn(anchor, plan)

        self.assertIn("does not yet separately budget probe", plan)
        self.assertIn("wall-clock deadline", plan)


if __name__ == "__main__":
    unittest.main()
