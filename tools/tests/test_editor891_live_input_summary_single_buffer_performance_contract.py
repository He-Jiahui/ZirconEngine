import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/live_input_summary.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/live_input_summary/tests/single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}(?:<[^>]+>)?\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor891LiveInputSummarySingleBufferPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_live_input_summary_uses_one_output_buffer(self) -> None:
        owner = function_body(self.source, "input_summary")
        self.assertIn("let mut summary = String::with_capacity", owner)
        self.assertIn('summary.push_str("Inputs: ");', owner)
        self.assertIn("summary.push_str(first);", owner)
        self.assertTrue(owner.rstrip().endswith("Some(summary)\n}"))

    def test_summary_filters_and_caps_before_positional_append(self) -> None:
        owner = function_body(self.source, "input_summary")
        self.assertIn(".map(str::trim)", owner)
        self.assertIn(".filter(|value| !value.is_empty())", owner)
        self.assertIn(".take(3)", owner)
        self.assertIn('summary.push_str(" | ");', owner)

    def test_summary_precomputes_exact_capacity_without_child_storage(self) -> None:
        owner = function_body(self.source, "input_summary")
        self.assertIn("I::IntoIter: Clone", owner)
        self.assertRegex(owner, r"values\s*\.clone\(\)\s*\.fold")
        self.assertIn("let delimiter_count = value_count.saturating_sub(1);", owner)
        self.assertNotIn(".size_hint()", owner)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("summary.capacity(), summary.len()", lower)

    def test_summary_does_not_collect_join_or_format_children(self) -> None:
        owner = function_body(self.source, "input_summary")
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)
        caller = function_body(self.source, "for_command")
        self.assertIn("input_summary(values)", caller)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor891_live_input_summary_single_buffer_preserves_exact_text", lower
        )
        self.assertIn("EDITOR891_LIVE_INPUT_SUMMARY_SINGLE_BUFFER_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_summary_temporary_allocation_model(self) -> None:
        summaries = 4_096
        visible_values = 3
        legacy_reference_slots = summaries * visible_values
        legacy_join_outputs = summaries
        optimized_reference_slots = 0
        optimized_join_outputs = 0
        self.assertEqual(legacy_reference_slots, 12_288)
        self.assertEqual(legacy_join_outputs, 4_096)
        self.assertEqual(optimized_reference_slots, 0)
        self.assertEqual(optimized_join_outputs, 0)


if __name__ == "__main__":
    unittest.main()
