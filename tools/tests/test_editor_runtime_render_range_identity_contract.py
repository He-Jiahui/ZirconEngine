from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
FRAME_EXTRACT = ROOT / (
    "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
)
VIEW_DATA = ROOT / "zircon_editor/src/ui/layouts/views/view_data.rs"
MATERIALIZATION = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/materialization.rs"
)
UI_ASSET_NODE_PROJECTION = ROOT / "zircon_editor/src/ui/asset_editor/node_projection.rs"
TEMPLATE_CONVERSION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs"
)
PROJECTION_CACHE = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs"
)


class EditorRuntimeRenderRangeIdentityContractTests(unittest.TestCase):
    def test_published_runtime_commands_own_a_node_range_index(self) -> None:
        source = FRAME_EXTRACT.read_text(encoding="utf-8")

        self.assertIn(
            "command_ranges: Arc<HashMap<UiNodeId, Range<usize>>>",
            source,
        )
        self.assertIn("pub fn command_range(&self, node_id: UiNodeId)", source)
        self.assertIn("pub fn commands_for_node(", source)
        self.assertIn("node_id: UiNodeId", source)

    def test_fixed_cardinality_patch_reuses_range_identity(self) -> None:
        source = FRAME_EXTRACT.read_text(encoding="utf-8")
        patch_body = source.split("fn patch_ranges(", 1)[1]

        self.assertIn("patched_node_identity_is_stable", patch_body)
        self.assertIn("Arc::clone(&self.command_ranges)", patch_body)
        self.assertNotIn("build_command_ranges(source)", patch_body)

    def test_runtime_command_refs_resolve_through_the_owner_range(self) -> None:
        source = FRAME_EXTRACT.read_text(encoding="utf-8")

        self.assertIn("pub struct UiRenderFrameCommandRef", source)
        self.assertIn("pub node_command_index: u32", source)
        resolver = source.split("pub fn command_by_ref(", 1)[1]
        resolver = resolver.split("\n    }", 1)[0]
        self.assertIn("self.command_range(command_ref.node_id)", resolver)
        self.assertIn("checked_add(command_ref.node_command_index as usize)", resolver)
        self.assertIn("(index < range.end).then", resolver)

    def test_surface_materialization_preserves_runtime_node_identity(self) -> None:
        view_data = VIEW_DATA.read_text(encoding="utf-8")
        materialization = MATERIALIZATION.read_text(encoding="utf-8")
        ui_asset_projection = UI_ASSET_NODE_PROJECTION.read_text(encoding="utf-8")

        self.assertIn("pub surface_node_id: Option<UiNodeId>", view_data)
        self.assertIn("surface_node_id: Some(tree_node.node_id)", materialization)
        self.assertIn("surface_node_id: Some(node.node_id)", ui_asset_projection)

    def test_surface_materialization_preserves_exact_node_command_index(self) -> None:
        view_data = VIEW_DATA.read_text(encoding="utf-8")
        materialization = MATERIALIZATION.read_text(encoding="utf-8")
        ui_asset_projection = UI_ASSET_NODE_PROJECTION.read_text(encoding="utf-8")

        self.assertIn(
            "pub surface_render_command_ref: Option<UiRenderFrameCommandRef>",
            view_data,
        )
        self.assertIn("render_commands_with_refs(commands)", materialization)
        self.assertIn("surface_render_command_ref: command_ref", materialization)
        self.assertIn("surface_render_command_ref: None", ui_asset_projection)

    def test_template_bridge_forwards_identity_instead_of_erasing_it(self) -> None:
        source = TEMPLATE_CONVERSION.read_text(encoding="utf-8")
        initializer = source.split("host_contract::TemplatePaneNodeData {", 1)[1]
        initializer = initializer.split("has_workbench_icon_tooltip", 1)[0]

        self.assertIn("surface_node_id: data.surface_node_id", initializer)
        self.assertNotIn("surface_node_id: None", initializer)
        self.assertIn(
            "surface_render_command_ref: data.surface_render_command_ref",
            source,
        )

    def test_incremental_topology_validation_includes_exact_command_identity(self) -> None:
        materialization = MATERIALIZATION.read_text(encoding="utf-8")
        projection_cache = PROJECTION_CACHE.read_text(encoding="utf-8")

        self.assertIn("render_command_ref: Option<UiRenderFrameCommandRef>", materialization)
        self.assertIn(
            "current.render_command_ref != cached.render_command_ref",
            projection_cache,
        )
        self.assertIn("signature.render_command_ref", projection_cache)


if __name__ == "__main__":
    unittest.main()
