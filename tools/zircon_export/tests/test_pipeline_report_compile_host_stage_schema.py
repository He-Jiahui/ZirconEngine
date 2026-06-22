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

    def test_report_stage_rejects_compile_host_padded_command_entry(self) -> None:
        command = list(_compile_host_plan()["command"])
        command[0] = " cargo "

        self._assert_compile_host_command_diagnostic(
            command,
            "compile_host report command[0] must be a non-empty trimmed string",
        )

    def test_report_stage_rejects_compile_host_command_entry_non_string_before_array_shape(
        self,
    ) -> None:
        self._assert_compile_host_command_diagnostic(
            ["cargo", 42],
            "compile_host report command[1] must be a string",
            unexpected_diagnostic=(
                "compile_host report command must be a string array"
            ),
        )

    def test_report_stage_rejects_compile_host_log_line_entry_non_string_before_array_shape(
        self,
    ) -> None:
        cases = (
            ("stdout_lines", ["ok", 42]),
            ("stderr_lines", ["warning", None]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                self._assert_compile_host_report_mutation_diagnostic(
                    lambda report, field=field, value=value: report.__setitem__(
                        field,
                        value,
                    ),
                    f"compile_host report {field}[1] must be a string",
                    unexpected_diagnostic=(
                        f"compile_host report {field} must be a string array"
                    ),
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

    def test_report_stage_rejects_compile_host_link_plan_padded_feature_entry(
        self,
    ) -> None:
        cases = (
            ("app_features", [" target-client "]),
            ("runtime_features", [" target-client "]),
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
                            f"{field}[0] must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_link_plan_feature_entry_non_string_before_array_shape(
        self,
    ) -> None:
        cases = (
            ("app_features", ["target-client", 42]),
            ("runtime_features", ["target-client", None]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                self._assert_compile_host_report_mutation_diagnostic(
                    lambda report, field=field, value=value: report[
                        "link_plan"
                    ].__setitem__(field, value),
                    f"compile_host report link_plan.{field}[1] must be a string",
                    unexpected_diagnostic=(
                        f"compile_host report link_plan.{field} "
                        "must be a string array"
                    ),
                )

    def test_report_stage_rejects_compile_host_link_plan_duplicate_feature_entry(
        self,
    ) -> None:
        cases = (
            ("app_features", ["target-client", "target-client"]),
            ("runtime_features", ["target-client", "target-client"]),
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
                            f"{field}[1] duplicates entry 0"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_link_plan_duplicate_expected_plugin(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_compile_host_report(
                out,
                out / "compile" / "zircon_runtime.exe",
            )
            compile_report_path = out / "stages" / "compile_host" / "report.json"
            compile_report = json.loads(
                compile_report_path.read_text(encoding="utf-8")
            )
            compile_report["link_plan"]["expected_runtime_plugins"] = [
                "rendering",
                "rendering",
            ]
            compile_report_path.write_text(
                json.dumps(compile_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertTrue(
                any(
                    "compile_host report link_plan.expected_runtime_plugins[1] "
                    "duplicates entry 0"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_does_not_compare_fatal_compile_host_link_plan_with_validate(
        self,
    ) -> None:
        cases = (
            (
                "app_features",
                [" target-client "],
                "compile_host report link_plan.app_features[0] "
                "must be a non-empty trimmed string",
                "compile_host report link_plan.app_features does not match "
                "validate report plan_summary.library_embed_compile_host.app_features",
            ),
            (
                "runtime_features",
                [" target-client "],
                "compile_host report link_plan.runtime_features[0] "
                "must be a non-empty trimmed string",
                "compile_host report link_plan.runtime_features does not match "
                "validate report plan_summary.library_embed_compile_host.runtime_features",
            ),
            (
                "expected_runtime_plugins",
                [" rendering "],
                "compile_host report link_plan.expected_runtime_plugins[0] "
                "must be a non-empty trimmed project plugin id",
                "compile_host report link_plan.expected_runtime_plugins does not match "
                "validate report plan_summary.library_embed_compile_host.expected_runtime_plugins",
            ),
        )
        for field, value, expected, unexpected in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        (
                            out
                            / "stages"
                            / "compile_host"
                            / "target"
                            / "release"
                            / "zircon_runtime.exe"
                        ),
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(
                        out,
                        out / "bundle" / "windows-release" / "assets.zrpack",
                    )
                    _write_stage_report(out, "platform_bundle", fatal=False)
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
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(expected in diagnostic for diagnostic in report["diagnostics"]),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            unexpected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "CompileHost report does not contain "
                            "host_executable evidence" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def _assert_compile_host_command_diagnostic(
        self,
        command: list[object],
        expected_diagnostic: str,
        *,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        self._assert_compile_host_report_mutation_diagnostic(
            lambda report: report.__setitem__("command", command),
            expected_diagnostic,
            unexpected_diagnostic=unexpected_diagnostic,
        )

    def _assert_compile_host_report_mutation_diagnostic(
        self,
        mutate_report: object,
        expected_diagnostic: str,
        *,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            compile_report_path = out / "stages" / "compile_host" / "report.json"
            compile_report = json.loads(compile_report_path.read_text(encoding="utf-8"))
            mutate_report(compile_report)
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
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
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
