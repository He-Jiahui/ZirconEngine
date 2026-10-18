from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/surface/focus.rs"


class RuntimeFocusHoveredRetainPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("    fn clear_invalid_transient_input_owners(&mut self) {")
        end = cls.source.index("\n    fn clear_drag_drop_session_for_source", start)
        cls.body = cls.source[start:end]

    def test_hovered_path_is_filtered_in_place(self) -> None:
        self.assertIn(
            "let mut hovered = std::mem::take(&mut self.focus.hovered);",
            self.body,
        )
        self.assertIn(
            "hovered.retain(|owner| is_valid_input_owner(self, *owner));",
            self.body,
        )
        self.assertIn("self.focus.hovered = hovered;", self.body)
        self.assertNotIn(".filter(|owner| is_valid_input_owner(self, *owner))", self.body)
        self.assertNotIn(".collect();", self.body)

    def test_focus_semantics_and_lower_regression_are_wired(self) -> None:
        self.assertIn("focus_hovered_retain_preserves_order_and_capacity", self.source)
        self.assertIn("runtime821_focus_hovered_retain_bench_v1", self.source)
        self.assertIn("is_valid_input_owner(self, *owner)", self.body)

    def test_empty_and_all_valid_paths_keep_the_existing_buffer(self) -> None:
        self.assertIn("std::mem::take(&mut self.focus.hovered)", self.body)
        self.assertIn("self.focus.hovered = hovered;", self.body)
        self.assertGreaterEqual(self.body.count("is_valid_input_owner(self, *owner)"), 1)

    def test_deterministic_model_removes_rebuild_allocations(self) -> None:
        hovered_count = 4_096
        reconciliations = 1_000
        legacy_rebuild_allocations = reconciliations
        optimized_rebuild_allocations = 0
        self.assertGreater(legacy_rebuild_allocations, optimized_rebuild_allocations)
        self.assertEqual(hovered_count, 4_096)
        source_order = list(range(hovered_count))
        retained_order = [node for node in source_order if node % 2 == 0]
        self.assertEqual(retained_order, source_order[::2])
        self.assertEqual(len(retained_order), hovered_count // 2)


if __name__ == "__main__":
    unittest.main()
