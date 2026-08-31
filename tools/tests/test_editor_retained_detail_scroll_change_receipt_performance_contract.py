import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/detail_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class DetailScrollChangeReceiptPerformanceContract(unittest.TestCase):
    def test_bridge_owns_only_scalar_layout_and_state(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/"
            "scroll_surface_pointer_bridge.rs"
        )

        for required in [
            "layout: ScrollSurfacePointerLayout",
            "state: ScrollSurfacePointerState",
        ]:
            self.assertIn(required, bridge)
        for forbidden in [
            "UiSurface",
            "UiPointerDispatcher",
            "EditorRouteIntentMap",
            "tree_id",
            "path_prefix",
        ]:
            self.assertNotIn(forbidden, bridge)

    def test_generic_surface_owners_are_deleted(self):
        for name in ["base_state.rs", "bridge_constants.rs", "rebuild_surface.rs"]:
            self.assertFalse((OWNER / name).exists(), name)

    def test_direct_scroll_receipt_carries_changed_without_generic_dispatch(self):
        handler = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/handle_scroll.rs"
        )
        dispatch = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/"
            "scroll_surface_pointer_dispatch.rs"
        )

        self.assertIn("viewport_frame(&self.layout).contains_point(point)", handler)
        self.assertIn("let previous_offset = self.state.scroll_offset", handler)
        self.assertIn("changed:", handler)
        self.assertIn("pub(crate) changed: bool", dispatch)
        for forbidden in [
            "UiPointerEvent",
            "dispatch_pointer_event",
            "detail_route_for_pointer_dispatch",
            "Result<ScrollSurfacePointerDispatch",
        ]:
            self.assertNotIn(forbidden, handler)

    def test_scroll_layout_and_state_are_copy_scalar_values(self):
        layout = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/"
            "scroll_surface_pointer_layout.rs"
        )
        state = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/"
            "scroll_surface_pointer_state.rs"
        )

        self.assertIn("Clone, Copy", layout)
        self.assertIn("Clone, Copy", state)

    def test_host_state_does_not_duplicate_bridge_scroll_state(self):
        host = source("zircon_editor/src/ui/retained_host/scroll_surface_host.rs")
        production = host.split("#[cfg(test)]", 1)[0]

        self.assertNotIn("state: ScrollSurfacePointerState", production)
        self.assertIn("pub(crate) fn handle_scroll", production)
        self.assertIn("-> bool", production)
        self.assertIn("dispatch.changed", production)
        self.assertNotIn("Result<(), String>", production)

    def test_scroll_constructors_have_no_mirror_identity_strings(self):
        bridge_new = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/new.rs"
        )
        host = source("zircon_editor/src/ui/retained_host/scroll_surface_host.rs")
        startup = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/"
            "state/interaction.rs"
        )

        self.assertIn("pub(crate) fn new() -> Self", bridge_new)
        self.assertIn("pub(crate) fn new() -> Self", host)
        self.assertEqual(startup.count("ScrollSurfaceHostState::new()"), 3)
        for forbidden in ["zircon.editor.console.pointer", "editor.asset_details"]:
            self.assertNotIn(forbidden, startup)

    def test_all_detail_scroll_adapters_publish_only_changed_offsets(self):
        for name in ["console.rs", "asset_browser.rs"]:
            adapter = source(
                "zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/" + name
            )
            self.assertIn("if self", adapter, name)
            self.assertIn(".handle_scroll(UiPoint::new(x, y), delta)", adapter, name)
            self.assertNotIn("match self", adapter, name)
            self.assertNotIn("Ok(())", adapter, name)

        inspector = source(
            "zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs"
        )
        self.assertIn("route_workbench_inspector_scroll", inspector)
        self.assertIn("Ok(true) => return", inspector)
        self.assertIn("Ok(false) => {}", inspector)
        self.assertIn(
            ".handle_scroll(UiPoint::new(x, y), delta)", inspector
        )
        self.assertIn("self.apply_dispatch_effects(effects)", inspector)

    def test_detail_layout_sync_publishes_only_changed_offsets(self):
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs"
        )

        for pattern in [
            r"if\s+self\s*\.console_scroll_surface\s*\.sync_following_tail",
            r"if\s+self\s*\.inspector_scroll_surface\s*\.sync",
            r"if\s+self\s*\.browser_asset_details_scroll_surface\s*\.sync",
        ]:
            self.assertRegex(
                pointer_layout,
                re.compile(pattern),
            )
        self.assertEqual(pointer_layout.count("apply_console_pointer_state_to_ui();"), 1)
        self.assertEqual(pointer_layout.count("apply_inspector_pointer_state_to_ui();"), 1)
        self.assertEqual(
            pointer_layout.count("apply_browser_asset_details_pointer_state_to_ui();"),
            1,
        )

    def test_detail_route_intent_mirror_binding_is_removed(self):
        route_intent = source(
            "zircon_editor/src/ui/retained_host/route_intent/map.rs"
        )

        self.assertNotIn("Detail(ScrollSurfacePointerRoute)", route_intent)
        self.assertNotIn("detail_route_for_pointer_dispatch", route_intent)

    def test_direct_cut_preserves_viewport_origin_clamp_and_tail_policy(self):
        handler = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/handle_scroll.rs"
        )
        viewport = source(
            "zircon_editor/src/ui/retained_host/detail_pointer/viewport_frame.rs"
        )
        host = source("zircon_editor/src/ui/retained_host/scroll_surface_host.rs")

        self.assertIn("viewport_origin_y", viewport)
        self.assertIn("self.clamp_scroll_offset()", handler)
        self.assertIn("sync_following_tail", host)
        self.assertIn("SCROLL_END_EPSILON_PX", host)
        self.assertNotIn("self.state = self.bridge.state()", host)


if __name__ == "__main__":
    unittest.main()
