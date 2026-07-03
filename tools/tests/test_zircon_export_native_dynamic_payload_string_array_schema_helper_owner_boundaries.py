import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_schema_helpers.py"
)
PAYLOAD_STRING_ARRAY_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_string_array_schema_helpers.py"
)
MATERIALIZED_PACKAGES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_materialized_packages_schema.py"
)


class ZirconExportNativeDynamicPayloadStringArrayHelperOwnerBoundaryTests(
    unittest.TestCase
):
    def test_object_array_string_array_helpers_live_in_string_array_owner(self):
        self.assertTrue(
            PAYLOAD_STRING_ARRAY_HELPERS.exists(),
            "NativeDynamic payload object-array string-array helper rules need a dedicated owner",
        )
        parent_text = PAYLOAD_SCHEMA_HELPERS.read_text(encoding="utf-8")
        string_array_text = PAYLOAD_STRING_ARRAY_HELPERS.read_text(encoding="utf-8")
        materialized_text = MATERIALIZED_PACKAGES_SCHEMA.read_text(encoding="utf-8")

        moved_functions = (
            "object_array_loadable_artifacts_schema_diagnostics",
            "object_array_string_array_no_blank_entries_schema_diagnostics",
            "object_array_string_array_trimmed_non_empty_entries_schema_diagnostics",
            "object_array_string_array_safe_relative_path_schema_diagnostics",
            "object_array_string_array_unique_entries_schema_diagnostics",
            "object_array_integer_matches_string_array_length_schema_diagnostics",
        )
        for function_name in moved_functions:
            self.assertNotIn(
                f"def {function_name}(",
                parent_text,
                f"{function_name} belongs in the object-array string-array helper owner",
            )
            self.assertIn(f"def {function_name}(", string_array_text)

        self.assertIn(
            "from .pipeline_report_native_dynamic_payload_string_array_schema_helpers import",
            materialized_text,
            "materialized_packages schema should consume the string-array helper owner directly",
        )
        if "from .pipeline_report_native_dynamic_payload_schema_helpers import (" in materialized_text:
            parent_imports = materialized_text.split(
                "from .pipeline_report_native_dynamic_payload_schema_helpers import (",
                1,
            )[1].split(")", 1)[0]
            for function_name in moved_functions:
                self.assertNotIn(
                    function_name,
                    parent_imports,
                    f"{function_name} should not be imported from the parent helper owner",
                )

        self.assertNotIn(
            "pipeline_report_native_dynamic_payload_schema_helpers",
            string_array_text,
            "string-array helper owner must not import the parent helper owner",
        )

    def test_payload_helper_owners_stay_under_line_budgets(self):
        parent_line_count = len(
            PAYLOAD_SCHEMA_HELPERS.read_text(encoding="utf-8").splitlines()
        )
        string_array_line_count = len(
            PAYLOAD_STRING_ARRAY_HELPERS.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            parent_line_count,
            260,
            "NativeDynamic payload helper parent should stay below 260 lines after string-array split",
        )
        self.assertLess(
            string_array_line_count,
            260,
            "NativeDynamic payload string-array helper owner should stay below 260 lines",
        )


if __name__ == "__main__":
    unittest.main()
