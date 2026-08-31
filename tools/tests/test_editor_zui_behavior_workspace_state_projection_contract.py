import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
BEHAVIOR_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/"
    "workbench_behavior_workspace.zui"
)


class EditorZuiBehaviorWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_behavior_selection_is_initialized_by_runtime_projection(self):
        with BEHAVIOR_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "behavior_selector_row",
            "behavior_attack_row",
            "behavior_node_row_01",
            "behavior_node_row_02",
            "behavior_node_row_03",
        ):
            with self.subTest(row=row_name):
                props = nodes[row_name].get("props", {})
                self.assertNotIn("selected", props)
                self.assertNotIn("checked", props)

    def test_tree_and_graph_actions_share_one_behavior_profile_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "behavior_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("initialize_behavior_workspace_state()?", actions)
        self.assertIn("apply_behavior_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.behavior.selector_row.select",
            "workbench.module.behavior.attack_row.select",
            "workbench.module.behavior.node_selector.select",
            "workbench.module.behavior.node_attack.select",
            "workbench.module.behavior.node_cooldown.select",
            "workbench.module.behavior.validate.invoke",
            "WorkbenchBehaviorCenterTitle",
            "WorkbenchBehaviorBlackboardField",
            "WorkbenchBehaviorAiField",
            "WorkbenchBehaviorStateField",
            "WorkbenchBehaviorOutputRow",
            "BEHAVIOR_TREE_ROWS",
            "BEHAVIOR_GRAPH_ROWS",
        ):
            self.assertIn(required, state)

    def test_validate_feedback_does_not_embed_the_selector_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("Validation: selector branch is reachable", feedback)

    def test_validation_reads_current_behavior_details(self):
        state = (WORKBENCH_BRIDGE_ROOT / "behavior_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        validation = state.split("fn apply_behavior_validation_feedback", 1)[1].split(
            "fn set_behavior_string", 1
        )[0]

        for control_id in (
            "WorkbenchBehaviorBlackboardField",
            "WorkbenchBehaviorAiField",
            "WorkbenchBehaviorStateField",
        ):
            with self.subTest(control_id=control_id):
                self.assertIn(
                    f'control_string("{control_id}", "value")', validation
                )
        self.assertNotIn("profile.validation", validation)


if __name__ == "__main__":
    unittest.main()
