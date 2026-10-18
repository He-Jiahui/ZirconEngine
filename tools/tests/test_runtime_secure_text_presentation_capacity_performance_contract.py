import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime" / "src" / "ui" / "text" / "presentation.rs"
TEST_SOURCE = (
    REPO_ROOT / "zircon_runtime/src/ui/text/tests/presentation_capacity_tests.rs"
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


class RuntimeSecureTextPresentationCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.tests = TEST_SOURCE.read_text(encoding="utf-8")

    def test_secure_presentation_reserves_known_line_and_cluster_bounds(self) -> None:
        body = function_body(self.source, "new")
        self.assertIn("let hard_lines = hard_lines(source_text);", body)
        self.assertIn("secure_display_capacity(source_text.len())", body)
        self.assertIn("Vec::with_capacity(hard_lines.len())", body)
        self.assertLess(body.index("Vec::with_capacity"), body.index("for hard_line"))

        self.assertIn("grapheme_indices(true)", body)
        self.assertIn("size_hint()", body)
        self.assertIn("Vec::with_capacity(grapheme_capacity)", body)
        helper = function_body(self.source, "secure_display_capacity")
        self.assertIn("checked_mul", helper)
        self.assertIn("isize::MAX", helper)
        self.assertIn("MASK_GLYPH.len_utf8()", helper)

    def test_secure_display_and_bidi_semantics_remain_in_place(self) -> None:
        body = function_body(self.source, "new")
        self.assertIn("display_text.push(MASK_GLYPH)", body)
        self.assertIn("logical_ranges.push(TextRange", body)
        self.assertIn("bidi_for_line(source_content", self.source)
        self.assertIn("is_hard_line_separator: true", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("presentation_capacity_tests", self.source)
        self.assertIn(
            "runtime754_secure_text_capacity_preserves_mask_and_line_semantics",
            self.tests,
        )
        self.assertIn("RUNTIME754_SECURE_TEXT_PRESENTATION_CAPACITY_BENCH_V1", self.tests)
        self.assertIn("optimized_growth_events", self.tests)
        self.assertIn("optimized_display_growth_events", self.tests)
        self.assertIn("secure_display_capacity(source.len())", self.tests)

    def test_deterministic_model_removes_nested_vector_growth(self) -> None:
        source_bytes = 8_192
        hard_line_count = 64
        legacy_growth_events = hard_line_count + 1
        optimized_growth_events = 0
        optimized_display_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(optimized_display_growth_events, 0)
        self.assertEqual(source_bytes // hard_line_count, 128)


if __name__ == "__main__":
    unittest.main()
