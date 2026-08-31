import unittest

from tools.runtime_ui_analog_ownership_pressure import pressure_report


class RuntimeUiAnalogOwnershipPressureTests(unittest.TestCase):
    def test_canonical_control_avoids_event_clone_and_normalization_ownership(self):
        report = pressure_report(100_000, 12)

        self.assertEqual(
            report["prior_owned_string_baseline"]["transient_string_allocations"],
            200_000,
        )
        self.assertEqual(
            report["prior_owned_string_baseline"]["minimum_control_bytes_copied"],
            2_400_000,
        )
        self.assertEqual(
            report["candidate_borrowed_canonical_control"][
                "transient_string_allocations"
            ],
            0,
        )
        self.assertEqual(
            report["candidate_borrowed_canonical_control"][
                "minimum_control_bytes_copied"
            ],
            0,
        )
        self.assertFalse(report["implementation_evidence"])
        self.assertTrue(report["implementation_source_contract"])
        self.assertFalse(report["is_product_timing"])

    def test_report_is_bound_to_zircon_and_unreal_sources(self):
        report = pressure_report(1, 1)

        self.assertEqual(len(report["source_binding"]["implementation"]), 3)
        self.assertEqual(len(report["source_binding"]["primary_reference"]), 1)
        for binding in [
            *report["source_binding"]["implementation"],
            *report["source_binding"]["primary_reference"],
        ]:
            self.assertGreater(binding["bytes"], 0)
            self.assertEqual(len(binding["sha256"]), 64)

    def test_rejects_invalid_inputs(self):
        for values in ((0, 1), (1, 0), (-1, 1), (1, -1)):
            with self.assertRaises(ValueError):
                pressure_report(*values)


if __name__ == "__main__":
    unittest.main()
