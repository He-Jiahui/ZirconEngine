import unittest

from tools.runtime_ui_drag_drop_ownership_pressure import pressure_report


DEFAULTS = {
    "drag_update_events": 100_000,
    "target_transitions": 100,
    "payload_bytes": 4096,
    "payload_string_fields": 8,
    "route_depth": 8,
    "focus_depth": 6,
    "popup_entries": 2,
    "surface_snapshot_entries": 50_000,
}


class RuntimeUiDragDropOwnershipPressureTests(unittest.TestCase):
    def test_shared_payload_eliminates_deep_payload_copy_work(self) -> None:
        report = pressure_report(**DEFAULTS)

        self.assertEqual(
            report["prior_owned_payload_baseline"]["payload_deep_clones"],
            600_000,
        )
        self.assertEqual(
            report["prior_owned_payload_baseline"]["payload_string_allocations"],
            4_800_000,
        )
        self.assertEqual(
            report["prior_owned_payload_baseline"]["minimum_payload_bytes_copied"],
            2_457_600_000,
        )
        self.assertEqual(report["candidate_shared_payload"]["payload_deep_clones"], 0)
        self.assertEqual(report["candidate_shared_payload"]["arc_reference_clones"], 300_000)

    def test_steady_target_updates_avoid_whole_surface_snapshots(self) -> None:
        report = pressure_report(**DEFAULTS)
        snapshots = report["steady_target_transaction_snapshot"]

        self.assertEqual(snapshots["prior_full_surface_snapshots"], 100_000)
        self.assertEqual(snapshots["candidate_full_surface_snapshots"], 100)
        self.assertEqual(snapshots["eliminated_full_surface_snapshots"], 99_900)
        self.assertEqual(snapshots["prior_retained_entry_clone_units"], 5_000_000_000)
        self.assertEqual(snapshots["candidate_retained_entry_clone_units"], 5_000_000)

    def test_summary_skips_route_trace_steps_and_popup_string_projection(self) -> None:
        report = pressure_report(**DEFAULTS)
        diagnostics = report["optional_diagnostics_projection"]

        self.assertEqual(diagnostics["prior_route_node_writes"], 6_000_000)
        self.assertEqual(diagnostics["candidate_summary_route_node_writes"], 0)
        self.assertEqual(diagnostics["candidate_full_route_node_writes"], 3_000_000)
        self.assertEqual(diagnostics["prior_route_step_writes"], 900_000)
        self.assertEqual(diagnostics["prior_popup_string_clones"], 400_000)
        self.assertEqual(diagnostics["candidate_summary_popup_string_clones"], 0)

    def test_report_is_bound_to_zircon_and_unreal_sources(self) -> None:
        report = pressure_report(**DEFAULTS)

        self.assertEqual(len(report["source_binding"]["implementation"]), 8)
        self.assertEqual(len(report["source_binding"]["primary_reference"]), 2)
        for binding in [
            *report["source_binding"]["implementation"],
            *report["source_binding"]["primary_reference"],
        ]:
            self.assertGreater(binding["bytes"], 0)
            self.assertEqual(len(binding["sha256"]), 64)
        self.assertFalse(report["implementation_evidence"])
        self.assertTrue(report["implementation_source_contract"])
        self.assertFalse(report["is_product_timing"])

    def test_rejects_invalid_inputs(self) -> None:
        for key in (
            "drag_update_events",
            "payload_bytes",
            "payload_string_fields",
            "route_depth",
            "surface_snapshot_entries",
        ):
            values = dict(DEFAULTS)
            values[key] = 0
            with self.assertRaises(ValueError):
                pressure_report(**values)
        for key in ("target_transitions", "focus_depth", "popup_entries"):
            values = dict(DEFAULTS)
            values[key] = -1
            with self.assertRaises(ValueError):
                pressure_report(**values)
        values = dict(DEFAULTS)
        values["target_transitions"] = values["drag_update_events"] + 1
        with self.assertRaises(ValueError):
            pressure_report(**values)


if __name__ == "__main__":
    unittest.main()
