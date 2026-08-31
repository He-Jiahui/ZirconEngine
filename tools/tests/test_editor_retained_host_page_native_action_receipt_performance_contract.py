import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/host_page_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class HostPageNativeActionReceiptPerformanceContract(unittest.TestCase):
    def test_native_hit_authority_distinguishes_close_before_tab_body(self):
        route = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/"
            "chrome/tabs/host_page.rs"
        )
        close_hit = route.index("contains(&tab.close_frame, x, y)")
        body_hit = route.index("contains(&tab.frame, x, y)")

        self.assertLess(close_hit, body_hit)
        self.assertIn("close: true", route)
        self.assertIn("close: false", route)
        self.assertNotIn("tab_x:", route)
        self.assertNotIn("local_x:", route)

    def test_host_page_callback_carries_action_not_geometry(self):
        globals_source = source(
            "zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs"
        )
        callback_storage = source(
            "zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs"
        )
        callback_line = next(
            line
            for line in globals_source.splitlines()
            if "callback_methods!(ui_callbacks, on_host_page_pointer_clicked" in line
        )

        self.assertIn("close: bool", callback_line)
        self.assertIn("Option<Callback2<i32, bool>>", callback_storage)
        self.assertNotIn("Option<Callback5<i32, f32, f32, f32, f32>>", callback_storage)
        for forbidden in ["tab_x", "tab_width", "point_x", "point_y"]:
            self.assertNotIn(forbidden, callback_line)

    def test_bridge_no_longer_owns_a_mirror_hit_surface(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/host_page_pointer/"
            "host_page_pointer_bridge.rs"
        )

        self.assertNotIn("UiSurface", bridge)
        self.assertNotIn("UiPointerDispatcher", bridge)
        self.assertNotIn("measured_frames", bridge)
        self.assertNotIn("route_intents", bridge)
        self.assertIn("activation_target_for_route", bridge)
        self.assertIn("close_target_for_route", bridge)

    def test_mirror_geometry_measurement_and_dispatch_owners_are_deleted(self):
        retired = [
            "base_state.rs",
            "dispatch_event.rs",
            "error.rs",
            "handle_overflow_click.rs",
            "rebuild_surface.rs",
            "register_handled_pointer_node.rs",
            "root_frame.rs",
            "tab_node_id.rs",
            "tab_strip_geometry.rs",
            "update_measured_frame.rs",
        ]

        for name in retired:
            self.assertFalse((OWNER / name).exists(), name)

    def test_receipt_route_is_copy_and_target_identity_stays_typed(self):
        route = source(
            "zircon_editor/src/ui/retained_host/host_page_pointer/"
            "host_page_pointer_route.rs"
        )
        item = source(
            "zircon_editor/src/ui/retained_host/host_page_pointer/"
            "host_page_pointer_item.rs"
        )

        self.assertIn("Clone, Copy", route)
        self.assertNotIn("String", route)
        self.assertIn("page_id: MainPageId", item)
        self.assertIn("close_instance_id: Option<ViewInstanceId>", item)
        self.assertNotIn("title:", item)

    def test_receipt_projection_has_no_paint_or_measurement_inputs(self):
        build = source(
            "zircon_editor/src/ui/retained_host/host_page_pointer/"
            "build_host_page_pointer_layout.rs"
        )
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs"
        )

        for forbidden in [
            "UiFrame",
            "WorkbenchChromeMetrics",
            "BuiltinHostOuterShellFrames",
            "allocate_host_page_tabs",
            "title",
        ]:
            self.assertNotIn(forbidden, build)
        self.assertIn("build_host_page_pointer_layout(model)", pointer_layout)

    def test_callback_borrows_typed_receipt_target_without_pointer_dispatch(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/host_page.rs"
        )
        click = source(
            "zircon_editor/src/ui/retained_host/host_page_pointer/handle_click.rs"
        )

        self.assertRegex(callback, re.compile(r"pointer_bridge\s*\.activation_target_for_route"))
        self.assertRegex(callback, re.compile(r"pointer_bridge\s*\.close_target_for_route"))
        for implementation in (callback, click):
            self.assertNotIn("UiPointerEvent", implementation)
            self.assertNotIn("dispatch_event", implementation)
            self.assertNotIn("UiPoint", implementation)
            self.assertNotIn("tab_width", implementation)

    def test_host_page_route_intent_mirror_binding_is_removed(self):
        route_intent = source(
            "zircon_editor/src/ui/retained_host/route_intent/map.rs"
        )

        self.assertNotIn("HostPage(HostPagePointerRoute)", route_intent)
        self.assertNotIn("host_page_route_for_pointer_dispatch", route_intent)


if __name__ == "__main__":
    unittest.main()
