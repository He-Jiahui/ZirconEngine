"""Boundary tests for NativeDynamic operation-audit schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
OPERATION_AUDIT_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_schema.py"
)
OPERATION_AUDIT_IDENTITY_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_identity_schema.py"
)
OPERATION_AUDIT_PLATFORM_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_operation_audit_platform_schema.py"
)
OPERATION_AUDIT_TEST_SUPPORT = (
    REPO_ROOT
    / "tools/zircon_export/tests/native_dynamic_operation_audit_test_support.py"
)

IDENTITY_TEST_METHODS = (
    "test_report_stage_rejects_native_dynamic_operation_audit_empty_required_identity_string",
    "test_report_stage_rejects_native_dynamic_operation_audit_padded_required_identity_string",
    "test_report_stage_rejects_native_dynamic_operation_audit_invalid_hash_string",
)

PLATFORM_TEST_METHODS = (
    "test_report_stage_rejects_native_dynamic_operation_audit_blank_target_platform",
    "test_report_stage_rejects_native_dynamic_operation_audit_blank_profile",
    "test_report_stage_rejects_native_dynamic_operation_audit_padded_summary_string",
    "test_report_stage_rejects_native_dynamic_operation_audit_duplicate_allowed_platform",
    "test_report_stage_rejects_native_dynamic_operation_audit_padded_duplicate_allowed_platform_before_uniqueness",
    "test_report_stage_rejects_native_dynamic_operation_audit_padded_allowed_platform_entry",
    "test_report_stage_rejects_native_dynamic_operation_audit_non_string_allowed_platform_entry_before_array_shape",
    "test_report_stage_rejects_native_dynamic_operation_audit_platform_allowed_mismatch",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicOperationAuditSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_identity_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            OPERATION_AUDIT_IDENTITY_TEST.exists(),
            "NativeDynamic operation-audit identity test owner is missing",
        )

        root_text = OPERATION_AUDIT_SCHEMA_TEST.read_text(encoding="utf-8")
        identity_text = OPERATION_AUDIT_IDENTITY_TEST.read_text(encoding="utf-8")

        for method_name in IDENTITY_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Operation-audit root test should not own identity gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    identity_text,
                    "Identity owner is missing coverage",
                )

    def test_platform_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            OPERATION_AUDIT_PLATFORM_TEST.exists(),
            "NativeDynamic operation-audit platform test owner is missing",
        )

        root_text = OPERATION_AUDIT_SCHEMA_TEST.read_text(encoding="utf-8")
        platform_text = OPERATION_AUDIT_PLATFORM_TEST.read_text(encoding="utf-8")

        for method_name in PLATFORM_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Operation-audit root test should not own platform gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    platform_text,
                    "Platform owner is missing coverage",
                )

    def test_operation_audit_root_keeps_artifact_evidence_tests(self):
        root_text = OPERATION_AUDIT_SCHEMA_TEST.read_text(encoding="utf-8")
        identity_text = (
            OPERATION_AUDIT_IDENTITY_TEST.read_text(encoding="utf-8")
            if OPERATION_AUDIT_IDENTITY_TEST.exists()
            else ""
        )
        platform_text = (
            OPERATION_AUDIT_PLATFORM_TEST.read_text(encoding="utf-8")
            if OPERATION_AUDIT_PLATFORM_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field",
            "test_report_stage_rejects_native_dynamic_operation_audit_unsafe_relative_artifact",
            "test_report_stage_rejects_native_dynamic_operation_audit_duplicate_package_relative_artifact",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", identity_text)
                self.assertNotIn(f"def {method_name}(", platform_text)

    def test_operation_audit_test_support_has_dedicated_owner(self):
        self.assertTrue(
            OPERATION_AUDIT_TEST_SUPPORT.exists(),
            "Operation-audit test support owner is missing",
        )
        for path in (
            OPERATION_AUDIT_SCHEMA_TEST,
            OPERATION_AUDIT_IDENTITY_TEST,
            OPERATION_AUDIT_PLATFORM_TEST,
        ):
            with self.subTest(path=path.name):
                text = path.read_text(encoding="utf-8") if path.exists() else ""
                self.assertNotIn("def _write_native_dynamic_reports(", text)
                self.assertIn(
                    "from tools.zircon_export.tests.native_dynamic_operation_audit_test_support import",
                    text,
                )

    def test_operation_audit_schema_test_owners_stay_small(self):
        self.assertLess(
            _line_count(OPERATION_AUDIT_SCHEMA_TEST),
            620,
            "Operation-audit root test should stay focused on artifact evidence",
        )
        for path, budget, description in (
            (OPERATION_AUDIT_IDENTITY_TEST, 340, "identity schema"),
            (OPERATION_AUDIT_PLATFORM_TEST, 440, "platform schema"),
            (OPERATION_AUDIT_TEST_SUPPORT, 80, "test support"),
        ):
            with self.subTest(owner=description):
                self.assertTrue(path.exists(), f"{description} owner is missing")
                self.assertLess(
                    _line_count(path),
                    budget,
                    f"{description} owner should stay focused",
                )


if __name__ == "__main__":
    unittest.main()
