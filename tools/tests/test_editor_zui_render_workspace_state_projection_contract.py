import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
RENDER_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/"
    "workbench_render_workspace.zui"
)


class EditorZuiRenderWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_pass_graph_and_platform_state_are_runtime_owned(self):
        with RENDER_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "render_pass_row_01",
            "render_pass_row_02",
            "render_graph_row_01",
            "render_graph_row_02",
            "render_resource_row",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                self.assertNotIn("selected", row.get("props", {}))
                self.assertNotIn("checked", row.get("props", {}))
                self.assertNotIn("workbench-row-selected", row.get("classes", []))

        platform = nodes["render_platform_dropdown"]["props"]
        self.assertNotIn("text", platform)
        self.assertEqual("windows_dx12", platform["value"])
        self.assertEqual("Windows DX12", platform["value_text"])
        self.assertEqual(
            [
                "windows_dx12|label=Windows DX12",
                "vulkan|label=Vulkan",
                "metal|label=Metal",
            ],
            platform["options"],
        )

    def test_pass_graph_and_compile_actions_share_one_render_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "render_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        navigation = (WORKBENCH_BRIDGE_ROOT / "module_navigation.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("initialize_render_workspace_state()?", actions)
        self.assertIn("apply_render_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.render.lighting_pass.select",
            "workbench.module.render.bloom_pass.select",
            "workbench.module.render.frame_start.select",
            "workbench.module.render.lighting_node.select",
            "workbench.module.render.scene_color.select",
            "workbench.module.render.compile.invoke",
            "WorkbenchRenderCenterTitle",
            "WorkbenchRenderCaptureRow",
            "RENDER_PASS_ROWS",
            "RENDER_GRAPH_ROWS",
        ):
            self.assertIn(required, state)
        self.assertIn("RENDER_PASS_ROW_CONTROLS", navigation)
        self.assertIn("RENDER_GRAPH_ROW_CONTROLS", navigation)

    def test_render_compile_does_not_embed_context_free_capture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn(
            "Windows DX12   30 fps   GPU 6.24 ms   compiled", feedback
        )

    def test_pass_and_compile_feedback_use_the_selected_platform_label(self):
        state = (WORKBENCH_BRIDGE_ROOT / "render_workspace_state.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            'control_string(RENDER_PLATFORM_DROPDOWN, "value_text")',
            state,
        )
        self.assertNotIn("Pass: {} selected   Windows DX12", state)
        self.assertNotIn('"Windows DX12   {} compiled   {} selected"', state)

    def test_compile_feedback_reads_current_pipeline_and_frame(self):
        state = (WORKBENCH_BRIDGE_ROOT / "render_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        compile_feedback = state.split("fn apply_render_compile_feedback", 1)[1].split(
            "fn render_platform_label", 1
        )[0]

        self.assertIn(
            'control_string("WorkbenchRenderPipelineField", "value")',
            compile_feedback,
        )
        self.assertIn(
            'control_string("WorkbenchRenderFrameField", "value")',
            compile_feedback,
        )
        self.assertIn('"WorkbenchRenderCenterTitle"', compile_feedback)


if __name__ == "__main__":
    unittest.main()
