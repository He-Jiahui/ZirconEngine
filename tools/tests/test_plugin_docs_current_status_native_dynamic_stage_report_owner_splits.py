import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_native_dynamic_report_support import (
    assert_required_phrases,
    load_native_dynamic_report_sections,
)


class PluginDocsCurrentStatusNativeDynamicStageReportOwnerSplitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_native_dynamic_report_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_native_dynamic_stage_loader_manifest_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_stage_loader_manifest_owner_split",
                    "pipeline_report_native_dynamic_stage_loader_manifest.py",
                    "NativeDynamic stage loader manifest owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_stage_loader_manifest_owner_split",
                    "pipeline_report_native_dynamic_stage_loader_manifest.py",
                    "NativeDynamic stage loader manifest owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_stage_loader_manifest_owner_split",
                    "pipeline_report_native_dynamic_stage_loader_manifest.py",
                    "loader manifest package diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_stage_loader_manifest.py",
                    "NativeDynamic stage loader manifest owner",
                    "loader manifest package diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_stage_loader_manifest_owner_split",
                    "pipeline_report_native_dynamic_stage_loader_manifest.py",
                    "NativeDynamic stage loader manifest owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic stage loader manifest owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_stage_package_report_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_stage_package_report_owner_split",
                    "pipeline_report_native_dynamic_stage_package_report.py",
                    "NativeDynamic stage package report owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_stage_package_report_owner_split",
                    "pipeline_report_native_dynamic_stage_package_report.py",
                    "NativeDynamic stage package report owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_stage_package_report_owner_split",
                    "pipeline_report_native_dynamic_stage_package_report.py",
                    "NativeDynamic materialized package source/package-report diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_stage_package_report.py",
                    "NativeDynamic stage package report owner",
                    "NativeDynamic materialized package source/package-report diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_stage_package_report_owner_split",
                    "pipeline_report_native_dynamic_stage_package_report.py",
                    "NativeDynamic stage package report owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic stage package report owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_stage_payload_finalize_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_finalize_owner_split",
                    "native_dynamic_stage_payload_finalize.py",
                    "NativeDynamic stage payload finalization owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_finalize_owner_split",
                    "native_dynamic_stage_payload_finalize.py",
                    "NativeDynamic stage payload finalization owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_finalize_owner_split",
                    "native_dynamic_stage_payload_finalize.py",
                    "package report/loader manifest/file manifest finalization",
                ],
                "export tooling docs": [
                    "native_dynamic_stage_payload_finalize.py",
                    "NativeDynamic stage payload finalization owner",
                    "package report/loader manifest/file manifest finalization",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_finalize_owner_split",
                    "native_dynamic_stage_payload_finalize.py",
                    "NativeDynamic stage payload finalization owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic stage payload finalization owner split",
        )

    def test_current_export_plan_reflects_native_dynamic_stage_payload_operation_audit_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_operation_audit_owner_split",
                    "pipeline_report_native_dynamic_stage_payload_operation_audit.py",
                    "NativeDynamic stage payload operation-audit artifact owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_operation_audit_owner_split",
                    "pipeline_report_native_dynamic_stage_payload_operation_audit.py",
                    "NativeDynamic stage payload operation-audit artifact owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_operation_audit_owner_split",
                    "pipeline_report_native_dynamic_stage_payload_operation_audit.py",
                    "operation-audit artifact/package-relative diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_stage_payload_operation_audit.py",
                    "NativeDynamic stage payload operation-audit artifact owner",
                    "operation-audit artifact/package-relative diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_native_dynamic_stage_payload_operation_audit_owner_split",
                    "pipeline_report_native_dynamic_stage_payload_operation_audit.py",
                    "NativeDynamic stage payload operation-audit artifact owner",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic stage payload operation-audit owner split",
        )


if __name__ == "__main__":
    unittest.main()
