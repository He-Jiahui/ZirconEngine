from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor09AssetContentGenerationProjectionContract(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_asset_models_return_generation_owned_metadata_from_composition(self) -> None:
        activity = self.read("zircon_editor/src/ui/layouts/views/assets_activity.rs")
        browser = self.read("zircon_editor/src/ui/layouts/views/asset_browser.rs")
        composition = self.read(
            "zircon_editor/src/ui/layouts/views/view_projection/"
            "projection_composition.rs"
        )

        for source, surface in (
            (activity, "Activity"),
            (browser, "Browser"),
        ):
            compose = source.index("compose_view_template_node_model(")
            metadata = source.index("asset_content_paint_metadata(", compose)
            self.assertLess(compose, metadata)
            self.assertIn("asset_content_paint_metadata(", source)
            self.assertIn(f"AssetContentSurface::{surface}", source)

        self.assertIn("let metadata = compose(&mut nodes);", composition)
        self.assertIn("from_shared_rows_overlay_with_metadata(", composition)

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
        for forbidden in (
            "activity_content_identity(",
            "browser_content_identity(",
            "activity_reference_row_index(",
            "browser_reference_row_index(",
            "browser_source_tree_row_index(",
            ".identity(",
            ".is_scroll_node(",
            ".contains(",
        ):
            self.assertNotIn(forbidden, projector)

    def test_generation_publishes_dense_row_descriptors_for_all_paint_time_identity(self) -> None:
        metadata = self.read(
            "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs"
        )
        identity = self.read(
            "zircon_editor/src/ui/workbench/asset_content_layout/identity.rs"
        )

        self.assertIn("row_descriptors", metadata)
        self.assertIn("fn row_descriptor(", metadata)
        self.assertNotIn("fn identity(", metadata)
        self.assertNotIn("fn is_scroll_node(", metadata)
        self.assertIn("AssetContentRowDescriptor", identity)
        self.assertIn("describe_asset_content_row", identity)

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
        self.assertIn("fn transform_row(", transform)
        self.assertNotIn("fn transform(", transform)

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

    def test_profile_counters_keep_identity_parsing_in_generation_and_lookup_in_projection(self) -> None:
        metadata = self.read(
            "zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs"
        )
        activity = self.read("zircon_editor/src/ui/layouts/views/assets_activity.rs")
        browser = self.read("zircon_editor/src/ui/layouts/views/asset_browser.rs")
        projector = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs"
        )
        evidence_gate = self.read("tools/ui-profile-counter-evidence.ps1")

        self.assertIn("identity_parse_count: row_descriptors.len()", metadata)
        self.assertIn("fn identity_parse_count(&self)", metadata)
        for source in (activity, browser):
            self.assertIn("AssetContentGenerationIdentityParseCount", source)
            self.assertIn("metadata.identity_parse_count()", source)
            self.assertIn("#[cfg(feature = \"profiling\")]", source)
        self.assertIn("record_descriptor_lookup", projector)
        self.assertIn("AssetContentDescriptorLookupCount", projector)
        self.assertIn("asset_content_generation_identity_parse_count", evidence_gate)
        self.assertIn("asset_content_descriptor_lookup_count", evidence_gate)
        self.assertIn("template_node_visit_count", evidence_gate)
        self.assertIn("$descriptorLookupCount -gt $templateNodeVisitCount", evidence_gate)


if __name__ == "__main__":
    unittest.main()
