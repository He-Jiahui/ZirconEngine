from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _compile_host_args,
    _compile_host_plan,
    _run_compile_host_quiet,
    json_dumps,
    json_loads,
)


class CompileHostOutputGateTests(unittest.TestCase):
    def test_compile_host_rejects_plan_without_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan.pop("binary")
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan binary must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_without_cargo_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan.pop("cargo_profile")
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan cargo_profile must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_empty_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = []
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_blank_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", ""]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_dangling_target_dir_option(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", "build", "--target-dir"]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan command --target-dir must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_target_dir_option_with_option_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", "build", "--target-dir", "--release"]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan command --target-dir value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_duplicate_target_dir_option(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = [
                "cargo",
                "build",
                "--target-dir",
                str(root / "target-a"),
                "--target-dir",
                str(root / "target-b"),
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan command --target-dir must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_directory_host_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            def cargo_success(
                command: list[str],
                cwd: Path,
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                target_dir = Path(command[command.index("--target-dir") + 1])
                host_output = target_dir / "debug" / (
                    "zircon_runtime.exe" if target_dir.drive else "zircon_runtime"
                )
                host_output.mkdir(parents=True)
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout="compile stdout\n",
                    stderr="compile stderr\n",
                )

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                side_effect=cargo_success,
            ):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(Path(report["host_executable"]).is_dir())
            self.assertEqual(report["stdout_lines"], ["compile stdout"])
            self.assertEqual(report["stderr_lines"], ["compile stderr"])
            self.assertTrue(
                any(
                    "CompileHost output" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
