import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/drawer_header_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class DrawerHeaderNativeReceiptPerformanceContract(unittest.TestCase):
    def test_native_hit_authority_emits_the_committed_drawer_tab_receipt(self):
        route = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/"
            "chrome/tabs/drawer.rs"
        )
        dispatch = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "button_dispatch/chrome_press/tabs/drawer.rs"
        )

        self.assertIn("for (row, tab) in tabs.iter().enumerate()", route)
        self.assertIn("contains(&tab_frame, x, y)", route)
        self.assertIn("ChromePointerRoute::DrawerHeaderTab", route)
        self.assertIn("invoke_drawer_header_pointer_clicked", dispatch)

    def test_bridge_no_longer_owns_a_mirror_hit_surface(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/"
            "host_drawer_header_pointer_bridge.rs"
        )

        self.assertNotIn("UiSurface", bridge)
        self.assertNotIn("UiPointerDispatcher", bridge)
        self.assertNotIn("measured_frames", bridge)
        self.assertNotIn("route_intents", bridge)
        self.assertIn("target_for_route", bridge)

    def test_mirror_geometry_and_dispatch_owners_are_deleted(self):
        retired = [
            "base_state.rs",
            "constants.rs",
            "dispatch_event.rs",
            "drawer_slot_key.rs",
            "global_point.rs",
            "rebuild_surface.rs",
            "register_handled_pointer_node.rs",
            "root_frame.rs",
            "update_measured_frame.rs",
        ]

        for name in retired:
            self.assertFalse((OWNER / name).exists(), name)

    def test_receipt_route_is_copy_and_target_identity_stays_typed(self):
        route = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/"
            "host_drawer_header_pointer_route.rs"
        )
        item = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/"
            "host_drawer_header_pointer_item.rs"
        )

        self.assertIn("Clone, Copy", route)
        self.assertNotIn("String", route)
        self.assertIn("slot: ActivityDrawerSlot", item)
        self.assertIn("instance_id: ViewInstanceId", item)
        self.assertNotIn("slot: String", item)
        self.assertNotIn("instance_id: String", item)

    def test_receipt_projection_has_no_paint_geometry_inputs(self):
        build = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/"
            "build_host_drawer_header_pointer_layout.rs"
        )
        surface = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/"
            "host_drawer_header_pointer_surface.rs"
        )
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs"
        )

        self.assertNotIn("UiFrame", build)
        self.assertNotIn("WorkbenchChromeMetrics", build)
        self.assertNotIn("WorkbenchWindowLayoutFrames", build)
        self.assertNotIn("strip_frame", surface)
        self.assertIn("build_host_drawer_header_pointer_layout(model)", pointer_layout)

    def test_callback_borrows_native_receipt_target_without_pointer_dispatch(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/"
            "drawer_header.rs"
        )
        click = source(
            "zircon_editor/src/ui/retained_host/drawer_header_pointer/handle_click.rs"
        )

        self.assertRegex(callback, re.compile(r"pointer_bridge\s*\.target_for_route"))
        for implementation in (callback, click):
            self.assertNotIn("UiPointerEvent", implementation)
            self.assertNotIn("dispatch_event", implementation)
        self.assertNotIn("UiPoint", callback)
        self.assertNotIn("tab_width", callback)

    def test_drawer_command_boundary_accepts_typed_target(self):
        command = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/layout/drawer_toggle.rs"
        )

        self.assertIn("slot: ActivityDrawerSlot", command)
        self.assertIn("instance_id: &ViewInstanceId", command)
        self.assertNotIn("parse_activity_drawer_slot(slot)", command)
        self.assertNotIn("activity_binding_for_target", command)
        self.assertNotIn("active_activity_window_drawers()", command)


if __name__ == "__main__":
    unittest.main()
