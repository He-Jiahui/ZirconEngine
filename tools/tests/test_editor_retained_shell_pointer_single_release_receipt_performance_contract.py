import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class ShellPointerSingleReleaseReceiptPerformanceContract(unittest.TestCase):
    def test_drag_target_sync_returns_the_typed_route_receipt(self):
        drag_drop = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs"
        )

        self.assertIn(
            "fn sync_drag_target_group(\n        &mut self,\n        x: f32,\n        y: f32,\n"
            "    ) -> Option<HostShellPointerRoute>",
            drag_drop,
        )
        self.assertIn("route\n    }", drag_drop)

    def test_drag_release_routes_once_and_passes_the_receipt_to_resolution(self):
        drag_drop = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs"
        )
        route = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop/route.rs"
        )

        release = drag_drop.split("fn dispatch_drag_drop_from_pointer", 1)[1]
        self.assertIn("let pointer_route = self.sync_drag_target_group(x, y);", release)
        self.assertIn("pointer_route,", release)
        self.assertEqual(release.count("drag_route_at("), 0)
        self.assertEqual(route.count("drag_route_at("), 0)

    def test_drop_resolution_borrows_the_committed_layout_model_and_frames(self):
        route = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop/route.rs"
        )

        self.assertIn("pointer_route: Option<HostShellPointerRoute>", route)
        self.assertIn("self.committed_shell_state.as_ref()?", route)
        self.assertIn("&committed.layout", route)
        self.assertIn("&committed.model", route)
        self.assertIn("committed.layout_frames", route)
        for forbidden in [
            "runtime.current_layout()",
            "build_chrome()",
            "WorkbenchModelBuildCount",
            "project_command_eval_snapshot",
            "runtime.commands().lock()",
            "WorkbenchViewModel::build_with_context",
        ]:
            self.assertNotIn(forbidden, route)

    def test_resize_move_uses_the_captured_app_receipt_without_redispatch(self):
        movement = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/"
            "drawer_resize/movement.rs"
        )
        update = movement.split("fn update_drawer_resize_capture", 1)[1].split(
            "fn finish_drawer_resize_capture", 1
        )[0]

        self.assertNotIn("shell_pointer_bridge.update_resize", update)
        self.assertIn("apply_drawer_resize_pointer_position", update)

    def test_resize_release_dispatches_up_once_and_applies_final_coordinate_once(self):
        movement = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/"
            "drawer_resize/movement.rs"
        )
        finish = movement.split("fn finish_drawer_resize_capture", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertNotIn("self.update_drawer_resize_capture(x, y)", finish)
        self.assertEqual(finish.count("finish_resize(UiPoint::new(x, y))"), 1)
        self.assertEqual(
            finish.count("apply_drawer_resize_pointer_position(active, x, y)"), 1
        )

    def test_rejected_resize_setup_cancels_the_surface_capture(self):
        capture = source(
            "zircon_editor/src/ui/retained_host/app/workspace_docking/"
            "drawer_resize/capture.rs"
        )
        bridge = source(
            "zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs"
        )

        self.assertIn("cancel_resize", capture)
        self.assertGreaterEqual(capture.count("self.shell_pointer_bridge.cancel_resize();"), 2)
        self.assertIn("pub(crate) fn cancel_resize(&mut self)", bridge)
        self.assertIn("self.resize_surface.release_pointer_capture();", bridge)


if __name__ == "__main__":
    unittest.main()
