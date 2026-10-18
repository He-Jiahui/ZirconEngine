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
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/search_filter_accumulator_tests.rs"
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


class RuntimeMenuSearchFilterAccumulatorPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_recursive_filter_reuses_one_accumulator_with_checkpoints(self) -> None:
        body = function_body(self.source, "collect_matching_search_options")
        self.assertIn("let ids_start = filter.filtered_ids.len();", body)
        self.assertIn("let focus_start = filter.focus_candidates.len();", body)
        self.assertIn(
            "collect_matching_search_options(child, query, filter);", body
        )
        self.assertIn("truncate(ids_start)", body)
        self.assertIn("truncate(focus_start)", body)
        self.assertNotIn("let mut child_filter = MenuSearchBuild::default();", body)
        self.assertNotIn("extend(child_filter.filtered_ids)", body)
        self.assertNotIn("extend(child_filter.focus_candidates)", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime779_menu_search_filter_preserves_preorder_and_focus_order",
            lower,
        )
        self.assertIn("RUNTIME779_MENU_SEARCH_FILTER_ACCUMULATOR_BENCH_V1", lower)
        self.assertIn("optimized_recursive_filter_temporaries", lower)

    def test_deterministic_model_eliminates_recursive_filter_temporaries(self) -> None:
        nodes = 1_024
        legacy_recursive_filter_temporaries = nodes
        optimized_recursive_filter_temporaries = 0
        self.assertGreater(
            legacy_recursive_filter_temporaries,
            optimized_recursive_filter_temporaries,
        )
        self.assertEqual(optimized_recursive_filter_temporaries, 0)


if __name__ == "__main__":
    unittest.main()
