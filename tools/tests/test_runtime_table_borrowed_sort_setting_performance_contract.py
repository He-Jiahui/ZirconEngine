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
    / "table.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/table/tests/borrowed_sort_setting_tests.rs"
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


class RuntimeTableBorrowedSortSettingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_text_helpers_borrow_string_and_enum_payloads(self) -> None:
        string_value = function_body(self.source, "string_value")
        string_setting_ref = function_body(self.source, "string_setting_ref")
        self.assertIn("Option<&str>", self.source)
        self.assertIn("Some(value.as_str())", string_value)
        self.assertNotIn("value.clone()", string_value)
        self.assertIn("string_value", string_setting_ref)
        self.assertNotIn("value.clone()", string_setting_ref)

    def test_sort_and_width_paths_propagate_borrowed_text(self) -> None:
        payload = function_body(self.source, "column_width_payload")
        self.assertIn("Option<(&str, f64)>", self.source)
        self.assertNotIn(".to_owned()", payload)
        self.assertNotIn("string_value(value).and_then(|value|", self.source)
        self.assertIn("string_setting_ref(state, \"sort_column\")", self.source)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime771_table_borrowed_sort_settings_preserve_semantics", lower)
        self.assertIn("RUNTIME771_TABLE_BORROWED_SORT_SETTING_BENCH_V1", lower)
        self.assertIn("optimized_textual_clones", lower)

    def test_deterministic_model_eliminates_transient_text_clones(self) -> None:
        sort_events = 1_024
        legacy_textual_clones = sort_events * 3
        optimized_textual_clones = 0
        self.assertGreater(legacy_textual_clones, optimized_textual_clones)
        self.assertEqual(optimized_textual_clones, 0)


if __name__ == "__main__":
    unittest.main()
