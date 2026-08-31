from pathlib import Path
import unittest

from tools.editor_ui_frame_extraction_cache_pressure import (
    pressure_report,
    validate_output_path,
)


class EditorUiFrameExtractionCachePressureTests(unittest.TestCase):
    def test_render_input_refreshes_reuse_geometry_snapshot(self):
        report = pressure_report(1_000, 256)

        self.assertEqual(
            report["current_unconditional_extraction"]["frame_control_visits"],
            256_000,
        )
        self.assertEqual(
            report["cached_geometry_extraction"]["frame_control_visits"], 0
        )
        self.assertEqual(report["comparison"]["avoided_frame_control_visits"], 256_000)
        self.assertFalse(report["is_product_timing"])

    def test_rejects_non_positive_inputs(self):
        for values in ((0, 1), (1, 0), (-1, 1), (1, -1)):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\frame-cache.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\frame-cache.json").drive.upper(),
            "E:",
        )


if __name__ == "__main__":
    unittest.main()
