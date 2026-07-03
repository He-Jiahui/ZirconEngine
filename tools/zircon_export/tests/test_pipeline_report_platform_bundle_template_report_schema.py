from __future__ import annotations

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


class PlatformBundleTemplateReportSchemaTests(unittest.TestCase):
    def test_report_rejects_template_report_missing_success_evidence_field(
        self,
    ) -> None:
        cases = (
            ("bundle_format", "must be a string"),
            ("compatible_profiles", "must be a string array"),
            ("computed_content_hash", "must be a string"),
            ("content_hash", "must be a string"),
            ("diagnostics", "must be a string array"),
            ("engine_version", "must be a string"),
            ("expected_engine_version", "must be a string"),
            ("expected_format_version", "must be an integer"),
            ("expected_target_platform", "must be a string"),
            ("fatal", "must be a boolean"),
            ("files", "must be an object array"),
            ("format_version", "must be an integer"),
            ("host_artifact", "must be a string"),
            ("host_executable", "must be a string"),
            ("host_kind", "must be a string"),
            ("manifest", "must be a string"),
            ("plugin_strategy", "must be a string"),
            ("profile", "must be a string"),
            ("resource_strategy", "must be a string"),
            ("target_platform", "must be a string"),
            ("template_dir", "must be a string"),
            ("template_id", "must be a string"),
        )
        for field, expected_type in cases:
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
                    template.pop(field, None)
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
                            f"PlatformBundle report template.{field} {expected_type}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_padded_required_string_field(
        self,
    ) -> None:
        fields = (
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
        )
        for field in fields:
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
                    value = template[field]
                    self.assertIsInstance(value, str)
                    template[field] = f" {value} "
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
                            "must be a non-empty trimmed string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_padded_compatible_profile_entry(
        self,
    ) -> None:
        cases = (
            (
                [" windows-release "],
                "PlatformBundle report template.compatible_profiles[0] "
                "must be a non-empty trimmed string",
            ),
            (
                ["windows-release", " linux-release "],
                "PlatformBundle report template.compatible_profiles[1] "
                "must be a non-empty trimmed string",
            ),
        )
        for compatible_profiles, expected_diagnostic in cases:
            with self.subTest(compatible_profiles=compatible_profiles):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template["compatible_profiles"] = compatible_profiles
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

    def test_report_rejects_template_report_non_string_compatible_profile_entry_before_array_shape(
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
            template["compatible_profiles"] = [1, "windows-release"]
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn(
                "PlatformBundle report template.compatible_profiles[0] "
                "must be a string",
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report template.compatible_profiles "
                    "must be a string array" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_fatal_without_diagnostics(self) -> None:
        for diagnostics in ([], None):
            with self.subTest(diagnostics=diagnostics):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template["fatal"] = True
                    if diagnostics is None:
                        template.pop("diagnostics", None)
                    else:
                        template["diagnostics"] = diagnostics
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
                            "PlatformBundle report template fatal report "
                            "must include diagnostics" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_non_fatal_with_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["fatal"] = False
            template["diagnostics"] = ["forced warning"]
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
                    "PlatformBundle report template non-fatal report "
                    "must not include diagnostics" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
