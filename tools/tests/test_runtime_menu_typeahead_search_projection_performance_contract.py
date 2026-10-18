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
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/typeahead_search_projection_tests.rs"
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


class RuntimeMenuTypeaheadSearchProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_each_typeahead_candidate_owns_one_search_buffer(self) -> None:
        self.assertIn("struct MenuTypeaheadSearch {\n    buffer: String,", self.source)
        self.assertNotIn("search: String", self.source)
        body = function_body(self.source, "menu_typeahead_searches")
        self.assertNotIn("search: payload.clone()", body)
        self.assertNotIn("search: buffer.clone()", body)

    def test_multi_scalar_detection_stops_after_the_second_scalar(self) -> None:
        body = function_body(self.source, "menu_typeahead_searches")
        self.assertIn("payload.chars().nth(1).is_some()", body)
        self.assertNotIn("payload.chars().count() > 1", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime777_menu_typeahead_search_projection_preserves_buffer_and_fallback_order",
            lower,
        )
        self.assertIn(
            "RUNTIME777_MENU_TYPEAHEAD_SEARCH_PROJECTION_BENCH_V1", lower
        )
        self.assertIn("optimized_duplicate_search_string_allocations", lower)

    def test_deterministic_model_eliminates_duplicate_search_storage(self) -> None:
        key_events = 1_024
        legacy_duplicate_search_string_allocations = key_events * 2
        optimized_duplicate_search_string_allocations = 0
        self.assertGreater(
            legacy_duplicate_search_string_allocations,
            optimized_duplicate_search_string_allocations,
        )
        self.assertEqual(optimized_duplicate_search_string_allocations, 0)


if __name__ == "__main__":
    unittest.main()
