from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_template_report_helpers import (
    _template_manifest_text,
)
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateReportSemanticsTests(unittest.TestCase):
    def test_report_rejects_template_report_missing_profile_membership(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["profile"] = "windows-release"
            template["compatible_profiles"] = ["linux-release"]
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
                    "PlatformBundle report template.compatible_profiles "
                    "does not include profile windows-release" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_duplicate_compatible_profile_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["compatible_profiles"] = ["windows-release", "windows-release"]
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
                    "PlatformBundle report template.compatible_profiles "
                    "duplicate entry windows-release" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_padded_duplicate_compatible_profile_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["compatible_profiles"] = [
                " windows-release ",
                " windows-release ",
            ]
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
                    "PlatformBundle report template.compatible_profiles[0] "
                    "must be a non-empty trimmed string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report template.compatible_profiles "
                    "duplicate entry" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report template.compatible_profiles "
                    "does not include profile windows-release" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_enum_field_unknown_value(self) -> None:
        cases = (
            (
                "bundle_format",
                "unknown_format",
                "app_bundle, directory, web_static, zip",
            ),
            (
                "host_kind",
                "unknown_host",
                "browser, desktop, headless, mobile_app",
            ),
            (
                "host_artifact",
                "generated",
                "placeholder, precompiled",
            ),
            (
                "plugin_strategy",
                "unknown_plugin_strategy",
                "native_dynamic_allowed, static_source_or_vm_only",
            ),
            (
                "resource_strategy",
                "unknown_resource_strategy",
                "browser_fetch, filesystem_bundle, mobile_asset_bundle",
            ),
        )
        for field, value, expected_values in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template[field] = value
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
                            f"PlatformBundle report template.{field}={value!r} "
                            f"is not one of {expected_values}" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_engine_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["engine_version"] = "0.1.0"
            template["expected_engine_version"] = "9.9.9"
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
                    "PlatformBundle report template.engine_version 0.1.0 "
                    "does not match expected_engine_version 9.9.9" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_target_platform_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["target_platform"] = "windows-x86_64"
            template["expected_target_platform"] = "linux-x86_64"
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
                    "PlatformBundle report template.target_platform windows-x86_64 "
                    "does not match expected_target_platform linux-x86_64" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_content_hash_mismatch(self) -> None:
        for field in ("computed_content_hash", "content_hash"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template[field] = "0" * 64
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
                            f"PlatformBundle report template.{field} "
                            "does not match computed content hash"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_hash_field_malformed(self) -> None:
        for field in ("computed_content_hash", "content_hash"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template[field] = "not-a-hash"
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
                            f"PlatformBundle report template.{field} must be a SHA-256 hex digest"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_format_version_mismatch(self) -> None:
        for field in ("expected_format_version", "format_version"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template[field] = 999
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
                            f"PlatformBundle report template.{field} must be 1"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_string_field_blank(self) -> None:
        for field in (
            "bundle_format",
            "computed_content_hash",
            "content_hash",
            "engine_version",
            "expected_engine_version",
            "expected_target_platform",
            "host_artifact",
            "host_executable",
            "host_kind",
            "manifest",
            "plugin_strategy",
            "profile",
            "resource_strategy",
            "target_platform",
            "template_dir",
            "template_id",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template[field] = " "
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
                            f"PlatformBundle report template.{field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )



if __name__ == "__main__":
    unittest.main()
