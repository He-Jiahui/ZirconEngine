import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE_MODULE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core"
)

MODULES = {
    "ai/workbench_behavior_workspace.zui": {
        "panel": "behavior_left",
        "title": "behavior_title",
        "body": ["behavior_selector_row", "behavior_attack_row"],
    },
    "ai/workbench_perception_workspace.zui": {
        "panel": "perception_left",
        "title": "perception_title",
        "body": ["perception_guard_row", "perception_sniper_row"],
    },
    "assets/workbench_assets_workspace.zui": {
        "panel": "assets_left",
        "title": "assets_title",
        "body": ["assets_forest_row", "assets_material_row"],
    },
    "gameplay/workbench_ability_workspace.zui": {
        "panel": "ability_left",
        "title": "ability_title",
        "scroll": "ability_left_content",
        "body": ["ability_task_row_01", "ability_task_row_02", "ability_asset_row"],
    },
    "gameplay/workbench_effect_workspace.zui": {
        "panel": "effect_left",
        "title": "effect_title",
        "scroll": "effect_left_content",
        "body": [
            "effect_asset_search",
            "effect_health_regen_row",
            "effect_damage_fire_row",
            "effect_search_empty",
            "effect_stack_row",
            "effect_cue_row",
        ],
    },
    "gameplay/workbench_tags_workspace.zui": {
        "panel": "tags_left",
        "title": "tags_title",
        "fixed": ["tags_actions"],
        "body": ["tags_source_row"],
    },
    "rendering/workbench_material_workspace.zui": {
        "panel": "material_left",
        "title": "material_title",
        "body": [
            "material_base_color_row",
            "material_roughness_row",
            "material_normal_row",
        ],
    },
    "rendering/workbench_render_workspace.zui": {
        "panel": "render_left",
        "title": "render_title",
        "body": ["render_pass_row_01", "render_pass_row_02"],
    },
    "rendering/workbench_vfx_workspace.zui": {
        "panel": "vfx_left",
        "title": "vfx_title",
        "body": ["vfx_emitter_row", "vfx_curve_row"],
    },
    "ui/workbench_hud_workspace.zui": {
        "panel": "hud_left",
        "title": "hud_title",
        "body": ["hud_widget_text_row", "hud_widget_button_row"],
    },
}


def load_document(relative_path):
    path = CORE_MODULE_ROOT / relative_path
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiCoreModuleTabStripContractTests(unittest.TestCase):
    def test_core_module_sidebars_use_one_fixed_title_then_real_content(self):
        for relative_path, contract in MODULES.items():
            with self.subTest(module=relative_path):
                nodes = load_document(relative_path)["nodes"]
                panel_children = [
                    child["node"] for child in nodes[contract["panel"]]["children"]
                ]
                scroll = contract.get("scroll", f'{contract["panel"]}_content')
                self.assertEqual(
                    [contract["title"], *contract.get("fixed", []), scroll],
                    panel_children,
                )

                title = nodes[contract["title"]]
                self.assertEqual("WorkbenchSectionTitle", title["component"])
                self.assertEqual("Fixed", title["layout"]["height"]["stretch"])

    def test_scrollable_module_bodies_keep_title_fixed_and_rows_reachable(self):
        for relative_path, contract in MODULES.items():
            with self.subTest(module=relative_path):
                nodes = load_document(relative_path)["nodes"]
                scroll_name = contract.get("scroll", f'{contract["panel"]}_content')
                self.assertEqual(
                    contract["body"],
                    [child["node"] for child in nodes[scroll_name]["children"]],
                )
                scroll = nodes[scroll_name]
                self.assertEqual("ScrollableBox", scroll["component"])
                self.assertEqual("Receive", scroll["layout"]["input_policy"])
                self.assertEqual("Vertical", scroll["layout"]["container"]["axis"])

    def test_tags_actions_share_one_compact_horizontal_command_row(self):
        nodes = load_document("gameplay/workbench_tags_workspace.zui")["nodes"]
        actions = nodes["tags_actions"]
        self.assertEqual("HorizontalGroup", actions["component"])
        self.assertEqual("HorizontalBox", actions["layout"]["container"]["kind"])
        self.assertEqual(
            ["tags_action_add", "tags_action_rename"],
            [child["node"] for child in actions["children"]],
        )
        self.assertNotIn("events", actions)
        for action in actions["children"]:
            node = nodes[action["node"]]
            self.assertEqual("Stretch", node["layout"]["width"]["stretch"])


if __name__ == "__main__":
    unittest.main()
