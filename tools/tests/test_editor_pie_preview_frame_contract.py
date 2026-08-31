import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class EditorPiePreviewFrameContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_play_core_captures_the_runtime_default_viewport_and_releases_output(self) -> None:
        preview = self.read("zircon_editor/src/core/play/preview_frame.rs")
        controller = self.read("zircon_editor/src/core/play/controller.rs")

        self.assertIn("ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1", controller)
        self.assertIn("pub fn capture_preview_frame", controller)
        self.assertRegex(controller, r"play_gateway\s*\.capture_frame")
        self.assertIn("PlayPreviewFrame::copy_and_release", controller)
        self.assertIn("Arc::<[u8]>::from(frame.rgba())", preview)
        self.assertRegex(preview, r"frame\s*\.release\(\)")

    def test_host_viewport_images_are_split_by_world_presentation_role(self) -> None:
        image = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs"
        )
        root = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs"
        )
        painter = self.read(
            "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/viewport.rs"
        )

        self.assertIn("struct HostViewportImageSet", image)
        self.assertIn("scene: Option<Arc<HostViewportImageData>>", image)
        self.assertIn("game: Option<Arc<HostViewportImageData>>", image)
        self.assertIn("pub viewport_images: HostViewportImageSet", root)
        self.assertRegex(
            painter,
            re.escape("viewport_images")
            + r"\s*\.for_pane\(pane\.kind\.as_str\(\)\)",
        )
        self.assertNotIn('matches!(pane.kind.as_str(), "Scene" | "Game")', painter)

    def test_retained_tick_captures_after_runtime_tick_and_clears_terminal_game_frame(self) -> None:
        tick = self.read(
            "zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs"
        )
        preview = self.read(
            "zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs"
        )

        runtime_pump = tick.find("self.runtime.pump_runtime_event_consumers()")
        preview_poll = tick.find("self.poll_play_preview_frame_for_native_host()")
        self.assertGreaterEqual(runtime_pump, 0)
        self.assertGreater(preview_poll, runtime_pump)
        self.assertIn("game_viewport_visible", preview)
        self.assertIn("capture_preview_frame", preview)
        self.assertIn("set_game_viewport_frame", preview)
        self.assertIn("clear_game_viewport_image", preview)

    def test_play_focuses_game_and_restores_the_pre_play_view_on_all_terminal_paths(self) -> None:
        shell_state = self.read("zircon_editor/src/ui/workbench/shell_state.rs")
        menu = self.read(
            "zircon_editor/src/ui/host/editor_event_execution/menu_action.rs"
        )
        host = self.read("zircon_editor/src/ui/host/editor_host_event_controller.rs")
        enter = menu.split("MenuAction::EnterPlayMode =>", 1)[1].split(
            "MenuAction::ExitPlayMode =>", 1
        )[0]
        exit_play = menu.split("MenuAction::ExitPlayMode =>", 1)[1]

        self.assertIn("play_preview_restore_view", shell_state)
        self.assertIn("focus_play_preview_view", enter)
        self.assertIn("restore_pre_play_view", exit_play)
        self.assertIn("restore_pre_play_view", host)


if __name__ == "__main__":
    unittest.main()
