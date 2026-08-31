from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LAYOUT = ROOT / "zircon_editor/src/ui/workbench/asset_content_layout"
NATIVE_PANES = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract"
    / "paint_workbench_renderer/native_panes"
)


class EditorAssetTreeMetadataScrollbarPerformanceContractTests(unittest.TestCase):
    def test_generation_classifies_activity_tree_rows(self) -> None:
        controls = (LAYOUT / "controls.rs").read_text(encoding="utf-8")
        identity = (LAYOUT / "identity.rs").read_text(encoding="utf-8")

        self.assertIn("ACTIVITY_TREE_ROW_CONTROL_ID", controls)
        self.assertIn("ActivityTreeRow", identity)
        self.assertIn("ACTIVITY_TREE_ROW_CONTROL_ID", identity)

    def test_metadata_publishes_tree_count_and_activity_row_addresses(self) -> None:
        metadata = (LAYOUT / "paint_metadata.rs").read_text(encoding="utf-8")

        self.assertIn("activity_tree_rows: Vec<usize>", metadata)
        self.assertIn("fn asset_tree_row_count(", metadata)
        self.assertIn("self.browser_source_tree_groups.len()", metadata)
        self.assertIn("fn activity_tree_node_row(", metadata)

    def test_scrollbar_count_is_a_metadata_lookup(self) -> None:
        asset = (NATIVE_PANES / "scrollbar/asset.rs").read_text(encoding="utf-8")
        production = asset.split("#[cfg(test)]", 1)[0]
        scrollbar = (NATIVE_PANES / "scrollbar.rs").read_text(encoding="utf-8")
        scrollbar_production = scrollbar.split("#[cfg(test)]", 1)[0]

        self.assertIn("AssetContentScrollbarExtent::TreeRows", production)
        self.assertEqual(
            scrollbar_production.count("metadata::<AssetContentPaintMetadata>"), 1
        )
        self.assertIn("metadata.scrollbar_descriptors()", scrollbar_production)
        self.assertNotIn("matches_asset_tree_row", production)
        self.assertNotIn(".iter()\n        .filter", production)
        self.assertNotIn("ACTIVITY_ASSET_TREE_ROW_CONTROL", scrollbar_production)
        self.assertNotIn("BROWSER_ASSET_TREE_ROW_CONTROL", scrollbar_production)

    def test_hover_frame_is_an_indexed_live_row_lookup(self) -> None:
        frame = (NATIVE_PANES / "assets/frame.rs").read_text(encoding="utf-8")
        overlay = (NATIVE_PANES / "assets/overlay.rs").read_text(encoding="utf-8")
        row = (NATIVE_PANES / "assets/overlay/row.rs").read_text(encoding="utf-8")

        self.assertIn("metadata::<AssetContentPaintMetadata>", frame)
        self.assertIn("activity_tree_node_row", frame)
        self.assertIn("let node = nodes.get(row)?", frame)
        self.assertNotIn("for row in 0..nodes.row_count()", frame)
        self.assertNotIn("matches_asset_tree_row", frame)
        self.assertNotIn("row_control_id", overlay)
        self.assertNotIn("row_control_id", row)


if __name__ == "__main__":
    unittest.main()
