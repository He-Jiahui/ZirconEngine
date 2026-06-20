from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateDiagnosticsSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_blank_fatal_diagnostic_entry(self) -> None:
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
                    validate_report_path = out / "stages" / "validate" / "report.json"
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    validate_report["fatal_diagnostics"] = [diagnostic]
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertIn(
                        "validate report fatal_diagnostics must not contain blank entries",
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
