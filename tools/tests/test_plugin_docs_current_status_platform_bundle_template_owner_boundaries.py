import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONVERGENCE_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
)
PLATFORM_BUNDLE_TEMPLATE_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_platform_bundle_template_owner_splits.py"
)


class PluginDocsCurrentStatusPlatformBundleTemplateOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_template_docs_guards_live_in_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_TEMPLATE_DOCS_TEST.exists(),
            "PlatformBundle template docs status guards belong in a focused test owner",
        )
        convergence_text = CONVERGENCE_TEST.read_text(encoding="utf-8")
        template_docs_text = PLATFORM_BUNDLE_TEMPLATE_DOCS_TEST.read_text(
            encoding="utf-8"
        )

        moved_markers = (
            "test_current_export_plan_reflects_platform_bundle_template_resolution_row_schema_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_manifest_identity_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_report_semantics_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_resolution_path_semantics_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_copied_files_schema_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_bundle_files_schema_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_resolution_candidate_semantics_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_schema_path_helper_owner_split",
            "test_current_export_plan_reflects_platform_bundle_template_manifest_files_owner_split",
            "test_current_plugin_docs_reflect_platform_bundle_template_files_materialize_owner_split",
            "test_current_plugin_docs_reflect_platform_bundle_template_resolution_failure_semantics_owner_split",
        )
        for marker in moved_markers:
            self.assertNotIn(
                marker,
                convergence_text,
                f"{marker} should move out of the broad convergence test owner",
            )
            self.assertIn(
                marker,
                template_docs_text,
                f"{marker} should be covered by the PlatformBundle template docs owner",
            )

        self.assertLessEqual(
            len(convergence_text.splitlines()),
            5500,
            "the broad current-status docs convergence test must keep shrinking",
        )
        self.assertLessEqual(
            len(template_docs_text.splitlines()),
            420,
            "the PlatformBundle template docs status owner should stay data-driven",
        )


if __name__ == "__main__":
    unittest.main()
