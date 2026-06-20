from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
)


class CompileHostStageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_compile_host_empty_command(self) -> None:
        self._assert_compile_host_command_diagnostic(
            [],
            "compile_host report command must be a non-empty string array",
        )

    def test_report_stage_rejects_compile_host_blank_command_entry(self) -> None:
        self._assert_compile_host_command_diagnostic(
            ["cargo", ""],
            "compile_host report command must not contain blank entries",
        )

    def test_report_stage_rejects_compile_host_command_missing_build_option(
        self,
    ) -> None:
        cases = (
            (
                ["cargo", "check"],
                "compile_host report command must run cargo build",
            ),
            (
                self._compile_host_command_without("-p", value=True),
                "compile_host report command must include -p/--package",
            ),
            (
                self._compile_host_command_without("--bin", value=True),
                "compile_host report command must include --bin",
            ),
            (
                self._compile_host_command_without("--no-default-features"),
                "compile_host report command must include --no-default-features",
            ),
            (
                self._compile_host_command_without("--features", value=True),
                "compile_host report command must include --features",
            ),
            (
                self._compile_host_command_without("--target-dir", value=True),
                "compile_host report command must include --target-dir",
            ),
            (
                [*_compile_host_plan()["command"], "--features", "target-client"],
                "compile_host report command --features must appear only once",
            ),
        )
        for command, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                self._assert_compile_host_command_diagnostic(
                    command,
                    expected_diagnostic,
                )

    def test_report_stage_rejects_compile_host_nonfatal_nonzero_exit_code(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            compile_report_path = out / "stages" / "compile_host" / "report.json"
            compile_report = json.loads(compile_report_path.read_text(encoding="utf-8"))
            compile_report["exit_code"] = 1
            compile_report_path.write_text(
                json.dumps(compile_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertTrue(
                any(
                    "compile_host report exit_code must be 0 for non-fatal report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_link_plan_blank_feature_entry(
        self,
    ) -> None:
        cases = (
            ("app_features", ["target-client", ""]),
            ("runtime_features", ["target-client", "   "]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    compile_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_report = json.loads(
                        compile_report_path.read_text(encoding="utf-8")
                    )
                    compile_report["link_plan"][field] = value
                    compile_report_path.write_text(
                        json.dumps(compile_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "compile_host report link_plan."
                            f"{field} must not contain blank entries"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def _assert_compile_host_command_diagnostic(
        self,
        command: list[object],
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            compile_report_path = out / "stages" / "compile_host" / "report.json"
            compile_report = json.loads(compile_report_path.read_text(encoding="utf-8"))
            compile_report["command"] = command
            compile_report_path.write_text(
                json.dumps(compile_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertTrue(
                any(expected_diagnostic in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def _compile_host_command_without(
        self,
        option: str,
        *,
        value: bool = False,
    ) -> list[str]:
        command = list(_compile_host_plan()["command"])
        if value:
            index = command.index(option)
            del command[index : index + 2]
        else:
            command.remove(option)
        return command
