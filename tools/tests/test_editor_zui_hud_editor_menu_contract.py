import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
HUD_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/ui/"
    "workbench_hud_workspace.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
HUD_EDITOR_MENU = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/hud_editor_menu.rs"
)
WINDOW_MENU_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/window_menu_state.rs"
)
CONTROL_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs"
)
MODULE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs"
)
NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs/ui_diagnostics.rs"
)
OBSERVABILITY_NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/extension_module_navigation/specs/ui_diagnostics/observability.rs"
)
EXTENSION_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/"
    "ui_diagnostics.rs"
)
OBSERVABILITY_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/"
    "ui_diagnostics/observability.rs"
)


TOOLS = [
    ("Console Diagnostics", "console_diagnostics"),
    ("Runtime Diagnostics", "runtime_diagnostics"),
    ("Telemetry Dashboard", "telemetry_dashboard"),
    ("Performance", "performance"),
    ("Font Atlas", "font_atlas"),
    ("Menu Flow", "menu_flow"),
    ("Accessibility Audit", "accessibility_audit"),
    ("Icon Library", "icon_library"),
    ("UI Binding", "ui_binding"),
    ("UI Asset Editor", "ui_asset_editor"),
]


class EditorZuiHudEditorMenuContractTests(unittest.TestCase):
    def test_hud_center_uses_one_tools_trigger_and_keeps_preview_separate(self):
        with HUD_WORKSPACE.open("rb") as source:
            document = tomllib.load(source)
        nodes = document["nodes"]
        self.assertEqual(
            ["hud_center_title", "hud_header_fill", "hud_tools_button", "hud_preview_button"],
            [child["node"] for child in nodes["hud_center_header"]["children"]],
        )
        self.assertNotIn("hud_extension_shortcut_scroll", nodes)
        trigger = nodes["hud_tools_button"]
        self.assertEqual("WorkbenchHudTools", trigger["control_id"])
        self.assertEqual("HUD Tools", trigger["props"]["text"])
        self.assertEqual("workbench.module.hud.tools.open", trigger["events"][0]["route"])

    def test_window_overlay_owns_the_anchored_hud_tools_menu(self):
        source = WORKBENCH_WINDOW.read_text("utf-8")
        self.assertIn('{ node = "hud_tools_menu" }', source)
        self.assertIn('control_id = "WorkbenchHudToolsMenu"', source)
        self.assertIn('control_id = "WorkbenchHudTools"', source)
        for _, action in TOOLS:
            self.assertIn(f"menu.item.hud.{action}", source)

    def test_rust_menu_authority_maps_every_item_to_existing_extension_action(self):
        source = HUD_EDITOR_MENU.read_text("utf-8")
        self.assertIn("HUD_EDITOR_MENU_COMMANDS", source)
        for _, action in TOOLS:
            self.assertIn(f'menu_action_id: "menu.item.hud.{action}"', source)
            self.assertIn(
                f'extension_action_id: "workbench.extension.{action}.open"',
                source,
            )
        self.assertIn("dispatch_workbench_hud_editor_menu_item_state", CONTROL_DISPATCH.read_text("utf-8"))

    def test_hud_navigation_uses_menu_identity_and_removes_direct_openers(self):
        state = WINDOW_MENU_STATE.read_text("utf-8")
        self.assertIn('trigger_control_id: "WorkbenchHudTools"', state)
        self.assertIn('menu_control_id: "WorkbenchHudToolsMenu"', state)
        self.assertIn('"workbench.module.hud.tools.open"', state)

        bindings = MODULE_BINDINGS.read_text("utf-8")
        self.assertIn('"HudToolsOpen"', bindings)
        self.assertIn('"workbench.module.hud.tools.open"', bindings)

        navigation_sources = (
            NAVIGATION.read_text("utf-8"),
            OBSERVABILITY_NAVIGATION.read_text("utf-8"),
        )
        for source in navigation_sources:
            self.assertNotRegex(source, r'WorkbenchHud(?:Console|Runtime|Telemetry|Performance|FontAtlas|MenuFlow|AccessibilityAudit|IconLibrary|UiBinding|UiAssetEditor)Button')
        self.assertEqual(
            20,
            sum(source.count('"WorkbenchHudToolsMenu"') for source in navigation_sources),
        )

        for path in (EXTENSION_BINDINGS, OBSERVABILITY_BINDINGS):
            source = path.read_text("utf-8")
            for _, action in TOOLS:
                self.assertNotIn(f'click("{action.title().replace("_", "")}Open"', source)


if __name__ == "__main__":
    unittest.main()
