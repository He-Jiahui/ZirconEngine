import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOLBAR = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_top_toolbar.zui"
)
ACTIVITY_RAIL = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_activity_rail.zui"
)
COMPONENT_DRAWER = REPO_ROOT / (
    "zircon_editor/assets/ui/editor/components/workbench/shell/"
    "workbench_component_drawer.zui"
)
ICON_TOOLTIP = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/icon_tooltip.rs"
)
ICON_TOOLTIP_RESOLVER = REPO_ROOT / (
    "zircon_editor/src/ui/template_runtime/workbench_tooltip.rs"
)
TOOLBAR_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/toolbar_layout.rs"
)


def load_document(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


class EditorZuiWorkbenchToolbarHintContractTests(unittest.TestCase):
    def test_density_collapsed_module_commands_keep_explicit_action_hints(self):
        nodes = load_document(TOOLBAR)["nodes"]
        expected = {
            "module_save": "Save Current Module",
            "module_browse": "Browse Current Module",
            "module_compile": "Compile Current Module",
        }

        for node_id, tooltip in expected.items():
            node = nodes[node_id]
            self.assertIn("workbench-toolbar-action", node["classes"])
            self.assertEqual(tooltip, node["props"].get("tooltip"))

    def test_repeated_module_utility_commands_use_icon_only_density(self):
        nodes = load_document(TOOLBAR)["nodes"]
        expected = {
            "module_save": "Save Current Module",
            "module_browse": "Browse Current Module",
        }

        for node_id, label in expected.items():
            node = nodes[node_id]
            self.assertEqual("WorkbenchIconButton", node["component"])
            self.assertEqual(label, node["props"].get("label"))
            self.assertNotIn("text", node["props"])
            width = node["layout"]["width"]
            self.assertEqual(34.0, width["min"])
            self.assertEqual(34.0, width["preferred"])
            self.assertEqual(34.0, width["max"])

        command_width = nodes["toolbar_module_commands"]["layout"]["width"]
        self.assertEqual(300.0, command_width["preferred"])
        self.assertEqual(300.0, command_width["max"])

    def test_responsive_projection_keeps_repeated_utility_commands_icon_only(self):
        source = TOOLBAR_LAYOUT.read_text(encoding="utf-8")

        self.assertEqual(2, source.count("always_icon_only: true"))
        self.assertEqual(1, source.count("always_icon_only: false"))
        self.assertIn(
            "let icon_only = command.always_icon_only || ultra;",
            source,
        )
        self.assertNotIn('label: "Save",\n        regular_width: 72.0', source)
        self.assertNotIn('label: "Browse",\n        regular_width: 92.0', source)

    def test_explicit_tooltip_is_not_restricted_to_icon_button_class(self):
        source = ICON_TOOLTIP_RESOLVER.read_text(encoding="utf-8")
        tooltip_lookup = '.get("tooltip")'
        icon_class_lookup = ".classes\n        .iter()"

        self.assertIn(tooltip_lookup, source)
        self.assertIn(icon_class_lookup, source)
        self.assertLess(
            source.index(tooltip_lookup),
            source.index(icon_class_lookup),
            "an explicit tooltip must resolve before the icon-button label fallback",
        )

    def test_activity_rail_labels_use_the_shared_icon_tooltip_fallback(self):
        nodes = load_document(ACTIVITY_RAIL)["nodes"]
        expected = {
            "rail_scene": "Scene",
            "rail_cube": "Entities",
            "rail_graph": "Graph",
            "rail_image": "Assets",
            "rail_audio": "Audio",
            "rail_code": "Code",
        }

        for node_id, label in expected.items():
            node = nodes[node_id]
            self.assertIn("workbench-rail-button", node["classes"])
            self.assertEqual(label, node["props"].get("label"))

        source = ICON_TOOLTIP_RESOLVER.read_text(encoding="utf-8")
        self.assertIn('const RAIL_BUTTON_CLASS: &str = "workbench-rail-button";', source)
        self.assertIn("class.as_str() == RAIL_BUTTON_CLASS", source)

    def test_non_icon_class_glyph_action_declares_an_explicit_tooltip(self):
        delete = load_document(COMPONENT_DRAWER)["nodes"]["button_delete"]

        self.assertEqual("", delete["props"].get("text"))
        self.assertNotIn("workbench-icon-button", delete["classes"])
        self.assertEqual("Delete", delete["props"].get("tooltip"))


if __name__ == "__main__":
    unittest.main()
