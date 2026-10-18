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
    / "text_search.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/text_search/tests/streaming_contains_tests.rs"
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


class RuntimeTextSearchContainsStreamingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_unicode_contains_uses_streaming_helper(self) -> None:
        body = function_body(self.source, "contains_lowercase_query")
        self.assertIn("unicode_lowercase_contains", body)
        self.assertNotIn("value.to_lowercase().contains", body)

    def test_streaming_contains_restarts_without_materializing_text(self) -> None:
        body = function_body(self.source, "unicode_lowercase_contains")
        self.assertIn("lowered.clone()", body)
        self.assertIn("flat_map(char::to_lowercase)", body)
        self.assertNotIn("collect::<String>()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime774_text_search_streaming_contains_preserves_unicode_case",
            lower,
        )
        self.assertIn("RUNTIME774_TEXT_SEARCH_STREAMING_CONTAINS_BENCH_V1", lower)
        self.assertIn("optimized_lowercase_allocations", lower)

    def test_deterministic_model_removes_unicode_contains_allocations(self) -> None:
        search_events = 1_024
        legacy_lowercase_allocations = search_events
        optimized_lowercase_allocations = 0
        self.assertGreater(
            legacy_lowercase_allocations, optimized_lowercase_allocations
        )
        self.assertEqual(optimized_lowercase_allocations, 0)


if __name__ == "__main__":
    unittest.main()
