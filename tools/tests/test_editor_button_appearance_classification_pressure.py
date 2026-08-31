from pathlib import Path
import unittest

from tools.editor_button_appearance_classification_pressure import (
    pressure_report,
    pressure_suite,
)


ROOT = Path(__file__).resolve().parents[2]
BUTTON_IDENTITY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_buttons/identity.rs"
)
BUTTON_GLYPH = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_buttons/content/glyph.rs"
)
BUTTON_COMMANDS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_buttons/commands.rs"
)
BUTTON_CONTENT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/"
    "template_buttons/content/entry.rs"
)
class EditorButtonAppearanceClassificationPressureTests(unittest.TestCase):
    def test_secondary_button_worst_case_counts_every_current_identity_scan(self):
        report = pressure_report(
            candidates_per_repaint=1,
            repaint_count=1,
            field_lengths=[32, 24, 24, 9, 16, 8],
        )

        self.assertEqual(report["inputs"]["paint_candidate_visits"], 1)
        self.assertEqual(
            report["current_string_classification"]["identity_array_projections"], 2
        )
        self.assertEqual(
            report["current_string_classification"]["field_needle_scans"], 102
        )
        self.assertGreater(
            report["current_string_classification"]["substring_windows"], 0
        )
        self.assertGreater(
            report["current_string_classification"]["upper_bound_byte_comparisons"],
            report["current_string_classification"]["substring_windows"],
        )
        self.assertEqual(report["published_typed_appearance"]["field_needle_scans"], 0)
        self.assertEqual(report["published_typed_appearance"]["typed_field_reads"], 2)
        self.assertEqual(report["allocation_reduction_claim"], 0)

    def test_default_suite_scales_by_candidate_visits_and_is_not_product_timing(self):
        suite = pressure_suite(
            scenarios=[(1, 10_000), (32, 10_000), (512, 1_000)],
            field_lengths=[32, 24, 24, 9, 16, 8],
        )

        visits = [
            scenario["inputs"]["paint_candidate_visits"]
            for scenario in suite["scenarios"]
        ]
        scans = [
            scenario["current_string_classification"]["field_needle_scans"]
            for scenario in suite["scenarios"]
        ]
        self.assertEqual(visits, [10_000, 320_000, 512_000])
        self.assertEqual(scans, [visit * 102 for visit in visits])
        self.assertFalse(suite["is_product_timing"])
        self.assertEqual(suite["classification_case"], "worst_case_nonmatching_identity")

    def test_rejects_invalid_pressure_inputs(self):
        with self.assertRaises(ValueError):
            pressure_report(0, 1, [1] * 6)
        with self.assertRaises(ValueError):
            pressure_report(1, 0, [1] * 6)
        with self.assertRaises(ValueError):
            pressure_report(1, 1, [1] * 5)
        with self.assertRaises(ValueError):
            pressure_report(1, 1, [1, 1, 1, 1, 1, -1])
        with self.assertRaises(ValueError):
            pressure_suite([], [1] * 6)

    def test_model_is_bound_to_current_paint_chain(self):
        identity = BUTTON_IDENTITY.read_text(encoding="utf-8")
        glyph = BUTTON_GLYPH.read_text(encoding="utf-8")
        commands = BUTTON_COMMANDS.read_text(encoding="utf-8")
        content = BUTTON_CONTENT.read_text(encoding="utf-8")

        self.assertIn("button_identity_values(node)", identity)
        self.assertIn(".windows(needle.len())", identity)
        self.assertIn("button_identity_values(node)", glyph)
        self.assertIn("let kind = button_kind(node);", commands)
        self.assertIn("let glyph = button_glyph(node);", content)


if __name__ == "__main__":
    unittest.main()
