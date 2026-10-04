import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
PANEL_HEADER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/chrome/"
    "workbench_panel_header.zui"
)
STRICT_THEME = REPO_ROOT / "zircon_editor/assets/ui/theme/editor_workbench_strict.zui"
BUTTON = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/"
    "workbench_button.zui"
)
SECTION_TITLE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/"
    "workbench_section_title.zui"
)
BLEND_SPACE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/modules/extensions/"
    "animation/workbench_extension_blend_space_workspace.zui"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiWorkbenchPanelHeaderLayoutContractTests(unittest.TestCase):
    def test_panel_header_is_a_continuous_pane_band_not_a_nested_card(self):
        panel_header = load_document(PANEL_HEADER)["nodes"]["root"]
        props = panel_header["props"]
        self.assertEqual(0.0, props["corner_radius"])
        self.assertEqual(0.0, props["border_width"])
        self.assertEqual("bottom", props["separator_edge"])
        self.assertEqual("$editor.separator.soft", props["separator_color"])
        self.assertEqual(
            "$editor.control.border_width", props["separator_thickness"]
        )

        strict_theme = STRICT_THEME.read_text(encoding="utf-8")
        rule_start = strict_theme.index('selector = ".workbench-panel-header"')
        next_rule = strict_theme.index("[[stylesheets.rules]]", rule_start)
        panel_header_rule = strict_theme[rule_start:next_rule]
        self.assertIn('background_color = "$workbench_panel_raised"', panel_header_rule)
        self.assertIn("border_width = 0.0", panel_header_rule)
        self.assertIn("radius = 0.0", panel_header_rule)

    def test_panel_header_contains_standard_title_and_action_height(self):
        tokens = load_document(TOKENS)
        panel_header_height = tokens["chrome"]["panel_header_height"]

        self.assertEqual(30.0, panel_header_height)
        self.assertLessEqual(panel_header_height, tokens["controls"]["compact_height"])

        panel_header = load_document(PANEL_HEADER)["nodes"]
        self.assertEqual(
            {
                "min": "$editor.chrome.panel_header.height",
                "preferred": "$editor.chrome.panel_header.height",
                "max": "$editor.chrome.panel_header.height",
                "stretch": "Fixed",
            },
            panel_header["root"]["layout"]["height"],
        )

        title_height = load_document(SECTION_TITLE)["nodes"]["root"]["layout"][
            "height"
        ]
        self.assertLessEqual(tokens["controls"]["dense_height"], panel_header_height)
        self.assertEqual("$editor.control.height.dense", title_height["preferred"])

        button_height = load_document(BUTTON)["nodes"]["root"]["layout"]["height"]
        self.assertEqual("$editor.control.height.compact", button_height["preferred"])

    def test_product_panel_header_actions_do_not_exceed_the_header(self):
        tokens = load_document(TOKENS)
        panel_header_height = tokens["chrome"]["panel_header_height"]
        nodes = load_document(BLEND_SPACE)["nodes"]

        for node_id in (
            "blend_space_preview_button",
            "blend_space_apply_button",
        ):
            height = nodes[node_id]["layout"]["height"]
            self.assertLessEqual(height["min"], panel_header_height, node_id)
            self.assertLessEqual(height["preferred"], panel_header_height, node_id)


if __name__ == "__main__":
    unittest.main()
