import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE_ROOT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
TAGS_ASSET = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_tags_workspace.zui"
)


class EditorZuiTagsWorkspaceStateProjectionContractTests(unittest.TestCase):
    def test_source_and_tag_selection_are_runtime_owned(self):
        with TAGS_ASSET.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        for row_name in (
            "tags_source_row",
            "tags_table_row_01",
            "tags_table_row_02",
        ):
            with self.subTest(row=row_name):
                row = nodes[row_name]
                self.assertNotIn("selected", row.get("props", {}))
                self.assertNotIn("checked", row.get("props", {}))
                self.assertNotIn("workbench-row-selected", row.get("classes", []))

    def test_search_selection_and_commands_share_one_tag_projection(self):
        state = (WORKBENCH_BRIDGE_ROOT / "tags_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        actions = (WORKBENCH_BRIDGE_ROOT / "reference_menu_actions.rs").read_text(
            encoding="utf-8"
        )
        navigation = (WORKBENCH_BRIDGE_ROOT / "module_navigation.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("initialize_tags_workspace_state()?", actions)
        self.assertIn("apply_tags_workspace_action(action_id)?", actions)
        for required in (
            "workbench.module.tags.source_default.select",
            "workbench.module.tags.ability_activate.select",
            "workbench.module.tags.state_stunned.select",
            "workbench.module.tags.search.edit",
            "workbench.module.tags.search.commit",
            "workbench.module.tags.add.invoke",
            "workbench.module.tags.rename.invoke",
            "WorkbenchTagsCenterTitle",
            "WorkbenchTagsRedirectField",
            "WorkbenchTagsOwnerField",
            "WorkbenchTagsValidationRow",
            "TAGS_TABLE_ROWS",
        ):
            self.assertIn(required, state)
        self.assertIn("TAGS_SOURCE_ROW_CONTROLS", navigation)
        self.assertIn("TAGS_TABLE_ROW_CONTROLS", navigation)

    def test_tag_commands_do_not_embed_context_free_feedback(self):
        feedback = (WORKBENCH_BRIDGE_ROOT / "module_command_feedback.rs").read_text(
            encoding="utf-8"
        )
        for fixture in (
            "Add Tag   pending registry update",
            "Rename Tag   pending redirect update",
        ):
            self.assertNotIn(fixture, feedback)

    def test_tag_commands_read_current_owner_and_redirect(self):
        state = (WORKBENCH_BRIDGE_ROOT / "tags_workspace_state.rs").read_text(
            encoding="utf-8"
        )
        commands = state.split("fn apply_tag_command_feedback", 1)[1].split(
            "fn set_tags_string", 1
        )[0]

        self.assertIn('control_string("WorkbenchTagsOwnerField", "value")', commands)
        self.assertIn(
            'control_string("WorkbenchTagsRedirectField", "value")', commands
        )
        self.assertIn('"WorkbenchTagsCenterTitle"', commands)


if __name__ == "__main__":
    unittest.main()
