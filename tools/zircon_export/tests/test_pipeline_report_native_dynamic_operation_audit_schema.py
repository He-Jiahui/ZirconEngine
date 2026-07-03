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


class PipelineReportNativeDynamicOperationAuditSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field(
        self,
    ) -> None:
        cases = (
            ("exit_code", "must be an integer"),
            ("before_sha256", "must be a string"),
            ("after_sha256", "must be a string"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(
                                artifacts=[
                                    _native_operation_audit_artifact_without(field)
                                ]
                            )
                        ]
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
                            "native_dynamic report native_signing packages[0] "
                            f"artifacts[0].{field} {expected_type}" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "NativeDynamic report native_signing is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_nonzero_exit_code(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(exit_code=1)
                        ]
                    )
                ]
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].exit_code must be 0 for non-fatal "
                    "operation audit"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_diagnostic_entry(
        self,
    ) -> None:
        cases = (
            ("native_signing", [""]),
            ("native_notarization", ["   "]),
        )
        for operation, diagnostics in cases:
            with self.subTest(operation=operation, diagnostics=diagnostics):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        diagnostics=diagnostics
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
                            f"{operation}.diagnostics must not contain blank entries"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_diagnostic_entry(
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
                        diagnostics=[" signing warning "]
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
                            f"{operation}.diagnostics[0] "
                            "must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_non_string_diagnostic_entry_before_array_shape(
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
                        diagnostics=["signing warning", 42]
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
                            f"{operation}.diagnostics[1] "
                            "must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.diagnostics "
                            "must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_command_non_string_entry_before_array_shape(
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
                        packages=[
                            _native_operation_audit_package(
                                artifacts=[
                                    _native_operation_audit_artifact(
                                        command=["signtool", 42],
                                    )
                                ]
                            )
                        ]
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
                            f"{operation} packages[0] "
                            "artifacts[0].command[1] must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation} packages[0] "
                            "artifacts[0].command must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_negative_counts(
        self,
    ) -> None:
        cases = (
            (
                "package_count",
                lambda audit: audit.__setitem__("package_count", -1),
                "native_dynamic report native_signing.package_count must be non-negative",
            ),
            (
                "artifact_count",
                lambda audit: audit["packages"][0].__setitem__(
                    "artifact_count",
                    -1,
                ),
                "native_dynamic report native_signing packages[0].artifact_count "
                "must be non-negative",
            ),
        )
        for field, mutate, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    audit = _native_operation_audit()
                    mutate(audit)
                    native_report["native_signing"] = audit
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
                    if field == "package_relative_artifact":
                        self.assertFalse(
                            any(
                                "native_dynamic report native_signing packages[0] "
                                "artifacts[0].package_relative_artifact "
                                "must be a safe relative path"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_native_dynamic_operation_audit_unsafe_relative_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    "../zircon_plugin_animation.dll"
                                )
                            )
                        ]
                    )
                ]
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].package_relative_artifact "
                    "must be a safe relative path"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(
                                artifact="plugins/animation/native/forged.dll"
                            )
                        ]
                    )
                ]
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
                    "native_dynamic report native_signing package animation "
                    "artifacts[0].artifact "
                    "plugins/animation/native/forged.dll does not match "
                    "package_relative_artifact "
                    "plugins/animation/native/zircon_plugin_animation.dll"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_duplicate_package_relative_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifact_count=2,
                        artifacts=[
                            _native_operation_audit_artifact(),
                            _native_operation_audit_artifact(),
                        ],
                    )
                ]
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts.package_relative_artifact "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_duplicate_package_relative_artifact_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = _write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifact_count=2,
                        artifacts=[
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    " native/zircon_plugin_animation.dll "
                                )
                            ),
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    " native/zircon_plugin_animation.dll "
                                )
                            ),
                        ],
                    )
                ]
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].package_relative_artifact "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_dynamic report native_signing packages[0] "
                    "artifacts.package_relative_artifact "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
