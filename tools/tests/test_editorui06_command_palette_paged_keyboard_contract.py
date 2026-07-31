from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class CommandPalettePagedKeyboardContractTests(unittest.TestCase):
    def test_runtime_reducer_emits_typed_window_requests_without_wrapping(self) -> None:
        reducer = source(
            "zircon_runtime/src/ui/component/state_reducer/command_palette.rs"
        )

        for token in [
            "WINDOW_REQUEST_OFFSET",
            "WINDOW_REQUEST_CURRENT_OFFSET",
            "WINDOW_REQUEST_FOCUS",
            "WINDOW_REQUEST_GENERATION",
            "MATCH_COUNT",
            "WINDOW_OFFSET",
            "UiComponentKeyboardAction::LargeIncrement",
            "UiComponentKeyboardAction::LargeDecrement",
            "request_window",
        ]:
            self.assertIn(token, reducer)
        self.assertNotIn("(self.current_index + 1) % self.rows.len()", reducer)

    def test_native_popup_navigation_distinguishes_row_and_window_moves(self) -> None:
        commands = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/commands.rs"
        )
        model = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/target/model.rs"
        )
        dispatch = source(
            "zircon_editor/src/ui/retained_host/host_contract/native_keyboard/dispatch.rs"
        )

        self.assertIn("NamedKey::PageDown", commands)
        self.assertIn("NamedKey::PageUp", commands)
        self.assertIn("PopupKeyboardMove::Window", model)
        self.assertIn("window_offset", model)
        self.assertIn("current_offset", model)
        self.assertIn("target_offset", model)
        self.assertIn("window_query", model)
        self.assertIn("total_count", model)
        self.assertIn("window_navigation_enabled", model)
        self.assertIn("self.current_index.checked_add(1)", model)
        self.assertIn("dispatch_popup_window_request", dispatch)

    def test_editor_host_requeries_existing_catalog_and_rejects_stale_generation(self) -> None:
        actions = source(
            "zircon_editor/src/ui/retained_host/app/command_palette_actions.rs"
        )
        bridge = source(
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/command_palette.rs"
        )

        self.assertIn("dispatch_workbench_command_palette_window_requested", actions)
        self.assertIn("catalog_generation", actions)
        self.assertIn("query != request_query", actions)
        self.assertIn("WindowRequestFocus", actions)
        self.assertIn("command_palette_query_window", actions)
        self.assertIn("WINDOW_COUNT", bridge)
        self.assertIn("VIRTUALIZATION_TOTAL_COUNT", bridge)


if __name__ == "__main__":
    unittest.main()
