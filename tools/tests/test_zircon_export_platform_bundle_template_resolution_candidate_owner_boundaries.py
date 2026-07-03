import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESOLUTION_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_semantics.py"
)
RESOLUTION_CANDIDATE_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_candidate_semantics.py"
)
RESOLUTION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_schema.py"
)


class ZirconExportPlatformBundleTemplateResolutionCandidateOwnerBoundaryTests(
    unittest.TestCase
):
    def test_resolution_candidate_semantics_live_in_dedicated_owner(self):
        self.assertTrue(
            RESOLUTION_CANDIDATE_SEMANTICS.exists(),
            "PlatformBundle template resolution candidate semantics need a dedicated owner",
        )
        semantics_text = RESOLUTION_SEMANTICS.read_text(encoding="utf-8")
        candidate_text = RESOLUTION_CANDIDATE_SEMANTICS.read_text(encoding="utf-8")
        schema_text = RESOLUTION_SCHEMA.read_text(encoding="utf-8")

        for function_name in (
            "template_resolution_candidate_profile_diagnostics",
            "template_resolution_candidate_identity_diagnostics",
            "template_resolution_candidate_bundle_format_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                semantics_text,
                f"{function_name} belongs in the resolution candidate semantics owner",
            )
            self.assertIn(
                f"def {function_name}(",
                candidate_text,
            )

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_resolution_candidate_semantics import",
            schema_text,
            "resolution schema should consume the candidate semantics owner directly",
        )
        self.assertNotIn(
            "EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS",
            semantics_text,
            "candidate bundle-format constants should not remain in resolution semantics",
        )
        self.assertNotIn(
            "normalize_target_platform",
            semantics_text,
            "candidate target-platform normalization should not remain in resolution semantics",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_semantics import",
            candidate_text,
            "candidate semantics owner must not import the resolution semantics owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_schema import",
            candidate_text,
            "candidate semantics owner must not import schema orchestration",
        )

    def test_resolution_semantics_owner_stays_under_large_file_threshold(self):
        line_count = len(RESOLUTION_SEMANTICS.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            460,
            "PlatformBundle template resolution semantics owner should stay below 460 lines after candidate split",
        )


if __name__ == "__main__":
    unittest.main()
