import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
ASSETS_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/"
    "workbench_assets_workspace.zui"
)


class EditorZuiAssetsWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_folder_and_asset_selection_are_runtime_owned(self):
        with ASSETS_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "assets_forest_row",
            "assets_material_row",
            "assets_table_row_01",
            "assets_table_row_02",
            "assets_table_row_03",
        ):
            with self.subTest(row=row_name):
                props = nodes[row_name].get("props", {})
                self.assertNotIn("selected", props)
                self.assertNotIn("checked", props)

    def test_folder_and_asset_actions_share_one_profile_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "assets_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("initialize_assets_workspace_state()?", actions)
        self.assertIn("apply_assets_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.assets.forest_row.select",
            "workbench.module.assets.material_row.select",
            "workbench.module.assets.table_tree.select",
            "workbench.module.assets.table_material.select",
            "workbench.module.assets.table_texture.select",
            "workbench.module.assets.import.invoke",
            "WorkbenchAssetsCenterTitle",
            "WorkbenchAssetsTableRow01",
            "WorkbenchAssetsTypeField",
            "WorkbenchAssetsPathField",
            "WorkbenchAssetsOwnerField",
            "WorkbenchAssetsOutputRow",
            "UiValue::Array",
        ):
            self.assertIn(required, state)

    def test_import_feedback_does_not_embed_the_tree_fixture(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("Import: queued SM_Tree_Oak_01 and dependencies", feedback)

    def test_import_feedback_reads_current_asset_metadata(self):
        state = (WORKBENCH_BRIDGE_ROOT / "assets_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        import_feedback = state.split("fn apply_asset_import_feedback", 1)[1].split(
            "fn set_asset_string", 1
        )[0]

        for control_id in (
            "WorkbenchAssetsTypeField",
            "WorkbenchAssetsPathField",
            "WorkbenchAssetsOwnerField",
        ):
            with self.subTest(control_id=control_id):
                self.assertIn(
                    f'control_string("{control_id}", "value")', import_feedback
                )


if __name__ == "__main__":
    unittest.main()
