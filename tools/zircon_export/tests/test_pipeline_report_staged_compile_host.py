import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.pipeline_report_stage_schema import (
    stage_report_schema_diagnostics,
)
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class StagedCompileHostReportTests(unittest.TestCase):
    def test_release_preset_overrides_default_debug_profile_in_full_report(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            staged_root = out / "stages" / "compile_host" / "staged"
            engine_root = staged_root / "ZirconEngine"
            host = engine_root / "zircon_hub.exe"
            _write_compile_host_report(out, host)
            compile_path = out / "stages" / "compile_host" / "report.json"
            compile_report = {
                "stage": "CompileHost",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "command": [
                    "python",
                    "E:/Git/ZirconEngine/tools/zircon_build.py",
                    "--targets",
                    "hub,editor,runtime",
                    "--out",
                    str(staged_root),
                    "--mode",
                    "release",
                    "--runtime-features",
                    "target-client",
                    "--cargo",
                    "cargo",
                ],
                "host_executable": str(host),
                "staged_engine_root": str(engine_root),
                "exit_code": 0,
                "stdout_lines": [],
                "stderr_lines": [],
            }
            compile_path.write_text(
                json.dumps(compile_report, indent=2), encoding="utf-8"
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_staged_build_report_is_accepted_by_final_report_schema(self):
        report = {
            "stage": "CompileHost",
            "profile": "desktop_windows",
            "fatal": False,
            "diagnostics": [],
            "command": [
                "python",
                "E:/Git/ZirconEngine/tools/zircon_build.py",
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
        self.assertEqual(
            stage_report_schema_diagnostics("compile_host", report), []
        )

    def test_legacy_cargo_build_report_is_rejected(self):
        report = {
            "stage": "CompileHost",
            "profile": "desktop_windows",
            "fatal": False,
            "diagnostics": [],
            "command": ["cargo", "build"],
            "host_executable": "D:/legacy.exe",
            "staged_engine_root": "D:/legacy",
            "exit_code": 0,
            "stdout_lines": [],
            "stderr_lines": [],
        }
        self.assertTrue(
            stage_report_schema_diagnostics("compile_host", report)
        )


if __name__ == "__main__":
    unittest.main()
