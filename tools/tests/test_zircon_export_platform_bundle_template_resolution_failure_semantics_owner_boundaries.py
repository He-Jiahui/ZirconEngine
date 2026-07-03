import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESOLUTION_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_semantics.py"
)
FAILURE_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_failure_semantics.py"
)
RESOLUTION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_platform_bundle_template_resolution_schema.py"
)


class ZirconExportPlatformBundleTemplateResolutionFailureSemanticsOwnerTests(
    unittest.TestCase
):
    def test_failure_semantics_live_in_dedicated_owner(self):
        self.assertTrue(
            FAILURE_SEMANTICS.exists(),
            "PlatformBundle template resolution failure semantics need a dedicated owner",
        )
        semantics_text = RESOLUTION_SEMANTICS.read_text(encoding="utf-8")
        failure_text = FAILURE_SEMANTICS.read_text(encoding="utf-8")
        schema_text = RESOLUTION_SCHEMA.read_text(encoding="utf-8")

        for function_name in (
            "template_resolution_fatal_candidate_count_diagnostics",
            "template_resolution_fatal_diagnostics_diagnostics",
            "template_resolution_fatal_diagnostic_family_diagnostics",
            "template_resolution_fatal_multiple_candidate_diagnostics",
            "template_resolution_fatal_no_candidate_diagnostics",
            "template_resolution_no_match_profile_diagnostics",
            "template_resolution_no_match_identity_diagnostics",
            "template_resolution_no_match_root_diagnostics",
            "template_resolution_root_failure_candidate_diagnostics",
            "template_resolution_root_failure_root_diagnostics",
            "template_resolution_object_row_count",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                semantics_text,
                f"{function_name} belongs in the failure semantics owner",
            )
            self.assertIn(f"def {function_name}(", failure_text)

        for diagnostic_marker in (
            "multiple export templates matched profile=",
            "no export template under ",
            "export template root ",
        ):
            self.assertNotIn(
                diagnostic_marker,
                semantics_text,
                f"{diagnostic_marker} diagnostics belong in the failure owner",
            )
            self.assertIn(diagnostic_marker, failure_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_template_resolution_failure_semantics import",
            schema_text,
            "resolution schema should consume the failure semantics owner directly",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_semantics import",
            failure_text,
            "failure semantics owner must not import resolution semantics",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle_template_resolution_schema import",
            failure_text,
            "failure semantics owner must not import schema orchestration",
        )

    def test_resolution_semantics_and_failure_owner_stay_small(self):
        semantics_lines = len(
            RESOLUTION_SEMANTICS.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            semantics_lines,
            260,
            "template resolution semantics owner should stay below 260 lines "
            "after failure split",
        )
        self.assertTrue(FAILURE_SEMANTICS.exists())
        failure_lines = len(FAILURE_SEMANTICS.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            failure_lines,
            260,
            "template resolution failure semantics owner should stay below 260 lines",
        )


if __name__ == "__main__":
    unittest.main()
