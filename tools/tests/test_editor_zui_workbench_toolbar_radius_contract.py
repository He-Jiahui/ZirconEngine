import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_workbench_strict.zui"
WORKBENCH_TAB = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_tab.zui"
)

QUIET_TOOLBAR_NODES = {
    "toolbar_menu",
    "toolbar_assets",
    "toolbar_open",
    "toolbar_save",
    "module_more",
    "module_save",
    "module_browse",
    "module_diff",
    "module_simulate",
    "tool_select",
    "tool_move",
    "tool_rotate",
    "tool_scale",
    "tool_snap",
    "run_play",
    "run_stop",
    "run_mode",
    "layout_grid",
    "theme_toggle",
}
TOOLBAR_MODULE_TAB_NODES = {
    "module_scene",
    "module_effect",
    "module_ability",
    "module_tags",
    "module_perception",
    "module_material",
    "module_behavior",
    "module_render",
    "module_assets",
    "module_vfx",
    "module_hud",
}


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiWorkbenchToolbarRadiusContractTests(unittest.TestCase):
    def test_toolbar_actions_use_the_larger_radius_tier(self):
        toolbar = load_document(TOOLBAR)
        nodes = toolbar["nodes"]
        actions = [
            node
            for node in nodes.values()
            if node.get("component") in {"WorkbenchIconButton", "WorkbenchButton"}
            and node.get("control_id", "").startswith(
                ("WorkbenchToolbar", "WorkbenchTool", "WorkbenchRun", "WorkbenchLayout", "WorkbenchTheme", "WorkbenchModule")
            )
        ]
        self.assertGreaterEqual(len(actions), 19)
        for node in actions:
            self.assertIn(
                "workbench-toolbar-action",
                node.get("classes", []),
                node.get("control_id"),
            )

    def test_toolbar_action_states_keep_the_large_radius(self):
        theme = load_document(THEME)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for stylesheet in theme["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        for selector in (
            ".workbench-toolbar-action",
            ".workbench-toolbar-action:hovered",
            ".workbench-toolbar-action:pressed",
            ".workbench-toolbar-action:selected",
            ".workbench-toolbar-action:focus-visible",
        ):
            self.assertEqual(
                "$editor.control.radius.large",
                rules[selector]["radius"],
                selector,
            )

    def test_regular_controls_keep_the_control_radius_tier(self):
        theme = load_document(THEME)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for stylesheet in theme["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        self.assertEqual(
            "$editor.control.radius.control",
            rules[".workbench-control-button"]["radius"],
        )

    def test_secondary_toolbar_actions_are_quiet_until_interaction(self):
        toolbar = load_document(TOOLBAR)
        nodes = toolbar["nodes"]

        self.assertEqual(19, len(QUIET_TOOLBAR_NODES))
        for node_id in QUIET_TOOLBAR_NODES:
            self.assertIn(
                "workbench-quiet-action",
                nodes[node_id].get("classes", []),
                node_id,
            )
        self.assertNotIn(
            "workbench-quiet-action",
            nodes["module_compile"].get("classes", []),
            "Compile is the persistent primary action and must keep its emphasis",
        )

        theme = load_document(THEME)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for stylesheet in theme["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        quiet = rules[".workbench-quiet-action"]
        self.assertEqual("transparent", quiet["background_color"])
        self.assertEqual("transparent", quiet["border_color"])
        for selector, expected_surface in (
            (".workbench-quiet-action:hovered", "$workbench_hover"),
            (".workbench-quiet-action:pressed", "$workbench_active"),
            (".workbench-quiet-action:checked", "$workbench_selected"),
            (".workbench-quiet-action:selected", "$workbench_selected"),
            (".workbench-quiet-action:popup_open", "$workbench_selected"),
        ):
            self.assertEqual(expected_surface, rules[selector]["background_color"])

    def test_toolbar_module_tabs_use_a_local_quiet_large_radius_tier(self):
        toolbar = load_document(TOOLBAR)
        nodes = toolbar["nodes"]

        self.assertEqual(11, len(TOOLBAR_MODULE_TAB_NODES))
        for node_id in TOOLBAR_MODULE_TAB_NODES:
            self.assertIn(
                "workbench-toolbar-tab",
                nodes[node_id].get("classes", []),
                node_id,
            )

        theme = load_document(THEME)
        rules = {
            rule["selector"]: rule["set"]["self"]
            for stylesheet in theme["stylesheets"]
            for rule in stylesheet.get("rules", [])
        }
        base = rules[".workbench-toolbar-tab"]
        self.assertEqual("transparent", base["background_color"])
        self.assertEqual("transparent", base["border_color"])
        for selector in (
            ".workbench-toolbar-tab",
            ".workbench-toolbar-tab:hovered",
            ".workbench-toolbar-tab:pressed",
            ".workbench-toolbar-tab:checked",
            ".workbench-toolbar-tab:selected",
            ".workbench-toolbar-tab:focus-visible",
        ):
            self.assertEqual(
                "$editor.control.radius.large",
                rules[selector]["radius"],
                selector,
            )
        self.assertEqual(
            "$workbench_selected",
            rules[".workbench-toolbar-tab:selected"]["background_color"],
        )

        regular_tab = load_document(WORKBENCH_TAB)["nodes"]["root"]
        self.assertEqual(
            "$editor.control.radius.small",
            regular_tab["props"]["corner_radius"],
        )


if __name__ == "__main__":
    unittest.main()
