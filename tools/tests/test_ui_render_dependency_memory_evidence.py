from pathlib import Path
import unittest

from tools.ui_render_dependency_memory_evidence import (
    MAX_IMAGE_POOL_BYTES,
    MAX_METADATA_BUDGET_BYTES,
    PREFIX,
    evaluate_memory_evidence,
    validate_output_path,
    validate_source_manifest,
)


PHASES = ("warmup", "pressure", "quiescent")


def counter(name, value):
    return {"name": name, "value": value}


def valid_timeline():
    counters = [
        counter(f"{PREFIX}.max_retained_generation_count", 3),
        counter(f"{PREFIX}.metadata_budget_bytes", MAX_METADATA_BUDGET_BYTES),
        counter(f"{PREFIX}.delta_publish_count", 8),
        counter(f"{PREFIX}.retirement_count", 8),
        counter(f"{PREFIX}.eviction_count", 2),
        counter(f"{PREFIX}.global_binding_scan_count", 0),
        counter(f"{PREFIX}.full_generation_payload_clone_bytes", 0),
        counter(f"{PREFIX}.present_liveness_scan_count", 0),
    ]
    values = {
        "warmup": (1, 0, 2_000_000, 20_000_000, 32, 32, 8_000_000),
        "pressure": (3, 2, 2_300_000, 24_000_000, 32, 32, 12_000_000),
        "quiescent": (1, 0, 2_000_000, 20_000_000, 32, 32, 8_000_000),
    }
    for phase in PHASES:
        (
            live,
            pending,
            metadata,
            source_payload,
            bindings,
            identities,
            image_bytes,
        ) = values[phase]
        counters.extend(
            (
                counter(f"{PREFIX}.{phase}.snapshot_count", 1),
                counter(f"{PREFIX}.{phase}.live_generation_count", live),
                counter(
                    f"{PREFIX}.{phase}.pending_retired_generation_count", pending
                ),
                counter(f"{PREFIX}.{phase}.metadata_bytes", metadata),
                counter(f"{PREFIX}.{phase}.source_payload_bytes", source_payload),
                counter(f"{PREFIX}.{phase}.binding_product_count", bindings),
                counter(
                    f"{PREFIX}.{phase}.unique_binding_identity_count", identities
                ),
                counter(
                    f"{PREFIX}.{phase}.image_shared_resident_bytes", image_bytes
                ),
                counter(f"{PREFIX}.{phase}.image_cache_resident_bytes", image_bytes),
                counter(
                    f"{PREFIX}.{phase}.image_cache_cpu_resident_bytes", image_bytes
                ),
            )
        )
    return {"counters": counters}


def valid_interaction():
    return {
        "interaction": {
            "scenario": "render_dependency_memory_pressure",
            "requested_delta_cycles": 8,
            "completed_delta_cycles": 8,
            "same_resource_identity_set": True,
            "process_id": 4242,
            "elapsed_ms": 2_000,
            "processor_time_delta_ms": 400,
            "cpu_core_utilization_percent": 20,
            "cpu_system_utilization_percent": 2.5,
            "logical_processor_count": 8,
            "start_working_set_bytes": 100_000_000,
            "end_working_set_bytes": 110_000_000,
            "peak_working_set_bytes": 120_000_000,
            "start_private_bytes": 80_000_000,
            "end_private_bytes": 90_000_000,
            "peak_private_bytes": 95_000_000,
            "quiescence_process_id": 4242,
            "quiescence_requested_ms": 2_000,
            "quiescence_elapsed_ms": 2_050,
            "quiescence_working_set_bytes": 108_000_000,
            "quiescence_private_bytes": 88_000_000,
            "quiescence_sampled": True,
        }
    }


def valid_manifest():
    required = (
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
        "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
        "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
        "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs",
        "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
        "zircon_editor/src/ui/retained_host/ui_perf.rs",
        "tools/ui-profile-process-evidence.ps1",
    )
    return {
        "schema_version": 2,
        "scenario": "render_dependency_memory_pressure",
        "capture": {
            "options": {
                "run_phase": "measured",
                "run_ordinal": 1,
                "measured_run_count": 3,
            }
        },
        "repository": {
            "git": {"revision": "a" * 40},
            "critical_source_files": [
                {"relative_path": path, "sha256": "B" * 64} for path in required
            ],
        },
    }


def blocker_codes(result):
    return {blocker["code"] for blocker in result["blockers"]}


class UiRenderDependencyMemoryEvidenceTests(unittest.TestCase):
    def test_accepts_complete_three_phase_bounded_memory_evidence(self):
        result = evaluate_memory_evidence(valid_timeline(), valid_interaction())

        self.assertTrue(result["ready"])
        self.assertEqual(result["phases"]["pressure"]["live_generation_count"], 3)
        self.assertEqual(result["process"]["completed_delta_cycles"], 8)
        self.assertFalse(result["scope"]["driver_gpu_residency_measured"])

    def test_rejects_missing_duplicate_or_invalid_counter(self):
        timeline = valid_timeline()
        timeline["counters"] = timeline["counters"][1:]
        self.assertIn(
            "missing_counter",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

        timeline = valid_timeline()
        timeline["counters"].append(
            counter(f"{PREFIX}.warmup.metadata_bytes", 2_000_000)
        )
        self.assertIn(
            "duplicate_snapshot_counter",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

        timeline = valid_timeline()
        timeline["counters"][0]["value"] = float("nan")
        self.assertIn(
            "invalid_counter_value",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

    def test_rejects_generation_or_metadata_budget_violation(self):
        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.pressure.live_generation_count":
                item["value"] = 4
        self.assertIn(
            "retained_generation_bound_exceeded",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.pressure.metadata_bytes":
                item["value"] = MAX_METADATA_BUDGET_BYTES + 1
        self.assertIn(
            "metadata_budget_exceeded",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

    def test_rejects_binding_or_image_pool_budget_violation(self):
        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.pressure.binding_product_count":
                item["value"] = 33
        self.assertIn(
            "binding_identity_conservation_failed",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.pressure.image_cache_resident_bytes":
                item["value"] = MAX_IMAGE_POOL_BYTES + 1
        self.assertIn(
            "image_pool_budget_exceeded",
            blocker_codes(evaluate_memory_evidence(timeline, valid_interaction())),
        )

    def test_rejects_missing_pressure_activity_or_forbidden_global_work(self):
        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.delta_publish_count":
                item["value"] = 0
            if item["name"] == f"{PREFIX}.global_binding_scan_count":
                item["value"] = 1
            if item["name"] == f"{PREFIX}.pressure.metadata_bytes":
                item["value"] = 2_000_000
            if item["name"] == f"{PREFIX}.pressure.source_payload_bytes":
                item["value"] = 20_000_000
        codes = blocker_codes(evaluate_memory_evidence(timeline, valid_interaction()))
        self.assertIn("insufficient_delta_pressure_activity", codes)
        self.assertIn("forbidden_global_dependency_work", codes)
        self.assertIn("pressure_memory_did_not_overlap", codes)

    def test_rejects_quiescent_generation_or_residency_leak(self):
        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.quiescent.live_generation_count":
                item["value"] = 2
            if item["name"] == f"{PREFIX}.quiescent.image_shared_resident_bytes":
                item["value"] = 8_000_001
        codes = blocker_codes(evaluate_memory_evidence(timeline, valid_interaction()))
        self.assertIn("quiescent_generation_retirement_incomplete", codes)
        self.assertIn("quiescent_residency_not_recovered", codes)

    def test_rejects_incoherent_phase_lifecycle_or_identity_set(self):
        timeline = valid_timeline()
        for item in timeline["counters"]:
            if item["name"] == f"{PREFIX}.warmup.live_generation_count":
                item["value"] = 0
            if item["name"] == f"{PREFIX}.pressure.pending_retired_generation_count":
                item["value"] = 4
            if item["name"] == f"{PREFIX}.pressure.unique_binding_identity_count":
                item["value"] = 31
        codes = blocker_codes(evaluate_memory_evidence(timeline, valid_interaction()))
        self.assertIn("warmup_generation_not_canonical", codes)
        self.assertIn("pending_retirement_count_incoherent", codes)
        self.assertIn("resource_identity_set_changed", codes)

    def test_rejects_invalid_or_excessive_process_memory_evidence(self):
        interaction = valid_interaction()
        interaction["interaction"]["quiescence_process_id"] = 4243
        self.assertIn(
            "invalid_process_quiescence",
            blocker_codes(evaluate_memory_evidence(valid_timeline(), interaction)),
        )

        interaction = valid_interaction()
        interaction["interaction"]["quiescence_private_bytes"] = 200_000_000
        interaction["interaction"]["peak_private_bytes"] = 200_000_000
        self.assertIn(
            "process_memory_growth_budget_exceeded",
            blocker_codes(evaluate_memory_evidence(valid_timeline(), interaction)),
        )

    def test_source_manifest_is_measured_scenario_and_owner_complete(self):
        self.assertEqual(validate_source_manifest(valid_manifest()), [])
        manifest = valid_manifest()
        manifest["scenario"] = "click"
        manifest["repository"]["critical_source_files"].pop()
        codes = {blocker["code"] for blocker in validate_source_manifest(manifest)}
        self.assertIn("invalid_source_manifest_scenario", codes)
        self.assertIn("missing_critical_source", codes)

    def test_output_artifacts_are_restricted_to_d_e_or_f(self):
        for path in (
            Path("D:/profiles/memory-evidence.json"),
            Path("E:/profiles/memory-evidence.json"),
            Path("F:/profiles/memory-evidence.json"),
        ):
            with self.subTest(path=path):
                self.assertEqual(validate_output_path(path), path.resolve())
        relative = Path("memory.json")
        self.assertEqual(validate_output_path(relative), relative.resolve())
        for path in (Path("C:/profiles/memory-evidence.json"),):
            with self.subTest(path=path):
                with self.assertRaises(ValueError):
                    validate_output_path(path)


if __name__ == "__main__":
    unittest.main()
