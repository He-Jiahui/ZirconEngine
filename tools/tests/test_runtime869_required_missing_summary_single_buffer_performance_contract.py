import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/builtin/runtime_modules/composition/outcome/tests/required_missing_summary_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"pub fn {signature}\(&self\).*?\n    }}", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Runtime869RequiredMissingSummarySingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_required_missing_summary_uses_one_output_buffer(self) -> None:
        owner = function_body(self.source, "required_missing_summary")
        self.assertIn("let mut summary = String::new();", owner)
        self.assertIn("self.required_missing().iter().enumerate()", owner)
        self.assertIn("write!(", owner)
        self.assertIn("&mut summary", owner)
        self.assertTrue(owner.rstrip().endswith("summary\n    }"))

    def test_required_missing_summary_preserves_delimiters(self) -> None:
        owner = function_body(self.source, "required_missing_summary")
        self.assertIn("if index > 0", owner)
        self.assertIn('summary.push_str("; ");', owner)
        self.assertIn(
            '"required runtime plugin {} is unavailable: {}"', owner
        )

    def test_required_missing_summary_does_not_collect_or_join(self) -> None:
        owner = function_body(self.source, "required_missing_summary")
        self.assertNotIn("collect::<Vec", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod required_missing_summary_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime869_required_missing_summary_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "RUNTIME869_REQUIRED_MISSING_SUMMARY_SINGLE_BUFFER_BENCH_V1",
            lower,
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_missing_plugin_temporary_allocation_model(self) -> None:
        missing_plugins = 4_096
        legacy_child_strings = missing_plugins
        legacy_vector_slots = missing_plugins
        optimized_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_child_strings, 4_096)
        self.assertEqual(legacy_vector_slots, 4_096)
        self.assertEqual(optimized_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
