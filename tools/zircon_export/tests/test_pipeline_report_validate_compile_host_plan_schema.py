from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateCompileHostPlanSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_compile_host_plan_non_object(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["plan_summary"] = {
                "library_embed_compile_host": "not-an-object"
            }
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
                    "validate report plan_summary.library_embed_compile_host must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_compile_host_plan_unknown_field(
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
            compile_host_plan = _compile_host_plan()
            compile_host_plan["unsigned_sidecar"] = "sidecar.bin"
            validate_report["plan_summary"] = {
                "library_embed_compile_host": compile_host_plan
            }
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "validate report plan_summary.library_embed_compile_host unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_compile_host_plan_string_fields_non_string(
        self,
    ) -> None:
        compile_host_string_fields = (
            "binary",
            "cargo_profile",
            "manifest_path",
            "package",
            "target_dir",
        )
        for field in compile_host_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan[field] = 42
                    validate_report["plan_summary"] = {
                        "library_embed_compile_host": compile_host_plan
                    }
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            "validate report plan_summary.library_embed_compile_host."
                            f"{field} must be a string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_compile_host_plan_release_non_bool(
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
            compile_host_plan = _compile_host_plan()
            compile_host_plan["release"] = "false"
            validate_report["plan_summary"] = {
                "library_embed_compile_host": compile_host_plan
            }
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "validate report plan_summary.library_embed_compile_host.release must be a boolean"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_compile_host_plan_string_array_fields_non_string_array(
        self,
    ) -> None:
        compile_host_string_array_fields = (
            "app_features",
            "command",
            "expected_runtime_plugins",
            "runtime_features",
        )
        for field in compile_host_string_array_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan[field] = ["target-client", 42]
                    validate_report["plan_summary"] = {
                        "library_embed_compile_host": compile_host_plan
                    }
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    expected_diagnostic = (
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field}[1] must be a string"
                    )
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
