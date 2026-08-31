import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE = REPO_ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)


class EditorEditControlIndexContractTests(unittest.TestCase):
    def test_inspector_edits_read_control_properties_through_the_retained_index(self):
        property_edit = (WORKBENCH_BRIDGE / "property_edit.rs").read_text(
            encoding="utf-8"
        )
        transform_edit = (WORKBENCH_BRIDGE / "transform_edit.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("self.control_bool", property_edit)
        self.assertIn(".control_string(", property_edit)
        self.assertIn(".control_string(", transform_edit)
        for filename, source in (
            ("property_edit.rs", property_edit),
            ("transform_edit.rs", transform_edit),
        ):
            self.assertNotIn("tree.nodes.values()", source, filename)
            self.assertNotIn("fn control_string(", source, filename)


if __name__ == "__main__":
    unittest.main()
