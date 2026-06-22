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


class CompileHostPlanArrayDuplicateTests(unittest.TestCase):
    def test_compile_host_rejects_plan_duplicate_array_evidence_entry(
        self,
    ) -> None:
        cases: tuple[tuple[str, dict[str, object], str], ...] = (
            (
                "app_features",
                {
                    "app_features": ["target-client", "target-client"],
                    "command": self._compile_host_command_with(
                        "--features",
                        "target-client target-client",
                    ),
                },
                "CompileHost plan app_features[1] duplicates entry 0",
            ),
            (
                "runtime_features",
                {"runtime_features": ["target-client", "target-client"]},
                "CompileHost plan runtime_features[1] duplicates entry 0",
            ),
            (
                "expected_runtime_plugins",
                {"expected_runtime_plugins": ["rendering", "rendering"]},
                "CompileHost plan expected_runtime_plugins[1] duplicates entry 0",
            ),
        )
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                report, exit_code, cargo_call = self._run_compile_host_plan(
                    overrides
                )

                self.assertEqual(exit_code, 2)
                cargo_call.assert_not_called()
                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertEqual(report["command"], [])
                self.assertIsNone(report["host_executable"])
                self.assertTrue(
                    any(
                        expected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def _run_compile_host_plan(
        self,
        overrides: dict[str, object],
    ) -> tuple[dict[str, object], int, mock.Mock]:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan.update(overrides)
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
                return_value=subprocess.CompletedProcess([], 0),
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            return report, exit_code, cargo_call

    def _compile_host_command_with(self, option: str, value: str) -> list[str]:
        command = list(_compile_host_plan()["command"])
        command[command.index(option) + 1] = value
        return command


if __name__ == "__main__":
    unittest.main()
