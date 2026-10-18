import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ACTIVITY_ROOT = ROOT / "zircon_editor/src/ui/retained_host/activity_rail_pointer"


class ActivityRailPointerRetainedGeometryContractTests(unittest.TestCase):
    def test_sync_classifies_geometry_and_topology_before_rebuild(self) -> None:
        source = (ACTIVITY_ROOT / "sync.rs").read_text(encoding="utf-8")

        self.assertIn("surface_delta_for_layout", source)
        self.assertIn("ActivityRailSurfaceDelta::NoChange", source)
        self.assertIn("ActivityRailSurfaceDelta::Geometry(changes)", source)
        self.assertIn("ActivityRailSurfaceDelta::Topology", source)
        self.assertIn("self.apply_geometry_delta(changes)", source)
        self.assertIn("self.rebuild_surface()", source)

    def test_geometry_path_publishes_authored_frames_and_fails_closed(self) -> None:
        source = (ACTIVITY_ROOT / "rebuild_surface.rs").read_text(encoding="utf-8")

        self.assertIn("publish_authored_geometry(", source)
        self.assertIn("layout_order_generation()", source)
        self.assertIn("surface_geometry_authority_miss_count", source)
        self.assertIn("self.rebuild_surface_from_scratch()", source)
        self.assertIn("surface.rebuild_authored_frames(", source)
        self.assertNotIn("surface.rebuild();", source)

    def test_delta_is_bounded_to_existing_nodes_and_preserves_semantic_routes(self) -> None:
        source = (ACTIVITY_ROOT / "surface_delta.rs").read_text(encoding="utf-8")

        self.assertIn("strip_node_count", source)
        self.assertIn("ActivityRailSurfaceDelta::Topology", source)
        self.assertIn("button_frame", source)
        self.assertNotIn("tabs.iter().map", source)

    def test_window_metrics_reuses_shared_semantic_tab_products(self) -> None:
        layout = (ACTIVITY_ROOT / "host_activity_rail_pointer_layout.rs").read_text(
            encoding="utf-8"
        )
        builder = (
            ACTIVITY_ROOT / "build_host_activity_rail_pointer_layout.rs"
        ).read_text(encoding="utf-8")
        shell_sync = (
            ROOT / "zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("Arc<[HostActivityRailPointerItem]>", layout)
        self.assertIn("Arc::clone(&previous.left_tabs)", builder)
        self.assertIn("Arc::clone(&previous.right_tabs)", builder)
        self.assertIn("sync_activity_rail_pointer_layout_for_target", shell_sync)
        self.assertIn("build_host_activity_rail_pointer_geometry_layout(", shell_sync)

    def test_window_metrics_skips_geometry_free_pointer_receipt_projection(self) -> None:
        recompute = (
            ROOT
            / "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs"
        ).read_text(encoding="utf-8")
        reuse_gate = recompute.split("if !window_metrics_target {", 1)[1].split(
            "} else {", 1
        )[0]

        self.assertIn("sync_host_page_pointer_layout(model)", reuse_gate)
        self.assertIn("sync_document_tab_pointer_layout(model)", reuse_gate)
        self.assertIn("sync_drawer_header_pointer_layout(model)", reuse_gate)
        self.assertIn(
            "ui.window_metrics.semantic_pointer_receipt_reuse_count", recompute
        )

    def test_window_metrics_uses_welcome_recent_viewport_fast_path(self) -> None:
        pointer_surfaces = (
            ROOT
            / "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/pointer_surfaces.rs"
        ).read_text(encoding="utf-8")
        welcome_layout = (
            ROOT
            / "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("sync_welcome_recent_pointer_size()", pointer_surfaces)
        self.assertIn(
            "ui.window_metrics.welcome_recent_semantic_reuse_count", pointer_surfaces
        )
        self.assertIn("fn sync_welcome_recent_pointer_size", welcome_layout)


if __name__ == "__main__":
    unittest.main()
