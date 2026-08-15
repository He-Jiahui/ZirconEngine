import csv
import hashlib
import json
import struct
import tempfile
import unittest
import zlib
from pathlib import Path
from unittest import mock

from tools.zircon_summarize_shader_pbr_profile import (
    _post_stage_hydration_elapsed_ns,
    _write_analysis_output,
    summarize_profile,
)
from tools.zircon_validate_shader_pbr_viewer_evidence import (
    ready_frame_evidence_summary,
    validate_current_ready_frame_evidence,
)


class ZirconSummarizeShaderPbrProfileTests(unittest.TestCase):
    def test_replays_every_ready_frame_in_process(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")

            with mock.patch(
                "tools.zircon_summarize_shader_pbr_profile.validate_current_ready_frame_evidence",
                wraps=validate_current_ready_frame_evidence,
            ) as validate_current:
                summarize_profile(summary_path)

            self.assertEqual(10, validate_current.call_count)

    def test_summarizes_five_run_cold_warm_matrix_with_bound_energy_data(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            profile_root = Path(temp_dir) / "profile"
            summary_path = self._write_profile_summary(profile_root)

            analysis = summarize_profile(summary_path)

            self.assertEqual(5, analysis["modes"]["cold"]["sample_count"])
            self.assertEqual(5, analysis["modes"]["warm"]["sample_count"])
            self.assertEqual("renderer_initialization", analysis["modes"]["cold"]["bottleneck"])
            self.assertEqual(15.0, analysis["modes"]["cold"]["energy_meter"]["mean_power_watts"])
            self.assertEqual("meter_instance_sum", analysis["modes"]["cold"]["energy_meter"]["scope"])
        self.assertTrue(analysis["modes"]["cold"]["cpu_sampling"]["attribution_ready"])
        self.assertEqual(10, analysis["modes"]["warm"]["gpu_pass_median_us"]["direct_scene_content"])

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
            summary["modes"]["cold"][1]["ordinal"] = 1
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "duplicate measured ordinal"):
                summarize_profile(summary_path)

            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            summary["modes"]["cold"][1]["ordinal"] = 2
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
            summary_path = self._write_profile_summary(Path(temp_dir) / "profile")
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            manifest_path = Path(summary["profile_manifest"]["path"])
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            provenance_path = Path(manifest["build_provenance"]["path"])
            provenance = json.loads(provenance_path.read_text(encoding="utf-8"))
            provenance["repository"]["git_revision"] = "stale-revision"
            provenance_path.write_text(json.dumps(provenance), encoding="utf-8")
            manifest["build_provenance"] = self._fingerprint(provenance_path)
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            summary["profile_manifest"] = self._fingerprint(manifest_path)
            summary_path.write_text(json.dumps(summary), encoding="utf-8")

            with self.assertRaisesRegex(RuntimeError, "build provenance Git revision"):
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

    def _write_profile_summary(self, profile_root: Path, energy_status: str = "captured") -> Path:
        profile_root.mkdir(parents=True, exist_ok=True)
        binary = self._write_file(profile_root / "zircon_shader_pbr_viewer.exe", b"viewer")
        hdri = self._write_file(profile_root / "input.hdr", b"hdri")
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
            "schema_version": 1,
            "provenance_kind": "zircon_local_viewer_capture_provenance",
            "binary": binary,
            "repository": {
                "git_revision": "fixture-revision",
                "source_manifest": {"critical.rs": source["sha256"]},
            },
            "source_validation_ticket": {
                "validation_ticket_id": "a" * 32,
                "status": "passed",
                "source_manifest_hash": receipt_manifest_hash,
                "source_manifest": receipt_sources,
            },
        }
        build_provenance = self._write_file(
            profile_root / "viewer-build-provenance.json",
            json.dumps(build_provenance_contents, sort_keys=True).encode(),
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
            },
            "binary": binary,
            "build_provenance": build_provenance,
            "input": {
                "hdri": hdri,
                "requested_source_face_size": 64,
                "requested_pmrem_face_size": 64,
            },
        }
        manifest = self._write_file(
            profile_root / "profile_manifest.json",
            json.dumps(manifest_contents, sort_keys=True).encode(),
        )
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
                    )
                )
        summary = {
            "schema_version": 1,
            "profile_kind": "zircon_shader_pbr_viewer_startup_matrix",
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

    def _run_report(
        self,
        run_directory: Path,
        mode: str,
        ordinal: int,
        staging_status: str,
        renderer_ns: int,
        ibl_ns: int,
        energy_status: str,
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
        sidecar_fields = {
            "schema": "zircon_shader_pbr_viewer_ready_frame_evidence_v12",
            "screenshot": ready_png_path.name,
            "screenshot_presentation": "cpu_readback",
            "interactive_direct_present_enabled": "false",
            "backend": "Dx12",
            "hdri_path": "input.hdr",
            "requested_source_face_size": "64",
            "requested_pmrem_face_size": "64",
            "active_source_cubemap_face_size": "64",
            "active_source_cubemap_mip_count": "7",
            "active_pmrem_face_size": "64",
            "active_pmrem_mip_count": "7",
            "render_profile": "environment_only_pbr_preview",
            "environment_only_base_prewarm_cache_hit": "false",
            "environment_only_base_prewarm_cache_scope": "process_local_mesh_pipeline_cache",
            "environment_only_base_prewarm_shader_source_resolution_ns": "0",
            "environment_only_base_prewarm_pipeline_creation_ns": "0",
            "environment_only_base_prewarm_elapsed_ns": "0",
            "viewport": "2x2",
            "camera_yaw_degrees": "0",
            "camera_pitch_degrees": "0",
            "ibl_bake_algorithm_version": "202608090006",
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
                        expected_backend="Dx12",
                    )
                ),
                sort_keys=True,
            ).encode(),
        )
        screenshot_sha256 = ready_png["sha256"]
        timing = self._write_file(
            run_directory / "gpu_timing.txt",
            (
                "schema=zircon_shader_pbr_viewer_gpu_timing_evidence_v1\n"
                "status=measured\n"
                "screenshot=ready.png\n"
                f"screenshot_sha256={screenshot_sha256}\n"
                "frame_generation=1\n"
                "pass.direct_gpu_scene_upload=0\n"
                "pass.direct_scene_content=10\n"
                "pass.direct_output_transfer=2\n"
                "pass.direct_overlays=1\n"
            ).encode(),
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
            "ready_sidecar": sidecar_fields,
            "artifacts": {
                "ready_png": ready_png,
                "ready_sidecar": sidecar,
                "ready_validation": ready_validation,
                "gpu_timing": timing,
                "cpu_sampling": {"status": "captured", "etl": etl},
                "energy_meter": energy,
                "renderdoc_capture": None,
            },
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


if __name__ == "__main__":
    unittest.main()
