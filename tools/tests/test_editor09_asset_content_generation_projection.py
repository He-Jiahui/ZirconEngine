from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor09AssetContentGenerationProjectionContract(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_asset_models_attach_generation_owned_paint_metadata(self) -> None:
        activity = self.read("zircon_editor/src/ui/layouts/views/assets_activity.rs")
        browser = self.read("zircon_editor/src/ui/layouts/views/asset_browser.rs")

        for source, surface in (
            (activity, "Activity"),
            (browser, "Browser"),
        ):
            self.assertIn("asset_content_paint_metadata(", source)
            self.assertIn(f"AssetContentSurface::{surface}", source)
            self.assertIn("ModelRc::with_metadata(nodes, metadata)", source)

    def test_painter_consumers_do_not_scan_or_parse_the_model(self) -> None:
        projector = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs"
        )
        scrollbar = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/native_panes/scrollbar/asset.rs"
        ).split("#[cfg(test)]", 1)[0]

        for source in (projector, scrollbar):
            self.assertNotIn("row_data(", source)
            self.assertNotIn("for row in 0..nodes.row_count()", source)
        self.assertNotIn("activity_content_identity(", projector)
        self.assertNotIn("browser_content_identity(", projector)

    def test_old_painter_owned_identity_parser_is_deleted(self) -> None:
        identity = ROOT / (
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/docks/pane/template_nodes/asset_content/identity.rs"
        )
        self.assertFalse(identity.exists())

    def test_draw_pipeline_supports_exact_generation_row_visits(self) -> None:
        transform = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_node_pipeline/transform.rs"
        )
        draw = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_template_nodes/template_node_pipeline/draw.rs"
        )

        self.assertIn("row_visit_indices", transform)
        self.assertIn("row_visit_indices", draw)
        self.assertIn("Some(rows)", draw)

    def test_template_dto_projection_preserves_generation_metadata(self) -> None:
        projection = self.read(
            "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
            "template_node_projection.rs"
        )

        self.assertIn("map_preserving_metadata", projection)

    def test_generation_metadata_owner_does_not_depend_on_layout_dtos(self) -> None:
        metadata = self.read(
            "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs"
        )

        self.assertNotIn("crate::ui::layouts", metadata)
        self.assertIn("AssetContentPaintNodeInput", metadata)


if __name__ == "__main__":
    unittest.main()
