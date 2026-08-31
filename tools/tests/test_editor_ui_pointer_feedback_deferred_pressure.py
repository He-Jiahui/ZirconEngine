import unittest

from tools.editor_ui_pointer_feedback_deferred_pressure import (
    pressure_report,
    validate_output_path,
)


class EditorUiPointerFeedbackDeferredPressureTests(unittest.TestCase):
    def test_feedback_rebuilds_coalesce_at_frame_boundary(self):
        report = pressure_report(1_000, 17)
        self.assertEqual(report["comparison"]["frame_count"], 59)
        self.assertEqual(report["frame_owned_refresh"]["surface_rebuild_dirty_count"], 59)
        self.assertEqual(report["comparison"]["avoided_surface_rebuilds"], 941)
        self.assertGreater(report["comparison"]["refresh_reduction_ratio"], 16.9)
        self.assertFalse(report["is_product_timing"])

    def test_one_event_per_frame_preserves_refresh_count(self):
        report = pressure_report(32, 1)
        self.assertEqual(report["comparison"]["frame_count"], 32)
        self.assertEqual(report["comparison"]["avoided_surface_rebuilds"], 0)

    def test_rejects_non_positive_inputs(self):
        for values in ((0, 1), (1, 0), (-1, 1), (1, -1)):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

    def test_artifact_output_rejects_the_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\pointer-feedback.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\pointer-feedback.json").drive.upper(),
            "E:",
        )


if __name__ == "__main__":
    unittest.main()
