from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "zircon_editor/src/ui"
METADATA = EDITOR / "retained_host/hierarchy_pointer/paint_metadata.rs"
HIERARCHY_VIEW = EDITOR / "layouts/views/hierarchy.rs"
HOST_PROJECTION = (
    EDITOR / "retained_host/ui/pane_data_conversion/hierarchy_projection.rs"
)
VIEWPORT = (
    EDITOR
    / "retained_host/host_contract/paint_workbench_renderer/native_panes"
    / "hierarchy/viewport.rs"
)


class EditorHierarchyViewportIndexPerformanceContractTests(unittest.TestCase):
    def test_generation_metadata_publishes_stable_anchor_rows(self) -> None:
        source = METADATA.read_text(encoding="utf-8")

        self.assertIn("struct HierarchyPaintMetadata", source)
        self.assertIn("viewport_node_rows: Vec<usize>", source)
        self.assertIn("HierarchyListPanel", source)
        self.assertIn("HierarchyTreeSlotAnchor", source)
        self.assertIn("let mut published = [false;", source)
        self.assertIn("std::mem::replace(&mut published[identity], true)", source)

    def test_both_hierarchy_model_construction_paths_attach_metadata(self) -> None:
        hierarchy = HIERARCHY_VIEW.read_text(encoding="utf-8")
        projection = HOST_PROJECTION.read_text(encoding="utf-8")

        self.assertIn("hierarchy_paint_metadata(projection.iter()", hierarchy)
        self.assertIn(".replacing_metadata(metadata)", hierarchy)
        self.assertIn("hierarchy_paint_metadata(nodes.iter()", projection)
        self.assertIn(".replacing_metadata(metadata)", projection)

    def test_viewport_uses_only_published_candidates_and_live_rows(self) -> None:
        source = VIEWPORT.read_text(encoding="utf-8")
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("metadata::<HierarchyPaintMetadata>", production)
        self.assertIn("HierarchyPaintMetadata::viewport_node_rows", production)
        self.assertIn("nodes.get(row)", production)
        self.assertNotIn("nodes.iter()", production)
        self.assertNotIn("row_count()", production)
        self.assertNotIn("control_id", production)


if __name__ == "__main__":
    unittest.main()
