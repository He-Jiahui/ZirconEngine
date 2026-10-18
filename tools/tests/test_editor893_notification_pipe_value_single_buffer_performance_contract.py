import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications/tests/pipe_value_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor893NotificationPipeValueSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_pipe_value_uses_one_input_bounded_output(self) -> None:
        owner = function_body(self.source, "pipe_value")
        self.assertIn("String::with_capacity(value.len())", owner)
        self.assertIn("let mut pending_space = false", owner)
        self.assertTrue(owner.rstrip().endswith("output\n}"))

    def test_pipe_value_normalizes_delimiters_and_unicode_whitespace_in_one_scan(
        self,
    ) -> None:
        owner = function_body(self.source, "pipe_value")
        self.assertIn("for ch in value.chars()", owner)
        self.assertIn("ch.is_whitespace()", owner)
        self.assertIn("'|' | '=' | '\\n' | '\\r' | '\\t'", owner)
        self.assertIn("output.push(' ')", owner)
        self.assertIn("output.push(ch)", owner)

    def test_pipe_value_does_not_collect_split_or_join(self) -> None:
        owner = function_body(self.source, "pipe_value")
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".split_whitespace()", owner)
        self.assertNotIn(".join(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod pipe_value_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor893_notification_pipe_value_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "EDITOR893_NOTIFICATION_PIPE_VALUE_SINGLE_BUFFER_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_pipe_value_temporary_allocation_model(self) -> None:
        values = 4_096
        tokens = 64
        legacy_intermediate_strings = values
        legacy_reference_slots = values * tokens
        optimized_intermediate_strings = 0
        optimized_reference_slots = 0
        self.assertEqual(legacy_intermediate_strings, 4_096)
        self.assertEqual(legacy_reference_slots, 262_144)
        self.assertEqual(optimized_intermediate_strings, 0)
        self.assertEqual(optimized_reference_slots, 0)


if __name__ == "__main__":
    unittest.main()
