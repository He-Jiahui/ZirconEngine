from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
MODULE = ROOT / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion"
PAINT_PIPELINE = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_node_pipeline"
)
CONSOLE_PROJECTOR = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/"
    "docks/pane/template_nodes/console_output.rs"
)
CONSOLE_METADATA = ROOT / "zircon_editor/src/ui/retained_host/console_output.rs"


class EditorInspectorHierarchyConsoleProjectionPerformanceContractTests(unittest.TestCase):
    def test_inspector_field_classification_and_key_encoding_avoid_temporary_strings(self) -> None:
        source = (MODULE / "inspector_fields.rs").read_text(encoding="utf-8")
        classification = source.split("fn inspector_numeric_kind", 1)[1].split(
            "fn inspector_body_frame", 1
        )[0]
        key_encoding = source.split("fn inspector_component_key", 1)[1].split(
            "fn inspector_numeric_kind", 1
        )[0]

        self.assertIn("eq_ignore_ascii_case", classification)
        self.assertNotIn("to_ascii_lowercase", classification)
        self.assertIn("write!(&mut key", key_encoding)
        self.assertNotIn('format!("{:x}"', key_encoding)

    def test_hierarchy_template_state_uses_one_node_pass(self) -> None:
        source = (MODULE / "hierarchy_projection.rs").read_text(encoding="utf-8")
        function = source.split("fn apply_hierarchy_template_state", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertEqual(function.count("for node in nodes"), 1)
        self.assertNotIn(".iter_mut()", function)

    def test_console_controls_are_projected_in_one_node_pass(self) -> None:
        source = (MODULE / "console_projection.rs").read_text(encoding="utf-8")
        controls = source.split("fn project_console_controls", 1)[1].split(
            "fn project_console_output_lines", 1
        )[0]

        self.assertEqual(controls.count("for node in nodes"), 1)
        self.assertNotIn(".iter_mut().find", controls.replace("\n", ""))

    def test_console_line_count_stays_inside_the_generation_owned_snapshot(self) -> None:
        source = (MODULE / "console_projection.rs").read_text(encoding="utf-8")
        function = source.split("fn project_console_output_lines", 1)[1].split(
            "fn console_output_text_tone", 1
        )[0]

        self.assertIn("new_virtualized_snapshot", function)
        self.assertIn("output.clone()", function)
        self.assertNotIn("output.levels()", function)
        self.assertNotIn("output.as_ref()", function)

    def test_console_paint_streams_visible_rows_without_a_visit_plan_allocation(self) -> None:
        transform = (PAINT_PIPELINE / "transform.rs").read_text(encoding="utf-8")
        draw = (PAINT_PIPELINE / "draw.rs").read_text(encoding="utf-8")
        projector = CONSOLE_PROJECTOR.read_text(encoding="utf-8")
        metadata = CONSOLE_METADATA.read_text(encoding="utf-8")
        implementation = projector.split(
            "impl TemplateNodePaintTransform for ConsoleOutputProjector", 1
        )[1].split("#[cfg(test)]", 1)[0]

        self.assertIn("fn stream_row_visit_indices", transform)
        self.assertIn("stream_row_visit_indices", draw)
        self.assertIn("fn stream_row_visit_indices", implementation)
        self.assertNotIn("fn row_visit_indices", implementation)
        self.assertNotIn(".visible_node_rows(", implementation)
        row_stream = metadata.split("fn stream_visible_node_rows", 1)[1].split(
            "fn visible_line_node_rows", 1
        )[0]
        self.assertIn("visit: &mut dyn FnMut(usize)", row_stream)
        self.assertNotIn("Vec", row_stream)
        self.assertIn("stream_visible_node_rows", implementation)


if __name__ == "__main__":
    unittest.main()
