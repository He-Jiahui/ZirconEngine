from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TEXT_ENTRY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_text.rs"
)
TEXT_COMMAND = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_text/command.rs"
)
TEXT_ELIGIBILITY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_text/eligibility.rs"
)
TEXT_GEOMETRY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_text/geometry.rs"
)
NODE_LABELS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_labels.rs"
)


class EditorTemplateTextLazyMaterializationPerformanceContractTests(unittest.TestCase):
    def test_fallback_text_rejects_unpaintable_geometry_before_resolving_label(self) -> None:
        source = TEXT_ENTRY.read_text(encoding="utf-8")
        eligibility = TEXT_ELIGIBILITY.read_text(encoding="utf-8")

        semantic_gate = source.index("should_skip_template_text_before_label(")
        label_presence_gate = source.index("template_node_has_label(node, text_input_focus)")
        geometry_gate = source.index("is_paintable_text_slot(")
        label_resolution = source.index("let label = template_node_label(")

        self.assertLess(label_presence_gate, geometry_gate)
        self.assertLess(semantic_gate, geometry_gate)
        self.assertLess(geometry_gate, label_resolution)
        self.assertIn("fn should_skip_template_text_before_label(", eligibility)
        self.assertIn("is_icon_only_node(node)", eligibility)

    def test_text_command_rejects_fully_clipped_rectangles(self) -> None:
        source = TEXT_COMMAND.read_text(encoding="utf-8")

        self.assertIn("pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_paintable_text_slot", source)
        self.assertIn("intersect(text_rect, clip).is_some()", source)
        self.assertIn("if !is_paintable_text_slot(text_rect, clip, font_size)", source)
        self.assertIn("fn text_outside_clip_does_not_emit_a_command", source)

    def test_icon_geometry_probes_label_presence_without_materializing_owned_text(self) -> None:
        geometry = TEXT_GEOMETRY.read_text(encoding="utf-8")
        labels = NODE_LABELS.read_text(encoding="utf-8")

        self.assertNotIn("template_node_label(node, None)", geometry)
        self.assertIn("template_node_has_label(node, None)", geometry)
        self.assertIn("fn template_node_has_label(", labels)
        self.assertIn("property_row_has_label(node)", labels)
        self.assertIn("fallback_node_has_label(node)", labels)


if __name__ == "__main__":
    unittest.main()
