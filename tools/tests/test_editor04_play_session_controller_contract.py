import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "zircon_editor/src"


class PlaySessionControllerContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (EDITOR / relative).read_text(encoding="utf-8")

    def test_core_play_hardcuts_the_old_backend_bridge_vocabulary(self) -> None:
        self.assertFalse((EDITOR / "core/play/bridge.rs").exists())
        production = "\n".join(
            self.source(path)
            for path in (
                "core/play/mod.rs",
                "ui/host/editor_host_event_controller.rs",
                "ui/host/editor_event_execution/menu_action.rs",
                "ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs",
            )
        )
        for retired in (
            "EditorRuntimePlayModeBackend",
            "EditorPlayBridge",
            "NativePluginEditorRuntimePlayModeBackend",
            "set_runtime_play_mode_backend",
        ):
            self.assertNotIn(retired, production)

    def test_controller_owns_edit_building_playing_transitions(self) -> None:
        mode = self.source("core/play/mode.rs")
        controller = self.source("core/play/controller.rs")
        tests = self.source("core/play/tests.rs")

        for variant in ("Edit", "Building", "Playing"):
            self.assertIn(variant, mode)
        for operation in (
            "request_play",
            "on_build_finished",
            "request_stop",
            "play_after_build",
        ):
            self.assertIn(operation, controller)
        for matrix_case in (
            "edit_request_play_without_build_enters_playing",
            "edit_request_play_with_build_waits_for_build_result",
            "playing_rejects_second_play_request",
            "stop_is_noop_in_edit_and_cancels_building",
        ):
            self.assertIn(matrix_case, tests)

    def test_command_eval_projects_the_controller_mode_not_chrome_mode(self) -> None:
        projection = self.source("ui/host/command_eval_projection.rs")
        reflection = self.source("ui/host/editor_event_runtime_reflection.rs")
        test_helper = self.source("ui/workbench/model/mod.rs")

        self.assertIn("PlayModeKind::Building", projection)
        self.assertIn("PlayStateKind::Building", projection)
        self.assertIn("self.play_sessions().mode()", projection)
        self.assertIn("self.play_sessions().mode()", reflection)
        self.assertNotIn("match chrome.session_mode", projection)
        self.assertIn("play_mode: crate::core::play::PlayModeKind", test_helper)
        self.assertNotIn("chrome.session_mode", test_helper)

    def test_menu_uses_controller_transition_api(self) -> None:
        menu = self.source("ui/host/editor_event_execution/menu_action.rs")

        self.assertIn(".play_sessions()", menu)
        self.assertIn(".request_play", menu)
        self.assertIn("play_sessions().request_stop", menu)
        self.assertNotIn(".backend().enter_play_mode", menu)
        self.assertNotIn(".backend().exit_play_mode", menu)

    def test_editor_profile_runtime_is_not_attached_as_a_play_world_at_startup(self) -> None:
        startup = self.source(
            "ui/retained_host/app/host_lifecycle/startup/with_viewport.rs"
        )
        product = startup.split("#[cfg(test)]", 1)[0]

        self.assertNotIn("attach_play_gateway(runtime_gateway", product)

    def test_backend_without_a_gateway_is_not_attachable_by_default(self) -> None:
        report = self.source("core/play/backend/report.rs")

        self.assertIn("gateway: None", report)
        self.assertIn("self.gateway.is_some()", report)
        self.assertNotIn("pub attachable: bool", report)

    def test_terminal_play_paths_detach_the_identity_qualified_gateway(self) -> None:
        shutdown = self.source(
            "ui/host/editor_host_event_controller/runtime_shutdown.rs"
        )
        core_controller = self.source("core/play/controller.rs")
        host_controller = self.source("ui/host/editor_host_event_controller.rs")
        menu = self.source("ui/host/editor_event_execution/menu_action.rs")

        self.assertIn("detach_terminal_play_gateway", shutdown)
        self.assertIn("fn detach_terminal_play_gateway", core_controller)
        self.assertIn("if mode.has_active_runtime()", core_controller)
        self.assertIn("detach_matching_identity", core_controller)
        self.assertIn("detach_terminal_play_gateway", host_controller)
        self.assertGreaterEqual(menu.count("detach_terminal_play_gateway"), 2)


if __name__ == "__main__":
    unittest.main()
