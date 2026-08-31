import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
OWNER = ROOT / "zircon_editor/src/ui/retained_host/welcome_recent_pointer"


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class WelcomeRecentDirectReceiptPerformanceContract(unittest.TestCase):
    def test_bridge_no_longer_owns_a_generic_mirror_hit_surface(self):
        bridge = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_bridge.rs"
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

    def test_generic_surface_and_route_conversion_owners_are_deleted(self):
        retired = [
            "constants.rs",
            "register_handled_pointer_node.rs",
            "route_conversion.rs",
            "welcome_recent_pointer_bridge_dispatch_event.rs",
            "welcome_recent_pointer_bridge_rebuild_surface.rs",
            "welcome_recent_pointer_route_intent.rs",
        ]

        for name in retired:
            self.assertFalse((OWNER / name).exists(), name)

    def test_route_is_copy_index_action_without_project_path(self):
        route = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_route.rs"
        )

        self.assertIn("Clone, Copy", route)
        self.assertIn("item_index: usize", route)
        self.assertIn("action: WelcomeRecentPointerAction", route)
        self.assertNotIn("path", route)
        self.assertNotIn("String", route)

    def test_handlers_use_direct_infallible_arithmetic_hit(self):
        handlers = "\n".join(
            source(
                "zircon_editor/src/ui/retained_host/welcome_recent_pointer/" + name
            )
            for name in [
                "welcome_recent_pointer_bridge_handle_click.rs",
                "welcome_recent_pointer_bridge_handle_move.rs",
                "welcome_recent_pointer_bridge_handle_scroll.rs",
            ]
        )
        route = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_bridge_project_route.rs"
        )

        self.assertIn("self.route_at_point(point)", handlers)
        self.assertIn(".floor() as usize", route)
        self.assertIn("recent_project_paths.len()", route)
        for forbidden in [
            "UiPointerEvent",
            "UiPointerEventKind",
            "dispatch_event",
            "Result<WelcomeRecentPointerDispatch",
            "dispatched_route",
        ]:
            self.assertNotIn(forbidden, handlers + route)

    def test_dispatch_carries_changed_and_state_is_copy(self):
        dispatch = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_dispatch.rs"
        )
        state = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_state.rs"
        )

        self.assertIn("pub changed: bool", dispatch)
        self.assertIn("Clone, Copy", dispatch)
        self.assertIn("Clone, Copy", state)

    def test_click_uses_committed_layout_without_chrome_snapshot(self):
        click = source(
            "zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/click.rs"
        )

        self.assertIn("use_committed_pointer_layout", click)
        self.assertNotIn("chrome_snapshot", click)
        self.assertNotIn("sync_welcome_recent_pointer_layout", click)

    def test_shared_click_borrows_one_committed_action_target(self):
        callback = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/shared_pointer/"
            "welcome_recent.rs"
        )
        route = source(
            "zircon_editor/src/ui/retained_host/welcome_recent_pointer/"
            "welcome_recent_pointer_bridge_project_route.rs"
        )

        self.assertIn("action_target_for_route", callback)
        self.assertIn("-> Option<(WelcomeRecentPointerAction, &str)>", route)
        self.assertNotIn("path, ..", callback)
        self.assertNotIn("path: String", route)

    def test_bridge_is_the_single_welcome_pointer_state_owner(self):
        app = source("zircon_editor/src/ui/retained_host/app.rs")
        startup = source(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/"
            "state/interaction.rs"
        )
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs"
        )

        self.assertNotIn("welcome_recent_pointer_state:", app)
        self.assertNotIn("welcome_recent_pointer_state:", startup)
        self.assertIn("welcome_recent_pointer_bridge.state()", pointer_layout)

    def test_adapters_publish_only_changed_state(self):
        click = source(
            "zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/click.rs"
        )
        move = source(
            "zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/motion.rs"
        )
        scroll = source(
            "zircon_editor/src/ui/retained_host/app/welcome_recent_pointer/scroll.rs"
        )
        pointer_layout = source(
            "zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs"
        )

        self.assertIn("size_state_changed || dispatch.pointer.changed", click)
        self.assertIn("size_state_changed || dispatch.changed", move)
        self.assertIn("size_state_changed || dispatch.changed", scroll)
        self.assertIn("sync_welcome_recent_pointer_size(&mut self) -> bool", pointer_layout)
        self.assertEqual(
            pointer_layout.count("apply_welcome_recent_pointer_state_to_ui();"), 1
        )

    def test_welcome_route_intent_mirror_binding_is_removed(self):
        route_intent = source(
            "zircon_editor/src/ui/retained_host/route_intent/map.rs"
        )

        self.assertNotIn("WelcomeRecent(WelcomeRecentPointerRouteIntent)", route_intent)
        self.assertNotIn("welcome_recent_route_for_pointer_dispatch", route_intent)

    def test_native_paint_remains_bounded_to_visible_rows(self):
        painter = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/welcome/recent_projects.rs"
        )
        rows = source(
            "zircon_editor/src/ui/retained_host/host_contract/"
            "paint_workbench_renderer/welcome/recent_projects/rows.rs"
        )

        self.assertIn("welcome_recent_visible_row_count", painter)
        self.assertIn("for index in 0..visible_rows", rows)


if __name__ == "__main__":
    unittest.main()
