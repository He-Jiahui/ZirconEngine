from pathlib import Path
import tempfile
import unittest

from tools.editor_svg_gpu_residency_pressure import (
    CRITICAL_SOURCE_CONTRACTS,
    SourceContractError,
    pressure_report,
    source_binding_report,
)


ROOT = Path(__file__).resolve().parents[2]


class EditorSvgGpuResidencyPressureTest(unittest.TestCase):
    def test_default_pressure_model_quantifies_retained_identity_work(self) -> None:
        result = pressure_report()

        baseline = result["repeated_per_command_reconstruction_baseline"]
        retained = result["retained_content_addressed_residency"]
        delta = result["delta"]
        budget = result["memory_budget_warning"]

        self.assertEqual(baseline["svg_file_reads"], 20_480_000)
        self.assertEqual(baseline["gpu_upload_writes"], 20_480_000)
        self.assertEqual(retained["cold_and_one_reload_svg_file_reads"], 257)
        self.assertEqual(retained["cold_and_one_reload_svg_rasterizations"], 1_028)
        self.assertEqual(retained["cold_and_one_reload_gpu_page_upload_writes"], 17)
        self.assertEqual(retained["stable_svg_file_reads"], 0)
        self.assertEqual(retained["stable_gpu_upload_writes"], 0)
        self.assertEqual(delta["svg_file_read_reduction_ratio"], 79_688.715953)
        self.assertEqual(delta["gpu_upload_write_reduction_ratio"], 1_204_705.882353)
        self.assertEqual(budget["configured_four_layer_ceiling_bytes"], 256 * 1024 * 1024)
        self.assertFalse(result["is_product_timing"])

    def test_invalid_change_counts_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(unique_svg_source_count=4, changed_svg_source_count=5)
        with self.assertRaises(ValueError):
            pressure_report(atlas_page_count=4, changed_atlas_page_count=5)

    def test_global_epoch_discards_every_inflight_product_for_unrelated_events(self) -> None:
        result = pressure_report(
            inflight_svg_product_count=8,
            unrelated_path_event_count=3,
            unchanged_path_event_count=2,
            affected_path_event_count=1,
            affected_svg_product_count=1,
        )

        current = result["global_epoch_invalidation"]
        targeted = result["source_generation_invalidation"]

        self.assertEqual(current["stale_discard_count"], 48)
        self.assertEqual(current["raster_attempt_count"], 56)
        self.assertEqual(
            current["status"], "historical_removed_current_source_baseline"
        )
        self.assertEqual(targeted["stale_discard_count"], 1)
        self.assertEqual(targeted["raster_attempt_count"], 9)
        self.assertEqual(targeted["status"], "current_source_contract")
        self.assertEqual(targeted["unrelated_event_stale_discard_count"], 0)
        self.assertEqual(targeted["unchanged_event_stale_discard_count"], 0)

    def test_no_asset_events_add_no_epoch_retry_work(self) -> None:
        result = pressure_report(
            inflight_svg_product_count=8,
            unrelated_path_event_count=0,
            unchanged_path_event_count=0,
            affected_path_event_count=0,
            affected_svg_product_count=0,
        )

        self.assertEqual(result["global_epoch_invalidation"]["raster_attempt_count"], 8)
        self.assertEqual(
            result["source_generation_invalidation"]["raster_attempt_count"], 8
        )

    def test_disjoint_surface_working_sets_remain_inside_device_budget(self) -> None:
        result = pressure_report(ui_surface_count=16)

        residency = result["multi_surface_residency_pressure"]

        self.assertEqual(
            residency["pre_ledger_reachable_unique_gpu_allocation_bytes"],
            1024 * 1024 * 1024,
        )
        self.assertEqual(
            residency["device_ledger_unique_allocation_bytes"], 64 * 1024 * 1024
        )
        self.assertEqual(
            residency["device_budget_rejected_working_set_bytes"],
            960 * 1024 * 1024,
        )
        self.assertEqual(residency["physical_budget_overshoot_bytes"], 0)
        self.assertEqual(residency["pre_ledger_budget_overshoot_ratio"], 16.0)
        self.assertEqual(residency["device_ledger_budget_ratio"], 1.0)
        self.assertEqual(
            residency["device_allocation_budget_bytes"], 64 * 1024 * 1024
        )

    def test_surface_count_must_be_positive(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(ui_surface_count=0)

    def test_external_provider_disables_stable_generation_fast_path(self) -> None:
        result = pressure_report(stable_present_count=10_000, atlas_page_count=16)

        pressure = result["external_provider_fast_path_pressure"]

        self.assertTrue(pressure["provider_installed_for_every_ui_surface"])
        self.assertFalse(pressure["generation_fast_path_enabled"])
        self.assertEqual(pressure["stable_image_source_count"], 16)
        self.assertEqual(pressure["current_provider_resolve_calls"], 160_000)
        self.assertEqual(pressure["current_registry_lock_acquisitions"], 160_000)
        self.assertEqual(pressure["target_provider_revision_checks"], 10_000)
        self.assertEqual(pressure["target_provider_resolve_calls"], 0)
        self.assertEqual(pressure["avoided_provider_resolve_calls"], 160_000)
        self.assertEqual(pressure["current_stable_complexity"], "O(P * R)")
        self.assertEqual(pressure["target_stable_complexity"], "O(P)")
        self.assertEqual(pressure["stable_svg_file_reads"], 0)
        self.assertEqual(pressure["stable_svg_tree_parses"], 0)
        self.assertEqual(pressure["stable_svg_rasterizations"], 0)
        self.assertEqual(pressure["stable_gpu_upload_writes"], 0)

    def test_source_binding_fails_closed_when_generation_authority_guard_changes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative_path, _ in CRITICAL_SOURCE_CONTRACTS:
                path = root / relative_path
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("authority intentionally changed\n", encoding="utf-8")

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_current_source_binding_is_ready_and_content_hashed(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"])
        self.assertEqual(len(binding["critical_sources"]), 20)
        critical_paths = {
            source["relative_path"] for source in binding["critical_sources"]
        }
        self.assertIn(
            "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/ui_perf.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_runtime/crates/zr_rhi_wgpu/src/production/device/native_recording.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_runtime/crates/zr_rhi_wgpu/src/production/submission/"
            "ui_image_retirement.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_runtime/src/graphics/runtime/render_framework/"
            "render_framework_trait_binding/wgpu_framework.rs",
            critical_paths,
        )
        self.assertIn(
            "zircon_runtime/src/graphics/runtime/render_framework/"
            "render_framework_state/viewport_product_registry.rs",
            critical_paths,
        )
        self.assertRegex(binding["source_set_sha256"], r"^[0-9A-F]{64}$")
        for source in binding["critical_sources"]:
            self.assertRegex(source["sha256"], r"^[0-9A-F]{64}$")
            self.assertGreater(source["byte_length"], 0)


if __name__ == "__main__":
    unittest.main()
