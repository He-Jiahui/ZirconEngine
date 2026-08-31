import unittest
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = "zircon_editor/src/ui/retained_host/activity_rail_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class ActivityRailStableRoutePerformanceContract(unittest.TestCase):
    def test_route_is_a_copy_index_without_owned_tab_payload(self):
        route = source(f"{OWNER}/host_activity_rail_pointer_route.rs")
        insert = source(f"{OWNER}/insert_strip.rs")

        self.assertIn("Clone, Copy", route)
        self.assertNotIn("slot: String", route)
        self.assertNotIn("instance_id: String", route)
        self.assertNotIn("slot: tab.slot.clone()", insert)
        self.assertNotIn("instance_id: tab.instance_id.clone()", insert)

    def test_layout_keeps_typed_slot_and_view_instance_identity(self):
        item = source(f"{OWNER}/host_activity_rail_pointer_item.rs")
        collect = source(f"{OWNER}/collect_tabs.rs")

        self.assertIn("slot: ActivityDrawerSlot", item)
        self.assertIn("instance_id: ViewInstanceId", item)
        self.assertIn("slot: stack.slot", collect)
        self.assertNotIn("drawer_slot_key", collect)
        self.assertNotIn(".to_string()", collect)

    def test_product_local_click_performs_exactly_one_dispatch(self):
        click = source(f"{OWNER}/handle_click.rs")
        local = click.split("pub(crate) fn handle_click(", 1)[1].split(
            "pub(crate) fn handle_click_at_global_point(", 1
        )[0]

        self.assertEqual(local.count("self.dispatch_event("), 1)
        self.assertNotIn("projected_route", local)
        self.assertIn("pub(crate) fn handle_click_at_global_point(", click)

    def test_shared_dispatch_borrows_target_from_pointer_layout(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/activity_rail.rs"
        )
        bridge = source(f"{OWNER}/host_activity_rail_pointer_bridge.rs")
        route_map = source("zircon_editor/src/ui/retained_host/route_intent/map.rs")

        self.assertRegex(callback, re.compile(r"pointer_bridge\s*\.target_for_button"))
        self.assertIn("pub(crate) fn target_for_button", bridge)
        self.assertIn("Some(*route)", route_map)


if __name__ == "__main__":
    unittest.main()
