from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateCommandSchemaTests(unittest.TestCase):
    def test_report_command_reports_manifest_and_target_option_errors(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = ["cargo", "build", "--manifest-path", "--target-dir"]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": command,
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
            self.assertIn(
                "SourceTemplate report command --manifest-path value must not be another option",
                report["diagnostics"],
            )
            self.assertIn(
                "SourceTemplate report command --target-dir must include a value",
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_missing_target_dir(
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
            ]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": command,
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
                    "SourceTemplate report command must include --target-dir"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_command_missing_target_dir(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            report_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            build_validation_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
            ]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": report_command,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": build_validation_command,
                        "stdout_lines": [],
                        "stderr_lines": [],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command must include --target-dir"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_command_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            project_dir = out / "stages" / "source_template" / "project"
            target_dir = out / "stages" / "source_template" / "target"
            report_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(target_dir),
            ]
            build_validation_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(root / "other" / "Cargo.toml"),
                "--target-dir",
                str(target_dir),
            ]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": report_command,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": build_validation_command,
                        "stdout_lines": [],
                        "stderr_lines": [],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command manifest-path must target current project Cargo.toml"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_command_target_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            project_dir = out / "stages" / "source_template" / "project"
            report_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            build_validation_command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(root / "other-target"),
            ]
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": report_command,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": build_validation_command,
                        "stdout_lines": [],
                        "stderr_lines": [],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command target-dir must match current SourceTemplate stage target"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
