from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_source_template_missing_release_evidence_field(
        self,
    ) -> None:
        missing_fields = (
            (
                "project",
                "SourceTemplate report project must be a non-empty string",
            ),
            (
                "validate_report",
                "SourceTemplate report validate_report must be a non-empty string",
            ),
            (
                "generated_files",
                "SourceTemplate report generated_files must be an object array",
            ),
            (
                "command",
                "SourceTemplate report command must be a non-empty string array",
            ),
            (
                "build_executed",
                "SourceTemplate report build_executed must be a boolean",
            ),
            (
                "build_validation",
                "SourceTemplate report build_validation must be an object",
            ),
        )
        for field, expected_diagnostic in missing_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _write_source_template_report(out)
                    report_path = out / "stages" / "source_template" / "report.json"
                    stage_report = json.loads(report_path.read_text(encoding="utf-8"))
                    stage_report.pop(field)
                    report_path.write_text(
                        json.dumps(stage_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("SourceTemplate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={"unsigned_sidecar": "sidecar.bin"},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("SourceTemplate", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate report unknown field unsigned_sidecar" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["generated_files"][0]["unsigned_sidecar"] = "sidecar.bin"
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("SourceTemplate", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate report generated_files[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_missing_field(self) -> None:
        missing_fields = (
            (
                "path",
                "SourceTemplate generated file path must be a non-empty string",
            ),
            (
                "purpose",
                "SourceTemplate generated file Cargo.toml purpose must be a string",
            ),
            (
                "size",
                "SourceTemplate generated file Cargo.toml size must be an integer",
            ),
            (
                "sha256",
                "SourceTemplate generated file Cargo.toml sha256 must be a string",
            ),
        )
        for field, expected_diagnostic in missing_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _write_source_template_report(out)
                    report_path = out / "stages" / "source_template" / "report.json"
                    stage_report = json.loads(report_path.read_text(encoding="utf-8"))
                    stage_report["generated_files"][0].pop(field)
                    report_path.write_text(
                        json.dumps(stage_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("SourceTemplate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_build_validation_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["build_validation"]["unsigned_sidecar"] = "sidecar.bin"
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("SourceTemplate", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["source_template_build"][
                "unsigned_sidecar"
            ] = "sidecar.bin"
            validate_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("Validate", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_generated_file_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"][0][
                "unsigned_sidecar"
            ] = "sidecar.bin"
            validate_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("Validate", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate Validate generated_files[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_generated_file_missing_field(
        self,
    ) -> None:
        missing_fields = (
            (
                "contents",
                "SourceTemplate Validate generated_files[0].contents must be a string",
            ),
            (
                "path",
                "SourceTemplate Validate generated file path must be a non-empty string",
            ),
            (
                "purpose",
                "SourceTemplate Validate generated_files[0].purpose must be a string",
            ),
        )
        for field, expected_diagnostic in missing_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    validate_path = out / "stages" / "validate" / "report.json"
                    validate_report = json.loads(
                        validate_path.read_text(encoding="utf-8")
                    )
                    validate_report["plan_summary"]["generated_files"][0].pop(field)
                    validate_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("Validate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
