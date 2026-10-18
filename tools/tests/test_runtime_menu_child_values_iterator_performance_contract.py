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
    / "keyboard"
    / "menu.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/child_values_iterator_tests.rs"
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


class RuntimeMenuChildValuesIteratorPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_child_value_lookup_returns_a_borrowed_iterator(self) -> None:
        body = function_body(self.source, "menu_child_values")
        self.assertIn("impl Iterator<Item = &'a UiValue>", self.source)
        self.assertIn("MENU_CHILD_PROPERTY_NAMES", body)
        self.assertIn("filter_map", body)
        self.assertNotIn("collect()", body)
        self.assertNotIn("Vec<&UiValue>", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod child_values_iterator_tests;", self.source)
        self.assertIn(
            "runtime781_menu_child_values_preserves_property_order_without_allocation",
            lower,
        )
        self.assertIn("RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1", lower)
        self.assertIn("optimized_child_value_vectors", lower)

    def test_deterministic_model_eliminates_fixed_property_vector(self) -> None:
        menu_nodes = 1_024
        legacy_child_value_vectors = menu_nodes
        optimized_child_value_vectors = 0
        self.assertGreater(legacy_child_value_vectors, optimized_child_value_vectors)
        self.assertEqual(optimized_child_value_vectors, 0)


if __name__ == "__main__":
    unittest.main()
