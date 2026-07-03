import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_schema_helpers.py"
)
PATH_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_path_schema_helpers.py"
)
TEMPLATE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template_schema.py"
)
TEMPLATE_BUNDLE_FILES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_bundle_files_schema.py"
)


class ZirconExportPlatformBundleTemplatePathSchemaHelperOwnerTests(
    unittest.TestCase
):
    def test_path_hash_schema_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            PATH_SCHEMA_HELPERS.exists(),
            "PlatformBundle template path/hash schema diagnostics need a dedicated owner",
        )
        schema_helpers_text = SCHEMA_HELPERS.read_text(encoding="utf-8")
        path_helpers_text = PATH_SCHEMA_HELPERS.read_text(encoding="utf-8")

        for function_name in (
            "table_sha256_hex_string_diagnostics",
            "sequence_sha256_hex_string_diagnostics",
            "table_safe_relative_path_string_diagnostics",
            "table_bundle_path_string_diagnostics",
            "sequence_safe_relative_path_string_diagnostics",
            "sequence_unique_path_diagnostics",
            "sequence_unique_relative_path_field_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_helpers_text,
                f"{function_name} belongs in the path/hash schema helper owner",
            )
            self.assertIn(f"def {function_name}(", path_helpers_text)

        for helper_name in (
            "is_safe_relative_path",
            "is_sha256_hex",
            "normalize_relative_path",
        ):
            self.assertNotIn(
                helper_name,
                schema_helpers_text,
                f"{helper_name} should not leak into the generic schema helper owner",
            )
            self.assertIn(helper_name, path_helpers_text)

        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_schema import",
            path_helpers_text,
            "path/hash schema helpers must not import template schema orchestration",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_bundle_files_schema import",
            path_helpers_text,
            "path/hash schema helpers must not import bundle/files schema orchestration",
        )

    def test_path_hash_schema_consumers_import_dedicated_owner_directly(self):
        template_schema_text = TEMPLATE_SCHEMA.read_text(encoding="utf-8")
        bundle_files_text = TEMPLATE_BUNDLE_FILES_SCHEMA.read_text(encoding="utf-8")

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_path_schema_helpers import",
            template_schema_text,
            "template schema should consume path/hash helpers directly",
        )
        self.assertIn(
            "from .pipeline_report_platform_bundle_template_path_schema_helpers import",
            bundle_files_text,
            "bundle/files schema should consume path/hash helpers directly",
        )
        self.assertIn("table_sha256_hex_string_diagnostics", template_schema_text)
        for function_name in (
            "sequence_safe_relative_path_string_diagnostics",
            "sequence_sha256_hex_string_diagnostics",
            "sequence_unique_path_diagnostics",
            "sequence_unique_relative_path_field_diagnostics",
            "table_bundle_path_string_diagnostics",
        ):
            self.assertIn(function_name, bundle_files_text)

    def test_schema_helper_owners_stay_under_large_file_thresholds(self):
        generic_line_count = len(SCHEMA_HELPERS.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            generic_line_count,
            430,
            "Generic PlatformBundle template schema helpers should stay below 430 lines "
            "after path/hash split",
        )
        self.assertTrue(
            PATH_SCHEMA_HELPERS.exists(),
            "PlatformBundle template path/hash schema diagnostics need a dedicated owner",
        )
        path_line_count = len(PATH_SCHEMA_HELPERS.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            path_line_count,
            180,
            "PlatformBundle template path/hash schema helper owner should stay below 180 lines",
        )


if __name__ == "__main__":
    unittest.main()
