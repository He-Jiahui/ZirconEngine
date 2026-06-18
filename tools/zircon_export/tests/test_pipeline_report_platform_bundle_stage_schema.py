from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.test_pipeline_report_platform_bundle import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleStageSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_platform_bundle_unknown_top_level_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report["unsigned_sidecar"] = {"path": "sidecar.bin"}
            _write_stage_report(out, "platform_bundle", platform_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("PlatformBundle", report["fatal_stages"])
            self.assertTrue(
                any(
                    "PlatformBundle report unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_bundle_nested_template_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["unsigned_sidecar"] = {"path": "sidecar.bin"}
            _write_stage_report(out, "platform_bundle", platform_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("PlatformBundle", report["fatal_stages"])
            self.assertTrue(
                any(
                    "PlatformBundle report template unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_bundle_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            ("bundle", "PlatformBundle report bundle must be a string"),
            (
                "host_executable",
                "PlatformBundle report host_executable must be a string",
            ),
            ("host_source", "PlatformBundle report host_source must be a string"),
            (
                "host_source_origin",
                "PlatformBundle report host_source_origin must be a string",
            ),
            ("pack", "PlatformBundle report pack must be a string"),
            ("pack_source", "PlatformBundle report pack_source must be a string"),
            (
                "pack_source_origin",
                "PlatformBundle report pack_source_origin must be a string",
            ),
            (
                "template_files",
                "PlatformBundle report template_files must be an object array",
            ),
            (
                "bundle_manifest",
                "PlatformBundle report bundle_manifest must be a string",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out)
                    platform_report = _read_stage_report(out, "platform_bundle")
                    platform_report.pop(field)
                    _write_stage_report(out, "platform_bundle", platform_report)
                    _write_bundle_manifest_from_platform_report(
                        fixture["bundle_manifest"],
                        platform_report,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("PlatformBundle", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
