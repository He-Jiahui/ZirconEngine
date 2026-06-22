from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateBuildValidationTests(unittest.TestCase):
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

    def test_report_rejects_malformed_source_template_build_validation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out, report_overrides={"build_validation": []})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report build_validation must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_executed_source_template_build_without_log_lines(
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
                        "requested": True,
                        "executed": True,
                        "status": "passed",
                        "exit_code": 0,
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
                    "SourceTemplate build_validation stdout_lines must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "SourceTemplate build_validation stderr_lines must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_malformed_source_template_build_log_lines(self) -> None:
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
                        "status": "passed",
                        "exit_code": 0,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "build"],
                        "stdout_lines": ["ok", 123],
                        "stderr_lines": "warning",
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation stdout_lines[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "SourceTemplate build_validation stderr_lines must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_non_string_log_line_entry_before_array_shape(
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
                        "requested": True,
                        "executed": True,
                        "status": "passed",
                        "exit_code": 0,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "build"],
                        "stdout_lines": ["ok", 123],
                        "stderr_lines": ["warning", None],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation stdout_lines[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "SourceTemplate build_validation stderr_lines[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "SourceTemplate build_validation stdout_lines must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "SourceTemplate build_validation stderr_lines must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_command_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": ["cargo", "build", "--manifest-path", "Cargo.toml"],
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": ["cargo", "check"],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command must match SourceTemplate report command"
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

    def test_report_rejects_empty_source_template_build_validation_command(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": [],
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(
                            out / "stages" / "source_template" / "project"
                        ),
                        "command": [],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_blank_source_template_build_validation_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = [
                "cargo",
                "",
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_blank_source_template_report_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": [
                        "cargo",
                        "",
                        "build",
                        "--manifest-path",
                        str(project_dir / "Cargo.toml"),
                    ],
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": [
                            "cargo",
                            "build",
                            "--manifest-path",
                            str(project_dir / "Cargo.toml"),
                        ],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_option_value(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = ["cargo", "build", "--manifest-path", "--release"]
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --manifest-path value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_dangling_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = ["cargo", "build", "--manifest-path"]
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --manifest-path must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_dangling_target_dir(
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --target-dir must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_target_dir_option_value(
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
                "--release",
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --target-dir value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_target_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = [
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
                    "command": command,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": command,
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command target-dir must match current SourceTemplate stage target"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_duplicate_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--manifest-path",
                str(root / "other" / "Cargo.toml"),
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --manifest-path must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_report_command_duplicate_target_dir(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            project_dir = out / "stages" / "source_template" / "project"
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(project_dir / "Cargo.toml"),
                "--target-dir",
                str(out / "stages" / "source_template" / "target"),
                "--target-dir",
                str(root / "other-target"),
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
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command --target-dir must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_command_dangling_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            project_dir = out / "stages" / "source_template" / "project"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        str(project_dir / "Cargo.toml"),
                    ],
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(project_dir),
                        "command": ["cargo", "build", "--manifest-path"],
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation command --manifest-path must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_working_dir_mismatch(
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
                        "exit_code": None,
                        "working_dir": str(
                            out / "stages" / "source_template" / "other"
                        ),
                        "command": command,
                    },
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation working_dir must match SourceTemplate report project"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_build_validation_working_dir_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_manifest = out / "stages" / "source_template" / "project" / "Cargo.toml"
            working_dir = out / "stages" / "source_template" / "other"
            command = ["cargo", "build", "--manifest-path", str(project_manifest)]
            _write_source_template_report(
                out,
                report_overrides={
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(working_dir),
                        "command": command,
                    },
                },
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == working_dir:
                    raise OSError(
                        "simulated source template working_dir resolve failure"
                    )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build_validation working_dir" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template working_dir resolve failure"
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
