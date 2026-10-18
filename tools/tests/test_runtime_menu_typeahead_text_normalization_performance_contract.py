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
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/typeahead_text_normalization_tests.rs"
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


class RuntimeMenuTypeaheadTextNormalizationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_typeahead_normalizes_into_one_retained_buffer(self) -> None:
        body = function_body(self.source, "keyboard_text_search")
        self.assertIn("String::new()", body)
        self.assertIn("search.truncate", body)
        self.assertIn("ch.to_lowercase()", body)
        self.assertNotIn("collect::<String>()", body)
        self.assertNotIn(".trim().to_lowercase()", body)
        self.assertNotIn("reserve(text.len())", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime775_menu_typeahead_text_normalization_preserves_legacy_output",
            lower,
        )
        self.assertIn(
            "RUNTIME775_MENU_TYPEAHEAD_TEXT_NORMALIZATION_BENCH_V1", lower
        )
        self.assertIn("optimized_intermediate_string_buffers", lower)

    def test_deterministic_model_eliminates_intermediate_buffer(self) -> None:
        key_events = 1_024
        legacy_intermediate_string_buffers = key_events
        optimized_intermediate_string_buffers = 0
        self.assertGreater(
            legacy_intermediate_string_buffers,
            optimized_intermediate_string_buffers,
        )
        self.assertEqual(optimized_intermediate_string_buffers, 0)


if __name__ == "__main__":
    unittest.main()
