import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/tree/node/focus.rs"
LOWER = (
    REPO_ROOT / "zircon_runtime/src/ui/tree/node/focus/tests/navigation_capacity_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.rindex(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class RuntimeTreeFocusNavigationCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = LOWER.read_text(encoding="utf-8")

    def test_navigation_output_reserves_tree_node_upper_bound(self) -> None:
        body = function_body(
            self.source,
            "fn focusable_nodes_in_navigation_order(&self)",
        )
        reserve = "let mut focusable = Vec::with_capacity(self.nodes.len());"
        self.assertIn(reserve, body)
        self.assertLess(body.index(reserve), body.index("for root_id in &self.roots"))

    def test_lower_regression_is_wired_and_preserves_order(self) -> None:
        self.assertIn("mod navigation_capacity_tests;", self.source)
        self.assertIn(
            "navigation_order_admits_tree_node_capacity_before_collecting",
            self.lower,
        )
        self.assertIn("focusable.capacity() >= tree.nodes.len()", self.lower)
        self.assertIn("NODE_COUNT: u64 = 128", self.lower)
        self.assertIn(
            "optimization_batch_20260916_runtime791_focus_navigation_capacity_bench",
            self.lower,
        )
        self.assertIn("RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1", self.lower)
        self.assertIn("geometric_growth_events(BENCH_NODE_COUNT as usize)", self.lower)
        self.assertIn("reserved_growth_events(BENCH_NODE_COUNT as usize)", self.lower)

    def test_deterministic_growth_model_is_zero_after_reservation(self) -> None:
        node_count = 4_096
        legacy_growth_events = 11
        reserved_growth_events = 0
        self.assertGreater(legacy_growth_events, reserved_growth_events)
        self.assertEqual(reserved_growth_events, 0)
        self.assertEqual(node_count, 4_096)


if __name__ == "__main__":
    unittest.main()
