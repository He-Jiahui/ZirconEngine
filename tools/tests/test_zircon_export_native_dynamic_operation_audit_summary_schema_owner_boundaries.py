"""Boundary tests for NativeDynamic operation-audit summary schema ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
OPERATION_AUDIT_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema.py"
)
OPERATION_AUDIT_SUMMARY_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_summary_schema.py"
)

SUMMARY_CONSTANTS = (
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_NON_EMPTY_STRING_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS",
    "NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS",
)

SUMMARY_FUNCTIONS = (
    "operation_audit_allowed_platforms_schema_diagnostics",
    "platform_bundle_native_plugins_operation_audit_schema_diagnostics",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicOperationAuditSummarySchemaOwnerBoundaryTests(unittest.TestCase):
    def test_operation_audit_summary_schema_owner_exists(self) -> None:
        self.assertTrue(
            OPERATION_AUDIT_SUMMARY_SCHEMA.exists(),
            "NativeDynamic operation-audit summary schema owner is missing",
        )

    def test_summary_schema_lives_in_summary_owner(self) -> None:
        schema_text = OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8")
        summary_text = (
            OPERATION_AUDIT_SUMMARY_SCHEMA.read_text(encoding="utf-8")
            if OPERATION_AUDIT_SUMMARY_SCHEMA.exists()
            else ""
        )

        failures: list[str] = []
        for constant in SUMMARY_CONSTANTS:
            definition = f"{constant} ="
            if definition in schema_text:
                failures.append(f"stage schema owner still owns {constant}")
            if definition not in summary_text:
                failures.append(f"summary schema owner missing {constant}")
        for function_name in SUMMARY_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"stage schema owner still owns {function_name}")
            if definition not in summary_text:
                failures.append(f"summary schema owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_stage_schema_imports_summary_owner_without_reverse_import(self) -> None:
        schema_text = OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8")
        summary_text = (
            OPERATION_AUDIT_SUMMARY_SCHEMA.read_text(encoding="utf-8")
            if OPERATION_AUDIT_SUMMARY_SCHEMA.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_operation_audit_summary_schema import (",
            schema_text,
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_operation_audit_schema import",
            summary_text,
        )

    def test_operation_audit_schema_parent_budget_stays_tight(self) -> None:
        self.assertLess(_line_count(OPERATION_AUDIT_SCHEMA), 260)
        self.assertTrue(
            OPERATION_AUDIT_SUMMARY_SCHEMA.exists(),
            "NativeDynamic operation-audit summary schema owner is missing",
        )
        self.assertLess(_line_count(OPERATION_AUDIT_SUMMARY_SCHEMA), 220)


if __name__ == "__main__":
    unittest.main()
