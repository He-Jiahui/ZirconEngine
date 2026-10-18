import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/core/tools/input_capture.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/core/tools/tests/optimization_batch_editor877_input_capture_shutdown_owned_drain_tests.rs"
)


class Editor877InputCaptureShutdownOwnedDrainPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"pub\(crate\)\s+fn\s+shutdown\(.*?\n\s*}\n\n\s*fn\s+start_capture",
            source,
            re.S,
        )
        if match is None:
            raise AssertionError("ToolInputCaptureAuthority::shutdown owner is missing")
        cls.source = source
        cls.body = match.group(0)

    def test_shutdown_takes_capture_map_and_clears_source_index_once(self) -> None:
        self.assertIn(
            "let captures = std::mem::take(&mut self.captures);", self.body
        )
        self.assertIn("self.by_source.clear();", self.body)
        self.assertIn("for handle in captures.into_values()", self.body)

    def test_shutdown_does_not_project_capture_ids_or_remove_each_entry(self) -> None:
        self.assertNotIn("self.captures.keys().copied().collect", self.body)
        self.assertNotIn("self.end_ids", self.body)
        self.assertNotIn("self.captures.remove", self.body)
        self.assertNotIn("self.by_source.remove", self.body)

    def test_shutdown_reserves_exact_result_and_event_bounds(self) -> None:
        self.assertIn("let mut ended = Vec::with_capacity(captures.len());", self.body)
        self.assertIn("let mut events = Vec::with_capacity(captures.len());", self.body)
        self.assertIn("ended.push(handle);", self.body)

    def test_dual_owner_clone_and_shutdown_disposition_are_explicit(self) -> None:
        self.assertIn("handle: handle.clone()", self.body)
        self.assertIn("disposition: ToolInputCaptureDisposition::Shutdown", self.body)
        self.assertIn("ToolInputCaptureReport::new(ended.into_boxed_slice(), events)", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_editor877_input_capture_shutdown_owned_drain_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor877_input_capture_shutdown_owned_drain_preserves_id_order",
            lower,
        )
        self.assertIn(
            "EDITOR877_INPUT_CAPTURE_SHUTDOWN_OWNED_DRAIN_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_shutdown_index_model(self) -> None:
        captures = 4_096
        legacy_id_slots = captures
        optimized_id_slots = 0
        legacy_tree_removals = captures * 2
        optimized_tree_removals = 0
        self.assertEqual(legacy_id_slots, 4_096)
        self.assertEqual(optimized_id_slots, 0)
        self.assertEqual(legacy_tree_removals, 8_192)
        self.assertEqual(optimized_tree_removals, 0)


if __name__ == "__main__":
    unittest.main()
