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


class PipelineReportSourceTemplateCommandSchemaTests(unittest.TestCase):
    def test_report_rejects_source_template_report_non_string_command_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            command = [
                "cargo",
                42,
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
                    "command": command,
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "SourceTemplate report command[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "SourceTemplate report command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_non_string_command_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            command = [
                "cargo",
                42,
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
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

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "SourceTemplate build_validation command "
                    "must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_padded_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            command = [
                " cargo ",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
                    "command": command,
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command[0] must be a non-empty "
                    "trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_padded_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            command = [
                " cargo ",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
            _write_source_template_report(
                out,
                report_overrides={
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
                    "SourceTemplate build_validation command[0] must be a "
                    "non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

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

    def test_report_rejects_source_template_command_target_triple_override(self) -> None:
        forbidden_suffixes = (
            ["--target", "x86_64-unknown-linux-gnu"],
            ["--target=x86_64-unknown-linux-gnu"],
        )
        for suffix in forbidden_suffixes:
            with self.subTest(suffix=suffix):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    project_dir = _write_source_template_report(out)
                    command = [
                        "cargo",
                        "build",
                        "--manifest-path",
                        str(project_dir / "Cargo.toml"),
                        "--target-dir",
                        str(out / "stages" / "source_template" / "target"),
                        *suffix,
                    ]
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    expected_reason = (
                        "must not include --target because export target descriptor "
                        "owns platform target selection"
                    )
                    self.assertTrue(
                        any(
                            diagnostic
                            == f"SourceTemplate report command {expected_reason}"
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertTrue(
                        any(
                            diagnostic
                            == f"SourceTemplate build_validation command {expected_reason}"
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_command_release_profile_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
            source_plan = validate_report["plan_summary"]["source_template_build"]
            source_plan["cargo_profile"] = "release"
            source_plan["release"] = True
            source_plan["command"] = [*source_plan["command"], "--release"]
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )
            project_dir = _write_source_template_report(out)
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
            ]
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

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn(
                "SourceTemplate report command must include --release for release profile",
                report["diagnostics"],
            )
            self.assertIn(
                "SourceTemplate build_validation command must include --release "
                "for release profile",
                report["diagnostics"],
            )

    def test_report_rejects_source_template_command_debug_release_flag(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
                "--release",
            ]
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

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn(
                "SourceTemplate report command must not include --release "
                "for debug profile",
                report["diagnostics"],
            )
            self.assertIn(
                "SourceTemplate build_validation command must not include --release "
                "for debug profile",
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
