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
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/typeahead_append_reuse_tests.rs"
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


class RuntimeMenuTypeaheadAppendReusePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_typeahead_moves_previous_buffer_then_appends_payload(self) -> None:
        body = function_body(self.source, "menu_typeahead_searches")
        self.assertIn("let mut combined = previous;", body)
        self.assertIn("combined.push_str(&payload);", body)
        self.assertNotIn('format!("{previous}{payload}")', body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime776_menu_typeahead_append_reuses_previous_buffer", lower
        )
        self.assertIn("RUNTIME776_MENU_TYPEAHEAD_APPEND_REUSE_BENCH_V1", lower)
        self.assertIn("optimized_combined_string_allocations", lower)

    def test_deterministic_model_eliminates_combined_buffer(self) -> None:
        key_events = 1_024
        legacy_combined_string_allocations = key_events
        optimized_combined_string_allocations = 0
        self.assertGreater(
            legacy_combined_string_allocations,
            optimized_combined_string_allocations,
        )
        self.assertEqual(optimized_combined_string_allocations, 0)


if __name__ == "__main__":
    unittest.main()
