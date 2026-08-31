import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
WORKSPACE_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/"
    "animation/workbench_extension_blend_space_workspace.zui"
)
DETAILS_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/animation/"
    "workbench_blend_space_details.zui"
)
EXTENSION_FEEDBACK = WORKBENCH_BRIDGE_ROOT / "extension_module_feedback.rs"


class EditorZuiBlendSpaceSelectionProjectionContractTests(unittest.TestCase):
    def test_authored_asset_rows_do_not_claim_selection_for_other_content(self):
        with WORKSPACE_ASSET.open("rb") as source:
            document = tomllib.load(source)

        idle_row = document["nodes"]["blend_space_idle_run_row"]
        self.assertNotIn("workbench-row-selected", idle_row.get("classes", []))
        self.assertFalse(idle_row.get("props", {}).get("selected", False))
        self.assertEqual(
            "Search blend assets",
            document["nodes"]["blend_space_search"]["props"]["placeholder"],
        )

    def test_asset_dropdown_offers_every_selectable_asset(self):
        with DETAILS_ASSET.open("rb") as source:
            document = tomllib.load(source)

        asset_props = document["nodes"]["asset"]["props"]
        options = asset_props["options"]
        self.assertEqual(
            ["BS_Locomotion", "BS_Idle_Run", "BS_Strafe_Grid", "BS_Sprint_Lean"],
            options,
        )
        self.assertNotIn("text", asset_props)
        self.assertEqual("BS_Locomotion", asset_props["value_text"])

        interpolation_props = document["nodes"]["interpolation"]["props"]
        self.assertNotIn("text", interpolation_props)
        self.assertEqual("Triangulated", interpolation_props["value_text"])

    def test_asset_selection_projects_one_profile_into_all_visible_readouts(self):
        projection_path = WORKBENCH_BRIDGE_ROOT / "blend_space_selection.rs"
        self.assertTrue(projection_path.is_file())
        projection = projection_path.read_text(encoding="utf-8")

        for required in (
            "workbench.extension.blend_space.idle_run_row.select",
            "workbench.extension.blend_space.strafe_row.select",
            "workbench.extension.blend_space.sprint_row.select",
            "workbench.extension.blend_space.asset.edit",
            "workbench.extension.blend_space.asset.commit",
            "workbench.extension.blend_space.run_sample_table_row.select",
            "workbench.extension.blend_space.walk_sample_table_row.select",
            "workbench.extension.blend_space.diagonal_sample_table_row.select",
            "workbench.extension.blend_space.idle_sample_table_row.select",
            "WorkbenchExtensionBlendSpaceAssetSummary",
            "WorkbenchExtensionBlendSpacePreviewAsset",
            "WorkbenchExtensionBlendSpacePreviewStatus",
            "WorkbenchExtensionBlendSpaceAssetDropdown",
            "WorkbenchExtensionBlendSpaceSamplePositionProperty",
            "WorkbenchExtensionBlendSpaceSampleRateProperty",
            "WorkbenchExtensionBlendSpacePreviewTimeline",
            "WorkbenchSampleWeightsDirectionValue",
            "WorkbenchSampleWeightsSpeedValue",
            "WorkbenchSampleWeightsRunForward",
            "WorkbenchSampleWeightsRunLeft",
            "WorkbenchSampleWeightsRunRight",
            "WorkbenchSampleWeightsIdle",
        ):
            self.assertIn(required, projection)

    def test_search_and_pointer_actions_share_the_selection_projection(self):
        search = (WORKBENCH_BRIDGE_ROOT / "blend_space_search.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("select_blend_space_asset_control(control_id)?", search)
        self.assertNotIn(
            "select_exclusive_selected(ASSET_ROW_CONTROLS, control_id)?", search
        )
        self.assertIn("apply_blend_space_asset_selection_action(action_id)?", actions)
        self.assertIn("apply_blend_space_sample_selection_action(action_id)?", actions)

    def test_transport_status_uses_the_selected_preview_sample(self):
        transport = (WORKBENCH_BRIDGE_ROOT / "blend_space_transport.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("PREVIEW_ASSET_CONTROL", transport)
        self.assertIn("control_string(PREVIEW_ASSET_CONTROL, \"value\")", transport)
        self.assertNotIn('"Run_Fwd  |', transport)

    def test_command_feedback_uses_the_current_asset_and_sample_context(self):
        projection = (WORKBENCH_BRIDGE_ROOT / "blend_space_selection.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        legacy_feedback = EXTENSION_FEEDBACK.read_text(encoding="utf-8")

        self.assertIn("apply_blend_space_contextual_command_feedback", projection)
        self.assertIn(
            "apply_blend_space_contextual_command_feedback(action_id)?", actions
        )
        for retired in (
            "Preview queued   BS_Locomotion",
            "Selected Idle Sample",
            "Selected Diagonal Sample",
        ):
            self.assertNotIn(retired, legacy_feedback)


if __name__ == "__main__":
    unittest.main()
