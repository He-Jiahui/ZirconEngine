import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_template_schema.py"
)
TEMPLATE_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_report_semantics.py"
)


class ZirconExportPlatformBundleTemplateReportOwnerBoundaryTests(unittest.TestCase):
    def test_template_report_semantic_diagnostics_live_in_semantics_owner(self):
        self.assertTrue(
            TEMPLATE_SEMANTICS.exists(),
            "PlatformBundle template report semantic diagnostics need a dedicated owner",
        )
        schema_text = TEMPLATE_SCHEMA.read_text(encoding="utf-8")
        semantics_text = TEMPLATE_SEMANTICS.read_text(encoding="utf-8")

        for function_name in (
            "template_report_identity_match_diagnostics",
            "template_report_profile_membership_diagnostics",
            "template_report_file_entry_is_schema_clean",
            "template_report_file_source_hash_diagnostics",
            "template_report_content_hash_diagnostics",
            "template_report_host_executable_membership_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the PlatformBundle template report semantics owner",
            )
            self.assertIn(f"def {function_name}(", semantics_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_report_semantics import",
            schema_text,
            "Template schema orchestration should consume the report semantics owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_schema import",
            semantics_text,
            "report semantics owner must not import schema orchestration",
        )

    def test_template_report_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(TEMPLATE_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            620,
            "PlatformBundle template schema owner should stay below 620 lines after semantics split",
        )


if __name__ == "__main__":
    unittest.main()
