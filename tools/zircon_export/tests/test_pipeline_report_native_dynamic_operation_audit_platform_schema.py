from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.native_dynamic_operation_audit_test_support import (
    _native_operation_audit,
    _native_operation_audit_artifact,
    _native_operation_audit_artifact_without,
    _native_operation_audit_package,
    _write_native_dynamic_reports,
)


class PipelineReportNativeDynamicOperationAuditPlatformSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_native_dynamic_operation_audit_blank_target_platform(
        self,
    ) -> None:
        cases = (
            ("native_signing", ""),
            ("native_notarization", "   "),
        )
        for operation, target_platform in cases:
            with self.subTest(operation=operation, target_platform=target_platform):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        target_platform=target_platform
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "native_dynamic report "
                            f"{operation}.target_platform "
                            "must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_profile(
        self,
    ) -> None:
        cases = (
            ("native_signing", ""),
            ("native_notarization", "   "),
        )
        for operation, profile in cases:
            with self.subTest(operation=operation, profile=profile):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        profile=profile
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "native_dynamic report "
                            f"{operation}.profile "
                            "must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_summary_string(
        self,
    ) -> None:
        cases = (
            (
                "target_platform",
                "native_signing",
                {"target_platform": " windows-x86_64 "},
                "native_dynamic report native_signing.target_platform "
                "must be a non-empty trimmed string",
            ),
            (
                "profile",
                "native_notarization",
                {"profile": " windows-store "},
                "native_dynamic report native_notarization.profile "
                "must be a non-empty trimmed string",
            ),
        )
        for field, operation, overrides, expected_diagnostic in cases:
            with self.subTest(field=field, operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        **overrides
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    if field == "target_platform":
                        self.assertFalse(
                            any(
                                "native_dynamic report "
                                f"{operation}.platform_allowed does not match "
                                "target_platform and allowed_platforms"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_native_dynamic_operation_audit_duplicate_allowed_platform(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=["windows", "windows"]
            )
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_duplicate_allowed_platform_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=[" windows ", " windows "]
            )
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report native_signing.allowed_platforms[0] "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_allowed_platform_entry(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        allowed_platforms=[" windows "],
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "native_dynamic report "
                            f"{operation}.allowed_platforms[0] "
                            "must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.platform_allowed does not match "
                            "target_platform and allowed_platforms"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_non_string_allowed_platform_entry_before_array_shape(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        allowed_platforms=["windows", 42],
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "native_dynamic report "
                            f"{operation}.allowed_platforms[1] "
                            "must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.allowed_platforms "
                            "must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.platform_allowed does not match "
                            "target_platform and allowed_platforms"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_platform_allowed_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=["macos"],
                platform_allowed=True,
            )
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report native_signing.platform_allowed "
                    "does not match target_platform and allowed_platforms"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
