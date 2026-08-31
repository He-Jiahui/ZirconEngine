import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class ViewportToolbarPointerGenerationPerformanceContract(unittest.TestCase):
    def test_applied_cursor_uses_hit_domain_identity_not_outer_frame_generation(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/"
            "viewport_toolbar_pointer_bridge.rs"
        )
        sync = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs"
        )

        self.assertIn("Weak<UiHitTestGrid>", bridge)
        self.assertIn("surface_frame: &Arc<UiSurfaceFrame>", sync)
        self.assertIn("applied_frame.as_ptr()", sync)
        self.assertIn("Arc::as_ptr(&surface_frame.hit_grid)", sync)
        self.assertIn("Arc::downgrade(&surface_frame.hit_grid)", sync)
        self.assertNotIn("Arc::as_ptr(surface_frame)", sync)
        self.assertNotIn("surface_frame.generation != 0", sync)

    def test_frame_projection_consumes_authoritative_hit_entries(self):
        sync = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs"
        )
        compact = "".join(sync.split())

        self.assertIn("surface_frame.hit_grid.entries", compact)
        self.assertNotIn("surface_frame.arranged_tree.nodes", compact)

    def test_control_validation_uses_one_borrowed_descriptor(self):
        route = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/route_for_control.rs"
        )
        click = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs"
        )
        frame = source(
            "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs"
        )
        module = source("zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/mod.rs")

        self.assertIn("enum ViewportToolbarControlRoute", route)
        self.assertIn("fn control_route_for_id", route)
        self.assertNotIn("for route in [", route)
        self.assertIn("validate_control_id(surface_key, control_id)?", click)
        self.assertIn("control_route_for_id(control_id).is_none()", frame)
        self.assertLess(
            frame.index("control_route_for_id(control_id)"),
            frame.index("control_id.to_string()"),
        )
        for obsolete in [
            "align_view_route",
            "cycle_display_mode_route",
            "cycle_grid_mode_route",
            "frame_selection_route",
            "play_mode_route",
            "set_projection_mode_route",
            "set_scene_mode_route",
            "set_transform_space_route",
            "snap_routes",
            "toggle_routes",
        ]:
            self.assertNotIn(f"mod {obsolete};", module)
            self.assertFalse((OWNER / f"{obsolete}.rs").exists())

    def test_stable_product_click_compares_before_layout_allocation(self):
        app = source(
            "zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs"
        )
        sync = source("zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync.rs")

        self.assertIn(".sync_single_surface(", app)
        self.assertNotIn("build_viewport_toolbar_pointer_layout_with_size", app)
        compare = sync.index("existing.key == surface_key")
        allocation = sync.index("surface_key.to_string()")
        self.assertLess(compare, allocation)

    def test_route_dispatch_never_builds_full_chrome_snapshot(self):
        mapping = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/viewport/route_mapping.rs"
        )
        snapshot = source("zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs")

        self.assertNotIn("runtime.chrome_snapshot()", mapping)
        self.assertIn("runtime.scene_viewport_settings()", mapping)
        self.assertIn("pub fn scene_viewport_settings(&self)", snapshot)
        self.assertIn("self.shell().lock().state.scene_viewport_settings()", snapshot)


if __name__ == "__main__":
    unittest.main()
