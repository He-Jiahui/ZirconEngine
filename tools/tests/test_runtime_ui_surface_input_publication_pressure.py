from pathlib import Path
import tempfile
import unittest

from tools.runtime_ui_surface_input_publication_pressure import (
    CRITICAL_SOURCE_CONTRACTS,
    SourceContractError,
    pressure_report,
    source_binding_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiSurfaceInputPublicationPressureTests(unittest.TestCase):
    def test_default_model_separates_event_work_publication_work_and_memory(self):
        report = pressure_report()

        baseline = report["legacy_reverse_surface_fanout_baseline"]
        current = report["current_input_publication_cutover"]
        retained = report["retained_input_publication"]
        memory = report["publication_memory_payload_estimate"]
        fallback = report["fallback_admission_policy"]
        delta = report["delta"]

        self.assertEqual(
            report["schema"],
            "zircon.runtime.ui_surface_input_publication_pressure.v11",
        )

        self.assertEqual(baseline["combined_surface_dispatches"], 25_600_000)
        self.assertEqual(current["combined_surface_dispatches"], 400_000)
        self.assertEqual(current["event_payload_clones"], 100_000)
        self.assertEqual(current["event_path_rebuild_probes"], 0)
        self.assertEqual(current["text_owner_sync_calls"], 400_000)
        self.assertEqual(current["focused_owner_surface_dispatches"], 100_000)
        self.assertEqual(current["navigation_owner_surface_dispatches"], 100_000)
        self.assertEqual(current["unrouted_mouse_motion_surface_dispatches"], 0)
        self.assertEqual(current["unrouted_session_counter_updates"], 100_000)
        self.assertEqual(current["dirty_first_event_node_scale_work"], 0)
        self.assertEqual(current["pre_input_publication_node_scale_work"], 640_000)

        self.assertEqual(retained["combined_surface_dispatches"], 400_000)
        self.assertEqual(retained["event_payload_clones"], 100_000)
        self.assertEqual(retained["event_path_rebuild_probes"], 0)
        self.assertEqual(retained["dirty_first_event_node_scale_work"], 0)
        self.assertEqual(retained["pre_input_publication_node_scale_work"], 640_000)

        self.assertEqual(memory["cell_count"], 510)
        self.assertEqual(memory["surface_cell_membership_count"], 32_640)
        self.assertEqual(memory["u32_payload_bytes"], 263_164)
        patch_scratch = report["publication_patch_scratch"]
        self.assertEqual(patch_scratch["cell_count"], 510)
        self.assertEqual(
            patch_scratch["removed_per_patch_occupancy_allocations"], 64
        )
        self.assertEqual(
            patch_scratch["removed_per_patch_occupancy_bytes"], 32_640
        )
        self.assertEqual(
            patch_scratch["removed_per_patch_footprint_allocations"], 64
        )
        self.assertEqual(patch_scratch["current_warm_occupancy_allocations"], 0)
        self.assertEqual(patch_scratch["current_warm_footprint_allocations"], 0)
        self.assertEqual(patch_scratch["retained_stamp_scratch_bytes"], 2_040)
        self.assertEqual(patch_scratch["removed_footprint_sort_invocations"], 64)
        self.assertEqual(patch_scratch["removed_footprint_sort_input_items"], 32_640)
        self.assertEqual(patch_scratch["current_footprint_sort_invocations"], 0)
        self.assertEqual(
            patch_scratch["removed_per_entry_cell_vector_allocations"], 640_000
        )
        self.assertEqual(
            patch_scratch["current_per_entry_cell_vector_allocations"], 0
        )
        self.assertEqual(delta["implemented_avoided_surface_dispatches"], 25_200_000)
        self.assertEqual(delta["implemented_avoided_event_payload_clones"], 25_100_000)
        self.assertEqual(delta["remaining_surface_dispatches_to_remove"], 0)
        self.assertEqual(delta["baseline_to_target_surface_dispatch_ratio"], 64.0)
        self.assertEqual(fallback["invalid_pointer_surface_dispatches"], 0)
        self.assertEqual(fallback["invalid_pointer_event_path_rebuild_probes"], 0)
        self.assertIn("cold reverse-fanout", fallback["unpublished_pointer_policy"])
        self.assertEqual(
            delta["current_to_target_surface_dispatch_ratio"],
            1.0,
        )
        self.assertFalse(report["is_product_timing"])

    def test_true_pointer_overlap_remains_visible_in_target_cost(self):
        one = pressure_report(candidate_surface_count=1)
        eight = pressure_report(candidate_surface_count=8)

        self.assertEqual(
            one["retained_input_publication"]["pointer_surface_dispatches"],
            100_000,
        )
        self.assertEqual(
            eight["retained_input_publication"]["pointer_surface_dispatches"],
            800_000,
        )
        self.assertEqual(
            eight["retained_input_publication"]["event_payload_clones"],
            700_000,
        )

    def test_sparse_footprints_bound_added_publication_payload(self):
        report = pressure_report(
            surface_count=16,
            candidate_surface_count=2,
            occupied_cells_per_surface=8,
        )

        memory = report["publication_memory_payload_estimate"]
        self.assertEqual(memory["surface_cell_membership_count"], 128)
        self.assertEqual(memory["u32_payload_bytes"], 3_068)

    def test_rejects_invalid_counts_and_grid_dimensions(self):
        with self.assertRaises(ValueError):
            pressure_report(surface_count=0)
        with self.assertRaises(ValueError):
            pressure_report(surface_count=4, candidate_surface_count=5)
        with self.assertRaises(ValueError):
            pressure_report(surface_count=4, dirty_surface_count=5)
        with self.assertRaises(ValueError):
            pressure_report(occupied_cells_per_surface=511)

    def test_current_fanout_model_is_bound_to_exact_current_sources(self):
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 10)
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")
        paths = {
            source["relative_path"] for source in binding["critical_sources"]
        }
        self.assertEqual(paths, {path for path, _ in CRITICAL_SOURCE_CONTRACTS})
        for source in binding["critical_sources"]:
            self.assertRegex(source["sha256"], r"^[0-9A-F]{64}$")
            self.assertGreater(source["byte_length"], 0)

    def test_current_source_binding_requires_the_unrouted_fast_path(self):
        runtime_contract = dict(CRITICAL_SOURCE_CONTRACTS)[
            "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
        ]

        self.assertIn(
            "if matches!(&event, UiInputEvent::MouseMotion(_))", runtime_contract
        )
        self.assertIn(
            "ui.surface_set.input.unrouted_reject_count", runtime_contract
        )
        self.assertIn("focused_surface: Option<usize>", runtime_contract)
        self.assertIn("input_requires_focus_owner(&event)", runtime_contract)
        self.assertIn("input_requires_navigation_owner(&event)", runtime_contract)
        self.assertIn(
            "ui.surface_set.input.focus_direct_route_count", runtime_contract
        )
        self.assertIn(
            ".query(viewport_size, point, previous_point)", runtime_contract
        )
        self.assertIn("RuntimeUiInputQueryAdmission::Rejected(reason)", runtime_contract)
        self.assertIn(
            "ui.surface_set.input.publication_unavailable_fallback_count",
            runtime_contract,
        )

    def test_source_binding_requires_retained_publication_patch_scratch(self):
        publication_contract = dict(CRITICAL_SOURCE_CONTRACTS)[
            "zircon_runtime/src/dynamic_api/session/runtime_ui/input_publication.rs"
        ]

        for token in (
            "cell_visit_stamps: Vec<u32>",
            "next_cell_visit_stamp: u32",
            "std::mem::take(&mut self.surface_footprints[surface_index])",
            "fn visit_bounded_cells(",
        ):
            self.assertIn(token, publication_contract)

    def test_source_binding_requires_resize_query_plumbing(self):
        contracts = dict(CRITICAL_SOURCE_CONTRACTS)

        self.assertIn(
            "physical_point: UiPoint",
            contracts[
                "zircon_runtime/src/dynamic_api/session/runtime_ui/input_publication.rs"
            ],
        )
        self.assertIn(
            "pub(crate) fn dispatch_input_event_with_query(",
            contracts["zircon_runtime/src/ui/dispatch/input_manager/manager.rs"],
        )
        self.assertIn(
            "dispatch_pointer_event_with_query_and_modifiers(",
            contracts["zircon_runtime/src/ui/surface/input/pointer.rs"],
        )

    def test_source_binding_fails_closed_without_publication_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
                if relative_path.endswith("runtime_ui/input_publication.rs"):
                    continue
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("\n".join(required_tokens), encoding="utf-8")

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_profile_output_rejects_system_drive(self):
        with self.assertRaises(ValueError):
            validate_output_path(Path("C:/zircon-profiles/forbidden.json"))


if __name__ == "__main__":
    unittest.main()
