import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
PERCEPTION_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/"
    "workbench_perception_workspace.zui"
)


class EditorZuiPerceptionWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_agent_selection_is_owned_by_runtime_projection(self):
        with PERCEPTION_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "perception_guard_row",
            "perception_sniper_row",
            "perception_map_row_01",
            "perception_map_row_02",
            "perception_map_row_03",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                props = row.get("props", {})
                self.assertNotIn("selected", props)
                self.assertNotIn("checked", props)
                self.assertNotIn("workbench-row-selected", row.get("classes", []))
        config = nodes["perception_config_dropdown"]["props"]
        self.assertNotIn("text", config)
        self.assertEqual("Guard_Perception", config["value"])
        self.assertEqual("Guard Perception", config["value_text"])

    def test_selection_projects_agent_context_into_map_and_details(self):
        state = (WORKBENCH_BRIDGE_ROOT / "perception_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("initialize_perception_workspace_state()?", actions)
        self.assertIn("apply_perception_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.perception.guard.select",
            "workbench.module.perception.sniper.select",
            "workbench.module.perception.sight_cone.select",
            "workbench.module.perception.hearing_pulse.select",
            "workbench.module.perception.stimulus.select",
            "workbench.module.perception.simulate.invoke",
            "WorkbenchPerceptionCenterTitle",
            "WorkbenchPerceptionSightConeRow",
            "WorkbenchPerceptionHearingPulseRow",
            "WorkbenchPerceptionStimulusRow",
            "WorkbenchPerceptionEventRow",
            "WorkbenchPerceptionConfigDropdown",
            "WorkbenchPerceptionLosField",
            "WorkbenchPerceptionTeamField",
            "PERCEPTION_MAP_ROWS",
            '"value_text"',
        ):
            self.assertIn(required, state)

    def test_simulate_feedback_does_not_embed_the_guard_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("AI_Guard_01   simulation tick   00:12.4", feedback)

    def test_simulation_reads_current_perception_configuration(self):
        state = (WORKBENCH_BRIDGE_ROOT / "perception_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        simulation = state.split(
            "fn apply_perception_simulation_feedback", 1
        )[1].split("fn set_perception_string", 1)[0]

        self.assertIn(
            'control_string("WorkbenchPerceptionConfigDropdown", "value_text")',
            simulation,
        )
        self.assertIn(
            'control_string("WorkbenchPerceptionLosField", "value")', simulation
        )
        self.assertIn(
            'control_string("WorkbenchPerceptionTeamField", "value")', simulation
        )
        self.assertIn('"WorkbenchPerceptionCenterTitle"', simulation)


if __name__ == "__main__":
    unittest.main()
