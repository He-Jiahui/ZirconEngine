import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_MANIFEST_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_manifest_schema.py"
)
TEMPLATE_MANIFEST_FILES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_manifest_files_schema.py"
)
TEMPLATE_MANIFEST_LOADER = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_manifest_loader.py"
)


class ZirconExportPlatformBundleTemplateManifestFilesOwnerBoundaryTests(
    unittest.TestCase
):
    def test_template_manifest_loading_lives_in_dedicated_owner(self):
        self.assertTrue(
            TEMPLATE_MANIFEST_LOADER.exists(),
            "PlatformBundle template manifest loading needs a dedicated owner",
        )
        schema_text = TEMPLATE_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        loader_text = TEMPLATE_MANIFEST_LOADER.read_text(encoding="utf-8")

        self.assertNotIn("import tomllib", schema_text)
        self.assertNotIn("def template_report_manifest_load(", schema_text)
        self.assertIn("import tomllib", loader_text)
        self.assertIn("def template_report_manifest_load(", loader_text)
        self.assertIn(
            "from .pipeline_report_platform_bundle_template_manifest_loader import",
            schema_text,
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_manifest_schema import",
            loader_text,
        )
        self.assertLess(
            len(loader_text.splitlines()),
            100,
            "template manifest loader should remain a focused leaf",
        )

    def test_template_manifest_files_schema_lives_in_dedicated_owner(self):
        self.assertTrue(
            TEMPLATE_MANIFEST_FILES_SCHEMA.exists(),
            "PlatformBundle template manifest [[files]] rules need a dedicated owner",
        )
        schema_text = TEMPLATE_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        files_schema_text = TEMPLATE_MANIFEST_FILES_SCHEMA.read_text(encoding="utf-8")

        for function_name in (
            "template_manifest_files_schema_diagnostic",
            "template_manifest_files_presence_diagnostic",
            "template_manifest_files_unique_diagnostic",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the PlatformBundle template manifest files owner",
            )
            self.assertIn(f"def {function_name}(", files_schema_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_manifest_files_schema import",
            schema_text,
            "Template manifest schema orchestration should consume the files owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_manifest_schema import",
            files_schema_text,
            "files owner must not import schema orchestration",
        )

    def test_template_manifest_files_schema_owners_stay_under_file_budgets(self):
        schema_line_count = len(
            TEMPLATE_MANIFEST_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            schema_line_count,
            360,
            "PlatformBundle template manifest schema owner should stay below 360 lines after files split",
        )
        self.assertTrue(
            TEMPLATE_MANIFEST_FILES_SCHEMA.exists(),
            "PlatformBundle template manifest [[files]] rules need a dedicated owner",
        )
        files_schema_line_count = len(
            TEMPLATE_MANIFEST_FILES_SCHEMA.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            files_schema_line_count,
            140,
            "PlatformBundle template manifest files schema owner should stay below 140 lines",
        )


if __name__ == "__main__":
    unittest.main()
