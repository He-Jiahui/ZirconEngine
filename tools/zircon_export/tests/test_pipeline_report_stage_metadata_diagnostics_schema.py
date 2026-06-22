from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.stage_handoff import stage_report_path_handoff_diagnostic
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportStageMetadataDiagnosticsSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_non_string_stage_diagnostic_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["diagnostics"] = ["pack warning", 42]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("Pack", report["fatal_stages"])
            self.assertIn(
                "pack report diagnostics[1] must be a string",
                report["diagnostics"],
            )
            self.assertNotIn(
                "pack report diagnostics must be a string array",
                report["diagnostics"],
            )

    def test_handoff_rejects_non_string_stage_diagnostic_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            report_dir = out / "stages" / "pack"
            report_dir.mkdir(parents=True)
            report_dir.joinpath("report.json").write_text(
                json.dumps(
                    {
                        "stage": "Pack",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": ["pack warning", 42],
                        "pack": str(out / "assets.zrpack"),
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )

            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report diagnostics[1] must be a string",
            )

    def test_report_stage_rejects_blank_stage_diagnostic_entry(self) -> None:
        for diagnostic in ("", "   "):
            with self.subTest(diagnostic=repr(diagnostic)):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    pack_report_path = out / "stages" / "pack" / "report.json"
                    pack_report = json.loads(
                        pack_report_path.read_text(encoding="utf-8")
                    )
                    pack_report["diagnostics"] = [diagnostic]
                    pack_report_path.write_text(
                        json.dumps(pack_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("Pack", report["fatal_stages"])
                    self.assertIn(
                        "pack report diagnostics must not contain blank entries",
                        report["diagnostics"],
                    )

    def test_handoff_rejects_blank_stage_diagnostic_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            report_dir = out / "stages" / "pack"
            report_dir.mkdir(parents=True)
            report_dir.joinpath("report.json").write_text(
                json.dumps(
                    {
                        "stage": "Pack",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [" "],
                        "pack": str(out / "assets.zrpack"),
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )

            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report diagnostics must not contain blank entries",
            )

    def test_report_stage_rejects_padded_stage_diagnostic_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["diagnostics"] = [" pack warning "]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("Pack", report["fatal_stages"])
            self.assertIn(
                "pack report diagnostics[0] must be a non-empty trimmed string",
                report["diagnostics"],
            )

    def test_handoff_rejects_padded_stage_diagnostic_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir)
            report_dir = out / "stages" / "pack"
            report_dir.mkdir(parents=True)
            report_dir.joinpath("report.json").write_text(
                json.dumps(
                    {
                        "stage": "Pack",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [" pack warning "],
                        "pack": str(out / "assets.zrpack"),
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )

            self.assertEqual(
                stage_report_path_handoff_diagnostic(
                    out,
                    "pack",
                    "windows-release",
                    "pack",
                ),
                "Pack report diagnostics[0] must be a non-empty trimmed string",
            )


if __name__ == "__main__":
    unittest.main()
