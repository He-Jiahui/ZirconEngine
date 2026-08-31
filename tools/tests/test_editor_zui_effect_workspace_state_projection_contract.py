import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
EFFECT_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_effect_workspace.zui"
)


class EditorZuiEffectWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_effect_search_uses_the_shared_search_control_and_has_an_empty_state(self):
        with EFFECT_ASSET.open("rb") as source:
            document = tomllib.load(source)

        search = document["nodes"]["effect_asset_search"]
        self.assertEqual("WorkbenchSearchInput", search["component"])
        self.assertEqual("", search["props"]["query"])
        self.assertEqual("Search effects", search["props"]["placeholder"])
        self.assertIn("effect_search_empty", document["nodes"])
        self.assertIn(
            {"node": "effect_search_empty"},
            document["nodes"]["effect_left_content"]["children"],
        )

    def test_effect_asset_selection_is_initialized_by_rust_state(self):
        with EFFECT_ASSET.open("rb") as source:
            document = tomllib.load(source)

        for row_name in (
            "effect_health_regen_row",
            "effect_damage_fire_row",
            "effect_stack_row",
            "effect_cue_row",
            "effect_mod_health_row",
            "effect_mod_healing_row",
            "effect_mod_cap_row",
            "effect_graph_row",
            "effect_attribute_preview_row",
        ):
            with self.subTest(row=row_name):
                row = document["nodes"][row_name]
                for property_name in ("selected", "checked"):
                    self.assertNotIn(property_name, row.get("props", {}))

        state = (WORKBENCH_BRIDGE_ROOT / "effect_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("initialize_effect_workspace_state()?", actions)
        self.assertIn("apply_effect_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.effect.health_regen_row.select",
            "workbench.module.effect.damage_fire_row.select",
            "workbench.module.effect.search.edit",
            "workbench.module.effect.search.commit",
            "workbench.module.effect.apply.invoke",
            "workbench.module.effect.modifier_health.select",
            "workbench.module.effect.modifier_healing.select",
            "workbench.module.effect.modifier_cap.select",
            "workbench.module.effect.graph_select.select",
            "workbench.module.effect.attribute_preview.select",
            "WorkbenchEffectCenterTitle",
            "WorkbenchEffectNameField",
            "WorkbenchEffectTagField",
            "WorkbenchEffectOutputRow",
            "EFFECT_MODIFIER_ROWS",
            "EFFECT_GRAPH_ROWS",
            "EFFECT_PREVIEW_ROWS",
        ):
            self.assertIn(required, state)

    def test_effect_apply_feedback_does_not_embed_the_health_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("Simulation Output: applied +50 health preview", feedback)

    def test_effect_apply_reads_every_editable_effect_property(self):
        state = (WORKBENCH_BRIDGE_ROOT / "effect_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        apply_feedback = state.split("fn apply_effect_feedback", 1)[1].split(
            "fn set_effect_string", 1
        )[0]

        for control_id, property_name in (
            ("WorkbenchEffectNameField", "value"),
            ("WorkbenchEffectTagField", "value"),
            ("WorkbenchEffectMagnitudeField", "value"),
            ("WorkbenchEffectPolicyDropdown", "value_text"),
            ("WorkbenchEffectStackField", "value"),
        ):
            with self.subTest(control_id=control_id):
                self.assertIn(
                    f'control_string("{control_id}", "{property_name}")',
                    apply_feedback,
                )
        self.assertNotIn("profile.applied_output", apply_feedback)


if __name__ == "__main__":
    unittest.main()
