import csv
import hashlib
import json
import struct
import subprocess
import tempfile
import unittest
import zlib
from pathlib import Path
from unittest import mock

from tools.zircon_summarize_shader_pbr_profile import (
    _post_stage_hydration_elapsed_ns,
    _write_analysis_output,
    summarize_profile,
    validate_profile_completion_receipt,
)
from tools.zircon_validate_shader_pbr_viewer_evidence import (
    _CURRENT_IBL_BAKE_ALGORITHM_VERSION,
    ready_frame_evidence_summary,
    validate_current_ready_frame_evidence,
)


_PROFILE_TOOL_PATHS = (
    "tools/performance-machine-manifest.ps1",
    "tools/profile-capture-manifest.ps1",
    "tools/shader-pbr-profile-contract.ps1",
    "tools/shader-pbr-profile-evidence-identity.ps1",
    "tools/shader-pbr-profile-publication.ps1",
    "tools/shader-pbr-profile-runtime-evidence.ps1",
    "tools/shader-pbr-profile-toolchain.ps1",
    "tools/write_zircon_shader_pbr_build_provenance.ps1",
    "tools/zircon_pbr_visual_oracle.py",
    "tools/zircon_profile_shader_pbr_viewer.ps1",
    "tools/zircon_shader_pbr_evidence_identity.py",
    "tools/zircon_shader_pbr_profile_tool_identity.py",
    "tools/zircon_summarize_shader_pbr_profile.py",
    "tools/zircon_validate_shader_pbr_gpu_timing_evidence.py",
    "tools/zircon_validate_shader_pbr_renderdoc_replay.py",
    "tools/zircon_validate_shader_pbr_viewer_evidence.py",
)


class ZirconSummarizeShaderPbrProfileTests(unittest.TestCase):
    def test_completion_receipt_binds_the_entire_profile_root_before_a_consumer_accepts_it(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            receipt_path = self._write_completion_receipt(summary_path)

            receipt = validate_profile_completion_receipt(summary_path, receipt_path)
            analysis = summarize_profile(
                summary_path,
                completion_receipt_path=receipt_path,
            )

        self.assertEqual("completed", receipt["status"])
        self.assertEqual(5, analysis["modes"]["cold"]["sample_count"])

    def test_completion_receipt_rejects_a_profile_artifact_changed_after_commit(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            receipt_path = self._write_completion_receipt(summary_path)
            summary_path.write_text("tampered", encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "completion receipt artifact SHA-256"):
                validate_profile_completion_receipt(summary_path, receipt_path)

    def test_replays_every_ready_frame_in_process(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            with mock.patch(
                "tools.zircon_summarize_shader_pbr_profile.validate_current_ready_frame_evidence",
                wraps=validate_current_ready_frame_evidence,
            ) as validate_current:
                summarize_profile(summary_path)

            self.assertEqual(10, validate_current.call_count)

    def test_replays_each_ready_frame_with_its_bound_display_oracle(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            first_report = summary["modes"]["cold"][0]
            reference_png = profile_root / "display-oracle-reference.png"
            reference_png.write_bytes(
                Path(first_report["artifacts"]["ready_png"]["path"]).read_bytes()
            )
            oracle_path = profile_root / "display-oracle.json"
            oracle_path.write_text(
                json.dumps(
                    {
                        "schema": "zircon_pbr_display_visual_oracle_v1",
                        "reference_png": reference_png.name,
                        "reference_png_sha256": hashlib.sha256(
                            reference_png.read_bytes()
                        ).hexdigest(),
                        "expected_metadata": {
                            "schema": "zircon_shader_pbr_viewer_ready_frame_evidence_v17",
                            "material_fixture": "metal-mirror",
                            "required_material_base_pipeline_kind": "environment-only-pbr-base",
                            "required_material_base_pipeline_ready_at_capture": "true",
                            "environment_only_base_prewarm_requested": "true",
                        },
                        "comparison": {
                            "max_mean_abs_error": 0.0,
                            "max_p99_abs_error": 0,
                            "exceeding_abs_error": 0,
                            "max_exceeding_pixel_fraction": 0.0,
                        },
                        "semantic_regions": [],
                    },
                    sort_keys=True,
                ),
                encoding="utf-8",
            )
            oracle_fingerprint = self._fingerprint(oracle_path)
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["capture"] = {"display_visual_oracle": oracle_fingerprint}
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            for reports in summary["modes"].values():
                for report in reports:
                    report["display_visual_oracle"] = oracle_fingerprint
                    ready_png = Path(report["artifacts"]["ready_png"]["path"])
                    ready_validation_path = Path(
                        report["artifacts"]["ready_validation"]["path"]
                    )
                    ready_validation_path.write_text(
                        json.dumps(
                            ready_frame_evidence_summary(
                                validate_current_ready_frame_evidence(
                                    ready_png,
                                    expected_backend="wgpu(dx12)",
                                    visual_oracle_path=oracle_path,
                                )
                            ),
                            sort_keys=True,
                        ),
                        encoding="utf-8",
                    )
                    report["artifacts"]["ready_validation"] = self._fingerprint(
                        ready_validation_path
                    )
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            analysis = summarize_profile(summary_path)

        self.assertEqual("metal-mirror", analysis["material_fixture"])
        self.assertEqual(oracle_fingerprint, analysis["display_visual_oracle"])

    def test_summarizes_five_run_cold_warm_matrix_with_bound_energy_data(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)

            analysis = summarize_profile(summary_path)

            self.assertEqual(5, analysis["modes"]["cold"]["sample_count"])
            self.assertEqual(5, analysis["modes"]["warm"]["sample_count"])
            self.assertEqual("metal-mirror", analysis["material_fixture"])
            self.assertEqual("renderer_initialization", analysis["modes"]["cold"]["bottleneck"])
            self.assertEqual(15.0, analysis["modes"]["cold"]["energy_meter"]["mean_power_watts"])
            self.assertEqual("meter_instance_sum", analysis["modes"]["cold"]["energy_meter"]["scope"])
        self.assertTrue(analysis["modes"]["cold"]["cpu_sampling"]["attribution_ready"])
        self.assertEqual(10, analysis["modes"]["warm"]["gpu_pass_median_us"]["direct_scene_content"])
        self.assertEqual(
            {
                "advanced_pbr_opaque_command_count": 0,
                "cached_command_hit_count": 1,
                "command_rebuild_count": 0,
                "dynamic_command_count": 0,
                "opaque_command_count": 1,
            },
            analysis["modes"]["warm"]["gpu_mesh_submission"],
        )

    def test_summarizes_source_bound_shader_pipeline_runtime_spans(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            analysis = summarize_profile(summary_path)

        cold = analysis["modes"]["cold"]["shader_pipeline_cpu"]
        warm = analysis["modes"]["warm"]["shader_pipeline_cpu"]
        self.assertEqual("microseconds", cold["unit"])
        self.assertEqual(
            "per_run_stage_sum_then_upper_nearest_percentile",
            cold["aggregation"],
        )
        self.assertEqual(
            "inclusive_per_span; different stages may overlap",
            cold["duration_semantics"],
        )
        self.assertEqual(
            {
                "run_sample_count": 5,
                "run_presence_count": 5,
                "span_count": 10,
                "total_duration_us": 1_520,
                "per_run_duration_us": {
                    "p50": 304,
                    "p95": 506,
                    "p99": 506,
                    "max": 506,
                },
                "per_run_span_count": {
                    "p50": 2,
                    "p95": 2,
                    "p99": 2,
                    "max": 2,
                },
            },
            cold["stages"]["mesh_source_build"],
        )
        self.assertEqual(154, warm["stages"]["mesh_source_build"]["per_run_duration_us"]["p50"])
        self.assertEqual(5, cold["stages"]["disk_cache_write"]["run_presence_count"])

    def test_rejects_runtime_profile_trace_or_stage_summary_tampering(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            report = summary["modes"]["cold"][0]
            runtime_profile = report["artifacts"]["runtime_profile"]
            timeline_path = Path(runtime_profile["artifacts"]["timeline"]["path"])
            timeline = json.loads(timeline_path.read_text(encoding="utf-8"))
            timeline["spans"][0]["duration_us"] += 1
            timeline_path.write_text(json.dumps(timeline), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "runtime_profile_timeline SHA-256"):
                summarize_profile(summary_path)

            runtime_profile["artifacts"]["timeline"] = self._fingerprint(timeline_path)
            runtime_profile["shader_pipeline_stage_counts"]["material_requirement_admission"] += 1
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "stage count does not match"):
                summarize_profile(summary_path)

    def test_rejects_runtime_profile_with_lost_samples(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            report = summary["modes"]["warm"][0]
            runtime_profile = report["artifacts"]["runtime_profile"]
            timeline_path = Path(runtime_profile["artifacts"]["timeline"]["path"])
            timeline = json.loads(timeline_path.read_text(encoding="utf-8"))
            timeline["recorder_retention"][0]["spans"]["overwritten"] = 1
            timeline["recorder_retention"][0]["spans"]["written"] += 1
            timeline_path.write_text(json.dumps(timeline), encoding="utf-8")
            runtime_profile["artifacts"]["timeline"] = self._fingerprint(timeline_path)
            runtime_profile["recorder_retention"] = timeline["recorder_retention"]
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "lost spans samples"):
                summarize_profile(summary_path)

    def test_rejects_profile_tool_changed_after_manifest_capture(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            tool = manifest["repository"]["profile_tool_files"][0]
            tool_path = Path(manifest["repository"]["root"]) / tool["relative_path"]
            original = tool_path.read_bytes()
            replacement = b"X" if original[:1] != b"X" else b"Y"
            tool_path.write_bytes(replacement + original[1:])

            with self.assertRaisesRegex(RuntimeError, "profile tool SHA-256"):
                summarize_profile(summary_path)

    def test_rejects_incomplete_profile_tool_closure(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["repository"]["profile_tool_files"].pop()
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "exact profile tool closure"):
                summarize_profile(summary_path)

    def test_rejects_noncanonical_profile_tool_path(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            first_tool = manifest["repository"]["profile_tool_files"][0]
            first_tool["relative_path"] = first_tool["relative_path"].replace(
                "tools/", "tools//", 1
            )
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "unsafe profile tool path"):
                summarize_profile(summary_path)

    def test_summarizes_complete_managed_build_provenance_schema_two(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            analysis = summarize_profile(summary_path)

        self.assertEqual(5, analysis["modes"]["cold"]["sample_count"])
        self.assertEqual(5, analysis["modes"]["warm"]["sample_count"])

    def test_marks_legacy_capture_unqualified_for_baseline_comparison(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            analysis = summarize_profile(summary_path)

        self.assertFalse(
            analysis["performance_qualification"]["cross_machine_baseline_eligible"]
        )
        self.assertEqual(
            [
                "cache_contract_legacy_unqualified",
                "machine_manifest_unavailable",
                "coordinator_comparison_receipt_missing",
            ],
            analysis["performance_qualification"]["blocking_reasons"],
        )

    def test_rejects_strict_cold_claim_when_a_cache_layer_is_uncontrolled(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["capture"] = {
                "cache_layers": {
                    "engine_cache": {"control_state": "controlled"},
                    "shader_cache": {"control_state": "uncontrolled"},
                    "os_file_cache": {"control_state": "uncontrolled"},
                    "driver_cache": {"control_state": "uncontrolled"},
                },
                "strict_cold_eligible": True,
                "comparison_scope": "process_and_caller_owned_engine_cache",
            }
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "strict cold"):
                summarize_profile(summary_path)

    def test_rejects_legacy_local_build_provenance_schema(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(
                Path(temp_dir) / "profile", managed_provenance=False
            )

            with self.assertRaisesRegex(RuntimeError, "build provenance has an unexpected schema"):
                summarize_profile(summary_path)

    def test_summarizes_writer_capture_manifest_in_a_temporary_profile_root(self):
        with tempfile.TemporaryDirectory(dir=Path.cwd()) as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            binary = self._write_file(profile_root / "zircon_shader_pbr_viewer.exe", b"viewer")
            hdri = self._write_file(profile_root / "input.hdr", b"hdri")
            profile_manifest_path = self._export_managed_profile_manifest(
                profile_root,
                Path(binary["path"]),
                Path(hdri["path"]),
            )
            summary_path = self._write_profile_summary(
                profile_root,
                profile_manifest_path=profile_manifest_path,
            )

            analysis = summarize_profile(summary_path)
            profile_manifest = json.loads(profile_manifest_path.read_text(encoding="utf-8"))
            toolchain_manifest_path = Path(
                profile_manifest["capture"]["toolchain"]["manifest"]["path"]
            )
            toolchain_manifest_path.write_text("tampered toolchain", encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "capture toolchain manifest"):
                summarize_profile(summary_path)

        self.assertEqual(5, analysis["modes"]["cold"]["sample_count"])
        self.assertEqual(5, analysis["modes"]["warm"]["sample_count"])
        self.assertEqual("scoped", analysis["cache_contract"]["status"])
        self.assertFalse(analysis["cache_contract"]["strict_cold_eligible"])
        self.assertEqual(
            "uncontrolled",
            analysis["cache_contract"]["layers"]["driver_cache"],
        )
        self.assertEqual(
            "dx12",
            analysis["cache_contract"]["toolchain"]["wgpu_backend"],
        )
        self.assertEqual(
            "wgpu(dx12)",
            analysis["cache_contract"]["toolchain"]["evidence_backend"],
        )
        self.assertIsInstance(
            analysis["cache_contract"]["machine_manifest"]["all_required_observed"],
            bool,
        )
        self.assertEqual(
            {
                "cpu",
                "gpu",
                "memory",
                "bios",
                "os",
                "display_modes",
                "power_policy",
                "thermal_frequency",
                "background_load",
                "virtualization",
            },
            set(analysis["cache_contract"]["machine_manifest"]["categories"]),
        )
        self.assertFalse(
            analysis["performance_qualification"]["cross_machine_baseline_eligible"]
        )
        self.assertIn(
            "strict_cold_cache_scope",
            analysis["performance_qualification"]["blocking_reasons"],
        )
        self.assertIn(
            "coordinator_comparison_receipt_missing",
            analysis["performance_qualification"]["blocking_reasons"],
        )

    def test_rejects_a_scoped_run_backend_that_differs_from_its_toolchain(self):
        with tempfile.TemporaryDirectory(dir=Path.cwd()) as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            binary = self._write_file(profile_root / "zircon_shader_pbr_viewer.exe", b"viewer")
            hdri = self._write_file(profile_root / "input.hdr", b"hdri")
            profile_manifest_path = self._export_managed_profile_manifest(
                profile_root,
                Path(binary["path"]),
                Path(hdri["path"]),
            )
            summary_path = self._write_profile_summary(
                profile_root,
                profile_manifest_path=profile_manifest_path,
            )
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            summary["modes"]["cold"][0]["backend"] = "wgpu(vulkan)"
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "run backend does not match its capture toolchain"):
                summarize_profile(summary_path)

    def test_rejects_a_machine_manifest_with_missing_required_categories(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            profile_manifest_path = Path(summary["profile_manifest"]["path"])
            profile_manifest = json.loads(profile_manifest_path.read_text(encoding="utf-8"))
            toolchain_manifest = self._write_file(
                profile_root / "capture-toolchain.json", b"toolchain"
            )
            profile_manifest["capture"] = {
                "cache_layers": {
                    "engine_cache": {"control_state": "controlled"},
                    "shader_cache": {"control_state": "uncontrolled"},
                    "os_file_cache": {"control_state": "uncontrolled"},
                    "driver_cache": {"control_state": "uncontrolled"},
                },
                "strict_cold_eligible": False,
                "comparison_scope": "process_and_caller_owned_engine_cache",
                "toolchain": {
                    "manifest": toolchain_manifest,
                    "graphics": {"wgpu_backend": "dx12", "evidence_backend": "wgpu(dx12)"},
                },
                "machine_manifest": {
                    "schema_version": 1,
                    "manifest_kind": "zircon_performance_machine_snapshot",
                    "captured_utc": "2026-08-24T00:00:00+00:00",
                    "required_categories": ["cpu"],
                    "all_required_observed": True,
                    "cpu": {"status": "captured"},
                },
            }
            profile_manifest_path.write_text(json.dumps(profile_manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(profile_manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "machine manifest required categories"):
                summarize_profile(summary_path)

    def test_reports_bound_pmrem_layout_and_staging_phase_medians(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            analysis = summarize_profile(summary_path)

            self.assertEqual(
                {
                    "requested_source_face_size": 64,
                    "requested_pmrem_face_size": 64,
                },
                analysis["requested_layout"],
            )
            self.assertEqual(
                {
                    "source_cubemap_face_size": 64,
                    "source_cubemap_mip_count": 7,
                    "pmrem_face_size": 64,
                    "pmrem_mip_count": 7,
                },
                analysis["modes"]["cold"]["active_layout"],
            )
            self.assertEqual(
                3,
                analysis["modes"]["cold"]["ibl_staging_median_ns"]["pmrem_build"],
            )
            self.assertEqual(
                0,
                analysis["modes"]["warm"]["ibl_staging_median_ns"]["pmrem_build"],
            )
            self.assertEqual(
                12,
                analysis["modes"]["cold"]["ibl_staging_parallel_work_item_median"][
                    "pmrem_build"
                ],
            )
            self.assertEqual(
                0,
                analysis["modes"]["warm"]["ibl_staging_parallel_work_item_median"][
                    "pmrem_build"
                ],
            )
            self.assertEqual(
                5,
                analysis["modes"]["cold"]["ibl_post_stage_hydration_median_ns"],
            )
            self.assertEqual(
                1,
                analysis["modes"]["warm"]["ibl_post_stage_hydration_median_ns"],
            )

    def test_rejects_parallel_work_total_that_does_not_match_phase_attribution(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            report = summary["modes"]["cold"][0]
            sidecar_path = Path(report["artifacts"]["ready_sidecar"]["path"])
            sidecar_path.write_text(
                sidecar_path.read_text(encoding="utf-8").replace(
                    "ibl_staging_parallel_executor_work_items=18\n",
                    "ibl_staging_parallel_executor_work_items=17\n",
                ),
                encoding="utf-8",
            )
            report["ready_sidecar"]["ibl_staging_parallel_executor_work_items"] = "17"
            report["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "parallel-work total"):
                summarize_profile(summary_path)

    def test_rejects_ibl_total_shorter_than_staging(self):
        with self.assertRaisesRegex(RuntimeError, "total elapsed is shorter than staging"):
            _post_stage_hydration_elapsed_ns(
                {
                    "ibl_staging_elapsed_ns": "10",
                    "ibl_total_elapsed_ns": "9",
                },
                "cold",
                Path("profile_summary.json"),
            )

    def test_measures_hydration_before_aggregating_cold_runs(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            for report, (staging_ns, total_ns) in zip(
                summary["modes"]["cold"],
                ((10, 10), (10, 110), (10, 110), (110, 110), (110, 110)),
                strict=True,
            ):
                sidecar_path = Path(report["artifacts"]["ready_sidecar"]["path"])
                sidecar_path.write_text(
                    sidecar_path.read_text(encoding="utf-8")
                    .replace("ibl_staging_elapsed_ns=10\n", f"ibl_staging_elapsed_ns={staging_ns}\n")
                    .replace("ibl_total_elapsed_ns=15\n", f"ibl_total_elapsed_ns={total_ns}\n"),
                    encoding="utf-8",
                )
                report["ready_sidecar"]["ibl_staging_elapsed_ns"] = str(staging_ns)
                report["ready_sidecar"]["ibl_total_elapsed_ns"] = str(total_ns)
                report["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            analysis = summarize_profile(summary_path)

            self.assertEqual(
                0,
                analysis["modes"]["cold"]["ibl_post_stage_hydration_median_ns"],
            )

    def test_rejects_wrong_cache_status_and_duplicate_ordinal(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            duplicate_summary = json.loads(json.dumps(summary))
            duplicate_summary["modes"]["cold"][1] = json.loads(
                json.dumps(duplicate_summary["modes"]["cold"][0])
            )
            summary_path.write_text(json.dumps(duplicate_summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "duplicate measured ordinal"):
                summarize_profile(summary_path)

            summary["modes"]["warm"][0]["ready_sidecar"]["ibl_staging_status"] = "Written"
            summary_path.write_text(json.dumps(summary), encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "expected Reused"):
                summarize_profile(summary_path)

    def test_rejects_ready_layout_that_is_not_bound_to_the_profile_manifest(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            report = summary["modes"]["cold"][0]
            sidecar_path = Path(report["artifacts"]["ready_sidecar"]["path"])
            sidecar_path.write_text(
                sidecar_path.read_text(encoding="utf-8").replace(
                    "requested_pmrem_face_size=64\n", "requested_pmrem_face_size=128\n"
                ),
                encoding="utf-8",
            )
            report["ready_sidecar"]["requested_pmrem_face_size"] = "128"
            report["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "requested layout does not match"):
                summarize_profile(summary_path)

    def test_rejects_material_fixture_that_is_not_bound_to_the_profile_manifest(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["input"]["material_fixture"] = "dielectric-ior"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(
                RuntimeError, "material fixture does not match its profile manifest"
            ):
                summarize_profile(summary_path)

    def test_accepts_automatic_layout_when_bound_sidecars_use_automatic_labels(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["input"]["requested_source_face_size"] = None
            manifest["input"]["requested_pmrem_face_size"] = None
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            for mode_reports in summary["modes"].values():
                for report in mode_reports:
                    sidecar_path = Path(report["artifacts"]["ready_sidecar"]["path"])
                    contents = sidecar_path.read_text(encoding="utf-8")
                    contents = contents.replace(
                        "requested_source_face_size=64\n",
                        "requested_source_face_size=automatic\n",
                    ).replace(
                        "requested_pmrem_face_size=64\n",
                        "requested_pmrem_face_size=automatic\n",
                    )
                    sidecar_path.write_text(contents, encoding="utf-8")
                    report["ready_sidecar"]["requested_source_face_size"] = "automatic"
                    report["ready_sidecar"]["requested_pmrem_face_size"] = "automatic"
                    report["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            analysis = summarize_profile(summary_path)

            self.assertEqual(
                {
                    "requested_source_face_size": None,
                    "requested_pmrem_face_size": None,
                },
                analysis["requested_layout"],
            )

    def test_rejects_active_layout_divergence_between_cold_and_warm(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["input"]["requested_source_face_size"] = None
            manifest["input"]["requested_pmrem_face_size"] = None
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            for mode, mode_reports in summary["modes"].items():
                for report in mode_reports:
                    sidecar_path = Path(report["artifacts"]["ready_sidecar"]["path"])
                    contents = sidecar_path.read_text(encoding="utf-8")
                    contents = contents.replace(
                        "requested_source_face_size=64\n",
                        "requested_source_face_size=automatic\n",
                    ).replace(
                        "requested_pmrem_face_size=64\n",
                        "requested_pmrem_face_size=automatic\n",
                    )
                    report["ready_sidecar"]["requested_source_face_size"] = "automatic"
                    report["ready_sidecar"]["requested_pmrem_face_size"] = "automatic"
                    if mode == "warm":
                        contents = contents.replace(
                            "active_pmrem_face_size=64\n",
                            "active_pmrem_face_size=128\n",
                        ).replace(
                            "active_pmrem_mip_count=7\n",
                            "active_pmrem_mip_count=8\n",
                        )
                        report["ready_sidecar"]["active_pmrem_face_size"] = "128"
                        report["ready_sidecar"]["active_pmrem_mip_count"] = "8"
                    sidecar_path.write_text(contents, encoding="utf-8")
                    report["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "across cold and warm"):
                summarize_profile(summary_path)

    def test_rejects_tampered_evidence_fingerprint(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(Path(temp_dir) / "sidecar-profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            evidence_path = Path(summary["modes"]["cold"][0]["artifacts"]["gpu_timing"]["path"])
            evidence_path.write_text("tampered", encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "SHA-256 does not match"):
                summarize_profile(summary_path)

    def test_rejects_tampered_profile_identity_and_sidecar_content(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest_path.write_text("tampered", encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "profile_manifest SHA-256 does not match"):
                summarize_profile(summary_path)

            summary_path = self._write_profile_summary(Path(temp_dir) / "sidecar-profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            sidecar_path = Path(summary["modes"]["warm"][0]["artifacts"]["ready_sidecar"]["path"])
            sidecar_path.write_text("schema=tampered\n", encoding="utf-8")
            summary["modes"]["warm"][0]["artifacts"]["ready_sidecar"] = self._fingerprint(sidecar_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "Ready sidecar does not match run report"):
                summarize_profile(summary_path)

    def test_rejects_build_provenance_that_is_not_bound_to_the_profile_manifest(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            provenance_path = Path(manifest["build_provenance"]["path"])
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["repository"]["root"] = str(profile_root / "different-source-repo")
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "build provenance repository root"):
                summarize_profile(summary_path)

    def test_rejects_managed_artifact_receipt_not_bound_to_validation_ticket(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            provenance_path = Path(manifest["build_provenance"]["path"])
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["artifact_receipt"]["source_manifest_hash"] = "b" * 64
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(
                RuntimeError, "managed artifact receipt source manifest does not match"
            ):
                summarize_profile(summary_path)

    def test_rejects_managed_artifact_receipt_with_unapproved_command(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            provenance_path = Path(manifest["build_provenance"]["path"])
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["artifact_receipt"]["command"][0] = "unapproved-tool"
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "allow-listed Cargo viewer build"):
                summarize_profile(summary_path)

    def test_rejects_forged_or_nonterminal_source_validation_ticket(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            provenance_path = Path(manifest["build_provenance"]["path"])
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["source_validation_ticket"]["source_manifest_hash"] = "b" * 64
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "validation ticket source manifest hash does not match"):
                summarize_profile(summary_path)

            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["source_validation_ticket"]["source_manifest_hash"] = hashlib.sha256(
                json.dumps(
                    provenance["source_validation_ticket"]["source_manifest"],
                    sort_keys=True,
                    separators=(",", ":"),
                    ensure_ascii=True,
                ).encode("utf-8")
            ).hexdigest()
            provenance["source_validation_ticket"]["status"] = "running"
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "requires a passed coordinator validation ticket"):
                summarize_profile(summary_path)

            provenance.pop("source_validation_ticket")
            provenance["managed_receipt"] = {
                "validation_ticket_id": "a" * 32,
                "status": "passed",
            }
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "requires a mapping for source_validation_ticket"):
                summarize_profile(summary_path)

    def test_rejects_saved_ready_validation_that_does_not_match_replayed_evidence(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            validation = summary["modes"]["cold"][0]["artifacts"]["ready_validation"]
            validation_path = Path(validation["path"])
            saved = json.loads(validation_path.read_text(encoding="utf-8"))
            saved["backend"] = "Vulkan"
            validation_path.write_text(json.dumps(saved), encoding="utf-8")
            summary["modes"]["cold"][0]["artifacts"]["ready_validation"] = self._fingerprint(
                validation_path
            )
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "unexpected backend"):
                summarize_profile(summary_path)

    def test_rejects_saved_ready_validation_with_altered_replayed_metric(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            validation = summary["modes"]["cold"][0]["artifacts"]["ready_validation"]
            validation_path = Path(validation["path"])
            saved = json.loads(validation_path.read_text(encoding="utf-8"))
            saved["startup_timing_ns"]["viewer_ready_elapsed_ns"] += 1
            validation_path.write_text(json.dumps(saved), encoding="utf-8")
            summary["modes"]["cold"][0]["artifacts"]["ready_validation"] = self._fingerprint(
                validation_path
            )
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "does not match replayed evidence"):
                summarize_profile(summary_path)

    def test_keeps_energy_unavailable_without_inventing_power(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root, energy_status="unavailable")

            analysis = summarize_profile(summary_path)

            self.assertEqual("unavailable", analysis["modes"]["cold"]["energy_meter"]["status"])
            self.assertNotIn("mean_power_watts", analysis["modes"]["cold"]["energy_meter"])

    def test_accepts_utf16_typeperf_energy_csv(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            energy = summary["modes"]["cold"][0]["artifacts"]["energy_meter"]
            energy_path = Path(energy["output_path"])
            energy_path.write_text(
                "(PDH-CSV 4.0),\\\\machine\\Energy Meter(core0)\\Power\n"
                "2026-08-13 20:00:00,10\n"
                "2026-08-13 20:00:01,20\n",
                encoding="utf-16",
            )
            energy["csv_fingerprint"] = self._fingerprint(energy_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            analysis = summarize_profile(summary_path)

            self.assertEqual(15.0, analysis["modes"]["cold"]["energy_meter"]["mean_power_watts"])

    def test_rejects_incomplete_matrix_and_nonfinite_power(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            del summary["modes"]["warm"]
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "exactly cold and warm"):
                summarize_profile(summary_path)

            summary_path = self._write_profile_summary(Path(temp_dir) / "power-profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            energy = summary["modes"]["cold"][0]["artifacts"]["energy_meter"]
            energy_path = Path(energy["output_path"])
            energy_path.write_text(
                "(PDH-CSV 4.0),\\\\machine\\Energy Meter(core0)\\Power\n"
                "2026-08-13 20:00:00,nan\n"
                "2026-08-13 20:00:01,10\n",
                encoding="utf-8",
            )
            energy["csv_fingerprint"] = self._fingerprint(energy_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "non-finite"):
                summarize_profile(summary_path)

    def test_rejects_energy_csv_output_path_not_bound_to_its_fingerprint(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            energy = summary["modes"]["cold"][0]["artifacts"]["energy_meter"]
            substituted_csv = profile_root / "substituted-energy.csv"
            substituted_csv.write_text(
                "(PDH-CSV 4.0),\\\\machine\\Energy Meter(core0)\\Power\n"
                "2026-08-13 20:00:00,100\n"
                "2026-08-13 20:00:01,200\n",
                encoding="utf-8",
            )
            energy["output_path"] = str(substituted_csv)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "output path does not match"):
                summarize_profile(summary_path)

    def test_requires_wpr_cpu_attribution_and_confines_analysis_output(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            summary["modes"]["warm"][0]["artifacts"]["cpu_sampling"] = {
                "status": "not_requested",
                "etl": None,
            }
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "cannot attribute startup without WPR"):
                summarize_profile(summary_path)

            serialized = "{\"profile\": \"analysis\"}"
            with self.assertRaisesRegex(RuntimeError, "must not overwrite"):
                _write_analysis_output(summary_path, summary_path, serialized)
            with self.assertRaisesRegex(RuntimeError, "must remain under"):
                _write_analysis_output(summary_path, profile_root.parent / "outside.json", serialized)
            c_summary_path = Path(r"C:\zircon_shader_pbr_profile_test\profile_summary.json")
            with self.assertRaisesRegex(RuntimeError, "must not be written beneath C:"):
                _write_analysis_output(
                    c_summary_path,
                    c_summary_path.parent / "profile_analysis.json",
                    serialized,
                )

        with tempfile.TemporaryDirectory(dir=Path.cwd()) as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)
            allowed_output = profile_root / "profile_analysis.json"
            _write_analysis_output(summary_path, allowed_output, serialized)
            self.assertEqual(serialized + "\n", allowed_output.read_text(encoding="utf-8"))

    def _write_profile_summary(
        self,
        profile_root: Path,
        energy_status: str = "captured",
        managed_provenance: bool = True,
        profile_manifest_path: Path | None = None,
    ) -> Path:
        profile_root.mkdir(parents=True, exist_ok=True)
        binary = self._write_file(profile_root / "zircon_shader_pbr_viewer.exe", b"viewer")
        hdri = self._write_file(profile_root / "input.hdr", b"hdri")
        if profile_manifest_path is None:
            source_root = profile_root / "source-repo"
            source = self._write_file(source_root / "critical.rs", b"critical source")
            receipt_sources = {"critical.rs": source["sha256"]}
            receipt_manifest_hash = hashlib.sha256(
                json.dumps(
                    receipt_sources,
                    sort_keys=True,
                    separators=(",", ":"),
                    ensure_ascii=True,
                ).encode("utf-8")
            ).hexdigest()
            build_provenance_contents = {
                "schema_version": 2 if managed_provenance else 1,
                "provenance_kind": (
                    "zircon_managed_viewer_artifact_provenance"
                    if managed_provenance
                    else "zircon_local_viewer_capture_provenance"
                ),
                "binary": binary,
                "repository": (
                    {
                        "root": str(source_root),
                        "source_manifest": {"critical.rs": source["sha256"]},
                    }
                    if managed_provenance
                    else {
                        "git_revision": "fixture-revision",
                        "source_manifest": {"critical.rs": source["sha256"]},
                    }
                ),
                "source_validation_ticket": {
                    "validation_ticket_id": "a" * 32,
                    "status": "passed",
                    "source_manifest_hash": receipt_manifest_hash,
                    "source_manifest": receipt_sources,
                },
            }
            if managed_provenance:
                build_provenance_contents["artifact_receipt"] = {
                    "artifact_receipt_id": "f" * 32,
                    "status": "passed",
                    "artifact_kind": "shader-pbr-viewer",
                    "job_id": "c" * 32,
                    "run_id": "e" * 32,
                    "validation_ticket_id": "a" * 32,
                    "input_manifest_hash": "b" * 64,
                    "source_manifest_hash": receipt_manifest_hash,
                    "target_relative_path": "release/zircon_shader_pbr_viewer.exe",
                    "artifact_path": binary["path"],
                    "sha256": binary["sha256"],
                    "byte_length": binary["byte_length"],
                    "command_sha256": "9" * 64,
                    "command": [
                        "cargo",
                        "build",
                        "-p",
                        "zircon_app",
                        "--bin",
                        "zircon_shader_pbr_viewer",
                        "--locked",
                        "--release",
                    ],
                }
            build_provenance = self._write_file(
                profile_root / "viewer-build-provenance.json",
                json.dumps(build_provenance_contents, sort_keys=True).encode(),
            )
            profile_tools = []
            for relative_path in _PROFILE_TOOL_PATHS:
                tool = self._write_file(
                    source_root / relative_path,
                    f"fixture profile tool: {relative_path}\n".encode(),
                )
                profile_tools.append(
                    {
                        "relative_path": relative_path,
                        "sha256": tool["sha256"],
                        "byte_length": tool["byte_length"],
                    }
                )
            manifest_contents = {
                "schema_version": 1,
                "profile_kind": "zircon_shader_pbr_viewer_startup",
                "repository": {
                    "root": str(source_root),
                    "git": {"revision": "fixture-revision"},
                    "critical_source_files": [
                        {
                            "relative_path": "critical.rs",
                            "sha256": source["sha256"],
                            "byte_length": source["byte_length"],
                        }
                    ],
                    "profile_tool_files": profile_tools,
                },
                "binary": binary,
                "build_provenance": build_provenance,
                "input": {
                    "hdri": hdri,
                    "requested_source_face_size": 64,
                    "requested_pmrem_face_size": 64,
                    "material_fixture": "metal-mirror",
                },
            }
            manifest = self._write_file(
                profile_root / "profile_manifest.json",
                json.dumps(manifest_contents, sort_keys=True).encode(),
            )
        else:
            manifest = self._fingerprint(profile_manifest_path)
        identity_manifest = json.loads(Path(str(manifest["path"])).read_text(encoding="utf-8"))
        identity_provenance = json.loads(
            Path(str(identity_manifest["build_provenance"]["path"])).read_text(encoding="utf-8")
        )
        evidence_identity_sources = {
            "profile_id": "shader-pbr-test-000001",
            "binary": self._identity_fingerprint(identity_manifest["binary"]),
            "hdri": self._identity_fingerprint(identity_manifest["input"]["hdri"]),
            "build_provenance": self._identity_fingerprint(
                identity_manifest["build_provenance"]
            ),
            "source_manifest_sha256": identity_provenance["source_validation_ticket"]["source_manifest_hash"],
        }
        modes = {"cold": [], "warm": []}
        for mode, staging_status, renderer_ns, ibl_ns in (
            ("cold", "Written", 10_000_000, 2_000_000),
            ("warm", "Reused", 5_000_000, 1_000_000),
        ):
            for ordinal in range(1, 6):
                run_directory = profile_root / mode / f"measured-{ordinal:02d}"
                run_directory.mkdir(parents=True)
                modes[mode].append(
                    self._run_report(
                        run_directory,
                        mode,
                        ordinal,
                        staging_status,
                        renderer_ns,
                        ibl_ns,
                        energy_status,
                        evidence_identity_sources,
                    )
                )
        summary = {
            "schema_version": 1,
            "profile_kind": "zircon_shader_pbr_viewer_startup_matrix",
            "profile_id": evidence_identity_sources["profile_id"],
            "profile_root": str(profile_root),
            "profile_manifest": manifest,
            "repetitions_per_mode": 5,
            "source_binary": binary,
            "source_hdri": hdri,
            "modes": modes,
        }
        summary_path = profile_root / "profile_summary.json"
        summary_path.write_text(json.dumps(summary), encoding="utf-8")
        return summary_path

    def _write_completion_receipt(self, summary_path: Path) -> Path:
        profile_root = summary_path.parent.resolve()
        artifacts = []
        for path in sorted(profile_root.rglob("*")):
            if path.is_file():
                relative_path = path.relative_to(profile_root).as_posix()
                fingerprint = self._fingerprint(path)
                artifacts.append(
                    {
                        "relative_path": relative_path,
                        "sha256": fingerprint["sha256"],
                        "byte_length": fingerprint["byte_length"],
                    }
                )
        receipt = {
            "schema_version": 1,
            "receipt_kind": "zircon_shader_pbr_profile_completion",
            "status": "completed",
            "profile_id": "shader-pbr-test-000001",
            "profile_root": str(profile_root),
            "completed_utc": "2026-08-25T00:00:00Z",
            "artifacts": artifacts,
        }
        receipt_path = profile_root.parent / "profile-completion.json"
        receipt_path.write_text(json.dumps(receipt), encoding="utf-8")
        return receipt_path

    @staticmethod
    def _export_managed_profile_manifest(
        profile_root: Path,
        binary_path: Path,
        hdri_path: Path,
    ) -> Path:
        repository_root = Path(__file__).resolve().parents[2]
        fixture_script = (
            Path(__file__).resolve().parent
            / "fixtures"
            / "export-managed-shader-pbr-profile-manifest.ps1"
        )
        result = subprocess.run(
            [
                "pwsh",
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                str(fixture_script),
                "-RepoRoot",
                str(repository_root),
                "-ProfileRoot",
                str(profile_root),
                "-ViewerExe",
                str(binary_path),
                "-HdriPath",
                str(hdri_path),
            ],
            cwd=repository_root,
            capture_output=True,
            check=False,
            encoding="utf-8",
            errors="replace",
            text=True,
        )
        if result.returncode != 0:
            raise AssertionError(
                "temporary managed writer/capture fixture failed:\n"
                f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
            )
        manifest_path = Path(result.stdout.strip())
        if not manifest_path.is_file():
            raise AssertionError(
                f"temporary managed writer/capture fixture did not emit a profile manifest: {result.stdout!r}"
            )
        return manifest_path

    def _run_report(
        self,
        run_directory: Path,
        mode: str,
        ordinal: int,
        staging_status: str,
        renderer_ns: int,
        ibl_ns: int,
        energy_status: str,
        evidence_identity_sources: dict[str, object],
    ) -> dict[str, object]:
        ready_png_path = run_directory / "ready.png"
        self._write_rgba_png(
            ready_png_path,
            2,
            2,
            [(18, 26, 48, 255), (160, 112, 64, 255), (64, 144, 196, 255), (236, 232, 212, 255)],
        )
        ready_png = self._fingerprint(ready_png_path)
        staging_elapsed_ns = 10 if staging_status == "Written" else 0
        hydration_elapsed_ns = 5 if staging_status == "Written" else 1
        equirect_projection_work_items = 6 if staging_status == "Written" else 0
        source_mip_build_work_items = 0
        pmrem_build_work_items = 12 if staging_status == "Written" else 0
        irradiance_cube_build_work_items = 0
        staging_work_items = (
            equirect_projection_work_items
            + source_mip_build_work_items
            + pmrem_build_work_items
            + irradiance_cube_build_work_items
        )
        source_sample_visits = 37 if staging_status == "Written" else 0
        profile_id = str(evidence_identity_sources["profile_id"])
        source_manifest_sha256 = str(evidence_identity_sources["source_manifest_sha256"])
        evidence_run_id = f"{profile_id}-{mode}-measured-{ordinal:02d}"
        identity_payload = {
            "schema": "zircon_shader_pbr_viewer_evidence_identity_v1",
            "run_id": evidence_run_id,
            "validation_policy": "zircon_shader_pbr_viewer_ready_frame_v17",
            "source_manifest_sha256": source_manifest_sha256,
            "viewer_binary": evidence_identity_sources["binary"],
            "hdri": evidence_identity_sources["hdri"],
            "build_provenance": evidence_identity_sources["build_provenance"],
        }
        evidence_identity = self._write_file(
            run_directory / "evidence_identity.json",
            json.dumps(identity_payload, sort_keys=True).encode(),
        )
        sidecar_fields = {
            "schema": "zircon_shader_pbr_viewer_ready_frame_evidence_v17",
            "screenshot_sha256": str(ready_png["sha256"]),
            "screenshot_byte_length": str(ready_png["byte_length"]),
            "evidence_identity_schema": "zircon_shader_pbr_viewer_evidence_identity_v1",
            "evidence_run_id": evidence_run_id,
            "evidence_validation_policy": "zircon_shader_pbr_viewer_ready_frame_v17",
            "evidence_identity_path": str(evidence_identity["path"]),
            "evidence_identity_sha256": str(evidence_identity["sha256"]),
            "evidence_identity_byte_length": str(evidence_identity["byte_length"]),
            "viewer_binary_path": str(evidence_identity_sources["binary"]["path"]),
            "viewer_binary_sha256": str(evidence_identity_sources["binary"]["sha256"]),
            "viewer_binary_byte_length": str(evidence_identity_sources["binary"]["byte_length"]),
            "hdri_sha256": str(evidence_identity_sources["hdri"]["sha256"]),
            "hdri_byte_length": str(evidence_identity_sources["hdri"]["byte_length"]),
            "build_provenance_path": str(evidence_identity_sources["build_provenance"]["path"]),
            "build_provenance_sha256": str(evidence_identity_sources["build_provenance"]["sha256"]),
            "build_provenance_byte_length": str(evidence_identity_sources["build_provenance"]["byte_length"]),
            "source_manifest_sha256": source_manifest_sha256,
            "screenshot": ready_png_path.name,
            "screenshot_presentation": "cpu_readback",
            "interactive_direct_present_enabled": "false",
            "host_mode": "offscreen-diagnostic",
            "host_composition_id": "zircon_shader_pbr_viewer_standalone_diagnostic_v1",
            "scene_id": "single_pbr_mirror_sphere",
            "capture_target": "offscreen-scene-renderer-cpu-readback",
            "gpu_scene_surface_present_count": "0",
            "backend": "wgpu(dx12)",
            "hdri_path": "input.hdr",
            "requested_source_face_size": "64",
            "requested_pmrem_face_size": "64",
            "active_source_cubemap_face_size": "64",
            "active_source_cubemap_mip_count": "7",
            "active_pmrem_face_size": "64",
            "active_pmrem_mip_count": "7",
            "render_profile": "environment_only_pbr_preview",
            "material_fixture": "metal-mirror",
            "required_material_base_pipeline_kind": "environment-only-pbr-base",
            "required_material_base_pipeline_ready_at_capture": "true",
            "environment_only_base_prewarm_requested": "true",
            "environment_only_base_prewarm_cache_hit": "false",
            "environment_only_base_prewarm_cache_scope": "process_local_mesh_pipeline_cache",
            "environment_only_base_prewarm_shader_source_resolution_ns": "0",
            "environment_only_base_prewarm_pipeline_creation_ns": "0",
            "environment_only_base_prewarm_elapsed_ns": "0",
            "viewport": "2x2",
            "camera_yaw_degrees": "0",
            "camera_pitch_degrees": "0",
            "ibl_bake_algorithm_version": _CURRENT_IBL_BAKE_ALGORITHM_VERSION,
            "ibl_staging_status": staging_status,
            "ibl_staging_elapsed_ns": str(staging_elapsed_ns),
            "ibl_total_elapsed_ns": str(staging_elapsed_ns + hydration_elapsed_ns),
            "ready_frame_render_elapsed_ns": "1",
            "ready_frame_extract_ns": "0",
            "ready_frame_renderer_call_ns": "1",
            "ready_frame_readback_and_completion_ns": "0",
            "environment_only_base_prewarm_pipeline_ready": "true",
            "environment_only_base_pipeline_ready_at_capture": "true",
            "scene_startup_hdri_decode_ns": "0",
            "scene_startup_project_assets_ns": "0",
            "scene_startup_runtime_bootstrap_ns": "0",
            "scene_startup_project_open_ns": "0",
            "scene_startup_world_load_ns": "0",
            "scene_startup_renderer_initialization_ns": str(renderer_ns),
            "scene_startup_renderer_backend_initialization_ns": "0",
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialized": "true",
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_cache_wait_ns": "3",
            "scene_startup_renderer_environment_brdf_lut_builtin_payload_materialization_ns": "2",
            "scene_startup_renderer_environment_brdf_lut_texture_upload_submission_ns": "1",
            "scene_startup_renderer_deferred_initialization_ns": str(renderer_ns // 2),
            "scene_startup_renderer_deferred_standard_pipeline_ns": str(renderer_ns // 2),
            "scene_startup_resource_streamer_initialization_ns": "0",
            "scene_startup_ibl_restore_ns": str(ibl_ns),
            "scene_startup_total_ns": str(renderer_ns + ibl_ns),
            "one_shot_base_pipeline_wait_elapsed_ns": "0",
            "viewer_scene_load_elapsed_ns": str(renderer_ns + ibl_ns),
            "viewer_ready_elapsed_ns": str(renderer_ns + ibl_ns + 1),
            "ibl_staging_source_decode_ns": "0",
            "ibl_staging_cubemap_build_ns": str(staging_elapsed_ns),
            "ibl_staging_equirect_projection_ns": "4" if staging_status == "Written" else "0",
            "ibl_staging_source_mip_build_ns": "2" if staging_status == "Written" else "0",
            "ibl_staging_pmrem_build_ns": "3" if staging_status == "Written" else "0",
            "ibl_staging_sh9_build_ns": "1" if staging_status == "Written" else "0",
            "ibl_staging_irradiance_cube_build_ns": "0",
            "ibl_staging_bundle_write_ns": "0",
            "ibl_staging_source_zcube_bytes": "1024",
            "ibl_staging_asset_derived_bytes": "2048",
            "ibl_staging_parallel_executor_work_items": str(staging_work_items),
            "ibl_staging_equirect_projection_parallel_work_items": str(
                equirect_projection_work_items
            ),
            "ibl_staging_source_mip_build_parallel_work_items": str(
                source_mip_build_work_items
            ),
            "ibl_staging_pmrem_build_parallel_work_items": str(pmrem_build_work_items),
            "ibl_staging_irradiance_cube_build_parallel_work_items": str(
                irradiance_cube_build_work_items
            ),
            "ibl_staging_irradiance_cube_source_sample_visits": str(source_sample_visits),
            "registered_pipeline_variant_count": "16",
            "registered_shader_variant_count": "1",
            "texture_presence_normalized_pipeline_variant_count": "1",
            "texture_presence_equivalent_pipeline_variant_count": "15",
            "cached_render_pipeline_count": "8",
            "cached_shader_module_count": "2",
            "render_pipeline_creation_count": "8",
            "render_pipeline_creation_cpu_microseconds": "3000",
            "shader_module_creation_count": "2",
            "shader_module_creation_cpu_microseconds": "500",
            "async_base_pipeline_queue_wait_count": "1",
            "async_base_pipeline_queue_wait_microseconds": "100",
        }
        sidecar = self._write_file(
            run_directory / "ready.png.txt",
            "".join(f"{key}={value}\n" for key, value in sidecar_fields.items()).encode(),
        )
        ready_validation = self._write_file(
            run_directory / "ready_validation.json",
            json.dumps(
                ready_frame_evidence_summary(
                    validate_current_ready_frame_evidence(
                        ready_png_path,
                        expected_backend="wgpu(dx12)",
                    )
                ),
                sort_keys=True,
            ).encode(),
        )
        screenshot_sha256 = ready_png["sha256"]
        timing = self._write_file(
            run_directory / "gpu_timing.txt",
            _gpu_timing_distribution(screenshot_sha256).encode(),
        )
        runtime_profile = self._write_runtime_profile(
            run_directory,
            evidence_run_id,
            mode,
            ordinal,
        )
        etl = self._write_file(run_directory / "cpu_sampling.etl", b"etl")
        energy_path = run_directory / "energy_meter.csv"
        if energy_status == "captured":
            with energy_path.open("w", newline="", encoding="utf-8") as energy_file:
                writer = csv.writer(energy_file)
                writer.writerow(["(PDH-CSV 4.0)", r"\\machine\Energy Meter(core0)\Power", r"\\machine\Energy Meter(core1)\Power"])
                writer.writerow(["2026-08-13 20:00:00", "7", "8"])
                writer.writerow(["2026-08-13 20:00:01", "9", "6"])
            energy = {
                "status": "captured",
                "output_path": str(energy_path),
                "counter_paths": [r"\Energy Meter(core0)\Power", r"\Energy Meter(core1)\Power"],
                "counter_units": [{"counter_suffix": "Power", "unit": "watts"}],
                "sample_interval_seconds": 1,
                "csv_fingerprint": self._fingerprint(energy_path),
            }
        else:
            energy = {
                "status": "unavailable",
                "output_path": str(energy_path),
                "counter_paths": [],
                "counter_units": [],
                "sample_interval_seconds": 1,
            }
        return {
            "schema_version": 1,
            "profile_kind": "zircon_shader_pbr_viewer_startup_run",
            "mode": mode,
            "role": "measured",
            "ordinal": ordinal,
            "expected_ibl_staging_status": staging_status,
            "backend": "wgpu(dx12)",
            "ready_sidecar": sidecar_fields,
            "artifacts": {
                "ready_png": ready_png,
                "ready_sidecar": sidecar,
                "evidence_identity": evidence_identity,
                "ready_validation": ready_validation,
                "gpu_timing": timing,
                "runtime_profile": runtime_profile,
                "cpu_sampling": {"status": "captured", "etl": etl},
                "energy_meter": energy,
                "renderdoc_capture": None,
            },
        }

    def _write_runtime_profile(
        self,
        run_directory: Path,
        session_id: str,
        mode: str,
        ordinal: int,
    ) -> dict[str, object]:
        stages = (
            "material_requirement_admission",
            "mesh_source_build",
            "module_include_resolution",
            "template_assembly",
            "source_hash",
            "naga_validation",
            "disk_cache_lookup",
            "disk_cache_write",
            "wgpu_pipeline_error_scope_pop",
        )
        output_root = run_directory / "runtime-profile"
        export_root = output_root / session_id
        spans = []
        duration_step = 100 if mode == "cold" else 50
        for stage_index, stage in enumerate(stages):
            spans.append(
                {
                    "id": len(spans) + 1,
                    "parent_id": None,
                    "frame_index": None,
                    "stream": "render",
                    "category": "shader_pipeline",
                    "name": stage,
                    "path": f"shader_pipeline/{stage}",
                    "start_us": len(spans) * 1_000,
                    "duration_us": ordinal * duration_step + stage_index,
                    "depth": 0,
                }
            )
        spans.append(
            {
                "id": len(spans) + 1,
                "parent_id": None,
                "frame_index": None,
                "stream": "render",
                "category": "shader_pipeline",
                "name": "mesh_source_build",
                "path": "shader_pipeline/mesh_source_build/repeated",
                "start_us": len(spans) * 1_000,
                "duration_us": ordinal,
                "depth": 0,
            }
        )
        counters = [
            {
                "stream": "render",
                "name": "mesh_shader_source_bytes",
                "value": 42,
                "timestamp_us": 1,
                "frame_index": None,
            }
        ]
        retention = [
            {
                "frames": {
                    "capacity": 4_096,
                    "written": 0,
                    "overwritten": 0,
                    "retained": 0,
                    "oldest_sequence": None,
                    "newest_sequence": None,
                },
                "spans": {
                    "capacity": 262_144,
                    "written": len(spans),
                    "overwritten": 0,
                    "retained": len(spans),
                    "oldest_sequence": 0,
                    "newest_sequence": len(spans) - 1,
                },
                "counters": {
                    "capacity": 262_144,
                    "written": len(counters),
                    "overwritten": 0,
                    "retained": len(counters),
                    "oldest_sequence": 0,
                    "newest_sequence": len(counters) - 1,
                },
            }
        ]
        timeline = {
            "session_id": session_id,
            "output_root": str(output_root),
            "active": False,
            "feature_enabled": True,
            "frame_budget_ms": 16.67,
            "frames": [],
            "spans": spans,
            "counters": counters,
            "recorder_retention": retention,
        }
        timeline_artifact = self._write_file(
            export_root / "timeline.zrtrace.json",
            json.dumps(timeline, sort_keys=True).encode(),
        )
        artifacts = {
            "timeline": timeline_artifact,
            "hotspots": self._write_file(export_root / "hotspots.json", b"{}"),
            "counter_hotspots": self._write_file(
                export_root / "counter_hotspots.json", b"{}"
            ),
            "summary": self._write_file(export_root / "summary.md", b"# fixture\n"),
        }
        stage_counts = {
            stage: sum(
                1
                for span in spans
                if span["category"] == "shader_pipeline" and span["name"] == stage
            )
            for stage in stages
        }
        return {
            "schema": "zircon_shader_pbr_runtime_profile_v1",
            "session_id": session_id,
            "output_root": str(output_root),
            "span_count": len(spans),
            "counter_count": len(counters),
            "recorder_retention": retention,
            "shader_pipeline_stage_counts": stage_counts,
            "artifacts": artifacts,
        }

    @staticmethod
    def _write_file(path: Path, contents: bytes) -> dict[str, object]:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(contents)
        return {
            "path": str(path),
            "sha256": hashlib.sha256(contents).hexdigest(),
            "byte_length": len(contents),
        }

    @staticmethod
    def _fingerprint(path: Path) -> dict[str, object]:
        contents = path.read_bytes()
        return {
            "path": str(path),
            "sha256": hashlib.sha256(contents).hexdigest(),
            "byte_length": len(contents),
        }

    @staticmethod
    def _identity_fingerprint(fingerprint: dict[str, object]) -> dict[str, object]:
        return {
            field: fingerprint[field] for field in ("path", "sha256", "byte_length")
        }

    @staticmethod
    def _write_rgba_png(
        path: Path,
        width: int,
        height: int,
        pixels: list[tuple[int, int, int, int]],
    ) -> None:
        if len(pixels) != width * height:
            raise ValueError("fixture pixels must match PNG dimensions")
        rows = []
        for row_index in range(height):
            row = pixels[row_index * width : (row_index + 1) * width]
            rows.append(b"\0" + b"".join(bytes(pixel) for pixel in row))
        compressed = zlib.compress(b"".join(rows))
        path.write_bytes(
            b"\x89PNG\r\n\x1a\n"
            + ZirconSummarizeShaderPbrProfileTests._png_chunk(
                b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
            )
            + ZirconSummarizeShaderPbrProfileTests._png_chunk(b"IDAT", compressed)
            + ZirconSummarizeShaderPbrProfileTests._png_chunk(b"IEND", b"")
        )

    @staticmethod
    def _png_chunk(kind: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )


def _gpu_timing_distribution(screenshot_sha256: str) -> str:
    pass_values = {
        "direct_gpu_scene_upload": 0,
        "direct_scene_content": 10,
        "direct_output_transfer": 2,
        "direct_overlays": 1,
    }
    pass_names = sorted(pass_values)
    lines = [
        "schema=zircon_shader_pbr_viewer_gpu_timing_evidence_v3",
        "status=measured",
        "screenshot=ready.png",
        f"screenshot_sha256={screenshot_sha256}",
        "screenshot_frame_generation=1",
        "warmup_sample_count=5",
        "warmup_first_frame_generation=2",
        "warmup_last_frame_generation=6",
        "measured_sample_count=31",
        "first_measured_frame_generation=7",
        "last_measured_frame_generation=37",
        "timestamp_period_ns_bits=1065353216",
        "timestamp_period_ns=1.000000000",
        "timestamp_frequency_hz=1000000000.000",
        "percentile_policy=nearest_rank",
        "outlier_policy=none_all_samples_retained",
        f"pass_coverage={','.join(pass_names)}",
        "total.min_us=13",
        "total.median_us=13",
        "total.p95_us=13",
        "total.max_us=13",
    ]
    for pass_name in pass_names:
        value = pass_values[pass_name]
        lines.extend(
            [
                f"pass.{pass_name}.min_us={value}",
                f"pass.{pass_name}.median_us={value}",
                f"pass.{pass_name}.p95_us={value}",
                f"pass.{pass_name}.max_us={value}",
            ]
        )
    for index, generation in enumerate(range(7, 38)):
        lines.append(f"sample.{index:03}.frame_generation={generation}")
        lines.append("sample.{:03}.total_us=13".format(index))
        for pass_name in pass_names:
            lines.append(
                f"sample.{index:03}.pass.{pass_name}_us={pass_values[pass_name]}"
            )
        lines.extend(
            [
                f"sample.{index:03}.mesh.opaque_command_count=1",
                f"sample.{index:03}.mesh.advanced_pbr_opaque_command_count=0",
                f"sample.{index:03}.mesh.cached_command_hit_count=1",
                f"sample.{index:03}.mesh.command_rebuild_count=0",
                f"sample.{index:03}.mesh.dynamic_command_count=0",
            ]
        )
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    unittest.main()
