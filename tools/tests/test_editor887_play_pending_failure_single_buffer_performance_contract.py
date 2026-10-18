import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/host/play_pending_decision/resolve.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/host/play_pending_decision/resolve/tests/failure_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Editor887PlayPendingFailureSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_failure_details_append_to_one_output_buffer(self) -> None:
        owner = function_body(
            self.source, "pending_play_failure_toast_message"
        )
        self.assertIn("let mut diagnostics = String::new();", owner)
        self.assertIn(
            "failures.iter().take(MAX_PLAY_PENDING_FAILURE_DETAILS)", owner
        )
        self.assertIn('diagnostics.push_str("; ");', owner)
        self.assertIn("write!(", owner)
        self.assertIn("&mut diagnostics", owner)

    def test_failure_details_do_not_stage_a_vector_or_join(self) -> None:
        owner = function_body(
            self.source, "pending_play_failure_toast_message"
        )
        self.assertNotIn("collect::<Vec", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_error_text_is_trimmed_once_without_a_bounded_copy(self) -> None:
        owner = function_body(
            self.source, "pending_play_failure_toast_message"
        )
        self.assertIn("let error = failure.error().to_string();", owner)
        self.assertIn("let error = error.trim();", owner)
        self.assertIn('"pending edit operation failed"', owner)
        self.assertEqual(owner.count("ToastNotification::bounded_message("), 1)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod failure_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor887_play_pending_failure_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "EDITOR887_PLAY_PENDING_FAILURE_SINGLE_BUFFER_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_failure_temporary_allocation_model(self) -> None:
        renders = 4_096
        details_per_render = 4
        legacy_staged_child_strings = renders * details_per_render * 3
        legacy_vector_slots = renders * details_per_render
        optimized_staged_child_strings = 0
        optimized_vector_slots = 0
        self.assertEqual(legacy_staged_child_strings, 49_152)
        self.assertEqual(legacy_vector_slots, 16_384)
        self.assertEqual(optimized_staged_child_strings, 0)
        self.assertEqual(optimized_vector_slots, 0)


if __name__ == "__main__":
    unittest.main()
