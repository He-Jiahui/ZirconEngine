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


class CompileHostPlanCommandSchemaTests(unittest.TestCase):
    def test_compile_host_rejects_plan_with_non_string_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            command = list(compile_plan["command"])
            command[1] = 42
            compile_plan["command"] = command
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
                return_value=subprocess.CompletedProcess(command, 0),
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
                    "CompileHost plan command[1] must be a string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "CompileHost plan command must be a string array" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_padded_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            command = list(compile_plan["command"])
            command[0] = " cargo "
            compile_plan["command"] = command
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
                return_value=subprocess.CompletedProcess(command, 0),
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
                    "CompileHost plan command[0] must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
