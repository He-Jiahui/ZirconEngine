import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SYNC = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/"
    "host_menu_pointer_bridge_sync.rs"
)
SURFACE_DELTA = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/"
    "host_menu_pointer_bridge_surface_delta.rs"
)
LAYOUT = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_layout.rs"
)
LAYOUT_BUILDER = ROOT / (
    "zircon_editor/src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs"
)
HOST_MENU_SYNC = ROOT / (
    "zircon_editor/src/ui/retained_host/app/pointer_layout/menu.rs"
)
POINTER_SURFACES = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/"
    "pointer_surfaces.rs"
)


class EditorMenuPointerRetainedGeometryContractTests(unittest.TestCase):
    def test_sync_classifies_surface_work_instead_of_rebuilding_unconditionally(self) -> None:
        source = SYNC.read_text(encoding="utf-8")

        self.assertIn("let surface_state_changed =", source)
        self.assertIn("if layout_changed || surface_state_changed", source)
        self.assertIn("self.apply_surface_delta();", source)
        self.assertNotIn("self.rebuild_surface();", source)

    def test_hover_only_sync_does_not_refresh_popup_items(self) -> None:
        source = SYNC.read_text(encoding="utf-8")

        self.assertIn(
            "if popup_semantics_changed || self.popup_menu_index != self.state.open_menu_index",
            source,
        )

    def test_geometry_delta_uses_exact_runtime_authored_publication(self) -> None:
        source = SURFACE_DELTA.read_text(encoding="utf-8")

        self.assertIn("publish_authored_geometry(", source)
        self.assertIn("changed_node_ids", source)
        self.assertIn("layout_order_generation()", source)
        self.assertIn("MenuPointerSurfaceDelta::Geometry", source)

    def test_hover_only_delta_has_no_tree_or_command_scan(self) -> None:
        source = SURFACE_DELTA.read_text(encoding="utf-8")

        self.assertIn("MenuPointerSurfaceDelta::NoChange", source)
        self.assertNotIn("tree.nodes.iter()", source)
        self.assertNotIn("render_extract", source)
        self.assertNotIn("commands.iter()", source)

    def test_topology_fallback_is_typed_and_rebuilds_from_scratch(self) -> None:
        source = SURFACE_DELTA.read_text(encoding="utf-8")

        self.assertIn("MenuPointerSurfaceDelta::Topology", source)
        self.assertIn("self.rebuild_surface_from_scratch();", source)
        self.assertIn("expected_nodes.len() != self.surface.tree.nodes.len()", source)

    def test_window_metrics_layout_uses_shared_semantic_geometry_builder(self) -> None:
        layout = LAYOUT.read_text(encoding="utf-8")
        builder = LAYOUT_BUILDER.read_text(encoding="utf-8")
        host_sync = HOST_MENU_SYNC.read_text(encoding="utf-8")
        pointer_surfaces = POINTER_SURFACES.read_text(encoding="utf-8")

        self.assertIn("Arc<[Vec<MenuItemSpec>]>", layout)
        self.assertIn("intrinsic_popup_widths: Arc<[f32]>", layout)
        self.assertIn("pub(crate) fn build_host_menu_pointer_geometry_layout(", builder)
        geometry_body = builder.split(
            "pub(crate) fn build_host_menu_pointer_geometry_layout(", 1
        )[1].split("\nfn ", 1)[0]
        self.assertNotIn("pointer_menus(", geometry_body)
        self.assertNotIn("measured_root_popup_widths(", geometry_body)
        self.assertIn("window_metrics_target", host_sync)
        self.assertIn("build_host_menu_pointer_geometry_layout(", host_sync)
        self.assertIn("preset_names, window_metrics_target", pointer_surfaces)


if __name__ == "__main__":
    unittest.main()
