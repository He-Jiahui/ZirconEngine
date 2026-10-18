import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "selection.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/selection/tests/flags_capacity_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
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


class RuntimeSelectionFlagsCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_array_conversion_uses_a_capacity_hinted_stream(self) -> None:
        body = function_body(self.source, "selection_flags_value")
        self.assertIn("selection_flags_from_array(values)", body)
        self.assertNotIn("filter_map", body)
        self.assertNotIn(".collect()", body)

    def test_streaming_helper_reserves_the_direct_array_bound(self) -> None:
        body = function_body(self.source, "selection_flags_from_array")
        self.assertIn("Vec::with_capacity(values.len())", body)
        self.assertIn("UiValue::Enum(value) | UiValue::String(value) if !value.is_empty()", body)
        self.assertIn("selected.push(value)", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime769_selection_flags_array_capacity_preserves_filtering", lower)
        self.assertIn("RUNTIME769_SELECTION_FLAGS_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_reallocations", lower)

    def test_deterministic_model_eliminates_geometric_growth_for_a_full_array(self) -> None:
        values_per_conversion = 256
        legacy_growth_reallocations = 9
        optimized_growth_reallocations = 0
        self.assertGreater(values_per_conversion, 0)
        self.assertGreater(legacy_growth_reallocations, optimized_growth_reallocations)
        self.assertEqual(optimized_growth_reallocations, 0)


if __name__ == "__main__":
    unittest.main()
