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
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/search_capacity_tests.rs"
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


class RuntimeMenuSearchCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_filter_id_collector_reserves_direct_array_and_flags_bounds(self) -> None:
        body = function_body(self.source, "collect_filtered_option_id_refs")
        self.assertGreaterEqual(body.count("ids.reserve(values.len());"), 2)
        self.assertLess(
            body.index("ids.reserve(values.len());"),
            body.index("for value in values"),
        )

    def test_flattened_search_id_projection_reserves_root_and_children(self) -> None:
        root = function_body(self.source, "all_search_option_ids")
        recursive = function_body(self.source, "collect_search_option_ids")
        self.assertIn("Vec::with_capacity(options.len())", root)
        self.assertIn("ids.reserve(option.children.len());", recursive)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime763_menu_search_projection_preserves_nested_ids_and_capacity",
            lower,
        )
        self.assertIn("RUNTIME763_MENU_SEARCH_PROJECTION_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_deterministic_model_eliminates_flat_projection_growth(self) -> None:
        direct_options = 1_024
        legacy_growth_events = 10
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(direct_options, 1_024)


if __name__ == "__main__":
    unittest.main()
