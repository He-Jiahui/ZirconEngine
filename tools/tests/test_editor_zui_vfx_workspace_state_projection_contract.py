import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
VFX_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/"
    "workbench_vfx_workspace.zui"
)


class EditorZuiVfxWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_context_and_parameter_selection_are_runtime_owned(self):
        with VFX_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "vfx_emitter_row",
            "vfx_curve_row",
            "vfx_spawn_row",
            "vfx_lifetime_row",
            "vfx_material_row",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                self.assertNotIn("selected", row.get("props", {}))
                self.assertNotIn("checked", row.get("props", {}))
                self.assertNotIn("workbench-row-selected", row.get("classes", []))

    def test_context_parameter_and_simulate_share_one_vfx_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "vfx_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        navigation = (WORKBENCH_BRIDGE_ROOT / "module_navigation.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("initialize_vfx_workspace_state()?", actions)
        self.assertIn("apply_vfx_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.vfx.emitter_row.select",
            "workbench.module.vfx.curve_row.select",
            "workbench.module.vfx.spawn_row.select",
            "workbench.module.vfx.lifetime_row.select",
            "workbench.module.vfx.material_row.select",
            "workbench.module.vfx.simulate.invoke",
            "WorkbenchVfxCenterTitle",
            "WorkbenchVfxOutputRow",
            "VFX_CONTEXT_ROWS",
            "VFX_PARAMETER_ROWS",
        ):
            self.assertIn(required, state)
        self.assertIn("VFX_CONTEXT_ROW_CONTROLS", navigation)
        self.assertIn("VFX_PARAMETER_ROW_CONTROLS", navigation)

    def test_panel_simulate_does_not_embed_context_free_feedback(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn(
            "Compile Output: simulation running, no errors", feedback
        )

    def test_simulation_reads_current_system_bounds_and_sort(self):
        state = (WORKBENCH_BRIDGE_ROOT / "vfx_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        simulation = state.split("fn apply_vfx_simulation_feedback", 1)[1].split(
            "fn set_vfx_string", 1
        )[0]

        for control_id in (
            "WorkbenchVfxSystemField",
            "WorkbenchVfxBoundsField",
            "WorkbenchVfxSortField",
        ):
            with self.subTest(control_id=control_id):
                self.assertIn(
                    f'control_string("{control_id}", "value")', simulation
                )


if __name__ == "__main__":
    unittest.main()
