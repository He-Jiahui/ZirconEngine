import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_native_dynamic_report_support import (
    assert_required_phrases,
    load_native_dynamic_report_sections,
)


class PluginDocsCurrentStatusNativeDynamicReportSchemaOwnerSplitTests(
    unittest.TestCase
):
    def setUp(self) -> None:
        self.sections = load_native_dynamic_report_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_native_dynamic_package_report_schema_helper_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_package_report_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_package_report_schema_helpers.py",
                    "NativeDynamic package report schema helper owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_package_report_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_package_report_schema_helpers.py",
                    "NativeDynamic package report schema helper owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_package_report_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_package_report_schema_helpers.py",
                    "package report reusable field diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_package_report_schema_helpers.py",
                    "NativeDynamic package report schema helper owner",
                    "package report reusable field diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_package_report_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_package_report_schema_helpers.py",
                    "NativeDynamic package report schema helper owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic package report schema helper owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_build_execution_packages_schema_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_packages_schema_owner_split",
                    "pipeline_report_native_dynamic_build_execution_packages_schema.py",
                    "NativeDynamic build execution packages schema owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_packages_schema_owner_split",
                    "pipeline_report_native_dynamic_build_execution_packages_schema.py",
                    "NativeDynamic build execution packages schema owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_packages_schema_owner_split",
                    "pipeline_report_native_dynamic_build_execution_packages_schema.py",
                    "BuildExecution packages row schema diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_build_execution_packages_schema.py",
                    "NativeDynamic build execution packages schema owner",
                    "BuildExecution packages row schema diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_packages_schema_owner_split",
                    "pipeline_report_native_dynamic_build_execution_packages_schema.py",
                    "NativeDynamic build execution packages schema owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic build execution packages schema owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_build_plan_schema_helper_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_plan_schema_helpers.py",
                    "NativeDynamic build plan schema helper owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_plan_schema_helpers.py",
                    "NativeDynamic build plan schema helper owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_plan_schema_helpers.py",
                    "BuildPlan packages row and reusable field diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_build_plan_schema_helpers.py",
                    "NativeDynamic build plan schema helper owner",
                    "BuildPlan packages row and reusable field diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_plan_schema_helpers.py",
                    "NativeDynamic build plan schema helper owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic build plan schema helper owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_operation_audit_stage_packages_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_operation_audit_stage_packages_owner_split",
                    "pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py",
                    "NativeDynamic operation audit stage packages owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_operation_audit_stage_packages_owner_split",
                    "pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py",
                    "NativeDynamic operation audit stage packages owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_operation_audit_stage_packages_owner_split",
                    "pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py",
                    "packages[].artifacts[] schema diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py",
                    "NativeDynamic operation audit stage packages owner",
                    "packages[].artifacts[] schema diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_operation_audit_stage_packages_owner_split",
                    "pipeline_report_native_dynamic_operation_audit_stage_packages_schema.py",
                    "NativeDynamic operation audit stage packages owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic operation audit stage packages owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_build_plan_package_details_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_package_details_owner_split",
                    "pipeline_report_native_dynamic_build_plan_package_details.py",
                    "NativeDynamic build-plan package details owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_package_details_owner_split",
                    "pipeline_report_native_dynamic_build_plan_package_details.py",
                    "NativeDynamic build-plan package details owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_package_details_owner_split",
                    "pipeline_report_native_dynamic_build_plan_package_details.py",
                    "header match and expected artifact diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_build_plan_package_details.py",
                    "NativeDynamic build-plan package details owner",
                    "header match and expected artifact diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_build_plan_package_details_owner_split",
                    "pipeline_report_native_dynamic_build_plan_package_details.py",
                    "NativeDynamic build-plan package details owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic build-plan package details owner split",
        )


if __name__ == "__main__":
    unittest.main()
