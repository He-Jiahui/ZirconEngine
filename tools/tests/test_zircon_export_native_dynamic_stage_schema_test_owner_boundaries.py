"""Boundary tests for NativeDynamic stage schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_SCHEMA_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_schema.py"
)
STAGE_OPERATION_AUDIT_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_operation_audit_schema.py"
)
STAGE_OPERATION_AUDIT_SUPPORT = (
    REPO_ROOT
    / "tools/zircon_export/tests/native_dynamic_stage_operation_audit_schema_test_support.py"
)

OPERATION_AUDIT_TEST_METHODS = (
    "test_report_stage_rejects_native_dynamic_operation_audit_unknown_field",
    "test_report_stage_rejects_native_dynamic_operation_audit_missing_stage_evidence_field",
    "test_report_stage_rejects_native_dynamic_operation_audit_package_missing_required_field",
    "test_report_stage_rejects_native_dynamic_operation_audit_artifact_empty_command",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicStageSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_stage_operation_audit_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            STAGE_OPERATION_AUDIT_TEST.exists(),
            "NativeDynamic stage operation-audit schema test owner is missing",
        )

        root_text = STAGE_SCHEMA_TEST.read_text(encoding="utf-8")
        audit_text = STAGE_OPERATION_AUDIT_TEST.read_text(encoding="utf-8")

        for method_name in OPERATION_AUDIT_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic stage root test should not own operation-audit gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    audit_text,
                    "Operation-audit owner is missing coverage",
                )

    def test_native_dynamic_stage_schema_root_keeps_package_export_tests(self):
        root_text = STAGE_SCHEMA_TEST.read_text(encoding="utf-8")
        audit_text = (
            STAGE_OPERATION_AUDIT_TEST.read_text(encoding="utf-8")
            if STAGE_OPERATION_AUDIT_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_stage_rejects_native_dynamic_package_export_unknown_field",
            "test_report_stage_rejects_native_dynamic_package_export_abi_missing_required_field",
            "test_report_stage_rejects_native_dynamic_file_manifest_field_types",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", audit_text)

    def test_stage_operation_audit_schema_support_has_dedicated_owner(self):
        self.assertTrue(
            STAGE_OPERATION_AUDIT_SUPPORT.exists(),
            "NativeDynamic stage operation-audit test support owner is missing",
        )

        for path in (STAGE_SCHEMA_TEST, STAGE_OPERATION_AUDIT_TEST):
            with self.subTest(path=path.name):
                text = path.read_text(encoding="utf-8") if path.exists() else ""
                self.assertNotIn("def _native_operation_audit(", text)

        audit_text = (
            STAGE_OPERATION_AUDIT_TEST.read_text(encoding="utf-8")
            if STAGE_OPERATION_AUDIT_TEST.exists()
            else ""
        )
        self.assertIn(
            "from tools.zircon_export.tests.native_dynamic_stage_operation_audit_schema_test_support import",
            audit_text,
        )

    def test_native_dynamic_stage_schema_test_owners_stay_small(self):
        self.assertLess(
            _line_count(STAGE_SCHEMA_TEST),
            940,
            "NativeDynamic stage schema root test should stay below large-file budget",
        )
        for path, budget, description in (
            (STAGE_OPERATION_AUDIT_TEST, 260, "operation-audit schema"),
            (STAGE_OPERATION_AUDIT_SUPPORT, 80, "operation-audit support"),
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
