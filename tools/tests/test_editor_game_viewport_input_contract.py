import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorGameViewportInputContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_hit_testing_preserves_scene_and_game_viewport_identity(self) -> None:
        target = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "routing/pane_route/target.rs"
        )
        mapping = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/native_pointer/"
            "routing/panes/target.rs"
        )

        self.assertIn("SceneViewport(&'a str)", target)
        self.assertIn("GameViewport(&'a str)", target)
        self.assertIn('"Scene" => PanePointerTarget::SceneViewport', mapping)
        self.assertIn('"Game" => PanePointerTarget::GameViewport', mapping)
        self.assertNotIn('"Scene" | "Game" => PanePointerTarget::Viewport', mapping)

    def test_native_host_uses_distinct_scene_and_game_callbacks(self) -> None:
        callbacks = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/pane.rs"
        )
        wiring = self.read(
            "zircon_editor/src/ui/retained_host/app/callback_wiring/"
            "pane_surface/viewport.rs"
        )

        self.assertIn("scene_viewport_pointer_event", callbacks)
        self.assertIn("game_viewport_pointer_event", callbacks)
        self.assertIn("on_scene_viewport_pointer_event", wiring)
        self.assertIn("on_game_viewport_pointer_event", wiring)
        self.assertNotIn("on_viewport_pointer_event", wiring)

    def test_play_controller_is_the_single_runtime_input_gate(self) -> None:
        controller = self.read("zircon_editor/src/core/play/controller.rs")
        route = controller.split("pub fn route_preview_input", 1)[1].split(
            "pub fn preview_input_active", 1
        )[0]

        self.assertIn("PlayMode::Playing", route)
        self.assertIn("kind: PlayKind::Play", route)
        self.assertIn("return Ok(false)", route)
        self.assertNotIn("PlayKind::Simulate", route)
        self.assertRegex(route, r"play_gateway\s*\.handle_event\(event\)")

    def test_game_keyboard_routes_before_editor_keymap_and_keeps_release_events(self) -> None:
        keyboard = self.read(
            "zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs"
        )
        game_input = self.read(
            "zircon_editor/src/ui/retained_host/app/viewport/game_input.rs"
        )
        unhandled = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/text_input/"
            "keyboard/unhandled.rs"
        )
        tick = self.read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs"
        )
        shell_state = self.read("zircon_editor/src/ui/workbench/shell_state.rs")

        game_route = keyboard.find("route_focused_game_keyboard_input")
        keymap = keyboard.find("dispatch_keyboard_keymap_command")
        self.assertGreaterEqual(game_route, 0)
        self.assertGreater(keymap, game_route)
        active_gate = game_input.find("play_preview_input_active")
        focused_gate = game_input.find("play_preview_view_focused")
        self.assertGreaterEqual(active_gate, 0)
        self.assertGreater(focused_gate, active_gate)
        self.assertNotIn("event.state != ElementState::Pressed", unhandled)
        self.assertIn("sync_play_preview_input_focus", tick)
        self.assertIn("clear_text_input_focus", tick)
        self.assertIn("play_preview_input_focus_active", tick)
        self.assertNotIn("current_view_instances", tick)
        focused_query = shell_state.split(
            "pub(crate) fn play_preview_view_focused", 1
        )[1].split("\n    }", 1)[0]
        self.assertIn("current_focused_view_matches", focused_query)
        self.assertNotIn("current_view_instances", focused_query)
        self.assertIn("play_preview_view_focus_active", tick)
        self.assertIn("route_play_preview_focus_lost", tick)

    def test_focused_game_window_lifecycle_clears_runtime_input_state(self) -> None:
        native_focus = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/"
            "events/focus.rs"
        )
        wiring = self.read(
            "zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/"
            "runtime.rs"
        )
        game_input = self.read(
            "zircon_editor/src/ui/retained_host/app/viewport/game_input.rs"
        )
        runtime_events = self.read("zircon_runtime/src/dynamic_api/session/events.rs")

        self.assertIn("invoke_native_window_focus_lost", native_focus)
        self.assertIn("native_window_focus_lost()", wiring)
        self.assertIn("ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1", game_input)
        self.assertIn("ZrRuntimeEventV1::lifecycle", game_input)
        self.assertIn("self.submit_input_event(InputEvent::FocusLost)", runtime_events)

    def test_simulate_injects_editor_camera_and_presents_the_play_world_in_scene(self) -> None:
        constants = self.read(
            "zircon_runtime_interface/src/runtime_api/constants.rs"
        )
        session = self.read(
            "zircon_runtime_interface/src/runtime_api/session/mod.rs"
        )
        runtime_events = self.read(
            "zircon_runtime/src/dynamic_api/session/events.rs"
        )
        runtime_camera = self.read(
            "zircon_runtime/src/dynamic_api/camera_controller.rs"
        )
        controller = self.read("zircon_editor/src/core/play/controller.rs")
        tick = self.read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs"
        )
        preview = self.read(
            "zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs"
        )
        images = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs"
        )
        menu = self.read(
            "zircon_editor/src/ui/host/editor_event_execution/menu_action.rs"
        )

        self.assertIn("ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1", constants)
        self.assertIn("ZrRuntimeViewportCameraV1", session)
        self.assertIn("ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1", runtime_events)
        self.assertIn("apply_editor_camera", runtime_camera)

        route = controller.split("pub fn route_simulate_camera", 1)[1].split(
            "pub fn", 1
        )[0]
        self.assertIn("kind: PlayKind::Simulate", route)
        self.assertNotIn("kind: PlayKind::Play", route)
        self.assertIn("ZrRuntimeEventV1::viewport_camera", route)

        camera_sync = tick.find("self.sync_simulate_preview_camera()")
        runtime_pump = tick.find("self.runtime.pump_runtime_event_consumers()")
        self.assertGreaterEqual(camera_sync, 0)
        self.assertGreater(runtime_pump, camera_sync)

        self.assertIn("simulate: Option<Arc<HostViewportImageData>>", images)
        self.assertRegex(
            images,
            r'"Scene"\s*=>\s*self\.simulate\(\)\.or_else\(\|\| self\.scene\(\)\)',
        )
        self.assertIn("PlayKind::Simulate", preview)
        self.assertIn("set_simulate_viewport_frame", preview)
        self.assertIn("clear_game_viewport_image", preview)

        enter = menu.split("MenuAction::EnterPlayMode =>", 1)[1].split(
            "MenuAction::ExitPlayMode =>", 1
        )[0]
        self.assertRegex(
            enter,
            r"backend_attachable\s*&&\s*play_kind\s*==\s*PlayKind::Play",
        )


if __name__ == "__main__":
    unittest.main()
