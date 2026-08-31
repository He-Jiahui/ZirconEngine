import unittest

from tools.runtime_ui_pointer_diagnostics_pressure import pressure_report


class RuntimeUiPointerDiagnosticsPressureTests(unittest.TestCase):
    def test_product_summary_removes_eager_trace_and_unrelated_handlers(self):
        report = pressure_report(100_000, 12, 12, 3, 5)

        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "diagnostic_identity_copies"
            ],
            2_800_000,
        )
        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "diagnostic_vector_allocations"
            ],
            400_000,
        )
        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "default_action_handler_probes"
            ],
            500_000,
        )
        self.assertEqual(
            report["eliminated_or_avoided"][
                "unrelated_default_action_handler_probes"
            ],
            400_000,
        )
        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "window_pump_diagnostic_string_allocations"
            ],
            300_000,
        )
        self.assertEqual(
            report["candidate_product_summary"][
                "window_pump_diagnostic_string_allocations"
            ],
            0,
        )
        self.assertEqual(
            report["candidate_product_summary"][
                "diagnostic_budget_presence_checks"
            ],
            1_600_000,
        )
        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "raw_mouse_motion_diagnostic_string_allocations"
            ],
            100_000,
        )
        self.assertEqual(
            report["candidate_product_summary"][
                "raw_mouse_motion_diagnostic_string_allocations"
            ],
            0,
        )
        self.assertEqual(
            report["candidate_product_summary"]["diagnostic_identity_copies"], 0
        )
        self.assertEqual(
            report["candidate_product_summary"]["ordinary_dispatch_path_allocations"],
            0,
        )
        self.assertEqual(
            report["candidate_explicit_full_capture"]["diagnostic_identity_copies"],
            4_000_000,
        )
        self.assertEqual(
            report["candidate_explicit_full_capture"][
                "diagnostic_vector_allocations"
            ],
            500_000,
        )
        self.assertFalse(report["implementation_evidence"])
        self.assertTrue(report["implementation_source_contract"])
        self.assertFalse(report["is_product_timing"])

    def test_empty_paths_only_retain_the_direct_step_in_the_current_model(self):
        report = pressure_report(1, 0, 0, 0, 0)

        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "diagnostic_identity_copies"
            ],
            1,
        )
        self.assertEqual(
            report["prior_eager_diagnostics_baseline"][
                "diagnostic_vector_allocations"
            ],
            1,
        )
        self.assertEqual(
            report["candidate_explicit_full_capture"]["diagnostic_identity_copies"],
            1,
        )
        self.assertEqual(
            report["eliminated_or_avoided"][
                "unrelated_default_action_handler_probes"
            ],
            0,
        )

    def test_rejects_invalid_inputs(self):
        with self.assertRaises(ValueError):
            pressure_report(0, 0, 0, 0, 0)
        for index in range(1, 5):
            values = [1, 0, 0, 0, 0]
            values[index] = -1
            with self.assertRaises(ValueError):
                pressure_report(*values)

    def test_report_is_bound_to_implementation_and_primary_reference_sources(self):
        report = pressure_report(1, 1, 1, 1, 1)

        implementation = report["source_binding"]["implementation"]
        reference = report["source_binding"]["primary_reference"]
        self.assertEqual(len(implementation), 13)
        self.assertEqual(len(reference), 2)
        for binding in [*implementation, *reference]:
            self.assertGreater(binding["bytes"], 0)
            self.assertEqual(len(binding["sha256"]), 64)

    def test_explicit_full_capture_is_bounded_for_adversarial_depth(self):
        report = pressure_report(1, 1_000, 1_000, 1_000, 1)

        self.assertEqual(
            report["candidate_explicit_full_capture"]["diagnostic_identity_copies"],
            401,
        )
        self.assertEqual(report["diagnostic_limits"]["route_nodes_per_path"], 128)
        self.assertEqual(report["diagnostic_limits"]["route_steps"], 256)
        self.assertEqual(report["diagnostic_limits"]["popup_entries"], 16)
        self.assertEqual(report["diagnostic_limits"]["combined_string_bytes"], 8192)


if __name__ == "__main__":
    unittest.main()
