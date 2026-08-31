import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOLBAR_LAYOUT = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/toolbar_layout.rs"
)


class EditorToolbarLayoutControlIndexContractTests(unittest.TestCase):
    def test_responsive_toolbar_width_updates_use_the_retained_control_index(self):
        source = TOOLBAR_LAYOUT.read_text(encoding="utf-8")

        self.assertIn("self.control_node_id", source)
        self.assertIn("self.apply_horizontal_content_width", source)
        self.assertIn("self.apply_fixed_control_width", source)
        self.assertNotIn("surface_control_node_id", source)
        self.assertNotIn("tree.nodes.values()", source)


if __name__ == "__main__":
    unittest.main()
