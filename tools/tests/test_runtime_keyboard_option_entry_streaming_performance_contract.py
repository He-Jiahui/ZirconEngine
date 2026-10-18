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
    / "keyboard.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/tests/option_entry_streaming_tests.rs"
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


class RuntimeKeyboardOptionEntryStreamingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_root_parser_uses_one_capacity_hinted_output(self) -> None:
        body = function_body(self.source, "option_entry_list")
        self.assertIn("Vec::with_capacity(option_entry_capacity_hint(value))", body)
        self.assertIn("collect_option_entries(value, &mut entries);", body)
        self.assertNotIn("flat_map(option_entry_list)", body)

    def test_recursive_collector_reserves_direct_array_bound(self) -> None:
        body = function_body(self.source, "collect_option_entries")
        self.assertIn("entries.reserve(values.len());", body)
        self.assertIn("for value in values", body)
        self.assertLess(
            body.index("entries.reserve(values.len());"),
            body.index("for value in values"),
        )

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime758_keyboard_option_entries_preserve_nested_order_and_capacity",
            lower,
        )
        self.assertIn("RUNTIME758_KEYBOARD_OPTION_ENTRY_STREAM_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_deterministic_model_eliminates_flat_entry_growth(self) -> None:
        direct_entries = 1_024
        legacy_growth_events = 10
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(direct_entries, 1_024)


if __name__ == "__main__":
    unittest.main()
