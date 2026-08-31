import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/document_tab_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class DocumentTabNativeReceiptPerformanceContract(unittest.TestCase):
    def test_native_hit_authority_emits_distinct_body_and_close_receipts(self):
        body = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/"
            "chrome/tabs/document/body.rs"
        )
        close = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/"
            "chrome/tabs/document/close.rs"
        )
        dispatch = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "button_dispatch/chrome_press/tabs/document.rs"
        )

        self.assertIn("ChromePointerRoute::DocumentTab", body)
        self.assertIn("close: false", body)
        self.assertIn("ChromePointerRoute::DocumentTab", close)
        self.assertIn("close: true", close)
        self.assertIn("dispatch_document_tab_body_press", dispatch)
        self.assertIn("dispatch_document_tab_close_press", dispatch)

    def test_bridge_no_longer_owns_a_mirror_hit_surface(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_bridge.rs"
        )

        self.assertNotIn("UiSurface", bridge)
        self.assertNotIn("UiPointerDispatcher", bridge)
        self.assertNotIn("measured_frames", bridge)
        self.assertNotIn("route_intents", bridge)
        self.assertIn("target_for_route", bridge)

    def test_mirror_geometry_and_dispatch_owners_are_deleted(self):
        retired = [
            "constants.rs",
            "helper.rs",
            "host_document_tab_pointer_bridge_dispatch_event.rs",
            "host_document_tab_pointer_bridge_global_point.rs",
            "host_document_tab_pointer_bridge_rebuild_surface.rs",
            "host_document_tab_pointer_bridge_update_measured_frame.rs",
            "register_handled_pointer_node.rs",
        ]

        for name in retired:
            self.assertFalse((OWNER / name).exists(), name)

    def test_receipt_route_is_copy_and_identity_stays_typed(self):
        route = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_route.rs"
        )
        item = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_item.rs"
        )

        self.assertIn("Clone, Copy", route)
        self.assertNotIn("String", route)
        self.assertIn("instance_id: ViewInstanceId", item)
        self.assertNotIn("instance_id: String", item)

    def test_receipt_projection_has_no_paint_geometry_inputs(self):
        build = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "build_host_document_tab_pointer_layout.rs"
        )
        surface = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_surface.rs"
        )
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs"
        )

        self.assertNotIn("UiFrame", build)
        self.assertNotIn("FloatingWindowProjectionBundle", build)
        self.assertNotIn("WorkbenchWindowLayoutFrames", build)
        self.assertNotIn("strip_frame", surface)
        self.assertIn("build_host_document_tab_pointer_layout(model)", pointer_layout)

    def test_callback_borrows_native_receipt_target_without_pointer_dispatch(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/document_tab.rs"
        )
        activate = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_bridge_activate.rs"
        )
        close = source(
            "zircon_editor/src/ui/retained_host/document_tab_pointer/"
            "host_document_tab_pointer_bridge_close.rs"
        )

        self.assertRegex(callback, re.compile(r"pointer_bridge\s*\.target_for_route"))
        for implementation in (callback, activate, close):
            self.assertNotIn("UiPointerEvent", implementation)
            self.assertNotIn("dispatch_event", implementation)
        self.assertNotIn("UiPoint", callback)
        self.assertNotIn("tab_width", callback)


if __name__ == "__main__":
    unittest.main()
