from __future__ import annotations

import unittest

from tools.zircon_export.pipeline_report_stage_schema import (
    stage_report_schema_diagnostics,
)


class CompileHostStageSchemaTests(unittest.TestCase):
    def test_staged_compile_host_report_schema_is_valid(self) -> None:
        self.assertEqual(stage_report_schema_diagnostics("compile_host", self._report()), [])

    def test_legacy_link_plan_field_is_rejected(self) -> None:
        report = self._report()
        report["link_plan"] = {}

        diagnostics = stage_report_schema_diagnostics("compile_host", report)

        self.assertIn("compile_host report unknown field link_plan", diagnostics)

    def test_legacy_cargo_command_is_rejected(self) -> None:
        report = self._report()
        report["command"] = ["cargo", "build"]

        diagnostics = stage_report_schema_diagnostics("compile_host", report)

        self.assertTrue(
            any("must run tools/zircon_build.py through Python" in value for value in diagnostics),
            diagnostics,
        )

    def test_staged_command_requires_every_owned_option(self) -> None:
        for option in ("--targets", "--out", "--mode", "--runtime-features", "--cargo"):
            with self.subTest(option=option):
                report = self._report()
                index = report["command"].index(option)
                del report["command"][index : index + 2]

                diagnostics = stage_report_schema_diagnostics("compile_host", report)

                self.assertTrue(
                    any(f"must include {option}" in value for value in diagnostics),
                    diagnostics,
                )

    def test_staged_command_rejects_unknown_mode(self) -> None:
        report = self._report()
        mode_index = report["command"].index("--mode")
        report["command"][mode_index + 1] = "profiling"

        diagnostics = stage_report_schema_diagnostics("compile_host", report)

        self.assertIn(
            "compile_host report command --mode must be debug or release",
            diagnostics,
        )

    def test_nonfatal_report_requires_zero_exit_code(self) -> None:
        report = self._report()
        report["exit_code"] = 7

        diagnostics = stage_report_schema_diagnostics("compile_host", report)

        self.assertIn(
            "compile_host report exit_code must be 0 for non-fatal report",
            diagnostics,
        )

    def test_report_arrays_are_typed_string_arrays(self) -> None:
        for field in ("command", "stdout_lines", "stderr_lines"):
            with self.subTest(field=field):
                report = self._report()
                report[field] = [7]

                diagnostics = stage_report_schema_diagnostics("compile_host", report)

                self.assertTrue(
                    any(f"compile_host report {field}[0] must be a string" in value for value in diagnostics),
                    diagnostics,
                )

    @staticmethod
    def _report() -> dict[str, object]:
        return {
            "stage": "CompileHost",
            "profile": "desktop_windows",
            "fatal": False,
            "diagnostics": [],
            "command": [
                "python",
                "tools/zircon_build.py",
                "--targets",
                "hub,editor,runtime",
                "--out",
                "D:/export/staged",
                "--mode",
                "release",
                "--runtime-features",
                "target-client",
                "--cargo",
                "cargo",
            ],
            "host_executable": "D:/export/staged/ZirconEngine/zircon_hub.exe",
            "staged_engine_root": "D:/export/staged/ZirconEngine",
            "exit_code": 0,
            "stdout_lines": [],
            "stderr_lines": [],
        }


if __name__ == "__main__":
    unittest.main()
