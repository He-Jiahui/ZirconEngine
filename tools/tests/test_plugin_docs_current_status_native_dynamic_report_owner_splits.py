import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_native_dynamic_report_support import (
    assert_required_phrases,
    load_native_dynamic_report_sections,
)


class PluginDocsCurrentStatusNativeDynamicReportOwnerSplitTests(unittest.TestCase):
    def test_current_export_plan_reflects_native_dynamic_report_owner_splits(self):
        sections = load_native_dynamic_report_sections(Path(__file__).resolve().parents[2])

        assert_required_phrases(
            self,
            sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_owner_split",
                    "plugins_13_m5_t1_native_dynamic_build_plan_command_owner_split",
                    "plugins_13_m5_t1_native_dynamic_operation_audit_schema_helper_owner_split",
                    "plugins_13_m5_t1_native_dynamic_payload_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_execution.py",
                    "pipeline_report_native_dynamic_build_plan_commands.py",
                    "pipeline_report_native_dynamic_operation_audit_schema_helpers.py",
                    "pipeline_report_native_dynamic_payload_schema_helpers.py",
                    "stage payload owner",
                    "build-plan schema owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_native_dynamic_build_execution_owner_split",
                    "plugins_13_m5_t1_native_dynamic_build_plan_command_owner_split",
                    "plugins_13_m5_t1_native_dynamic_operation_audit_schema_helper_owner_split",
                    "plugins_13_m5_t1_native_dynamic_payload_schema_helper_owner_split",
                    "pipeline_report_native_dynamic_build_execution.py",
                    "pipeline_report_native_dynamic_build_plan_commands.py",
                    "pipeline_report_native_dynamic_operation_audit_schema_helpers.py",
                    "pipeline_report_native_dynamic_payload_schema_helpers.py",
                ],
                "export tooling docs": [
                    "pipeline_report_native_dynamic_build_execution.py",
                    "pipeline_report_native_dynamic_build_plan_commands.py",
                    "pipeline_report_native_dynamic_operation_audit_schema_helpers.py",
                    "pipeline_report_native_dynamic_payload_schema_helpers.py",
                    "build execution report diagnostics",
                    "Cargo command semantics",
                    "Operation-audit helper diagnostics",
                    "Payload schema helper diagnostics",
                ],
            },
            "Current export/plugin docs do not reflect NativeDynamic report owner splits",
        )


if __name__ == "__main__":
    unittest.main()
