import re
import unittest
from pathlib import Path


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    open_brace = source.index("{", start)
    depth = 0
    for index in range(open_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[open_brace + 1 : index]
    raise AssertionError(f"unterminated function body: {signature}")


class EditorWorkbenchNodeVisibilityNonStringPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.source_path = (
            self.repo_root
            / "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/node_index.rs"
        )
        self.source = self.source_path.read_text(encoding="utf-8")
        self.visibility_body = _function_body(self.source, "fn visibility_is(")

    def test_non_string_visibility_values_do_not_format_temporary_strings(self) -> None:
        self.assertRegex(
            self.visibility_body,
            re.compile(
                r"RetainedUiHostValue::Integer\(_\)\s*\|\s*"
                r"RetainedUiHostValue::Float\(_\)\s*\|\s*"
                r"RetainedUiHostValue::Bool\(_\)\s*=> false,"
            ),
        )
        self.assertNotIn("to_string()", self.visibility_body)

    def test_string_and_structural_visibility_semantics_remain_explicit(self) -> None:
        self.assertIn("RetainedUiHostValue::String(value) => normalized_ascii_eq(value, expected)", self.visibility_body)
        self.assertIn(
            "RetainedUiHostValue::Datetime(_)\n        | RetainedUiHostValue::Array(_)\n        | RetainedUiHostValue::Table(_) => expected == \"visible\"",
            self.visibility_body,
        )

    def test_lower_regression_and_benchmark_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"node_index/tests/visibility_nonstring_tests.rs\"]",
            self.source,
        )
        lower = (
            self.repo_root / "zircon_editor/src/ui/retained_host/ui/workbench_window_projection/node_index/tests/visibility_nonstring_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260915_editor789_visibility_is_preserves_non_string_results",
            lower,
        )
        self.assertIn(
            "optimization_batch_20260915_editor789_visibility_is_p95",
            lower,
        )
        self.assertIn("EDITOR789_WORKBENCH_NODE_VISIBILITY_NON_STRING_BENCH_V1", lower)
        self.assertIn("legacy_visibility_is", lower)
        self.assertIn("visibility_is", lower)


if __name__ == "__main__":
    unittest.main()
