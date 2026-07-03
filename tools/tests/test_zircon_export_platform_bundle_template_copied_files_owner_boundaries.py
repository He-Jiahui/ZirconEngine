import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template_schema.py"
)
TEMPLATE_COPIED_FILES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_copied_files_schema.py"
)
PLATFORM_BUNDLE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_schema.py"
)
PLATFORM_BUNDLE_TEMPLATE = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template.py"
)


class ZirconExportPlatformBundleTemplateCopiedFilesOwnerBoundaryTests(
    unittest.TestCase
):
    def test_template_copied_files_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            TEMPLATE_COPIED_FILES_SCHEMA.exists(),
            "PlatformBundle template copied-files schema rules need a dedicated owner",
        )
        template_schema_text = TEMPLATE_SCHEMA.read_text(encoding="utf-8")
        copied_files_text = TEMPLATE_COPIED_FILES_SCHEMA.read_text(encoding="utf-8")
        platform_bundle_schema_text = PLATFORM_BUNDLE_SCHEMA.read_text(encoding="utf-8")
        platform_bundle_template_text = PLATFORM_BUNDLE_TEMPLATE.read_text(
            encoding="utf-8"
        )

        for name in (
            "PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS",
            "PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS",
        ):
            self.assertNotIn(
                f"{name} =",
                template_schema_text,
                f"{name} belongs in the copied-files schema owner",
            )
            self.assertIn(f"{name} =", copied_files_text)

        self.assertNotIn(
            "def platform_bundle_template_copied_files_schema_diagnostics(",
            template_schema_text,
            "copied-files schema diagnostics belong in the copied-files owner",
        )
        self.assertIn(
            "def platform_bundle_template_copied_files_schema_diagnostics(",
            copied_files_text,
        )
        self.assertIn(
            "from .pipeline_report_platform_bundle_template_copied_files_schema import",
            platform_bundle_schema_text,
            "PlatformBundle schema should consume the copied-files schema owner directly",
        )
        self.assertIn(
            "from .pipeline_report_platform_bundle_template_copied_files_schema import",
            platform_bundle_template_text,
            "PlatformBundle template diagnostics should consume the copied-files schema owner directly",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_schema import",
            copied_files_text,
            "copied-files schema owner must not import the template schema owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_schema import",
            copied_files_text,
            "copied-files schema owner must not import PlatformBundle schema orchestration",
        )

    def test_template_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(TEMPLATE_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            510,
            "PlatformBundle template schema owner should stay below 510 lines after copied-files split",
        )


if __name__ == "__main__":
    unittest.main()
