import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
MAIN_BAND = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_main_band.zui"
)
INSPECTOR_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_inspector_panel.zui"
)
WORKBENCH_WINDOW = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/windows/workbench_window.zui"
)
WORKBENCH_SKELETON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_skeleton.zui"
)
COMPONENT_DRAWER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_component_drawer.zui"
)
ACTIVITY_RAIL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_activity_rail.zui"
)
RAIL_BUTTON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/"
    "workbench_rail_button.zui"
)
LEGACY_WORKBENCH_SHELL = REPO_ROOT / "zircon_editor/assets/ui/editor/host/workbench_shell.zui"
SCENE_TREE_PANEL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_scene_tree_panel.zui"
)
STATUS_BAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_status_bar.zui"
)
SCENE_VIEWPORT_TOOLBAR = REPO_ROOT / "zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui"
COMMAND_PALETTE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/floating/"
    "workbench_command_palette.zui"
)
PREFERENCES = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/floating/"
    "workbench_preferences.zui"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiWorkbenchLayoutContractTests(unittest.TestCase):
    def test_preferences_shrink_inside_the_ultra_surface_with_safe_margins(self):
        tokens = load_document(TOKENS)["density"]
        window = load_document(WORKBENCH_WINDOW)
        root_children = window["nodes"]["root"]["children"]
        settings_slot = next(
            child["slot"]["layout"]
            for child in root_children
            if child["node"] == "settings_window"
        )

        safe_margin = "$editor.density.gap.large"
        self.assertEqual(
            {"left": safe_margin, "top": safe_margin, "right": safe_margin, "bottom": safe_margin},
            settings_slot["padding"],
        )
        self.assertEqual(
            {"horizontal": "Center", "vertical": "Center"},
            settings_slot["alignment"],
        )

        preferences = load_document(PREFERENCES)["nodes"]["preferences"]
        self.assertEqual(0.0, preferences["props"]["settings_category_scroll_offset"])
        self.assertEqual(0.0, preferences["props"]["settings_scroll_offset"])
        self.assertEqual(
            0.0,
            window["nodes"]["settings_window"]["props"]["settings_category_scroll_offset"],
        )
        self.assertEqual(
            0.0,
            window["nodes"]["settings_window"]["props"]["settings_scroll_offset"],
        )
        layout = preferences["layout"]
        for axis, ultra_extent in (
            ("width", tokens["ultra_minimum_window_width"]),
            ("height", tokens["ultra_minimum_window_height"]),
        ):
            constraint = layout[axis]
            self.assertEqual("Stretch", constraint["stretch"])
            self.assertLessEqual(
                constraint["min"] + 2.0 * tokens["gap_large"],
                ultra_extent,
                f"preferences {axis} must fit inside the Ultra surface",
            )

    def test_toast_shrinks_inside_the_ultra_surface_with_safe_side_margins(self):
        tokens = load_document(TOKENS)["density"]
        window = load_document(WORKBENCH_WINDOW)
        nodes = window["nodes"]
        toast_slot = next(
            child["slot"]["layout"]
            for child in nodes["root"]["children"]
            if child["node"] == "toast_overlay"
        )

        safe_margin = "$editor.density.gap.large"
        self.assertEqual(
            {"left": safe_margin, "right": safe_margin},
            toast_slot["padding"],
        )
        self.assertEqual(
            {"horizontal": "End", "vertical": "End"},
            toast_slot["alignment"],
        )

        layout = nodes["toast_overlay"]["layout"]
        self.assertEqual(0.0, layout["position"]["x"])
        self.assertEqual("Stretch", layout["width"]["stretch"])
        self.assertLessEqual(
            layout["width"]["min"] + 2.0 * tokens["gap_large"],
            tokens["ultra_minimum_window_width"],
        )

    def test_command_palette_uses_shared_overlay_size_constraints(self):
        palette = load_document(COMMAND_PALETTE)["nodes"]["palette"]["layout"]

        self.assertEqual(
            {
                "min": "$editor.density.command_palette.min_width",
                "preferred": "$editor.density.command_palette.preferred_width",
                "max": "$editor.density.command_palette.max_width",
                "stretch": "Fixed",
            },
            palette["width"],
        )
        self.assertEqual(
            {
                "min": "$editor.density.command_palette.min_height",
                "preferred": "$editor.density.command_palette.preferred_height",
                "max": "$editor.density.command_palette.max_height",
                "stretch": "Fixed",
            },
            palette["height"],
        )

    def test_two_row_toolbar_uses_dense_slate_rows_without_sacrificing_text_fit(self):
        tokens = load_document(TOKENS)
        typography = tokens["typography"]
        controls = tokens["controls"]
        chrome = tokens["chrome"]
        density = tokens["density"]

        self.assertEqual(66.0, chrome["workbench_toolbar_height"])
        self.assertEqual(34.0, chrome["workbench_toolbar_command_row_height"])
        self.assertEqual(28.0, controls["dense_height"])
        self.assertGreaterEqual(
            controls["dense_height"],
            typography["body_size"] * typography["line_height"]
            + 2.0 * density["gap_small"],
        )

    def test_toolbar_rows_exactly_fill_the_declared_chrome_height(self):
        tokens = load_document(TOKENS)

        self.assertEqual(
            tokens["chrome"]["workbench_toolbar_height"],
            tokens["chrome"]["workbench_toolbar_command_row_height"]
            + tokens["density"]["gap_small"]
            + tokens["controls"]["dense_height"],
        )

    def test_module_tabs_do_not_exceed_their_row_height(self):
        toolbar = load_document(TOOLBAR)
        nodes = toolbar["nodes"]
        module_row = nodes["toolbar_module_tabs"]
        row_height = module_row["layout"]["height"]

        self.assertEqual(
            {
                "min": "$editor.control.height.dense",
                "preferred": "$editor.control.height.dense",
                "max": "$editor.control.height.dense",
                "stretch": "Fixed",
            },
            row_height,
        )

        self.assertEqual("module_tab_scroll", module_row["children"][0]["node"])
        self.assertEqual("module_more", module_row["children"][1]["node"])
        scroll = nodes["module_tab_scroll"]
        self.assertEqual("ScrollableBox", scroll["component"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Horizontal",
                "gap": "$editor.density.gap.xsmall",
                "scrollbar_visibility": "Never",
            },
            scroll["layout"]["container"],
        )
        self.assertEqual("Receive", scroll["layout"]["input_policy"])
        self.assertTrue(scroll["props"]["input_hoverable"])
        self.assertNotIn("input_interactive", scroll["props"])
        self.assertEqual(0.0, scroll["layout"]["width"]["min"])
        module_tabs = [child["node"] for child in scroll["children"]]
        self.assertEqual(11, len(module_tabs))
        for node_id in module_tabs:
            self.assertEqual(
                row_height,
                nodes[node_id]["layout"]["height"],
                f"{node_id} must fit entirely inside the module row",
            )

    def test_module_overflow_action_remains_fixed_when_the_tab_strip_is_clipped(self):
        nodes = load_document(TOOLBAR)["nodes"]
        module_more = nodes["module_more"]
        self.assertEqual(
            {
                "min": 34.0,
                "preferred": 34.0,
                "max": 34.0,
                "stretch": "Fixed",
            },
            module_more["layout"]["width"],
        )

    def test_viewport_toolbar_keeps_the_right_camera_group_visible_when_tools_overflow(self):
        nodes = load_document(SCENE_VIEWPORT_TOOLBAR)["nodes"]
        left_group = nodes["left_group"]
        self.assertEqual("ScrollableBox", left_group["component"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Horizontal",
                "gap": "$editor.density.gap.small",
                "scrollbar_visibility": "Never",
            },
            left_group["layout"]["container"],
        )
        self.assertEqual("Receive", left_group["layout"]["input_policy"])
        self.assertTrue(left_group["props"]["input_hoverable"])
        self.assertEqual(14, len(left_group["children"]))
        self.assertEqual(
            ["set_projection_mode", "align_view"],
            [child["node"] for child in nodes["right_group"]["children"]],
        )

    def test_workbench_entry_chrome_consumes_shared_height_tokens(self):
        fixed_heights = {
            "top_toolbar": "$editor.chrome.workbench_toolbar.height",
            "document_tabs": "$editor.control.height.default",
            "status_bar": "$editor.chrome.status_bar.height",
        }
        skeleton_nodes = load_document(WORKBENCH_SKELETON)["nodes"]
        for node_id, token in fixed_heights.items():
            self.assertEqual(
                {
                    "min": token,
                    "preferred": token,
                    "max": token,
                    "stretch": "Fixed",
                },
                skeleton_nodes[node_id]["layout"]["height"],
                f"{node_id} must follow the shared chrome token generation",
            )

        window_top_toolbar = load_document(WORKBENCH_WINDOW)["nodes"]["top_toolbar"]
        self.assertEqual(
            {
                "min": "$editor.chrome.workbench_toolbar.height",
                "preferred": "$editor.chrome.workbench_toolbar.height",
                "max": "$editor.chrome.workbench_toolbar.height",
                "stretch": "Fixed",
            },
            window_top_toolbar["layout"]["height"],
        )

        status_nodes = load_document(STATUS_BAR)["nodes"]
        self.assertEqual(
            {
                "id": "Workbench/ToggleSnapFromStatus",
                "event": "Click",
                "route": "workbench.status.toggle_snap",
            },
            status_nodes["status_snap_icon"]["events"][0],
        )
        self.assertEqual(
            {
                "id": "Workbench/FrameSelectionFromStatus",
                "event": "Click",
                "route": "workbench.status.frame_selection",
            },
            status_nodes["status_target_icon"]["events"][0],
        )
        self.assertEqual(
            {
                "icon": "zircon_editor_shell/viewport/globe.svg",
                "label": "World",
                "responsive_min_tier": "wide",
                "input_interactive": False,
                "input_clickable": False,
                "input_hoverable": False,
                "input_focusable": False,
                "layout_min_width": "$editor.chrome.status_bar.height",
                "layout_min_height": "$editor.chrome.status_bar.height",
            },
            status_nodes["status_world_icon"]["props"],
        )

    def test_viewport_and_main_band_outrank_auxiliary_drawers(self):
        main_nodes = load_document(MAIN_BAND)["nodes"]
        priorities = {
            node_id: main_nodes[node_id]["layout"]["width"]["priority"]
            for node_id in (
                "viewport_panel",
                "activity_rail",
                "left_drawer_shell",
                "right_drawer_shell",
            )
        }

        self.assertGreater(priorities["viewport_panel"], priorities["activity_rail"])
        self.assertGreater(priorities["activity_rail"], priorities["left_drawer_shell"])
        self.assertGreater(priorities["left_drawer_shell"], priorities["right_drawer_shell"])
        self.assertEqual(
            "Stretch",
            main_nodes["viewport_panel"]["layout"]["width"]["stretch"],
        )

        window_nodes = load_document(WORKBENCH_WINDOW)["nodes"]
        self.assertGreater(
            window_nodes["main_band"]["layout"]["height"]["priority"],
            window_nodes["component_drawer_shell"]["layout"]["height"]["priority"],
        )

    def test_inspector_identity_header_replaces_nonfunctional_mode_tabs(self):
        panel_nodes = load_document(INSPECTOR_PANEL)["nodes"]
        header = panel_nodes["inspector_header"]

        self.assertEqual("RightDrawerHeaderRoot", header["control_id"])
        self.assertEqual([{"node": "inspector_title"}], header["children"])
        self.assertFalse(
            any(node.get("component") == "WorkbenchTab" for node in panel_nodes.values())
        )

    def test_scene_tree_rows_share_the_virtual_row_height_token(self):
        nodes = load_document(
            REPO_ROOT
            / "zircon_editor/assets/ui/editor/components/workbench/shell/"
            / "workbench_scene_tree_panel.zui"
        )["nodes"]
        row_nodes = [
            node_id
            for node_id, node in nodes.items()
            if node.get("component") == "WorkbenchTreeRow"
        ]

        self.assertEqual(10, len(row_nodes))
        expected_height = {
            "min": "$editor.density.row_height",
            "preferred": "$editor.density.row_height",
            "max": "$editor.density.row_height",
            "stretch": "Fixed",
        }
        for node_id in row_nodes:
            self.assertEqual(
                expected_height,
                nodes[node_id]["layout"]["height"],
                f"{node_id} must keep the virtual tree row cadence",
            )

    def test_inspector_content_is_scrollable_at_the_minimum_regular_window_height(self):
        nodes = load_document(INSPECTOR_PANEL)["nodes"]
        content = nodes["inspector_content"]

        self.assertEqual("ScrollableBox", content["component"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": "$editor.density.gap.large",
                "scrollbar_visibility": "Auto",
            },
            content["layout"]["container"],
        )
        self.assertEqual("Receive", content["layout"]["input_policy"])
        self.assertTrue(content["props"]["input_hoverable"])

    def test_inspector_transform_axes_wrap_before_the_regular_drawer_can_clip_them(self):
        nodes = load_document(INSPECTOR_PANEL)["nodes"]
        self.assertEqual("Fixed", nodes["inspector_transform"]["layout"]["height"]["stretch"])

        for axis_name in ("position", "rotation", "scale"):
            row = nodes[f"{axis_name}_row"]
            self.assertEqual("VerticalGroup", row["component"])
            values = nodes[f"{axis_name}_values"]
            self.assertEqual("WrapBox", values["component"])
            self.assertEqual("WrapBox", values["layout"]["container"]["kind"])
            self.assertEqual(76.0, values["layout"]["container"]["item_min_width"])
            for suffix in ("x", "y", "z"):
                group = nodes[f"{axis_name}_value_{suffix}_group"]
                self.assertEqual(
                    {"min": 70.0, "preferred": 76.0, "max": 80.0, "stretch": "Fixed"},
                    group["layout"]["width"],
                )

    def test_inspector_transform_wrap_capacity_matches_drawer_tiers(self):
        nodes = load_document(INSPECTOR_PANEL)["nodes"]
        density = load_document(TOKENS)["density"]
        container = nodes["position_values"]["layout"]["container"]
        item_min_width = container["item_min_width"]
        self.assertEqual("$editor.density.gap.small", container["horizontal_gap"])
        horizontal_gap = density["gap_small"]

        def columns_for(width):
            return int((width + horizontal_gap) // (item_min_width + horizontal_gap))

        self.assertEqual(2, columns_for(196.0))
        self.assertEqual(3, columns_for(248.0))

    def test_viewport_toolbar_reveals_secondary_display_controls_by_width_tier(self):
        nodes = load_document(
            REPO_ROOT
            / "zircon_editor/assets/ui/editor/components/workbench/shell/"
            / "workbench_viewport_panel.zui"
        )["nodes"]
        self.assertNotIn("responsive_min_tier", nodes["viewport_lit"]["props"])
        self.assertEqual("narrow", nodes["viewport_mode"]["props"]["responsive_min_tier"])
        self.assertEqual("regular", nodes["viewport_angle"]["props"]["responsive_min_tier"])
        self.assertEqual("wide", nodes["viewport_speed"]["props"]["responsive_min_tier"])

        gap = load_document(TOKENS)["density"]["gap_medium"]
        widths = {
            "ultra": nodes["viewport_lit"]["layout"]["width"]["preferred"],
            "narrow": 64.0 + 108.0 + gap,
            "regular": 64.0 + 108.0 + 70.0 + 2.0 * gap,
            "wide": 64.0 + 108.0 + 70.0 + 70.0 + 3.0 * gap,
        }
        self.assertEqual(64.0, widths["ultra"])
        self.assertEqual(180.0, widths["narrow"])
        self.assertEqual(258.0, widths["regular"])
        self.assertEqual(336.0, widths["wide"])

    def test_workbench_popup_triggers_use_control_anchors_and_explicit_placements(self):
        nodes = load_document(WORKBENCH_WINDOW)["nodes"]
        expected = {
            "toolbar_main_menu": ("WorkbenchToolbarMenu", "bottom-start"),
            "toolbar_run_mode_menu": ("WorkbenchRunMode", "bottom-end"),
            "toolbar_layout_menu": ("WorkbenchLayoutGrid", "bottom-end"),
            "toolbar_module_overflow_menu": ("WorkbenchModuleMore", "bottom-start"),
        }
        for node_id, (anchor, placement) in expected.items():
            node = nodes[node_id]
            self.assertEqual(
                {"kind": "control", "control_id": anchor},
                node["widget"]["popup_anchor"],
                f"{node_id} must follow the triggering control frame",
            )
            self.assertEqual(placement, node["props"]["placement"])
            self.assertFalse(node["props"]["popup_open"])
            self.assertEqual("collapsed", node["props"]["visibility"])
            self.assertGreaterEqual(node["layout"]["z_index"], 200)

    def test_closed_overlay_shells_are_inert_until_state_projection_opens_them(self):
        nodes = load_document(WORKBENCH_WINDOW)["nodes"]

        for node_id in (
            "toolbar_main_menu",
            "toolbar_run_mode_menu",
            "toolbar_layout_menu",
            "toolbar_module_overflow_menu",
            "command_palette",
            "toast_overlay",
            "context_menu",
            "icon_button_tooltip",
        ):
            props = nodes[node_id]["props"]
            self.assertFalse(
                props.get("popup_open", False),
                f"{node_id} must not start with an open popup state",
            )
            self.assertEqual(
                "collapsed",
                props["visibility"],
                f"{node_id} must remain out of the hit tree until projected open",
            )

        notification = nodes["notification_center"]["props"]
        self.assertFalse(notification["popup_open"])
        self.assertEqual("visible", notification["visibility"])
        self.assertTrue(notification["keep_mounted"])
        for property_name in (
            "input_interactive",
            "input_clickable",
            "input_hoverable",
            "input_focusable",
        ):
            self.assertFalse(
                notification[property_name],
                f"closed notification center must not capture {property_name}",
            )

    def test_component_drawer_uses_scrollable_two_column_gallery_without_row_clipping(self):
        nodes = load_document(COMPONENT_DRAWER)["nodes"]
        body = nodes["component_body"]
        top_row = nodes["component_top_row"]
        lower_row = nodes["component_lower_row"]
        feedback = nodes["component_feedback"]

        self.assertEqual("ScrollableBox", body["component"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": "$editor.density.gap.large",
                "scrollbar_visibility": "Auto",
            },
            body["layout"]["container"],
        )
        self.assertEqual("Receive", body["layout"]["input_policy"])
        self.assertTrue(body["props"]["input_hoverable"])

        self.assertEqual("MasonryBox", top_row["component"])
        self.assertEqual(
            {
                "kind": "MasonryBox",
                "columns": 2,
                "gap": "$editor.density.gap.medium",
            },
            top_row["layout"]["container"],
        )
        self.assertEqual("Fixed", top_row["layout"]["height"]["stretch"])
        self.assertFalse(top_row["layout"].get("clip", False))

        self.assertEqual("VerticalGroup", lower_row["component"])
        self.assertEqual("Fixed", lower_row["layout"]["height"]["stretch"])
        self.assertFalse(lower_row["layout"].get("clip", False))
        self.assertEqual("VerticalGroup", feedback["component"])
        self.assertEqual("Fixed", feedback["layout"]["height"]["stretch"])
        self.assertFalse(feedback["layout"].get("clip", False))

        top_cards = [
            "component_buttons",
            "component_icon_buttons",
            "component_inputs",
            "component_selection",
            "component_sliders",
            "component_labs",
            "component_list",
        ]
        max_card_width = max(
            nodes[node_id]["layout"]["width"]["max"] for node_id in top_cards
        )
        max_card_min_width = max(
            nodes[node_id]["layout"]["width"]["min"] for node_id in top_cards
        )
        gap = load_document(TOKENS)["density"]["gap_medium"]
        self.assertLessEqual(
            2.0 * max_card_width + gap,
            load_document(TOKENS)["density"]["minimum_window_width"],
            "two gallery columns must fit the regular minimum window without clipping",
        )
        self.assertLessEqual(
            nodes["component_table"]["layout"]["width"]["max"],
            load_document(TOKENS)["density"]["minimum_window_width"],
        )
        tokens = load_document(TOKENS)
        component_table_width = nodes["component_table"]["layout"]["width"]
        ultra_available = (
            tokens["density"]["ultra_minimum_window_width"]
            - tokens["chrome"]["activity_rail_width"]
        )
        self.assertLessEqual(2.0 * max_card_min_width + gap, ultra_available)
        for node_id in top_cards:
            self.assertEqual(
                "Stretch",
                nodes[node_id]["layout"]["width"]["stretch"],
                node_id,
            )
        self.assertLessEqual(component_table_width["min"], ultra_available)
        self.assertEqual("Stretch", component_table_width["stretch"])

        notification_width = {
            "min": "$editor.density.notification_panel.min_width",
            "preferred": "$editor.density.notification_panel.preferred_width",
            "max": "$editor.density.notification_panel.max_width",
            "stretch": "Stretch",
        }
        for node_id in ("feedback_alerts", "feedback_toast_column"):
            self.assertEqual(
                notification_width,
                nodes[node_id]["layout"]["width"],
                node_id,
            )

    def test_scene_tree_is_a_scrollable_virtualized_list_below_a_fixed_search_row(self):
        document = load_document(SCENE_TREE_PANEL)
        nodes = document["nodes"]
        content = nodes["scene_content"]
        search = nodes["scene_search_row"]
        tree = nodes["scene_tree"]

        self.assertEqual("VerticalGroup", content["component"])
        self.assertEqual(
            ["scene_search_row", "scene_tree"],
            [child["node"] for child in content["children"]],
        )
        self.assertEqual("Fixed", search["layout"]["height"]["stretch"])
        self.assertEqual("ScrollableBox", tree["component"])
        self.assertEqual("ScrollableBox", tree["layout"]["container"]["kind"])
        self.assertEqual("Vertical", tree["layout"]["container"]["axis"])
        self.assertEqual("Auto", tree["layout"]["container"]["scrollbar_visibility"])
        self.assertEqual("Receive", tree["layout"]["input_policy"])
        self.assertTrue(tree["props"]["input_hoverable"])
        self.assertEqual("virtual_rows", tree["repeat"]["kind"])
        self.assertEqual("Stretch", tree["layout"]["height"]["stretch"])

    def test_activity_rail_buttons_fit_the_rail_and_share_the_compact_hit_target(self):
        tokens = load_document(TOKENS)
        compact = "$editor.control.height.compact"
        rail = load_document(ACTIVITY_RAIL)["nodes"]
        button = load_document(RAIL_BUTTON)["nodes"]["root"]
        legacy = load_document(LEGACY_WORKBENCH_SHELL)["nodes"]

        self.assertEqual(compact, button["layout"]["width"]["preferred"])
        self.assertEqual(compact, button["layout"]["height"]["preferred"])
        self.assertEqual("small", button["props"]["button_size"])
        self.assertEqual("$editor.density.gap.small", button["props"]["layout_padding_left"])
        self.assertEqual(tokens["chrome"]["activity_rail_width"], 34.0)

        rail_buttons = [node for node in rail.values() if node.get("component") == "WorkbenchRailButton"]
        self.assertEqual(6, len(rail_buttons))
        for node in rail_buttons:
            self.assertEqual(compact, node["layout"]["width"]["preferred"])
            self.assertEqual(compact, node["layout"]["height"]["preferred"])

        for node_id in ("assets_toggle", "hierarchy_toggle", "console_toggle"):
            self.assertEqual(compact, legacy[node_id]["layout"]["width"]["preferred"])
            self.assertEqual(compact, legacy[node_id]["layout"]["height"]["preferred"])


if __name__ == "__main__":
    unittest.main()
