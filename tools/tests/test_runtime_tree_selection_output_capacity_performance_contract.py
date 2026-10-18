import unittest
from pathlib import Path


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for index in range(open_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[open_brace + 1 : index]
    raise AssertionError(f"unterminated function body: {signature}")


class RuntimeTreeSelectionOutputCapacityPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.model_path = (
            self.repo_root
            / "zircon_runtime/src/ui/component/state_reducer/state_model.rs"
        )
        self.index_path = (
            self.repo_root
            / "zircon_runtime/src/ui/component/state_reducer/state_model/tree_index.rs"
        )
        self.model_source = self.model_path.read_text(encoding="utf-8")
        self.index_source = self.index_path.read_text(encoding="utf-8")
        self.selection_body = _function_body(
            self.model_source, "fn apply_tree_selection_event("
        )

    def test_selection_projection_reserves_indexed_selected_count(self) -> None:
        self.assertIn(
            "Vec::with_capacity(self.tree_index().selected_count())",
            self.selection_body,
        )
        self.assertIn(
            "next_selected.extend(self.tree_index().selected_ids_ordered().map(str::to_owned));",
            self.selection_body,
        )
        self.assertNotIn(
            "selected_ids_ordered()\n                .map(str::to_owned)\n                .collect()",
            self.selection_body,
        )

    def test_tree_index_exposes_exact_selected_upper_bound(self) -> None:
        self.assertIn(
            "pub(crate) fn selected_count(&self) -> usize {\n        self.selected.len()\n    }",
            self.index_source,
        )

    def test_lower_regression_and_benchmark_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"state_model/tests/selected_output_capacity_tests.rs\"]",
            self.model_source,
        )
        lower = (
            self.repo_root / "zircon_runtime/src/ui/component/state_reducer/state_model/tests/selected_output_capacity_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260915_runtime788_tree_selection_output_preserves_order",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260915_runtime788_tree_selection_output_p95",
            lower,
        )
        self.assertIn("RUNTIME788_TREE_SELECTION_OUTPUT_CAPACITY_BENCH_V1", lower)
        self.assertIn("legacy_growth_events", lower)
        self.assertIn("reserved_growth_events", lower)


if __name__ == "__main__":
    unittest.main()
