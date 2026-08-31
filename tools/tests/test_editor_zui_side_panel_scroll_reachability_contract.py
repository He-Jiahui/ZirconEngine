import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSETS_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/assets/"
    "workbench_assets_workspace.zui"
)
BLEND_SPACE_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/"
    "animation/workbench_extension_blend_space_workspace.zui"
)
BLEND_SPACE_DETAILS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/animation/"
    "workbench_blend_space_details.zui"
)
VALIDATION_LOG = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/feedback/"
    "workbench_validation_log.zui"
)
SAMPLE_WEIGHTS = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/animation/"
    "workbench_sample_weights.zui"
)
ABILITY_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_ability_workspace.zui"
)
EFFECT_WORKSPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/"
    "workbench_effect_workspace.zui"
)
REFERENCE_MENU_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/reference_menu_actions.rs"
)


def load_nodes(path):
    with path.open("rb") as source:
        return tomllib.load(source)["nodes"]


def assert_vertical_scroll_region(test_case, node, gap):
    test_case.assertEqual("ScrollableBox", node["component"])
    test_case.assertTrue(node["layout"]["clip"])
    test_case.assertEqual("Receive", node["layout"]["input_policy"])
    test_case.assertEqual(
        {
            "kind": "ScrollableBox",
            "axis": "Vertical",
            "gap": gap,
            "scrollbar_visibility": "Auto",
        },
        node["layout"]["container"],
    )
    test_case.assertTrue(node["props"]["input_hoverable"])
    test_case.assertNotIn("input_interactive", node["props"])
    test_case.assertNotIn("input_clickable", node["props"])
    test_case.assertNotIn("input_focusable", node["props"])


class EditorZuiSidePanelScrollReachabilityContractTests(unittest.TestCase):
    def test_assets_wide_detail_actions_remain_reachable(self):
        nodes = load_nodes(ASSETS_WORKSPACE)
        root = nodes["assets_right"]
        self.assertEqual("VerticalGroup", root["component"])
        self.assertEqual("wide", root["props"]["responsive_min_tier"])
        self.assertEqual(
            ["assets_details_title", "assets_right_content"],
            [child["node"] for child in root["children"]],
        )

        content = nodes["assets_right_content"]
        assert_vertical_scroll_region(self, content, "$editor.density.gap.small")
        self.assertEqual(7, len(content["children"]))
        self.assertEqual(
            "assets_production_tools", content["children"][-1]["node"]
        )

    def test_blend_space_browser_and_details_remain_reachable(self):
        workspace = load_nodes(BLEND_SPACE_WORKSPACE)
        left = workspace["blend_space_left"]
        self.assertEqual("VerticalGroup", left["component"])
        self.assertEqual("narrow", left["props"]["responsive_min_tier"])
        self.assertEqual(
            ["blend_space_title", "blend_space_search", "blend_space_left_content"],
            [child["node"] for child in left["children"]],
        )

        left_content = workspace["blend_space_left_content"]
        assert_vertical_scroll_region(
            self, left_content, "$editor.density.gap.small"
        )
        left_children = [child["node"] for child in left_content["children"]]
        self.assertEqual(11, len(left_children))
        self.assertNotIn("blend_space_tabs", left_children)
        self.assertIn("blend_space_idle_run_row", left_children)
        self.assertEqual("blend_space_left_fill", left_children[-1])

        details = load_nodes(BLEND_SPACE_DETAILS)["root"]
        assert_vertical_scroll_region(self, details, "$editor.density.gap.xsmall")
        self.assertEqual(16, len(details["children"]))

    def test_blend_space_preview_keeps_its_header_fixed_and_secondary_content_reachable(self):
        nodes = load_nodes(BLEND_SPACE_WORKSPACE)
        preview = nodes["blend_space_preview_card"]
        self.assertEqual("VerticalGroup", preview["component"])
        self.assertEqual(
            ["blend_space_preview_header", "blend_space_preview_content"],
            [child["node"] for child in preview["children"]],
        )

        content = nodes["blend_space_preview_content"]
        assert_vertical_scroll_region(self, content, "$editor.density.gap.small")
        self.assertEqual(
            [
                "blend_space_preview_viewport",
                "blend_space_preview_divider",
                "blend_space_heatmap_header",
                "blend_space_weight_heatmap",
                "blend_space_preview_asset",
            ],
            [child["node"] for child in content["children"]],
        )
        self.assertNotIn("blend_space_preview_fill", nodes)

    def test_validation_chrome_stays_fixed_while_diagnostics_scroll(self):
        nodes = load_nodes(VALIDATION_LOG)
        root = nodes["validation_log_root"]
        self.assertEqual("VerticalGroup", root["component"])
        self.assertEqual(
            [
                "validation_log_header",
                "validation_log_filters",
                "validation_log_diagnostics",
                "validation_log_footer",
            ],
            [child["node"] for child in root["children"]],
        )

        diagnostics = nodes["validation_log_diagnostics"]
        assert_vertical_scroll_region(
            self, diagnostics, "$editor.density.gap.xsmall"
        )
        self.assertEqual(
            [
                "validation_log_info_axes",
                "validation_log_warning",
                "validation_log_info_range",
                "validation_log_info_duplicates",
            ],
            [child["node"] for child in diagnostics["children"]],
        )
        self.assertNotIn("validation_log_fill", nodes)

    def test_sample_weights_header_stays_fixed_while_weight_rows_scroll(self):
        nodes = load_nodes(SAMPLE_WEIGHTS)
        root = nodes["sample_weights_root"]
        self.assertEqual("VerticalGroup", root["component"])
        self.assertEqual(
            ["sample_weights_header", "sample_weights_body"],
            [child["node"] for child in root["children"]],
        )

        body = nodes["sample_weights_body"]
        assert_vertical_scroll_region(self, body, "$editor.density.gap.xsmall")
        self.assertEqual(
            [
                "sample_weights_axes",
                "sample_weights_run_forward_row",
                "sample_weights_run_left_row",
                "sample_weights_run_right_row",
                "sample_weights_idle_row",
                "sample_weights_footer",
            ],
            [child["node"] for child in body["children"]],
        )

    def test_validation_default_filter_uses_the_live_state_authority(self):
        nodes = load_nodes(VALIDATION_LOG)
        for name in (
            "validation_log_all",
            "validation_log_errors",
            "validation_log_warnings",
            "validation_log_infos",
        ):
            props = nodes[name].get("props", {})
            self.assertNotIn("selected", props, name)
            self.assertNotIn("checked", props, name)

        source = REFERENCE_MENU_ACTIONS.read_text(encoding="utf-8")
        self.assertIn(
            "self.select_exclusive(\n"
            "            BLEND_SPACE_VALIDATION_FILTER_CONTROLS,\n"
            '            "WorkbenchValidationLogAll",\n'
            "        )?;",
            source,
        )

    def test_gameplay_details_use_fixed_headers_and_scrollable_body(self):
        ability = load_nodes(ABILITY_WORKSPACE)
        self.assertEqual(
            ["ability_title", "ability_left_content"],
            [child["node"] for child in ability["ability_left"]["children"]],
        )
        self.assertEqual(
            ["ability_details_title", "ability_right_content"],
            [child["node"] for child in ability["ability_right"]["children"]],
        )
        assert_vertical_scroll_region(
            self, ability["ability_left_content"], "$editor.density.gap.small"
        )
        assert_vertical_scroll_region(
            self, ability["ability_right_content"], "$editor.density.gap.small"
        )
        self.assertEqual(3, len(ability["ability_left_content"]["children"]))
        self.assertEqual(4, len(ability["ability_right_content"]["children"]))

        effect = load_nodes(EFFECT_WORKSPACE)
        self.assertEqual(
            ["effect_title", "effect_left_content"],
            [child["node"] for child in effect["effect_left"]["children"]],
        )
        self.assertEqual(
            ["effect_details_title", "effect_right_content"],
            [child["node"] for child in effect["effect_right"]["children"]],
        )
        assert_vertical_scroll_region(
            self, effect["effect_left_content"], "$editor.density.gap.small"
        )
        assert_vertical_scroll_region(
            self, effect["effect_right_content"], "$editor.density.gap.small"
        )
        self.assertEqual(6, len(effect["effect_left_content"]["children"]))
        self.assertEqual(5, len(effect["effect_right_content"]["children"]))


if __name__ == "__main__":
    unittest.main()
