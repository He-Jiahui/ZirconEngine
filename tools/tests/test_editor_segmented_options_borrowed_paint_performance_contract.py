from pathlib import Path
import unittest

from tools.editor_segmented_options_paint_pressure import run


ROOT = Path(__file__).resolve().parents[2]
SEGMENTED = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_segmented_controls"
)


class EditorSegmentedOptionsBorrowedPaintPerformanceContractTests(unittest.TestCase):
    def test_option_projection_is_a_borrowed_iterator(self) -> None:
        source = (SEGMENTED / "options.rs").read_text(encoding="utf-8")
        projection = source.split("fn segmented_options", 1)[1].split(
            "pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selected_segment_value",
            1,
        )[0]

        self.assertIn("impl Iterator<Item = &str> + '_", projection)
        self.assertRegex(projection, r"node\.options\s*\.iter\(\)")
        self.assertNotIn("row_data", projection)
        self.assertNotIn("collect", projection)
        self.assertIn("fn segmented_option_count", source)

    def test_command_dispatch_does_not_materialize_an_option_vec(self) -> None:
        source = (SEGMENTED / "commands.rs").read_text(encoding="utf-8")
        branch = source.split("if is_segmented_control(node)", 1)[1].split(
            "if is_workbench_tab(node)", 1
        )[0]

        self.assertIn("let option_count = segmented_option_count(node);", branch)
        self.assertIn("option_count", branch)
        self.assertNotIn("let options =", branch)
        self.assertNotIn("&options", branch)

    def test_segment_paint_reiterates_borrowed_options(self) -> None:
        source = (SEGMENTED / "segments/body.rs").read_text(encoding="utf-8")

        self.assertIn("option_count: usize", source)
        self.assertIn("segmented_options(node).enumerate()", source)
        self.assertNotIn("options: &[SharedString]", source)
        self.assertNotIn("use crate::ui::retained_host::primitives::SharedString", source)

    def test_pressure_model_makes_the_extra_borrowed_pass_explicit(self) -> None:
        result = run(
            stable_paint_count=10_000,
            option_count=3,
            average_option_utf8_bytes=8,
        )

        self.assertEqual(
            result["delta"]["avoided_option_vec_allocation_count"], 10_000
        )
        self.assertEqual(
            result["delta"]["avoided_option_string_clone_count"], 30_000
        )
        self.assertEqual(
            result["delta"]["avoided_option_utf8_payload_bytes"], 240_000
        )
        self.assertEqual(
            result["delta"]["additional_borrowed_option_visit_count"], 30_000
        )


if __name__ == "__main__":
    unittest.main()
