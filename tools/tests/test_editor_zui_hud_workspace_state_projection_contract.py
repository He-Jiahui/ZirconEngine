import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
HUD_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/"
    "workbench_hud_workspace.zui"
)


class EditorZuiHudWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_widget_canvas_and_screen_state_are_runtime_owned(self):
        with HUD_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "hud_widget_text_row",
            "hud_widget_button_row",
            "hud_canvas_row_01",
            "hud_canvas_row_02",
            "hud_binding_row",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                self.assertNotIn("selected", row.get("props", {}))
                self.assertNotIn("checked", row.get("props", {}))
                self.assertNotIn("workbench-row-selected", row.get("classes", []))

        screen = nodes["hud_screen_dropdown"]["props"]
        self.assertNotIn("text", screen)
        self.assertEqual("gameplay_hud", screen["value"])
        self.assertEqual("Gameplay HUD", screen["value_text"])
        self.assertEqual(
            [
                "gameplay_hud|label=Gameplay HUD",
                "pause_menu|label=Pause Menu",
                "main_menu|label=Main Menu",
            ],
            screen["options"],
        )

    def test_widget_canvas_and_preview_share_one_hud_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "hud_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        navigation = (WORKBENCH_BRIDGE_ROOT / "module_navigation.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("initialize_hud_workspace_state()?", actions)
        self.assertIn("apply_hud_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.hud.widget_text.select",
            "workbench.module.hud.widget_button.select",
            "workbench.module.hud.minimap.select",
            "workbench.module.hud.ammo_panel.select",
            "workbench.module.hud.binding_ammo.select",
            "workbench.module.hud.preview.invoke",
            "WorkbenchHudCenterTitle",
            "WorkbenchHudValidationRow",
            "HUD_WIDGET_ROWS",
            "HUD_CANVAS_ROWS",
        ):
            self.assertIn(required, state)
        self.assertIn("HUD_WIDGET_ROW_CONTROLS", navigation)
        self.assertIn("HUD_CANVAS_ROW_CONTROLS", navigation)

    def test_panel_preview_does_not_embed_context_free_feedback(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        feedback_function = feedback.split("fn module_command_feedback(", 1)[1]
        self.assertNotIn(
            "Preview refreshed   localization warning remains", feedback_function
        )

    def test_widget_title_and_preview_use_the_selected_screen_label(self):
        state = (WORKBENCH_BRIDGE_ROOT / "hud_workspace_state.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            'control_string(HUD_SCREEN_DROPDOWN, "value_text")', state
        )
        self.assertNotIn('format!("Gameplay HUD / {}", profile.label)', state)

    def test_preview_feedback_reads_current_dpi_and_locale(self):
        state = (WORKBENCH_BRIDGE_ROOT / "hud_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        preview = state.split("fn apply_hud_preview_feedback", 1)[1].split(
            "fn hud_screen_label", 1
        )[0]

        self.assertIn('control_string("WorkbenchHudDpiField", "value")', preview)
        self.assertIn('control_string("WorkbenchHudLocaleField", "value")', preview)


if __name__ == "__main__":
    unittest.main()
