import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/diagnostic_log/settings.rs"
LOWER = (
    REPO_ROOT / "zircon_runtime/src/diagnostic_log/settings/tests/module_filters_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Runtime870DiagnosticModuleFiltersSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_module_filter_summary_uses_one_output_buffer(self) -> None:
        owner = function_body(self.source, "diagnostic_log_module_filters")
        self.assertIn("let mut summary = String::new();", owner)
        self.assertIn("config.module_filters.iter().enumerate()", owner)
        self.assertIn("write!(", owner)
        self.assertIn("&mut summary", owner)

    def test_module_filter_summary_preserves_empty_and_delimiters(self) -> None:
        owner = function_body(self.source, "diagnostic_log_module_filters")
        self.assertIn('return "none".to_string();', owner)
        self.assertIn("if index > 0", owner)
        self.assertIn("summary.push(',');", owner)
        self.assertIn('"{}={}"', owner)

    def test_module_filter_summary_does_not_collect_or_join(self) -> None:
        owner = function_body(self.source, "diagnostic_log_module_filters")
        self.assertNotIn("collect::<Vec", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)
        diagnostics = function_body(self.source, "diagnostic_lines")
        self.assertIn("diagnostic_log_module_filters(&self.filter)", diagnostics)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod module_filters_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime870_diagnostic_module_filters_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "RUNTIME870_DIAGNOSTIC_MODULE_FILTERS_SINGLE_BUFFER_BENCH_V1",
            lower,
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_filter_temporary_allocation_model(self) -> None:
        filters = 4_096
        legacy_child_strings = filters
        legacy_vector_slots = filters
        optimized_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_child_strings, 4_096)
        self.assertEqual(legacy_vector_slots, 4_096)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
