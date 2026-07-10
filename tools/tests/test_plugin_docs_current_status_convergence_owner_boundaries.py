"""Boundary tests for current-status plugin docs guard ownership."""

from __future__ import annotations

import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_GUARD = REPO_ROOT / "tools/tests/test_plugin_docs_current_status_convergence.py"
OWNER_FILES = {
    "native dynamic report/schema": REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_native_dynamic_report_owner_splits.py",
    "native dynamic build/materialize": REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_native_dynamic_build_owner_splits.py",
    "platform bundle": REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_platform_bundle_owner_splits.py",
    "source template/compile host": REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_source_template_compile_host_owner_splits.py",
    "export template/cook assets": REPO_ROOT
    / "tools/tests/test_plugin_docs_current_status_export_template_cook_assets_owner_splits.py",
}

MOVED_METHODS = (
    "test_current_export_plan_reflects_native_dynamic_report_owner_splits",
    "test_current_export_plan_reflects_platform_bundle_materialize_owner_split",
    "test_current_export_plan_reflects_platform_bundle_native_plugins_payload_owner_split",
    "test_current_export_plan_reflects_platform_bundle_stage_handoff_report_owner_split",
    "test_current_export_plan_reflects_native_dynamic_materialize_owner_split",
    "test_current_export_plan_reflects_export_template_manifest_owner_split",
    "test_current_export_plan_reflects_cook_assets_report_owner_split",
    "test_current_export_plan_reflects_platform_bundle_file_evidence_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_loader_manifest_owner_split",
    "test_current_export_plan_reflects_cook_assets_manifest_owner_split",
    "test_current_export_plan_reflects_compile_host_plan_owner_split",
    "test_current_export_plan_reflects_source_template_generated_files_owner_split",
    "test_current_export_plan_reflects_source_template_build_handoff_owner_split",
    "test_current_export_plan_reflects_source_template_generated_project_owner_split",
    "test_current_export_plan_reflects_pipeline_report_compile_host_owner_split",
    "test_current_export_plan_reflects_validate_compile_host_semantics_owner_split",
    "test_current_export_plan_reflects_native_dynamic_package_report_schema_helper_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_execution_packages_schema_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_plan_schema_helper_owner_split",
    "test_current_export_plan_reflects_native_build_workspace_owner_split",
    "test_current_export_plan_reflects_native_build_cargo_command_owner_split",
    "test_current_export_plan_reflects_native_dynamic_cli_options_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_package_report_owner_split",
    "test_current_export_plan_reflects_native_dynamic_operation_audit_stage_packages_owner_split",
    "test_current_export_plan_reflects_platform_bundle_argument_path_owner_split",
    "test_current_export_plan_reflects_cli_argument_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_payload_finalize_owner_split",
    "test_current_export_plan_reflects_native_dynamic_materialize_io_owner_split",
    "test_current_export_plan_reflects_export_template_resolution_owner_split",
    "test_current_export_plan_reflects_platform_bundle_strategy_handoff_owner_split",
    "test_current_export_plan_reflects_stage_handoff_strategy_owner_split",
    "test_current_export_plan_reflects_native_dynamic_stage_payload_operation_audit_owner_split",
    "test_current_export_plan_reflects_cook_assets_pack_trim_closure_owner_split",
    "test_current_export_plan_reflects_cook_assets_project_fallback_owner_split",
    "test_current_export_plan_reflects_native_dynamic_build_plan_package_details_owner_split",
    "test_current_plugin_docs_reflect_schema_string_array_owner_split",
    "test_current_plugin_docs_reflect_platform_bundle_report_payload_owner_split",
    "test_current_export_plan_reflects_validate_compile_host_command_semantics_owner_split",
    "test_current_export_plan_reflects_compile_host_plan_command_semantics_owner_split",
    "test_current_plugin_docs_reflect_platform_bundle_native_plugins_materialize_owner_split",
    "test_current_plugin_docs_reflect_source_template_plan_command_owner_split",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PluginDocsCurrentStatusConvergenceOwnerBoundaryTests(unittest.TestCase):
    def test_root_guard_stays_as_authority_status_owner(self):
        root_text = ROOT_GUARD.read_text(encoding="utf-8")

        self.assertLess(
            _line_count(ROOT_GUARD),
            1000,
            "root current-status convergence guard should stay below 1000 lines",
        )
        self.assertIn(
            "test_current_plugin_authority_docs_reflect_validate_all_and_no_stale_rollout_pending",
            root_text,
        )
        for method_name in MOVED_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(f"def {method_name}", root_text)

    def test_focused_owner_files_exist_and_stay_small(self):
        for owner_name, path in OWNER_FILES.items():
            with self.subTest(owner=owner_name):
                self.assertTrue(path.exists(), f"{owner_name} guard owner is missing")
                self.assertLess(
                    _line_count(path),
                    1000,
                    f"{owner_name} guard owner should stay below 1000 lines",
                )


if __name__ == "__main__":
    unittest.main()
