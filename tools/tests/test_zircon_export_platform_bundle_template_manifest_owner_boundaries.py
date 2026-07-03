import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_MANIFEST_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_manifest_schema.py"
)
TEMPLATE_MANIFEST_IDENTITY = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_manifest_identity.py"
)


class ZirconExportPlatformBundleTemplateManifestOwnerBoundaryTests(unittest.TestCase):
    def test_template_manifest_identity_diagnostics_live_in_identity_owner(self):
        self.assertTrue(
            TEMPLATE_MANIFEST_IDENTITY.exists(),
            "PlatformBundle template manifest identity checks need a dedicated owner",
        )
        schema_text = TEMPLATE_MANIFEST_SCHEMA.read_text(encoding="utf-8")
        identity_text = TEMPLATE_MANIFEST_IDENTITY.read_text(encoding="utf-8")

        for function_name in (
            "template_manifest_identity_diagnostic",
            "template_manifest_string_identity_diagnostic",
            "template_manifest_target_platform_identity_diagnostic",
            "template_manifest_compatible_profiles_identity_diagnostic",
            "template_manifest_host_executable_identity_diagnostic",
            "template_manifest_bundle_identity_diagnostic",
            "template_bundle_identity_value_is_schema_clean",
            "template_manifest_files_identity_diagnostic",
            "template_manifest_file_entry_identity_diagnostic",
            "template_manifest_file_bundle_path",
            "template_manifest_file_string_field_identity_diagnostic",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the PlatformBundle template manifest identity owner",
            )
            self.assertIn(f"def {function_name}(", identity_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_manifest_identity import",
            schema_text,
            "Template manifest schema orchestration should consume the identity owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_manifest_schema import",
            identity_text,
            "identity owner must not import schema orchestration",
        )

    def test_template_manifest_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(TEMPLATE_MANIFEST_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            620,
            "PlatformBundle template manifest schema owner should stay below 620 lines after identity split",
        )


if __name__ == "__main__":
    unittest.main()
