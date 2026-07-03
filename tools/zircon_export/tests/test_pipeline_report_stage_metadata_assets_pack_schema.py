from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pipeline_report_stage_metadata_test_support import (
    write_library_embed_reports,
)


class PipelineReportStageMetadataAssetsPackSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_cook_assets_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            cook_report["unsigned_sidecar"] = "sidecar.bin"
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CookAssets", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_cook_assets_string_fields_non_string(self) -> None:
        for field in (
            "asset_filter",
            "cooked_asset_manifest",
            "cooked_asset_manifest_sha256",
            "project_default_scene",
            "project_manifest",
            "source_asset_manifest",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = out / "stages" / "cook_assets" / "report.json"
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    cook_report[field] = 42
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CookAssets", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"cook_assets report {field} must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_count_fields_non_integer(self) -> None:
        for field in ("asset_count", "root_count"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = out / "stages" / "cook_assets" / "report.json"
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    cook_report[field] = "1"
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CookAssets", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"cook_assets report {field} must be an integer"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_generated_from_project_non_bool(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            write_library_embed_reports(out)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            cook_report["generated_from_project"] = "true"
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CookAssets", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report generated_from_project must be a boolean"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_cook_assets_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "cooked_asset_manifest",
                "cook_assets report cooked_asset_manifest must be a string",
            ),
            (
                "cooked_asset_manifest_sha256",
                "cook_assets report cooked_asset_manifest_sha256 must be a string",
            ),
            ("asset_count", "cook_assets report asset_count must be an integer"),
            ("root_count", "cook_assets report root_count must be an integer"),
            (
                "generated_from_project",
                "cook_assets report generated_from_project must be a boolean",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = out / "stages" / "cook_assets" / "report.json"
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    cook_report.pop(field)
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CookAssets", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_manifest_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            write_library_embed_reports(out)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            cook_report["cooked_asset_manifest_sha256"] = "0" * 64
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report cooked_asset_manifest" in diagnostic
                    and "does not match report cooked_asset_manifest_sha256"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_asset_manifest_not_from_cook_assets(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            write_library_embed_reports(out)
            stale_manifest = out / "stale" / "assets.json"
            stale_manifest.parent.mkdir(parents=True)
            stale_manifest.write_text(
                json.dumps({"roots": [], "assets": []}, indent=2),
                encoding="utf-8",
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["asset_manifest"] = str(stale_manifest)
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report asset_manifest" in diagnostic
                    and "does not match cook_assets report cooked_asset_manifest"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_cook_assets_manifest_count_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "asset_count",
                99,
                "cook_assets report asset_count 99 does not match "
                "cooked_asset_manifest assets length 0",
            ),
            (
                "root_count",
                99,
                "cook_assets report root_count 99 does not match "
                "cooked_asset_manifest roots length 0",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = out / "stages" / "cook_assets" / "report.json"
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    cook_report[field] = value
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["unsigned_sidecar"] = "sidecar.bin"
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("Pack", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report unknown field unsigned_sidecar" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_string_fields_non_string(self) -> None:
        for field in (
            "asset_manifest",
            "delta_pack",
            "pack",
            "previous_pack",
            "stage_output",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = 42
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"pack report {field} must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_count_fields_non_integer(self) -> None:
        for field in (
            "asset_count",
            "chunk_count",
            "delta_asset_count",
            "delta_chunk_count",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = "1"
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"pack report {field} must be an integer"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_string_array_fields_non_string_array(
        self,
    ) -> None:
        for field in (
            "deduplicated_assets",
            "delta_removed_assets",
            "delta_reused_assets",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = ["textures/albedo.png", 42]
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    expected_diagnostic = f"pack report {field}[1] must be a string"
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"pack report {field} must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_bool_fields_non_bool(self) -> None:
        for field in ("delta_apply_verified", "deterministic_double_run"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = "true"
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"pack report {field} must be a boolean"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_object_fields_non_object(self) -> None:
        for field in ("delta_manifest", "manifest", "trim_report"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report[field] = "not-an-object"
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"pack report {field} must be an object"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_pack_missing_release_evidence_field(self) -> None:
        cases = (
            ("asset_manifest", "pack report asset_manifest must be a string"),
            ("pack", "pack report pack must be a string"),
            ("stage_output", "pack report stage_output must be a string"),
            ("asset_count", "pack report asset_count must be an integer"),
            ("chunk_count", "pack report chunk_count must be an integer"),
            (
                "deduplicated_assets",
                "pack report deduplicated_assets must be a string array",
            ),
            (
                "deterministic_double_run",
                "pack report deterministic_double_run must be a boolean",
            ),
            ("trim_report", "pack report trim_report must be an object"),
            ("manifest", "pack report manifest must be an object"),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report.pop(field)
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_stage_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report_path.unlink()
            pack_report_path.mkdir()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("pack", report["missing_stages"])
            self.assertTrue(
                any(
                    "pack report" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_invalid_validate_metadata_without_defaulting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["diagnostics"] = "not-a-list"
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertTrue(
                any(
                    "validate report diagnostics must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()