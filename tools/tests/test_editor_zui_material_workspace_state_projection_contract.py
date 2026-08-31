import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
MATERIAL_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/"
    "workbench_material_workspace.zui"
)
TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_module_template_bindings.rs"
)


class EditorZuiMaterialWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_parameter_and_graph_selection_are_runtime_owned(self):
        with MATERIAL_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        self.assertEqual(
            [
                {"node": "material_base_color_row"},
                {"node": "material_roughness_row"},
                {"node": "material_normal_row"},
            ],
            nodes["material_left_content"]["children"],
        )
        for row_name in (
            "material_base_color_row",
            "material_roughness_row",
            "material_normal_row",
            "material_node_row_01",
            "material_node_row_02",
            "material_node_row_03",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                props = row.get("props", {})
                self.assertNotIn("selected", props)
                self.assertNotIn("checked", props)
                self.assertNotIn("workbench-row-selected", row.get("classes", []))

        roughness = nodes["material_roughness_row"]
        self.assertEqual("WorkbenchMaterialRoughnessRow", roughness["control_id"])
        self.assertEqual("Roughness", roughness["props"]["text"])
        self.assertEqual(
            "workbench.module.material.roughness",
            roughness["events"][0]["route"],
        )

    def test_parameter_and_graph_actions_share_one_material_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "material_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        bindings = TEMPLATE_BINDINGS.read_text(encoding="utf-8")

        self.assertIn("initialize_material_workspace_state()?", actions)
        self.assertIn("apply_material_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.material.base_color_row.select",
            "workbench.module.material.roughness_row.select",
            "workbench.module.material.normal_row.select",
            "workbench.module.material.node_albedo.select",
            "workbench.module.material.node_roughness.select",
            "workbench.module.material.node_normal.select",
            "workbench.module.material.compile.invoke",
            "WorkbenchMaterialCenterTitle",
            "WorkbenchMaterialOutputRow",
            "MATERIAL_PARAMETER_ROWS",
            "MATERIAL_GRAPH_ROWS",
        ):
            self.assertIn(required, state)
        self.assertIn("MaterialRoughnessRow", bindings)
        self.assertIn("workbench.module.material.roughness_row.select", bindings)

    def test_compile_feedback_does_not_embed_the_default_material_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("Shader Output: compile complete, 2 warnings", feedback)

    def test_compile_feedback_reads_current_domain_and_blend_labels(self):
        state = (WORKBENCH_BRIDGE_ROOT / "material_workspace_state.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            'control_string(control_id, "value_text")', state
        )
        self.assertIn('MATERIAL_DOMAIN_DROPDOWN, "Surface"', state)
        self.assertIn('MATERIAL_BLEND_DROPDOWN, "Opaque"', state)

    def test_compile_feedback_reads_current_preview_mesh(self):
        state = (WORKBENCH_BRIDGE_ROOT / "material_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        compile_feedback = state.split("fn apply_material_compile_feedback", 1)[
            1
        ].split("fn material_dropdown_label", 1)[0]

        self.assertIn(
            'control_string("WorkbenchMaterialPreviewField", "value")',
            compile_feedback,
        )
        self.assertIn('"WorkbenchMaterialCenterTitle"', compile_feedback)


if __name__ == "__main__":
    unittest.main()
