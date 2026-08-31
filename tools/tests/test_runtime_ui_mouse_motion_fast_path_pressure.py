import unittest

from tools.runtime_ui_mouse_motion_fast_path_pressure import pressure_report


class RuntimeUiMouseMotionFastPathPressureTests(unittest.TestCase):
    def test_unrouted_fast_path_removes_generic_route_materialization(self):
        report = pressure_report(100_000, 12, 4, 3)

        self.assertEqual(
            report["legacy_generic_route_annotation"]["event_payload_clones"],
            100_000,
        )
        self.assertEqual(
            report["legacy_generic_route_annotation"]["focused_route_queries"],
            100_000,
        )
        self.assertEqual(
            report["legacy_generic_route_annotation"]["route_identity_copies"],
            4_300_000,
        )
        self.assertEqual(
            report["legacy_generic_route_annotation"][
                "route_trace_vector_allocations"
            ],
            500_000,
        )
        self.assertEqual(report["unrouted_fast_path"]["route_identity_copies"], 0)
        self.assertEqual(
            report["unrouted_fast_path"][
                "retained_diagnostic_note_string_allocations"
            ],
            200_000,
        )
        self.assertFalse(report["is_product_timing"])

    def test_empty_surface_state_has_no_phantom_route_work(self):
        report = pressure_report(1, 0, 0, 0)

        self.assertEqual(
            report["legacy_generic_route_annotation"]["focused_route_queries"], 0
        )
        self.assertEqual(
            report["legacy_generic_route_annotation"]["route_identity_copies"], 0
        )
        self.assertEqual(
            report["legacy_generic_route_annotation"][
                "route_trace_vector_allocations"
            ],
            0,
        )

    def test_rejects_invalid_inputs(self):
        with self.assertRaises(ValueError):
            pressure_report(0, 0, 0, 0)
        with self.assertRaises(ValueError):
            pressure_report(1, -1, 0, 0)
        with self.assertRaises(ValueError):
            pressure_report(1, 0, -1, 0)
        with self.assertRaises(ValueError):
            pressure_report(1, 0, 0, -1)


if __name__ == "__main__":
    unittest.main()
