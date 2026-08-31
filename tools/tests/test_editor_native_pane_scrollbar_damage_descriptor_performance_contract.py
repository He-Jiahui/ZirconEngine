from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LAYOUT = ROOT / "zircon_editor/src/ui/workbench/asset_content_layout"
NATIVE_PANES = (
    ROOT
    / "zircon_editor/src/ui/retained_host/host_contract"
    / "paint_workbench_renderer/native_panes"
)


class EditorNativePaneScrollbarDamageDescriptorContractTests(unittest.TestCase):
    def test_generation_publishes_one_bounded_scrollbar_descriptor_slice(self) -> None:
        metadata = (LAYOUT / "paint_metadata.rs").read_text(encoding="utf-8")
        descriptors = (
            LAYOUT / "paint_metadata/scrollbar_descriptors.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "scrollbar_descriptors: AssetContentScrollbarDescriptors", metadata
        )
        self.assertIn("fn scrollbar_descriptors(&self)", metadata)
        self.assertIn("entries: [AssetContentScrollbarDescriptor; 4]", descriptors)
        self.assertIn("entries: [EMPTY_DESCRIPTOR; 4]", descriptors)
        self.assertNotIn("Vec<AssetContentScrollbarDescriptor>", descriptors)
        descriptor_body = descriptors[
            descriptors.index("pub(crate) struct AssetContentScrollbarDescriptor") :
            descriptors.index("const EMPTY_DESCRIPTOR")
        ]
        self.assertNotIn("viewport:", descriptor_body)
        self.assertNotIn("extent:", descriptor_body)
        self.assertIn("fn scrollbar_viewport(", descriptors)
        self.assertIn("fn scrollbar_extent(", descriptors)
        self.assertIn("AssetContentScrollbarKind::Tree", descriptors)
        self.assertIn("AssetContentScrollbarKind::Content", descriptors)
        self.assertIn("AssetContentScrollbarKind::References", descriptors)
        self.assertIn("AssetContentScrollbarKind::UsedBy", descriptors)

    def test_native_pane_uses_effective_damage_before_kind_dispatch(self) -> None:
        content = (NATIVE_PANES / "content.rs").read_text(encoding="utf-8")
        production = content.split("#[cfg(test)]", 1)[0]

        effective = production.index("effective_native_clip")
        dispatch = production.index("match pane.kind.as_str()")
        self.assertLess(effective, dispatch)
        self.assertIn("frame.paint_clip()", production)
        self.assertIn("paint_geometry::intersect", production)
        self.assertIn("return native_content_is_present(pane)", production)
        self.assertIn("fn native_content_is_present", production)
        self.assertIn("pane.hierarchy.hierarchy_nodes.row_count() > 0", production)

    def test_fallback_uses_logical_layer_presence_not_damage_pixel_hits(self) -> None:
        content = (
            NATIVE_PANES.parent / "docks/pane/content.rs"
        ).read_text(encoding="utf-8")
        template_nodes = (
            NATIVE_PANES.parent / "docks/pane/template_nodes.rs"
        ).read_text(encoding="utf-8")
        viewport = (NATIVE_PANES / "viewport.rs").read_text(encoding="utf-8")
        diagnostics = (NATIVE_PANES / "diagnostics.rs").read_text(encoding="utf-8")

        for presence in [
            "has_viewport_content",
            "has_native_content_before",
            "has_template_content",
            "has_native_content_after",
            "has_debug_overlay_content",
        ]:
            self.assertIn(presence, content)
        self.assertIn("if !has_template_nodes(nodes)", template_nodes)
        self.assertRegex(template_nodes, r"draw_template_nodes\([^;]+;\s*true")
        self.assertIn("if drew_base", viewport)
        self.assertIn("let content_present =", diagnostics)

    def test_asset_pane_resolves_the_descriptor_set_once(self) -> None:
        content = (NATIVE_PANES / "content.rs").read_text(encoding="utf-8")
        scrollbar = (NATIVE_PANES / "scrollbar.rs").read_text(encoding="utf-8")
        asset = (NATIVE_PANES / "scrollbar/asset.rs").read_text(encoding="utf-8")
        lower_tests = (NATIVE_PANES / "scrollbar/tests.rs").read_text(
            encoding="utf-8"
        )
        production = scrollbar.split("#[cfg(test)]", 1)[0]

        self.assertIn("draw_activity_asset_scrollbars", content)
        self.assertIn("draw_browser_asset_scrollbars", content)
        self.assertNotIn("draw_activity_asset_content_scrollbar", content)
        self.assertNotIn("draw_browser_asset_content_scrollbar", content)
        self.assertEqual(production.count("metadata::<AssetContentPaintMetadata>"), 1)
        self.assertIn("metadata.scrollbar_descriptors()", production)
        descriptor_loop = production[production.index("fn draw_asset_scrollbars") :]
        reject = descriptor_loop.index("intersect(&viewport, clip).is_none()")
        state = descriptor_loop.index("asset_scrollbar_interaction(")
        extent = descriptor_loop.index("metadata.scrollbar_extent(*descriptor)")
        self.assertLess(reject, state)
        self.assertLess(reject, extent)
        self.assertNotIn(".extent()", lower_tests)
        self.assertNotIn(".viewport()", lower_tests)
        self.assertNotIn(
            'production.contains("AssetContentScrollbarDescriptor")', asset
        )

    def test_damage_reject_precedes_scrollbar_style_and_geometry(self) -> None:
        paint = (NATIVE_PANES / "scrollbar/paint.rs").read_text(encoding="utf-8")
        production = paint[
            paint.index("fn draw_vertical_scrollbar") : paint.index(
                "#[cfg(test)]\npub(super) fn paint_scrollbar_component_for_test"
            )
        ]

        reject = production.index("intersect(viewport, clip).is_none()")
        metrics = production.index("workbench_scrollbar_metrics()")
        geometry = production.index("vertical_scrollbar_geometry(")
        palette = production.index("workbench_scrollbar_palette()")
        self.assertLess(reject, metrics)
        self.assertLess(reject, geometry)
        self.assertLess(reject, palette)


if __name__ == "__main__":
    unittest.main()
