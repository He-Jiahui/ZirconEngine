from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import write_library_embed_reports


class PipelineReportCookAssetsStageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_cook_assets_blank_or_padded_required_string(
        self,
    ) -> None:
        for field in ("cooked_asset_manifest", "cooked_asset_manifest_sha256"):
            for value_kind in ("blank", "padded"):
                with self.subTest(field=field, value_kind=value_kind):
                    with tempfile.TemporaryDirectory() as temp_dir:
                        out = Path(temp_dir) / "out"
                        write_library_embed_reports(out)
                        cook_report_path = (
                            out / "stages" / "cook_assets" / "report.json"
                        )
                        cook_report = json.loads(
                            cook_report_path.read_text(encoding="utf-8")
                        )
                        if value_kind == "blank":
                            cook_report[field] = "   "
                        else:
                            cook_report[field] = f" {cook_report[field]} "
                        cook_report_path.write_text(
                            json.dumps(cook_report, indent=2),
                            encoding="utf-8",
                        )

                        report = build_pipeline_report(out, "windows-release")

                        self.assertTrue(report["fatal"], report["diagnostics"])
                        self.assertIn("CookAssets", report["fatal_stages"])
                        self.assertEqual(report["missing_stages"], [])
                        self.assertTrue(
                            any(
                                f"cook_assets report {field} "
                                "must be a non-empty trimmed string"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_cook_assets_blank_or_padded_optional_string(
        self,
    ) -> None:
        for field in (
            "asset_filter",
            "project_default_scene",
            "project_manifest",
            "source_asset_manifest",
        ):
            for value in ("   ", f" {field}-value "):
                with self.subTest(field=field, value=value):
                    with tempfile.TemporaryDirectory() as temp_dir:
                        out = Path(temp_dir) / "out"
                        write_library_embed_reports(out)
                        cook_report_path = (
                            out / "stages" / "cook_assets" / "report.json"
                        )
                        cook_report = json.loads(
                            cook_report_path.read_text(encoding="utf-8")
                        )
                        cook_report[field] = value
                        cook_report_path.write_text(
                            json.dumps(cook_report, indent=2),
                            encoding="utf-8",
                        )

                        report = build_pipeline_report(out, "windows-release")

                        self.assertTrue(report["fatal"], report["diagnostics"])
                        self.assertIn("CookAssets", report["fatal_stages"])
                        self.assertEqual(report["missing_stages"], [])
                        self.assertTrue(
                            any(
                                f"cook_assets report {field} "
                                "must be a non-empty trimmed string when present"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )


if __name__ == "__main__":
    unittest.main()
