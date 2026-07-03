from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateBuildValidationStatusTests(unittest.TestCase):
    def test_report_rejects_failed_source_template_build_validation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "build_executed": True,
                    "build_validation": {
                        "requested": True,
                        "executed": True,
                        "status": "failed",
                        "exit_code": 42,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "build"],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation status failed is not publishable"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_execution_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "build_executed": True,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "build"],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation executed must match SourceTemplate report build_executed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_requested_source_template_build_validation_skip(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "build_validation": {
                        "requested": True,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "build"],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation requested build cannot be skipped"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_unrequested_source_template_build_validation_skip(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "build_executed": False,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": command,
                        "stdout_lines": [],
                        "stderr_lines": [],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation skipped status is not publishable"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_skipped_source_template_build_validation_exit_code(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(out / "stages" / "source_template" / "project" / "Cargo.toml"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": 0,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": command,
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation skipped status requires exit_code null"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_unrequested_source_template_build_validation_execution(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(out / "stages" / "source_template" / "project" / "Cargo.toml"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
                    "build_executed": True,
                    "build_validation": {
                        "requested": False,
                        "executed": True,
                        "status": "passed",
                        "exit_code": 0,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": command,
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation executed build must be requested"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
