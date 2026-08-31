import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INSPECTOR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_inspector_panel.zui"
)
WORKBENCH_BRIDGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
CONTROL_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiInspectorFilterContractTests(unittest.TestCase):
    def test_identity_and_filter_stay_fixed_above_the_scrolling_properties(self):
        nodes = load_document(INSPECTOR)["nodes"]

        self.assertEqual(
            [
                "inspector_header",
                "inspector_filter_row",
                "inspector_content",
            ],
            [child["node"] for child in nodes["inspector_panel"]["children"]],
        )
        self.assertEqual(
            "$editor.density.gap.small",
            nodes["inspector_panel"]["layout"]["container"]["gap"],
        )
        self.assertEqual(
            ["inspector_title"],
            [child["node"] for child in nodes["inspector_header"]["children"]],
        )
        scrolling_children = [
            child["node"] for child in nodes["inspector_content"]["children"]
        ]
        self.assertNotIn("inspector_title", scrolling_children)
        self.assertNotIn("inspector_filter_row", scrolling_children)
        self.assertIn("inspector_filter_empty", scrolling_children)

    def test_filter_is_a_value_bound_search_field_with_edit_and_commit_routes(self):
        document = load_document(INSPECTOR)
        nodes = document["nodes"]
        imports = document["imports"]["widgets"]
        search = nodes["inspector_filter_field"]

        self.assertIn(
            "res://ui/editor/components/workbench/primitives/inputs/"
            "workbench_search_input.zui#WorkbenchSearchInput",
            imports,
        )
        self.assertEqual("WorkbenchSearchInput", search["component"])
        self.assertEqual("WorkbenchInspectorFilter", search["control_id"])
        self.assertEqual("query", search["widget"]["value_property"])
        self.assertEqual("Filter properties...", search["props"]["placeholder"])
        self.assertEqual(
            [
                {
                    "id": "Workbench/InspectorSearchEdit",
                    "event": "Change",
                    "component_event": "ValueChanged",
                    "route": "workbench.inspector.search.edit",
                },
                {
                    "id": "Workbench/InspectorSearchCommit",
                    "event": "Submit",
                    "component_event": "Commit",
                    "route": "workbench.inspector.search.commit",
                },
            ],
            search["events"],
        )

    def test_filter_keeps_source_properties_separate_from_virtual_row_projection(self):
        componentized = (WORKBENCH_BRIDGE / "componentized_window.rs").read_text(
            encoding="utf-8"
        )
        data_sync = (WORKBENCH_BRIDGE / "data_sync.rs").read_text(encoding="utf-8")
        inspector_filter = (WORKBENCH_BRIDGE / "inspector_filter.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("inspector_source_properties", componentized)
        self.assertIn("component_properties", componentized)
        self.assertIn("set_inspector_filter_source", data_sync)
        self.assertIn("fn apply_inspector_filter", inspector_filter)
        self.assertIn("component_property_matches", inspector_filter)
        self.assertIn("component_property_item_keys", inspector_filter)
        self.assertIn("sync_component_property_binding", inspector_filter)

    def test_edit_and_pointer_clear_routes_share_the_bridge_filter_authority(self):
        control_dispatch = CONTROL_DISPATCH.read_text(encoding="utf-8")
        reference_actions = (
            WORKBENCH_BRIDGE / "reference_menu_actions.rs"
        ).read_text(encoding="utf-8")
        module = (WORKBENCH_BRIDGE / "mod.rs").read_text(encoding="utf-8")

        self.assertIn("mod inspector_filter;", module)
        self.assertIn(".edit_inspector_filter", control_dispatch)
        self.assertIn("is_inspector_filter_action", reference_actions)
        self.assertIn("apply_inspector_filter_action", reference_actions)

    def test_nonfunctional_inspector_history_mode_is_removed_end_to_end(self):
        document_text = INSPECTOR.read_text(encoding="utf-8")
        reference_actions = (
            WORKBENCH_BRIDGE / "reference_menu_actions.rs"
        ).read_text(encoding="utf-8")
        preview_actions = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
        ).read_text(encoding="utf-8")
        bindings = (
            REPO_ROOT
            / "zircon_editor/src/ui/template_runtime/builtin/"
            "workbench_window_template_bindings.rs"
        ).read_text(encoding="utf-8")

        for retired in (
            "WorkbenchInspectorTabInspector",
            "WorkbenchInspectorTabHistory",
            "PanelTab/InspectorMain",
            "PanelTab/InspectorHistory",
            "inspector.main_tab.select",
            "inspector.history_tab.select",
        ):
            with self.subTest(retired=retired):
                self.assertNotIn(retired, document_text)
                self.assertNotIn(retired, reference_actions)
                self.assertNotIn(retired, preview_actions)
                self.assertNotIn(retired, bindings)


if __name__ == "__main__":
    unittest.main()
