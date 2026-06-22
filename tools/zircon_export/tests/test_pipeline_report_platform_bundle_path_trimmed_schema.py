from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundlePathTrimmedSchemaTests(unittest.TestCase):
    def test_report_rejects_platform_bundle_report_padded_string_field(self) -> None:
        fields = (
            "bundle",
            "host_executable",
            "host_source",
            "host_source_origin",
            "pack",
            "pack_source",
            "pack_source_origin",
            "delta_pack",
            "delta_pack_source",
            "delta_pack_source_origin",
            "native_plugins",
            "bundle_manifest",
        )
        for field in fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out, with_delta=True)
                    platform_report = _read_stage_report(out, "platform_bundle")
                    value = platform_report[field]
                    self.assertIsInstance(value, str)
                    platform_report[field] = f" {value} "
                    _write_stage_report(out, "platform_bundle", platform_report)
                    _write_bundle_manifest_from_platform_report(
                        fixture["bundle_manifest"],
                        platform_report,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"PlatformBundle report {field} "
                            "must be a non-empty trimmed string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_platform_bundle_handoff_schema_before_semantics(
        self,
    ) -> None:
        cases = (
            (
                "host_source",
                "PlatformBundle report host_source must be a non-empty trimmed string",
                "PlatformBundle report host_source does not match CompileHost report host_executable",
            ),
            (
                "host_source_origin",
                "PlatformBundle report host_source_origin must be a non-empty trimmed string",
                "PlatformBundle report host_source_origin must be compile_host_report, argument, or template",
            ),
            (
                "pack_source",
                "PlatformBundle report pack_source must be a non-empty trimmed string",
                "PlatformBundle report pack_source does not match Pack report pack",
            ),
            (
                "pack_source_origin",
                "PlatformBundle report pack_source_origin must be a non-empty trimmed string",
                "PlatformBundle report pack_source_origin must be pack_report or argument",
            ),
            (
                "delta_pack_source",
                "PlatformBundle report delta_pack_source must be a non-empty trimmed string",
                "PlatformBundle report delta_pack_source does not match Pack report delta_pack",
            ),
            (
                "delta_pack_source_origin",
                "PlatformBundle report delta_pack_source_origin must be a non-empty trimmed string",
                "PlatformBundle report delta_pack_source_origin must be pack_report or argument",
            ),
        )
        for field, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out, with_delta=True)
                    platform_report = _read_stage_report(out, "platform_bundle")
                    value = platform_report[field]
                    self.assertIsInstance(value, str)
                    platform_report[field] = f" {value} "
                    _write_stage_report(out, "platform_bundle", platform_report)
                    _write_bundle_manifest_from_platform_report(
                        fixture["bundle_manifest"],
                        platform_report,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            unexpected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_platform_bundle_manifest_padded_string_field(self) -> None:
        fields = (
            "profile",
            "host_executable",
            "host_source",
            "host_source_origin",
            "pack",
            "pack_source",
            "pack_source_origin",
            "delta_pack",
            "delta_pack_source",
            "delta_pack_source_origin",
            "native_plugins",
        )
        for field in fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out, with_delta=True)
                    manifest = json.loads(
                        fixture["bundle_manifest"].read_text(encoding="utf-8")
                    )
                    value = manifest[field]
                    self.assertIsInstance(value, str)
                    manifest[field] = f" {value} "
                    fixture["bundle_manifest"].write_text(
                        json.dumps(manifest, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"PlatformBundle bundle_manifest {field} "
                            "must be a non-empty trimmed string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
