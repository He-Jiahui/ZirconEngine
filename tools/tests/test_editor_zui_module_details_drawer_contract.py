import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MODULE_ROOT = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules"
)
TOP_TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
RESPONSIVE_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/responsive_layout.rs"
)
BRIDGE_STATE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/componentized_window.rs"
)
ACTION_DISPATCH = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)
TOOLBAR_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/toolbar_layout.rs"
)
TEMPLATE_BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)
SCAN_ONLY_UI_ASSET_EDITOR = MODULE_ROOT / (
    "extensions/ui/workbench_extension_ui_asset_editor_workspace.zui"
)
DRAWER_ROLE = "module_details"
DRAWER_ACTION = "workbench.module.details_drawer.toggle"


def writable_module_workspaces():
    workspaces = []
    for path in sorted(MODULE_ROOT.rglob("*_workspace.zui")):
        if path == SCAN_ONLY_UI_ASSET_EDITOR:
            continue
        with path.open("rb") as source:
            document = tomllib.load(source)
        if len(document.get("components", {})) != 1:
            continue
        component = next(iter(document["components"].values()))
        root = document["nodes"][component["root"]]
        classes = set(root.get("classes", []))
        if not classes.intersection(
            {"workbench-module-body", "workbench-extension-module-body"}
        ):
            continue
        workspaces.append((path, document, component["root"]))
    return workspaces


class EditorZuiModuleDetailsDrawerContractTests(unittest.TestCase):
    def test_module_center_rows_scroll_between_fixed_header_and_output(self):
        workspaces = writable_module_workspaces()
        self.assertEqual(53, len(workspaces))

        for path, document, root_name in workspaces:
            nodes = document["nodes"]
            root = nodes[root_name]
            main_row = nodes[root["children"][0]["node"]]
            center_name = main_row["children"][2]["node"]
            center = nodes[center_name]
            context = path.relative_to(REPO_ROOT).as_posix()

            if center_name == "blend_space_center":
                work_area = nodes["blend_space_center_work_area"]
                self.assertEqual("HorizontalGroup", work_area["component"], context)
                self.assertEqual("Stretch", work_area["layout"]["height"]["stretch"])
                continue

            if center_name == "tags_center":
                self.assertEqual(
                    [
                        "tags_center_title",
                        "tags_table_header",
                        "tags_table_content",
                        "tags_validation_row",
                    ],
                    [child["node"] for child in center["children"]],
                    context,
                )

            stretch_children = [
                child["node"]
                for child in center["children"]
                if nodes[child["node"]]
                .get("layout", {})
                .get("height", {})
                .get("stretch")
                == "Stretch"
            ]
            self.assertEqual(1, len(stretch_children), context)
            body = nodes[stretch_children[0]]
            self.assertEqual("ScrollableBox", body["component"], context)
            self.assertTrue(body["layout"]["clip"], context)
            self.assertEqual("Receive", body["layout"]["input_policy"], context)
            self.assertEqual("ScrollableBox", body["layout"]["container"]["kind"], context)
            self.assertEqual("Vertical", body["layout"]["container"]["axis"], context)
            self.assertEqual("Auto", body["layout"]["container"]["scrollbar_visibility"], context)
            self.assertTrue(body["props"]["input_hoverable"], context)

            for child in center["children"]:
                if child["node"] == stretch_children[0]:
                    continue
                self.assertEqual(
                    "Fixed",
                    nodes[child["node"]]["layout"]["height"]["stretch"],
                    context,
                )

    def test_module_browsers_keep_tools_fixed_and_lists_scrollable(self):
        workspaces = writable_module_workspaces()
        self.assertEqual(53, len(workspaces))

        for path, document, root_name in workspaces:
            nodes = document["nodes"]
            root = nodes[root_name]
            main_row = nodes[root["children"][0]["node"]]
            left_name = main_row["children"][1]["node"]
            left = nodes[left_name]
            context = path.relative_to(REPO_ROOT).as_posix()

            self.assertEqual("VerticalGroup", left["component"], context)
            self.assertEqual("VerticalBox", left["layout"]["container"]["kind"], context)
            self.assertEqual(
                "$editor.density.gap.medium",
                left["layout"]["container"]["gap"],
                context,
            )
            title_name = left["children"][0]["node"]
            title = nodes[title_name]
            self.assertTrue(title_name.endswith("_title"), context)
            self.assertEqual("WorkbenchSectionTitle", title["component"], context)
            self.assertEqual("Fixed", title["layout"]["height"]["stretch"], context)

            expected_chrome = [title_name]
            if left_name == "tags_left":
                expected_chrome.append("tags_actions")
            elif left_name == "blend_space_left":
                expected_chrome.append("blend_space_search")

            content_name = f"{left_name}_content"
            self.assertEqual(
                expected_chrome + [content_name],
                [child["node"] for child in left["children"]],
                context,
            )
            content = nodes[content_name]
            self.assertEqual("ScrollableBox", content["component"], context)
            self.assertTrue(content["layout"]["clip"], context)
            self.assertEqual("Receive", content["layout"]["input_policy"], context)
            self.assertEqual(
                {
                    "kind": "ScrollableBox",
                    "axis": "Vertical",
                    "gap": "$editor.density.gap.small",
                    "scrollbar_visibility": "Auto",
                },
                content["layout"]["container"],
                context,
            )
            self.assertEqual("Stretch", content["layout"]["width"]["stretch"], context)
            self.assertEqual("Stretch", content["layout"]["height"]["stretch"], context)
            self.assertTrue(content["props"]["input_hoverable"], context)

    def test_module_details_keep_title_in_fixed_chrome_outside_scroll_root(self):
        workspaces = writable_module_workspaces()
        self.assertEqual(53, len(workspaces))

        for path, document, root_name in workspaces:
            nodes = document["nodes"]
            root = nodes[root_name]
            right_name = root["children"][1]["node"]
            right = nodes[right_name]
            context = path.relative_to(REPO_ROOT).as_posix()

            self.assertEqual("VerticalGroup", right["component"], context)
            self.assertNotEqual(
                "ScrollableBox", right["layout"]["container"]["kind"], context
            )
            title_name = right["children"][0]["node"]
            title = nodes[title_name]
            self.assertTrue(title_name.endswith("_details_title"), context)
            self.assertEqual("WorkbenchSectionTitle", title["component"], context)
            self.assertEqual("Fixed", title["layout"]["height"]["stretch"], context)

            self.assertEqual(2, len(right["children"]), context)
            content_name = right["children"][1]["node"]
            content = nodes[content_name]
            if content_name == "blend_space_details":
                self.assertEqual("WorkbenchBlendSpaceDetails", content["component"], context)
                continue
            self.assertTrue(content_name.endswith("_right_content"), context)
            self.assertEqual("ScrollableBox", content["component"], context)
            self.assertTrue(content["layout"]["clip"], context)
            self.assertEqual("Receive", content["layout"]["input_policy"], context)
            self.assertEqual(
                {
                    "kind": "ScrollableBox",
                    "axis": "Vertical",
                    "gap": "$editor.density.gap.small",
                    "scrollbar_visibility": "Auto",
                },
                content["layout"]["container"],
                context,
            )
            self.assertEqual("Stretch", content["layout"]["width"]["stretch"], context)
            self.assertEqual("Stretch", content["layout"]["height"]["stretch"], context)
            self.assertTrue(content["props"]["input_hoverable"], context)

    def test_module_details_use_wide_reserve_and_compact_overlay_drawer(self):
        workspaces = writable_module_workspaces()
        self.assertEqual(53, len(workspaces))

        for path, document, root_name in workspaces:
            nodes = document["nodes"]
            root = nodes[root_name]
            context = path.relative_to(REPO_ROOT).as_posix()
            self.assertEqual("Overlay", root["component"], context)
            self.assertEqual("Overlay", root["layout"]["container"]["kind"], context)
            self.assertEqual(2, len(root["children"]), context)

            main_row_name = root["children"][0]["node"]
            right_name = root["children"][1]["node"]
            right_slot = root["children"][1]["slot"]["layout"]
            self.assertEqual(
                {"horizontal": "End", "vertical": "Fill"},
                right_slot["alignment"],
                context,
            )
            self.assertGreater(right_slot["z_order"], 0, context)

            main_row = nodes[main_row_name]
            self.assertEqual("HorizontalGroup", main_row["component"], context)
            self.assertEqual(
                "HorizontalBox", main_row["layout"]["container"]["kind"], context
            )
            self.assertEqual(4, len(main_row["children"]), context)
            reserve_name = main_row["children"][-1]["node"]

            right = nodes[right_name]
            reserve = nodes[reserve_name]
            self.assertEqual("wide", right["props"]["responsive_min_tier"], context)
            self.assertEqual(
                DRAWER_ROLE,
                right["props"]["responsive_compact_drawer"],
                context,
            )
            self.assertEqual("Container", reserve["component"], context)
            self.assertEqual("wide", reserve["props"]["responsive_min_tier"], context)
            self.assertEqual(right["layout"], reserve["layout"], context)

    def test_toolbar_exposes_one_compact_details_drawer_toggle(self):
        with TOP_TOOLBAR.open("rb") as source:
            document = tomllib.load(source)
        node = document["nodes"]["module_details_drawer_toggle"]

        self.assertEqual("WorkbenchIconButton", node["component"])
        self.assertEqual("WorkbenchModuleDetailsDrawerToggle", node["control_id"])
        self.assertEqual(
            "editor_pages/workbench/dock/dock-right.svg", node["props"]["icon"]
        )
        self.assertEqual("Module Details", node["props"]["label"])
        self.assertEqual("narrow", node["props"]["responsive_min_tier"])
        self.assertEqual("regular", node["props"]["responsive_max_tier"])
        self.assertEqual(
            [
                {
                    "id": "Workbench/ToggleModuleDetailsDrawer",
                    "event": "Click",
                    "route": DRAWER_ACTION,
                }
            ],
            node["events"],
        )

    def test_retained_bridge_owns_drawer_state_and_responsive_override(self):
        responsive = RESPONSIVE_LAYOUT.read_text(encoding="utf-8")
        bridge = BRIDGE_STATE.read_text(encoding="utf-8")
        dispatch = ACTION_DISPATCH.read_text(encoding="utf-8")
        toolbar_layout = TOOLBAR_LAYOUT.read_text(encoding="utf-8")
        bindings = TEMPLATE_BINDINGS.read_text(encoding="utf-8")
        preview_actions = PREVIEW_ACTIONS.read_text(encoding="utf-8")

        self.assertIn('const RESPONSIVE_MAX_TIER_ATTRIBUTE: &str', responsive)
        self.assertIn('const RESPONSIVE_COMPACT_DRAWER_ATTRIBUTE: &str', responsive)
        self.assertIn("compact_module_details_drawer_open", responsive)
        self.assertIn("responsive_node_visible", responsive)
        self.assertIn("compact_module_details_drawer_open: bool", bridge)
        self.assertIn(
            "refresh_compact_module_details_toggle_visibility", bridge
        )
        self.assertIn(
            'MODULE_WORKSPACE_HOST_CONTROL_ID: &str = '
            '"WorkbenchMainBandModuleWorkspace"',
            toolbar_layout,
        )
        self.assertIn(
            "module_workspace_active && compact_tier", toolbar_layout
        )
        self.assertIn(DRAWER_ACTION, dispatch)
        self.assertIn("toggle_compact_module_details_drawer", dispatch)
        self.assertIn('"ToggleModuleDetailsDrawer"', bindings)
        self.assertIn(DRAWER_ACTION, bindings)
        self.assertIn(DRAWER_ACTION, preview_actions)


if __name__ == "__main__":
    unittest.main()
