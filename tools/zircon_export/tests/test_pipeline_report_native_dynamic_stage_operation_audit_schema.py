from __future__ import annotations

import unittest

from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
)
from tools.zircon_export.tests.native_dynamic_stage_operation_audit_schema_test_support import (
    _disabled_native_operation_audit_without,
    _native_operation_audit,
    _native_operation_audit_artifact,
    _native_operation_audit_package,
)


class PipelineReportNativeDynamicStageOperationAuditSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):
    def test_report_stage_rejects_native_dynamic_operation_audit_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_signing",
            {
                "enabled": False,
                "profile": None,
                "target_platform": "windows-x86_64",
                "allowed_platforms": [],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 0,
                "unsigned_sidecar": "plugins/animation/sidecar.bin",
            },
            "native_dynamic report native_signing unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_operation_audit_missing_stage_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "diagnostics",
                "native_dynamic report native_signing.diagnostics must be a string array",
            ),
            (
                "packages",
                "native_dynamic report native_signing packages must be an object array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    _disabled_native_operation_audit_without(field),
                    expected_diagnostic,
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_field_types(
        self,
    ) -> None:
        cases = (
            ("enabled", "true", "must be a boolean"),
            ("profile", 42, "must be a string"),
            ("target_platform", 42, "must be a string"),
            ("allowed_platforms", "windows-x86_64", "must be a string array"),
            ("allowed_platforms", [42], "must be a string array"),
            ("platform_allowed", "true", "must be a boolean"),
            ("fatal", "false", "must be a boolean"),
            ("package_count", "1", "must be an integer"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                expected_diagnostic = (
                    "native_dynamic report native_signing.allowed_platforms[0] "
                    "must be a string"
                    if field == "allowed_platforms" and isinstance(value, list)
                    else (
                        f"native_dynamic report native_signing.{field} "
                        f"{expected_type}"
                    )
                )
                audit = {
                    "enabled": False,
                    "profile": None,
                    "target_platform": "windows-x86_64",
                    "allowed_platforms": [],
                    "platform_allowed": True,
                    "fatal": False,
                    "package_count": 0,
                    field: value,
                }
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    expected_diagnostic,
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_allowed_platform_entry(
        self,
    ) -> None:
        for allowed_platforms in ([""], ["   "], ["windows", ""]):
            with self.subTest(allowed_platforms=allowed_platforms):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    _native_operation_audit(
                        allowed_platforms=allowed_platforms
                    ),
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain blank entries",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_package_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("package_id", "must be a string"),
            ("artifact_count", "must be an integer"),
            ("artifacts", "must be an object array"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                package = _native_operation_audit_package()
                package.pop(field)
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing "
                    f"packages[0].{field} {expected_type}",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("artifact", "must be a string"),
            ("package_relative_artifact", "must be a string"),
            ("stdout", "must be a string"),
            ("stderr", "must be a string"),
            ("command", "must be a string array"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                artifact = _native_operation_audit_artifact()
                artifact.pop(field)
                package = _native_operation_audit_package(artifacts=[artifact])
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing packages[0] "
                    f"artifacts[0].{field} {expected_type}",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_empty_command(
        self,
    ) -> None:
        for command in ([], ["signtool", ""], ["signtool", "   "]):
            with self.subTest(command=command):
                artifact = _native_operation_audit_artifact(command=command)
                package = _native_operation_audit_package(
                    artifacts=[artifact]
                )
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].command must be a non-empty string array",
                    "NativeDynamic report native_signing is malformed",
                )


if __name__ == "__main__":
    unittest.main()