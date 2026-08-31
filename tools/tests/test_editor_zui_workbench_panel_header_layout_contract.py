import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOKENS = REPO_ROOT / "zircon_editor/assets/ui/editor/theme/editor_tokens.zui"
PANEL_HEADER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/composites/chrome/"
    "workbench_panel_header.zui"
)
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
    def test_panel_header_contains_standard_title_and_action_height(self):
        tokens = load_document(TOKENS)
        panel_header_height = tokens["chrome"]["panel_header_height"]

        self.assertEqual(30.0, panel_header_height)
        self.assertEqual(tokens["controls"]["compact_height"], panel_header_height)

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
