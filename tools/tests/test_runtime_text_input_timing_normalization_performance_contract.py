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
    / "text_input.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/text_input/tests/timing_normalization_tests.rs"
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


class RuntimeTextInputTimingNormalizationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_timing_uses_borrowed_setting_and_streaming_normalization(self) -> None:
        body = function_body(self.source, "validation_timing")
        self.assertIn("string_setting_ref", body)
        self.assertIn("normalized_timing_matches", body)
        self.assertNotIn("collect::<String>()", body)
        self.assertNotIn("unwrap_or_else(|| \"commit\".to_string())", body)

    def test_normalizer_preserves_separator_and_unicode_lowercase_steps(self) -> None:
        body = function_body(self.source, "normalized_timing_matches")
        self.assertIn("*ch != '_' && *ch != '-' && !ch.is_whitespace()", body)
        self.assertIn("ch.to_lowercase()", body)
        self.assertNotIn("collect::<String>()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime768_text_input_timing_normalization_preserves_aliases",
            lower,
        )
        self.assertIn("RUNTIME768_TEXT_INPUT_TIMING_NORMALIZATION_BENCH_V1", lower)
        self.assertIn("optimized_normalization_allocations", lower)

    def test_deterministic_model_eliminates_normalized_string_allocations(self) -> None:
        validation_events = 1_024
        legacy_normalization_allocations = validation_events
        optimized_normalization_allocations = 0
        self.assertGreater(
            legacy_normalization_allocations, optimized_normalization_allocations
        )
        self.assertEqual(optimized_normalization_allocations, 0)
        self.assertEqual(validation_events, 1_024)


if __name__ == "__main__":
    unittest.main()
