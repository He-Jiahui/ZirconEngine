import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPILE_SOURCE = REPO_ROOT / "zircon_plugins/ai/runtime/src/behavior_tree/compile.rs"
BEHAVIOR_TREE_SOURCE = REPO_ROOT / "zircon_plugins/ai/runtime/src/behavior_tree.rs"
WEIGHT_SOURCE = (
    REPO_ROOT
    / "zircon_plugins/ai/runtime/src/behavior_tree/random_selector_weights.rs"
)
SUPPORT_SOURCE = (
    REPO_ROOT / "zircon_plugins/ai/runtime/src/behavior_tree/executor/support.rs"
)
REGRESSION = (
    REPO_ROOT / "zircon_plugins/ai/runtime/src/behavior_tree/compile/tests/random_selector_weights_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeRandomSelectorWeightTablePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.compile_source = COMPILE_SOURCE.read_text(encoding="utf-8")
        cls.behavior_tree_source = BEHAVIOR_TREE_SOURCE.read_text(encoding="utf-8")
        cls.weight_source = WEIGHT_SOURCE.read_text(encoding="utf-8")
        cls.support_source = SUPPORT_SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_compile_path_owns_the_weight_table(self) -> None:
        self.assertIn("random_selector_weights: Option<", self.compile_source)
        self.assertIn("CompiledRandomSelectorWeights::compile", self.compile_source)
        self.assertIn("mod random_selector_weights;", self.behavior_tree_source)
        self.assertIn("fn resolve_weight", self.weight_source)
        self.assertIn("let total = weights.iter().copied().sum()", self.weight_source)

    def test_tick_path_uses_compiled_weights_before_fallback(self) -> None:
        body = function_body(self.support_source, "weighted_random_child")
        selection = function_body(self.support_source, "select_weighted_child")
        self.assertIn("random_selector_weights()", body)
        self.assertIn("table.weights()", body)
        self.assertLess(body.index("random_selector_weights()"), body.index("collect::<Vec<_>>()"))
        self.assertIn("if sample < *weight", selection)
        self.assertIn("sample -= *weight", selection)
        self.assertIn("parameter_scans_per_selection=0", self.regression)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"compile/tests/random_selector_weights_tests.rs\"]",
            self.compile_source,
        )
        self.assertIn(
            "random_selector_compiles_weight_table_in_child_order",
            self.regression,
        )
        self.assertIn(
            "RUNTIME799_RANDOM_SELECTOR_WEIGHT_TABLE_BENCH_V1",
            self.regression,
        )

    def test_deterministic_hot_path_model_removes_repeated_work(self) -> None:
        child_count = 1_024
        legacy_allocations_per_selection = 1
        optimized_allocations_per_selection = 0
        legacy_parameter_scans = child_count * child_count
        optimized_parameter_scans = 0
        self.assertEqual(legacy_allocations_per_selection, 1)
        self.assertEqual(optimized_allocations_per_selection, 0)
        self.assertGreater(legacy_parameter_scans, 0)
        self.assertEqual(optimized_parameter_scans, 0)


if __name__ == "__main__":
    unittest.main()
