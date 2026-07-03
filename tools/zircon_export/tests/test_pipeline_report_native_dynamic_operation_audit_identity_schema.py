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


class PipelineReportNativeDynamicOperationAuditIdentitySchemaTests(unittest.TestCase):
    def test_report_stage_rejects_native_dynamic_operation_audit_empty_required_identity_string(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(package_id=""),
                    ]
                ),
                "native_dynamic report native_signing packages[0].package_id "
                "must be a non-empty string",
            ),
            (
                "artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(artifact="   ")
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].artifact must be a non-empty string",
            ),
            (
                "package_relative_artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    package_relative_artifact=""
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].package_relative_artifact "
                "must be a non-empty string",
            ),
            (
                "before_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    before_sha256="   "
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 must be a non-empty string",
            ),
            (
                "after_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(after_sha256="")
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 must be a non-empty string",
            ),
        )
        for field, make_audit, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = make_audit()
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

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_required_identity_string(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            package_id=" animation ",
                        ),
                    ]
                ),
                "native_dynamic report native_signing packages[0].package_id "
                "must be a non-empty trimmed string",
            ),
            (
                "artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    artifact=(
                                        " plugins/animation/native/"
                                        "zircon_plugin_animation.dll "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].artifact must be a non-empty trimmed string",
            ),
            (
                "package_relative_artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    package_relative_artifact=(
                                        " native/zircon_plugin_animation.dll "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].package_relative_artifact "
                "must be a non-empty trimmed string",
            ),
            (
                "before_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    before_sha256=(
                                        " 0000000000000000000000000000000000000000"
                                        "000000000000000000000000 "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 "
                "must be a non-empty trimmed string",
            ),
            (
                "after_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    after_sha256=(
                                        " 1111111111111111111111111111111111111111"
                                        "111111111111111111111111 "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 "
                "must be a non-empty trimmed string",
            ),
        )
        for field, make_audit, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = make_audit()
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

    def test_report_stage_rejects_native_dynamic_operation_audit_invalid_hash_string(
        self,
    ) -> None:
        cases = (
            (
                "before_sha256",
                _native_operation_audit_artifact(before_sha256="not-a-sha256"),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 must be a SHA-256 hex digest",
            ),
            (
                "after_sha256",
                _native_operation_audit_artifact(after_sha256="not-a-sha256"),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 must be a SHA-256 hex digest",
            ),
        )
        for field, artifact, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = _write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(artifacts=[artifact])
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
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
