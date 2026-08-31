import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE_MODULE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core"
)
PROPERTY_EDITOR_IMPORT = (
    "res://ui/editor/components/workbench/composites/inputs/"
    "workbench_property_editor_row.zui#WorkbenchPropertyEditorRow"
)

MODULES = {
    "ai/workbench_behavior_workspace.zui": (
        "behavior_right_content",
        {
            "behavior_blackboard_field": ("Blackboard", "BB_Enemy"),
            "behavior_ai_field": ("AI Controller", "AIController_Enemy"),
            "behavior_state_field": ("Preview State", "Running"),
        },
    ),
    "ai/workbench_perception_workspace.zui": (
        "perception_right_content",
        {
            "perception_config_dropdown": ("Configuration", "Guard_Perception"),
            "perception_los_field": ("Line of Sight", "On"),
            "perception_team_field": ("Team Filter", "All"),
        },
    ),
    "assets/workbench_assets_workspace.zui": (
        "assets_right_content",
        {
            "assets_type_field": ("Type", "Static Mesh"),
            "assets_path_field": ("Path", "/Game/Environment/Forest"),
            "assets_owner_field": ("Source", "glTF importer"),
        },
    ),
    "gameplay/workbench_ability_workspace.zui": (
        "ability_right_content",
        {
            "ability_name_field": ("Name", "GA_DashAttack"),
            "ability_net_policy_dropdown": ("Net Policy", "server_initiated"),
            "ability_cooldown_field": ("Cooldown", "4.00s"),
        },
    ),
    "gameplay/workbench_effect_workspace.zui": (
        "effect_right_content",
        {
            "effect_name_field": ("Name", "GE_HealthRegen"),
            "effect_tag_field": ("Gameplay Tag", "Effect.Health.Regen"),
            "effect_stack_field": ("Stacking", "Aggregate by Source"),
            "effect_magnitude_field": ("Magnitude", "10.0"),
        },
    ),
    "gameplay/workbench_tags_workspace.zui": (
        "tags_right_content",
        {
            "tags_search_field": ("Search", ""),
            "tags_redirect_field": ("Redirect", "Character.State.Stun"),
            "tags_owner_field": ("Owner", "DefaultGameplayTags.ini"),
        },
    ),
    "rendering/workbench_material_workspace.zui": (
        "material_right_content",
        {
            "material_domain_dropdown": ("Domain", "surface"),
            "material_blend_dropdown": ("Blend Mode", "opaque"),
            "material_preview_field": ("Preview Mesh", "Sphere"),
        },
    ),
    "rendering/workbench_render_workspace.zui": (
        "render_right_content",
        {
            "render_pipeline_field": ("Pipeline", "MainPipeline.rp"),
            "render_platform_dropdown": ("Platform", "windows_dx12"),
            "render_frame_field": ("Frame", "1234"),
        },
    ),
    "rendering/workbench_vfx_workspace.zui": (
        "vfx_right_content",
        {
            "vfx_system_field": ("System", "P_Bolt_01"),
            "vfx_bounds_field": ("Fixed Bounds", "400 cm"),
            "vfx_sort_field": ("Sort", "Depth"),
        },
    ),
    "ui/workbench_hud_workspace.zui": (
        "hud_right_content",
        {
            "hud_screen_dropdown": ("Screen", "gameplay_hud"),
            "hud_dpi_field": ("DPI Scale", "1.00"),
            "hud_locale_field": ("Locale", "en-US"),
        },
    ),
}


def load_document(relative_path):
    path = CORE_MODULE_ROOT / relative_path
    with path.open("rb") as source:
        return tomllib.load(source)


def property_row_name(control_name):
    for suffix in ("_field", "_dropdown"):
        if control_name.endswith(suffix):
            return f"{control_name.removesuffix(suffix)}_property_row"
    raise AssertionError(f"unsupported property control name: {control_name}")


class EditorZuiCoreModuleDetailsPropertyContractTests(unittest.TestCase):
    def test_details_use_shared_name_value_rows(self):
        for relative_path, (details_name, properties) in MODULES.items():
            with self.subTest(module=relative_path):
                document = load_document(relative_path)
                nodes = document["nodes"]
                self.assertIn(PROPERTY_EDITOR_IMPORT, document["imports"]["widgets"])

                details_children = [
                    child["node"] for child in nodes[details_name]["children"]
                ]
                for control_name, (label, _) in properties.items():
                    row_name = property_row_name(control_name)
                    row = nodes[row_name]
                    self.assertEqual("WorkbenchPropertyEditorRow", row["component"])
                    self.assertEqual(label, row["props"]["text"])
                    self.assertEqual(
                        [{"node": control_name, "slot": {"name": "value"}}],
                        row["children"],
                    )
                    self.assertEqual("Stretch", row["layout"]["width"]["stretch"])
                    self.assertEqual("Fixed", row["layout"]["height"]["stretch"])
                    self.assertIn(row_name, details_children)
                    self.assertNotIn(control_name, details_children)

    def test_property_values_do_not_repeat_their_labels(self):
        for relative_path, (_, properties) in MODULES.items():
            with self.subTest(module=relative_path):
                nodes = load_document(relative_path)["nodes"]
                for control_name, (_, value) in properties.items():
                    self.assertEqual(value, nodes[control_name]["props"]["value"])

                if relative_path.endswith("workbench_tags_workspace.zui"):
                    self.assertEqual(
                        "Search tags...",
                        nodes["tags_search_field"]["props"]["placeholder"],
                    )

    def test_value_controls_retain_their_edit_and_commit_routes(self):
        for relative_path, (_, properties) in MODULES.items():
            with self.subTest(module=relative_path):
                nodes = load_document(relative_path)["nodes"]
                for control_name in properties:
                    module, key_with_kind = control_name.split("_", 1)
                    key = key_with_kind.removesuffix("_field").removesuffix(
                        "_dropdown"
                    )
                    route_stem = f"workbench.module.{module}.{key}"
                    self.assertEqual(
                        [
                            ("Change", f"{route_stem}.edit"),
                            ("Submit", f"{route_stem}.commit"),
                        ],
                        [
                            (event["event"], event["route"])
                            for event in nodes[control_name]["events"]
                        ],
                    )


if __name__ == "__main__":
    unittest.main()
