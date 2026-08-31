from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TIMELINE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/performance_timeline.rs"
)


class EditorPerformanceTimelineMaterializationContractTests(unittest.TestCase):
    def test_timeline_rows_are_indexed_from_the_visible_clip_window(self) -> None:
        source = TIMELINE.read_text(encoding="utf-8")

        self.assertIn("fn visible_row_range(", source)
        for function_name in ("frame_row_nodes", "span_row_nodes", "hotspot_row_nodes"):
            start = source.index(f"fn {function_name}(")
            end = source.index("\nfn ", start + 1)
            body = source[start:end]
            self.assertIn("visible_row_range(", body)
            self.assertIn(".row_data(row)", body)
            self.assertNotIn(".iter().enumerate()", body)

    def test_large_logical_timelines_have_a_bounded_rust_regression(self) -> None:
        source = TIMELINE.read_text(encoding="utf-8")

        self.assertIn("large_timeline_materializes_only_rows_intersecting_the_list_clip", source)
        self.assertIn("const LOGICAL_ROWS: usize = 10_000", source)
        self.assertIn("nodes.len() < 100", source)


if __name__ == "__main__":
    unittest.main()
