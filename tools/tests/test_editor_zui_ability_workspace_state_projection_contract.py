import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
ABILITY_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_ability_workspace.zui"
)


class EditorZuiAbilityWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_ability_selection_and_net_policy_are_runtime_owned(self):
        with ABILITY_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "ability_task_row_01",
            "ability_task_row_02",
            "ability_asset_row",
            "ability_phase_row_01",
            "ability_phase_row_02",
            "ability_graph_row",
        ):
            with self.subTest(row=row_name):
                node = nodes[row_name]
                props = node.get("props", {})
                self.assertNotIn("selected", props)
                self.assertNotIn("checked", props)
                self.assertNotIn("workbench-row-selected", node.get("classes", []))

        net_policy = nodes["ability_net_policy_dropdown"]["props"]
        self.assertNotIn("text", net_policy)
        self.assertEqual("server_initiated", net_policy["value"])
        self.assertEqual("Server Initiated", net_policy["value_text"])
        self.assertEqual(
            [
                "server_initiated|label=Server Initiated",
                "client_predicted|label=Client Predicted",
                "local_only|label=Local Only",
            ],
            net_policy["options"],
        )

    def test_task_phase_and_graph_actions_share_one_ability_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "ability_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("initialize_ability_workspace_state()?", actions)
        self.assertIn("apply_ability_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.ability.task_activate.select",
            "workbench.module.ability.task_cost.select",
            "workbench.module.ability.asset_dash.select",
            "workbench.module.ability.phase_activate.select",
            "workbench.module.ability.phase_cost.select",
            "workbench.module.ability.graph_select.select",
            "workbench.module.ability.playtest.invoke",
            "WorkbenchAbilityCenterTitle",
            "WorkbenchAbilityOutputRow",
            "WorkbenchAbilityNetPolicyDropdown",
            "ABILITY_TASK_ROWS",
            "ABILITY_GRAPH_ROWS",
        ):
            self.assertIn(required, state)

    def test_task_navigation_does_not_reset_edited_ability_properties(self):
        state = (WORKBENCH_BRIDGE_ROOT / "ability_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        profile_projection = state.split("fn project_ability_profile", 1)[1].split(
            "fn apply_ability_playtest_feedback", 1
        )[0]

        self.assertIn("initialize_ability_properties()?", state)
        for control_id in (
            "WorkbenchAbilityNameField",
            "WorkbenchAbilityNetPolicyDropdown",
            "WorkbenchAbilityCooldownField",
        ):
            with self.subTest(control_id=control_id):
                self.assertNotIn(control_id, profile_projection)

    def test_playtest_feedback_does_not_embed_the_activation_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn(
            "Playtest queued   predicted activation   GA_DashAttack", feedback
        )

    def test_playtest_feedback_reads_current_ability_properties(self):
        state = (WORKBENCH_BRIDGE_ROOT / "ability_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        playtest = state.split("fn apply_ability_playtest_feedback", 1)[1].split(
            "fn set_ability_string", 1
        )[0]

        for control_id, property_name in (
            ("WorkbenchAbilityNameField", "value"),
            ("WorkbenchAbilityNetPolicyDropdown", "value_text"),
            ("WorkbenchAbilityCooldownField", "value"),
        ):
            with self.subTest(control_id=control_id):
                self.assertIn(
                    f'control_string("{control_id}", "{property_name}")', playtest
                )


if __name__ == "__main__":
    unittest.main()
