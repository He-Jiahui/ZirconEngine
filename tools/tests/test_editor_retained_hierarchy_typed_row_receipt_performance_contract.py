import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/hierarchy_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class HierarchyTypedRowReceiptPerformanceContract(unittest.TestCase):
    def test_pointer_layout_keeps_only_scalar_hit_inputs(self):
        layout = source(
            "zircon_editor/src/ui/retained_host/hierarchy_pointer/"
            "hierarchy_pointer_layout.rs"
        )

        self.assertIn("pub item_count: usize", layout)
        self.assertNotIn("node_ids", layout)
        self.assertNotIn("String", layout)

    def test_route_is_copy_and_does_not_own_entity_text(self):
        route = source(
            "zircon_editor/src/ui/retained_host/hierarchy_pointer/"
            "hierarchy_pointer_route.rs"
        )

        self.assertIn("Clone, Copy", route)
        self.assertIn("Node { item_index: usize }", route)
        self.assertNotIn("node_id", route)
        self.assertNotIn("String", route)

    def test_bridge_no_longer_owns_a_generic_mirror_hit_surface(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/hierarchy_pointer/"
            "hierarchy_pointer_bridge.rs"
        )

        for forbidden in [
            "UiSurface",
            "UiPointerDispatcher",
            "EditorRouteIntentMap",
            "surface:",
            "dispatcher:",
            "route_intents:",
        ]:
            self.assertNotIn(forbidden, bridge)

    def test_generic_surface_dispatch_owners_are_deleted(self):
        retired = [
            "base_state.rs",
            "constants.rs",
            "dispatch_event.rs",
            "rebuild_surface.rs",
            "register_handled_pointer_node.rs",
            "route_id.rs",
        ]

        for name in retired:
            self.assertFalse((OWNER / name).exists(), name)

    def test_pointer_projection_reuses_the_committed_typed_row_arc(self):
        layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/hierarchy.rs"
        )
        recompute = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/"
            "pointer_surfaces.rs"
        )

        self.assertIn("scene_entries: Arc<[WorldInspectionHierarchyRow]>", layout)
        self.assertIn("let item_count = scene_entries.len()", layout)
        self.assertIn("item_count,", layout)
        self.assertIn("hierarchy_rows_arc()", recompute)
        self.assertNotIn("Arc::from(scene_entries)", layout)
        self.assertNotIn(".to_string()", layout)

    def test_pointer_handlers_use_direct_infallible_arithmetic_routing(self):
        handlers = "\n".join(
            source(f"zircon_editor/src/ui/retained_host/hierarchy_pointer/{name}")
            for name in ["handle_click.rs", "handle_move.rs", "handle_scroll.rs"]
        )

        self.assertIn("self.route_at_point(point)", handlers)
        for forbidden in [
            "UiPointerEvent",
            "UiPointerEventKind",
            "dispatch_event",
            "Result<HierarchyPointerDispatch",
        ]:
            self.assertNotIn(forbidden, handlers)

    def test_click_dispatch_resolves_one_typed_row_without_parse(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/"
            "hierarchy.rs"
        )

        self.assertIn("scene_entries: &[WorldInspectionHierarchyRow]", callback)
        self.assertIn("selected_entity", callback)
        self.assertIn("entry.entity", callback)
        self.assertNotIn(".parse()", callback)
        self.assertNotIn("Invalid node id", callback)

    def test_direct_route_preserves_o1_hit_and_visible_row_paint(self):
        route = source(
            "zircon_editor/src/ui/retained_host/hierarchy_pointer/route_at_point.rs"
        )
        painter = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/native_panes/hierarchy.rs"
        )

        self.assertIn("item_index >= self.layout.item_count", route)
        self.assertIn(".floor() as usize", route)
        self.assertNotIn("dispatched_route", route)
        self.assertIn("visible_hierarchy_row_range", painter)

    def test_hierarchy_route_intent_mirror_binding_is_removed(self):
        route_intent = source(
            "zircon_editor/src/ui/retained_host/route_intent/map.rs"
        )

        self.assertNotIn("Hierarchy(HierarchyPointerRoute)", route_intent)
        self.assertNotIn("hierarchy_route_for_pointer_dispatch", route_intent)


if __name__ == "__main__":
    unittest.main()
