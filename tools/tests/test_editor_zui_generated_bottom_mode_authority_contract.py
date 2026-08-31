import unittest
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
GENERATED_BOTTOM = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/generated/"
    "workbench_generated_bottom_panel.zui"
)
EDITOR_TOKENS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
)
NAVIGATION = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/generated_bottom_panel_navigation.rs"
)
FEEDBACK = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/generated_bottom_panel_feedback.rs"
)
REFERENCE_MENU_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)
BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_generated_bottom_template_bindings.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)


def resolve_gap(value, tokens):
    if isinstance(value, (int, float)):
        return float(value)
    prefix = "$editor.density.gap."
    if not isinstance(value, str) or not value.startswith(prefix):
        raise AssertionError(f"unsupported gap token: {value}")
    return float(tokens["density"][f"gap_{value.removeprefix(prefix)}"])


class EditorZuiGeneratedBottomModeAuthorityContractTests(unittest.TestCase):
    def test_mode_tabs_are_the_only_authored_mode_trigger_authority(self):
        with GENERATED_BOTTOM.open("rb") as source:
            document = tomllib.load(source)

        self.assertFalse(
            any("#WorkbenchDropdown" in widget for widget in document["imports"]["widgets"])
        )
        nodes = document["nodes"]
        self.assertNotIn("generated_bottom_mode_dropdown", nodes)
        self.assertIn(
            "generated_bottom_filter_field",
            [child["node"] for child in nodes["generated_bottom_header"]["children"]],
        )
        self.assertIn(
            "generated_bottom_selected_route",
            [
                child["node"]
                for child in nodes["generated_bottom_detail_content"]["children"]
            ],
        )

        mode_nodes = [
            node
            for name, node in nodes.items()
            if name.startswith("generated_bottom_mode_")
        ]
        self.assertEqual(5, len(mode_nodes))
        self.assertTrue(all(node["component"] == "WorkbenchTab" for node in mode_nodes))
        self.assertTrue(
            all(node["events"][0]["event"] == "Click" for node in mode_nodes)
        )

    def test_header_filter_yields_width_and_route_detail_stays_reachable(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        self.assertEqual(
            {
                "min": 120.0,
                "preferred": 220.0,
                "max": 300.0,
                "weight": 1.0,
                "stretch": "Stretch",
            },
            nodes["generated_bottom_filter_field"]["layout"]["width"],
        )
        self.assertEqual(
            "Stretch",
            nodes["generated_bottom_selected_route"]["layout"]["width"]["stretch"],
        )

    def test_removed_dropdown_actions_have_no_runtime_mirror_path(self):
        for path in (NAVIGATION, FEEDBACK, BINDINGS, PREVIEW_ACTIONS):
            source = path.read_text(encoding="utf-8")
            for forbidden in (
                "WorkbenchGeneratedBottomModeDropdown",
                "workbench.generated_bottom.mode.edit",
                "workbench.generated_bottom.mode.commit",
                '"ModeEdit"',
                '"ModeCommit"',
            ):
                self.assertNotIn(forbidden, source, f"{path.name}: {forbidden}")

    def test_default_mode_is_initialized_by_the_live_state_authority(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        mode_nodes = [
            node
            for name, node in nodes.items()
            if name.startswith("generated_bottom_mode_")
        ]
        for node in mode_nodes:
            self.assertTrue(
                {"checked", "selected", "value"}.isdisjoint(node.get("props", {})),
                node["control_id"],
            )
        self.assertEqual(
            "Output",
            nodes["generated_bottom_selected_mode"]["props"]["value_text"],
        )

        initialization = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        feedback = FEEDBACK.read_text(encoding="utf-8")
        self.assertIn(
            'self.select_generated_bottom_mode("WorkbenchGeneratedBottomModeOutput")?',
            initialization,
        )
        self.assertIn(
            "pub(super) fn select_generated_bottom_mode(",
            feedback,
        )

    def test_route_table_is_a_reachable_vertical_scroll_region(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]
        route_table = nodes["generated_bottom_route_table"]

        self.assertEqual("ScrollableBox", route_table["component"])
        self.assertEqual({"input_hoverable": True}, route_table["props"])
        self.assertTrue(route_table["layout"]["clip"])
        self.assertEqual("Receive", route_table["layout"]["input_policy"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": 0.0,
                "scrollbar_visibility": "Auto",
            },
            route_table["layout"]["container"],
        )
        self.assertEqual(37, len(route_table["children"]))

        dense_row_height = {
            "min": "$editor.control.height.dense",
            "preferred": "$editor.control.height.dense",
            "max": "$editor.control.height.compact",
            "stretch": "Fixed",
        }
        for child in route_table["children"]:
            node = nodes[child["node"]]
            self.assertEqual(
                dense_row_height,
                node["layout"]["height"],
                node["control_id"],
            )

    def test_bottom_panel_consolidates_tools_into_one_fixed_header(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        root = nodes["generated_bottom_panel"]
        self.assertEqual(
            ["generated_bottom_header", "generated_bottom_body"],
            [child["node"] for child in root["children"]],
        )
        self.assertEqual(
            [
                "generated_bottom_title",
                "generated_bottom_modes",
                "generated_bottom_filter_field",
                "generated_bottom_open_button",
                "generated_bottom_pin_button",
            ],
            [child["node"] for child in nodes["generated_bottom_header"]["children"]],
        )
        self.assertNotIn("generated_bottom_header_fill", nodes)
        self.assertNotIn("generated_bottom_filter_bar", nodes)
        self.assertEqual(
            "Fixed",
            nodes["generated_bottom_header"]["layout"]["height"]["stretch"],
        )

    def test_header_and_mode_strip_share_the_toolbar_row_height_authority(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        expected_height = {
            "min": "$editor.chrome.workbench_toolbar.command_row.height",
            "preferred": "$editor.chrome.workbench_toolbar.command_row.height",
            "max": 36.0,
            "stretch": "Fixed",
        }
        self.assertEqual(
            expected_height,
            nodes["generated_bottom_header"]["layout"]["height"],
        )
        self.assertEqual(
            expected_height,
            nodes["generated_bottom_modes"]["layout"]["height"],
        )

    def test_compact_header_actions_and_detail_fields_share_control_height(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        compact_height = {
            "min": "$editor.control.height.compact",
            "preferred": "$editor.control.height.compact",
            "max": "$editor.control.height.default",
            "stretch": "Fixed",
        }
        for name in (
            "generated_bottom_open_button",
            "generated_bottom_pin_button",
            "generated_bottom_selected_module",
            "generated_bottom_selected_panel",
            "generated_bottom_selected_mode",
        ):
            self.assertEqual(compact_height, nodes[name]["layout"]["height"], name)

        self.assertEqual(
            {
                "min": "$editor.control.height.dense",
                "preferred": "$editor.control.height.dense",
                "max": "$editor.control.height.compact",
                "stretch": "Fixed",
            },
            nodes["generated_bottom_detail_title"]["layout"]["height"],
        )

    def test_route_detail_keeps_title_fixed_and_fields_scrollable(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]

        detail_panel = nodes["generated_bottom_detail_panel"]
        self.assertEqual("VerticalGroup", detail_panel["component"])
        self.assertEqual(
            ["generated_bottom_detail_title", "generated_bottom_detail_content"],
            [child["node"] for child in detail_panel["children"]],
        )
        self.assertEqual(
            "Fixed",
            nodes["generated_bottom_detail_title"]["layout"]["height"]["stretch"],
        )

        detail_content = nodes["generated_bottom_detail_content"]
        self.assertEqual("ScrollableBox", detail_content["component"])
        self.assertEqual({"input_hoverable": True}, detail_content["props"])
        self.assertTrue(detail_content["layout"]["clip"])
        self.assertEqual("Receive", detail_content["layout"]["input_policy"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Vertical",
                "gap": "$editor.density.gap.medium",
                "scrollbar_visibility": "Auto",
            },
            detail_content["layout"]["container"],
        )
        self.assertEqual(
            [
                "generated_bottom_selected_module",
                "generated_bottom_selected_panel",
                "generated_bottom_selected_mode",
                "generated_bottom_selected_route",
                "generated_bottom_selected_panel_row",
            ],
            [child["node"] for child in detail_content["children"]],
        )

    def test_mode_strip_and_detail_panel_yield_ultra_width(self):
        with GENERATED_BOTTOM.open("rb") as source:
            nodes = tomllib.load(source)["nodes"]
        with EDITOR_TOKENS.open("rb") as source:
            tokens = tomllib.load(source)

        mode_strip = nodes["generated_bottom_modes"]
        self.assertEqual("ScrollableBox", mode_strip["component"])
        self.assertEqual({"input_hoverable": True}, mode_strip["props"])
        self.assertEqual("Receive", mode_strip["layout"]["input_policy"])
        self.assertEqual(
            {
                "kind": "ScrollableBox",
                "axis": "Horizontal",
                "gap": "$editor.density.gap.regular",
                "scrollbar_visibility": "Never",
            },
            mode_strip["layout"]["container"],
        )
        self.assertEqual(0.0, mode_strip["layout"]["width"]["min"])
        self.assertEqual(
            "narrow",
            nodes["generated_bottom_detail_panel"]["props"][
                "responsive_min_tier"
            ],
        )

        title = nodes["generated_bottom_title"]
        self.assertEqual("narrow", title["props"]["responsive_min_tier"])
        header = nodes["generated_bottom_header"]
        child_names = [child["node"] for child in header["children"]]
        gap = resolve_gap(header["layout"]["container"]["gap"], tokens)
        authored_min = sum(
            nodes[name]["layout"]["width"].get("min", 0.0)
            for name in child_names
        ) + gap * (len(child_names) - 1)
        ultra_names = [name for name in child_names if name != "generated_bottom_title"]
        ultra_min = sum(
            nodes[name]["layout"]["width"].get("min", 0.0)
            for name in ultra_names
        ) + gap * (len(ultra_names) - 1)
        ultra_available = (
            640.0 / 1.5 - tokens["chrome"]["activity_rail_width"]
        )
        self.assertGreater(authored_min, ultra_available)
        self.assertLessEqual(ultra_min, ultra_available)


if __name__ == "__main__":
    unittest.main()
