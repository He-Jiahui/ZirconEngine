import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE_WORKSPACE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core"
)
CORE_WORKSPACES = (
    CORE_WORKSPACE_ROOT / "gameplay/workbench_effect_workspace.zui",
    CORE_WORKSPACE_ROOT / "gameplay/workbench_ability_workspace.zui",
    CORE_WORKSPACE_ROOT / "gameplay/workbench_tags_workspace.zui",
    CORE_WORKSPACE_ROOT / "ai/workbench_perception_workspace.zui",
    CORE_WORKSPACE_ROOT / "rendering/workbench_material_workspace.zui",
    CORE_WORKSPACE_ROOT / "ai/workbench_behavior_workspace.zui",
    CORE_WORKSPACE_ROOT / "rendering/workbench_render_workspace.zui",
    CORE_WORKSPACE_ROOT / "assets/workbench_assets_workspace.zui",
    CORE_WORKSPACE_ROOT / "rendering/workbench_vfx_workspace.zui",
    CORE_WORKSPACE_ROOT / "ui/workbench_hud_workspace.zui",
)
CORE_NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/module_navigation.rs"
)
REFERENCE_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)
TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_module_template_bindings.rs"
)
INTERACTION_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_projection/interaction.rs"
)
DOCUMENT_MODULE_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_projection/document_module.rs"
)
ASSETS_WORKSPACE_ROUTE_TESTS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_module_template_bindings/assets_workspace_routes.rs"
)
SURFACE_CHROME_ROUTE_TESTS = REPO_ROOT / (
    "zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/"
    "workbench_projection/surface_contract/chrome_routes.rs"
)
CORE_PANEL_TAB_ACTION = re.compile(
    r'workbench\.module\.[a-z_]+\.[a-z_]+_tab\.select'
)
CORE_PANEL_TAB_CONTROL = re.compile(
    r"Workbench(?:Effect|Material|Behavior|Assets|Vfx|Ability|Tags|Perception|Render|Hud)"
    r"[A-Za-z]+Tab\b"
)


class EditorZuiCoreWorkspaceModeAuthorityContractTests(unittest.TestCase):
    def test_core_workspaces_do_not_offer_tabs_without_distinct_content_stacks(self):
        for path in CORE_WORKSPACES:
            with self.subTest(workspace=path.name):
                with path.open("rb") as source:
                    document = tomllib.load(source)
                self.assertFalse(
                    any(
                        node.get("component") == "WorkbenchTab"
                        for node in document["nodes"].values()
                    )
                )
                self.assertFalse(
                    any("workbench_tab.zui#WorkbenchTab" in item for item in document["imports"]["widgets"])
                )

    def test_core_panel_tab_state_is_removed_from_every_runtime_registry(self):
        sources = (
            CORE_NAVIGATION,
            REFERENCE_ACTIONS,
            PREVIEW_ACTIONS,
            TEMPLATE_BINDINGS,
            INTERACTION_TESTS,
            DOCUMENT_MODULE_TESTS,
            ASSETS_WORKSPACE_ROUTE_TESTS,
            SURFACE_CHROME_ROUTE_TESTS,
        )
        for path in sources:
            with self.subTest(source=path.name):
                text = path.read_text(encoding="utf-8")
                self.assertIsNone(CORE_PANEL_TAB_ACTION.search(text))
                self.assertIsNone(CORE_PANEL_TAB_CONTROL.search(text))

        navigation = CORE_NAVIGATION.read_text(encoding="utf-8")
        reference_actions = REFERENCE_ACTIONS.read_text(encoding="utf-8")
        self.assertNotIn("PANEL_TAB_CONTROLS", navigation)
        self.assertNotIn("workbench_module_panel_tab_control_id", navigation)
        self.assertNotIn("workbench_module_panel_tab_group", navigation)
        self.assertNotIn("CORE_MODULE_DEFAULT_TAB_ACTIONS", reference_actions)


if __name__ == "__main__":
    unittest.main()
