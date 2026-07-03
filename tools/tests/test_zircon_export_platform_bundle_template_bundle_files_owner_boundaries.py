import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template_schema.py"
)
TEMPLATE_BUNDLE_FILES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_bundle_files_schema.py"
)


class ZirconExportPlatformBundleTemplateBundleFilesOwnerBoundaryTests(
    unittest.TestCase
):
    def test_template_bundle_files_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            TEMPLATE_BUNDLE_FILES_SCHEMA.exists(),
            "PlatformBundle embedded template bundle/files schema rules need a dedicated owner",
        )
        template_schema_text = TEMPLATE_SCHEMA.read_text(encoding="utf-8")
        bundle_files_text = TEMPLATE_BUNDLE_FILES_SCHEMA.read_text(encoding="utf-8")

        for name in (
            "PLATFORM_BUNDLE_TEMPLATE_BUNDLE_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_FILE_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_FILE_STRING_FIELDS",
        ):
            self.assertNotIn(
                f"{name} =",
                template_schema_text,
                f"{name} belongs in the bundle/files schema owner",
            )
            self.assertIn(f"{name} =", bundle_files_text)

        for function_name in (
            "template_report_host_executable_membership_diagnostics",
            "template_report_file_source_hash_diagnostics",
            "template_report_content_hash_diagnostics",
        ):
            self.assertNotIn(
                function_name,
                template_schema_text,
                f"{function_name} belongs behind the bundle/files schema owner",
            )
            self.assertIn(function_name, bundle_files_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_bundle_files_schema import",
            template_schema_text,
            "Template schema orchestration should consume the bundle/files schema owner",
        )
        self.assertIn(
            "def platform_bundle_template_bundle_files_schema_diagnostics(",
            bundle_files_text,
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_schema import",
            bundle_files_text,
            "bundle/files schema owner must not import schema orchestration",
        )

    def test_template_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(TEMPLATE_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            400,
            "PlatformBundle template schema owner should stay below 400 lines after bundle/files split",
        )

    def test_bundle_files_schema_owner_stays_under_large_file_threshold(self):
        self.assertTrue(
            TEMPLATE_BUNDLE_FILES_SCHEMA.exists(),
            "PlatformBundle embedded template bundle/files schema rules need a dedicated owner",
        )
        line_count = len(
            TEMPLATE_BUNDLE_FILES_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            220,
            "PlatformBundle template bundle/files schema owner should stay below 220 lines",
        )


if __name__ == "__main__":
    unittest.main()
