import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONVERGENCE_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
)
NATIVE_DYNAMIC_PAYLOAD_DOCS_TEST = (
    REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_native_dynamic_payload_owner_splits.py"
)


class PluginDocsCurrentStatusNativeDynamicPayloadOwnerBoundaryTests(unittest.TestCase):
    def test_native_dynamic_payload_docs_guards_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_PAYLOAD_DOCS_TEST.exists(),
            "NativeDynamic payload docs status guards belong in a focused test owner",
        )
        convergence_text = CONVERGENCE_TEST.read_text(encoding="utf-8")
        payload_docs_text = NATIVE_DYNAMIC_PAYLOAD_DOCS_TEST.read_text(encoding="utf-8")

        moved_markers = (
            "test_current_plugin_docs_reflect_native_dynamic_payload_file_manifest_schema_owner_split",
            "test_current_plugin_docs_reflect_native_dynamic_payload_materialized_packages_schema_owner_split",
            "test_current_plugin_docs_reflect_native_dynamic_payload_package_path_owner_split",
            "test_current_plugin_docs_reflect_native_dynamic_payload_bundle_evidence_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_file_manifest_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_loader_manifest_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_platform_bundle_handoff_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_platform_bundle_stage_report_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_operation_audit_summary_owner_split",
            "test_current_export_plan_reflects_native_dynamic_payload_directory_owner_split",
        )
        for marker in moved_markers:
            self.assertNotIn(
                marker,
                convergence_text,
                f"{marker} should move out of the broad convergence test owner",
            )
            self.assertIn(
                marker,
                payload_docs_text,
                f"{marker} should be covered by the NativeDynamic payload docs owner",
            )

        self.assertLessEqual(
            len(convergence_text.splitlines()),
            4600,
            "the broad current-status docs convergence test must keep shrinking",
        )
        self.assertLessEqual(
            len(payload_docs_text.splitlines()),
            420,
            "the NativeDynamic payload docs status owner should stay data-driven",
        )


if __name__ == "__main__":
    unittest.main()
