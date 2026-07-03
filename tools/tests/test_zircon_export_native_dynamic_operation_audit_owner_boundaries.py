import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
OPERATION_AUDIT_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema.py"
)
OPERATION_AUDIT_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_operation_audit_schema_helpers.py"
)


class ZirconExportNativeDynamicOperationAuditOwnerBoundaryTests(unittest.TestCase):
    def test_operation_audit_schema_helpers_live_in_helper_owner(self):
        self.assertTrue(
            OPERATION_AUDIT_SCHEMA_HELPERS.exists(),
            "NativeDynamic operation-audit schema helper rules need a dedicated owner",
        )
        schema_text = OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8")
        helper_text = OPERATION_AUDIT_SCHEMA_HELPERS.read_text(encoding="utf-8")

        for function_name in (
            "table_required_non_empty_string_schema_diagnostics",
            "table_required_trimmed_non_empty_string_schema_diagnostics",
            "operation_audit_platform_allowed_schema_diagnostics",
            "object_array_required_non_empty_string_schema_diagnostics",
            "object_array_required_trimmed_non_empty_string_schema_diagnostics",
            "table_sha256_hex_string_schema_diagnostics",
            "object_array_sha256_hex_string_schema_diagnostics",
            "string_array_unique_entries_schema_diagnostics",
            "object_array_unique_string_field_schema_diagnostics",
            "artifact_safe_relative_path_schema_diagnostics",
            "artifact_exit_code_success_schema_diagnostics",
            "table_non_negative_integer_schema_diagnostics",
            "object_array_non_negative_integer_schema_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the operation-audit helper owner",
            )
            self.assertIn(
                f"def {function_name}(",
                helper_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_operation_audit_schema_helpers import",
            schema_text,
            "operation-audit schema should consume the helper owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_operation_audit_schema import",
            helper_text,
            "helper owner must not import the operation-audit schema owner",
        )

    def test_operation_audit_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(OPERATION_AUDIT_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            600,
            "NativeDynamic operation-audit schema owner should stay below 600 lines after helper split",
        )


if __name__ == "__main__":
    unittest.main()
