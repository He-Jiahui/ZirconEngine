from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRIORITY = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/toolbar_layout/priority.rs"
)
TEMPLATE_SURFACE = ROOT / (
    "zircon_editor/src/ui/workbench/reference/template_surface.rs"
)


class Editor06WorkbenchToolbarPriorityContractTests(unittest.TestCase):
    def test_toolbar_width_resolution_indexes_control_nodes_once(self) -> None:
        source = PRIORITY.read_text(encoding="utf-8")

        self.assertIn("ToolbarControlIndex", source)
        self.assertIn("ToolbarControlIndex::new(surface)", source)
        self.assertEqual(source.count(".values()"), 1)
        self.assertNotIn("surface_control_node_id", source)

    def test_componentized_workbench_reuses_a_stable_control_node_index(self) -> None:
        source = TEMPLATE_SURFACE.read_text(encoding="utf-8")

        self.assertIn("control_nodes: HashMap<String, UiNodeId>", source)
        self.assertIn("build_control_node_index(&surface)", source)
        self.assertIn("self.control_node_id(control_id)", source)
        self.assertNotIn("fn control_frame(surface: &UiSurface, control_id: &str)", source)
        self.assertNotIn("fn visible_control_frame(surface: &UiSurface, control_id: &str)", source)


if __name__ == "__main__":
    unittest.main()
