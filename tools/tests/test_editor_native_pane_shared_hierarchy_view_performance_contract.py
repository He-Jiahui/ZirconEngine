from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
BASE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "native_panes"
)
CONTENT = BASE / "content.rs"
HIERARCHY = BASE / "hierarchy.rs"
SCROLLBAR = BASE / "scrollbar.rs"
ROW = BASE / "hierarchy/row.rs"
ROW_FRAME = BASE / "hierarchy/row/frame.rs"


class EditorNativePaneSharedHierarchyViewPerformanceContractTests(unittest.TestCase):
    def test_content_owner_computes_one_hierarchy_view_and_metric_snapshot(self) -> None:
        source = CONTENT.read_text(encoding="utf-8")
        branch = source.split('"Hierarchy" => {', 1)[1]
        branch = branch.split('"Assets" => {', 1)[0]

        self.assertEqual(branch.count("hierarchy_viewport_frame(pane, body)"), 1)
        self.assertEqual(branch.count("current_hierarchy_row_metrics()"), 1)
        self.assertIn("&viewport", branch)
        self.assertIn("row_metrics", branch)

    def test_hierarchy_rows_consume_precomputed_view_and_metrics(self) -> None:
        source = HIERARCHY.read_text(encoding="utf-8")
        function = source.split("fn draw_hierarchy_rows", 1)[1]
        function = function.split("fn inline_hierarchy_rename_value", 1)[0]

        self.assertIn("viewport: &FrameRect", function)
        self.assertIn("row_metrics: HierarchyRowMetrics", function)
        self.assertNotIn("hierarchy_viewport_frame(pane, body)", function)

    def test_hierarchy_scrollbar_consumes_precomputed_view_and_metrics(self) -> None:
        source = SCROLLBAR.read_text(encoding="utf-8")
        function = source.split("fn draw_hierarchy_scrollbar", 1)[1]
        function = function.split("fn draw_activity_asset_tree_scrollbar", 1)[0]

        self.assertIn("viewport: &FrameRect", function)
        self.assertIn("row_metrics: HierarchyRowMetrics", function)
        self.assertNotIn("hierarchy_viewport_frame", function)
        self.assertNotIn("current_hierarchy_row_metrics", function)

    def test_each_visible_row_reuses_the_metric_snapshot(self) -> None:
        row = ROW.read_text(encoding="utf-8")
        frame = ROW_FRAME.read_text(encoding="utf-8")

        self.assertIn("row_metrics: HierarchyRowMetrics", row)
        self.assertIn("metrics: HierarchyRowMetrics", frame)
        self.assertNotIn("current_hierarchy_row_metrics", frame)


if __name__ == "__main__":
    unittest.main()
