from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/v2/style/runtime_state.rs"
LOWER = SOURCE.parent / "runtime_state/tests/capacity_tests.rs"


def growth_events(item_count: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for length in range(1, item_count + 1):
        if length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


class RuntimeRuntimeStateCapacityPerformanceContractTests(unittest.TestCase):
    def test_runtime_collector_reserves_bounded_state_capacity(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        function = source.split(
            "pub(super) fn collect_runtime_pseudo_states", 1
        )[1].split("fn append_resolved_painter_state", 1)[0]
        compact = "".join(function.split())

        self.assertIn(
            "Vec::with_capacity(runtime_pseudo_state_initial_capacity(node,component_state))",
            compact,
        )
        self.assertNotIn("let mut states = Vec::new();", function)
        self.assertIn("collect_true_runtime_state_names", function)
        self.assertIn("append_resolved_painter_state", function)

    def test_capacity_helper_accounts_for_attributes_flags_and_aliases(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        helper = source.split(
            "fn runtime_pseudo_state_initial_capacity", 1
        )[1].split("pub(super) fn collect_runtime_pseudo_states", 1)[0]

        for token in (
            "template_metadata",
            "component_state",
            "focus_visible",
            "state_flags",
            "attributes.len()",
            "RUNTIME_PSEUDO_STATE_RESOLVED_ALIAS_CAPACITY",
        ):
            self.assertIn(token, helper)

    def test_lower_order_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("runtime_pseudo_state_capacity_preserves_order", lower)
        self.assertIn(
            "RUNTIME841_RUNTIME_PSEUDO_STATE_CAPACITY_BENCH_V1", lower
        )

    def test_dense_model_removes_geometric_growth(self) -> None:
        authored = 4_096
        output = authored + 20 + 4 + 2
        self.assertGreater(growth_events(output), 0)
        self.assertEqual(growth_events(output, output), 0)


if __name__ == "__main__":
    unittest.main()
