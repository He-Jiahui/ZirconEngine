import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCENE_TREE = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_scene_tree_panel.zui"
)
REFERENCE_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/"
    "reference_menu_actions.rs"
)
PREVIEW_ACTIONS = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/workbench_preview_actions.rs"
)
BINDINGS = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/builtin/"
    "workbench_window_template_bindings.rs"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiSceneTreeHeaderContractTests(unittest.TestCase):
    def test_scene_identity_header_precedes_search_and_virtual_tree_content(self):
        document = load_document(SCENE_TREE)
        nodes = document["nodes"]

        self.assertEqual(
            ["scene_header", "scene_content"],
            [child["node"] for child in nodes["scene_tree_panel"]["children"]],
        )
        self.assertEqual("LeftDrawerHeaderRoot", nodes["scene_header"]["control_id"])
        self.assertEqual([{"node": "scene_title"}], nodes["scene_header"]["children"])
        self.assertEqual("WorkbenchSectionTitle", nodes["scene_title"]["component"])
        self.assertEqual("Scene", nodes["scene_title"]["props"]["text"])
        self.assertFalse(
            any(node.get("component") == "WorkbenchTab" for node in nodes.values())
        )

    def test_nonfunctional_scene_layers_mode_is_removed_end_to_end(self):
        sources = [
            SCENE_TREE.read_text(encoding="utf-8"),
            REFERENCE_ACTIONS.read_text(encoding="utf-8"),
            PREVIEW_ACTIONS.read_text(encoding="utf-8"),
            BINDINGS.read_text(encoding="utf-8"),
        ]
        retired_identifiers = (
            "WorkbenchSceneTabScene",
            "WorkbenchSceneTabLayers",
            "PanelTab/SceneTreeScene",
            "PanelTab/SceneTreeLayers",
            "scene_tree.scene_tab.select",
            "scene_tree.layers_tab.select",
        )

        for retired in retired_identifiers:
            for source in sources:
                with self.subTest(retired=retired):
                    self.assertNotIn(retired, source)


if __name__ == "__main__":
    unittest.main()
