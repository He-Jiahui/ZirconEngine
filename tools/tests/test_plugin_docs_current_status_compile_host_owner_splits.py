import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_source_template_compile_host_support import (
    assert_required_phrases,
    load_source_template_compile_host_sections,
)


class PluginDocsCurrentStatusCompileHostOwnerSplitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_source_template_compile_host_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_compile_host_plan_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_compile_host_plan_owner_split",
                    "compile_host_plan.py",
                    "CompileHost plan owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_compile_host_plan_owner_split",
                    "compile_host_plan.py",
                    "CompileHost plan owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_compile_host_plan_owner_split",
                    "compile_host_plan.py",
                    "plan/evidence diagnostics",
                ],
                "export tooling docs": [
                    "compile_host_plan.py",
                    "CompileHost plan owner",
                    "plan/evidence diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_compile_host_plan_owner_split",
                    "compile_host_plan.py",
                    "CompileHost plan owner",
                ],
            },
            "Current export/plugin docs do not reflect CompileHost plan owner split",
        )

    def test_current_export_plan_reflects_pipeline_report_compile_host_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_pipeline_report_compile_host_owner_split",
                    "pipeline_report_compile_host.py",
                    "Pipeline Report CompileHost owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_pipeline_report_compile_host_owner_split",
                    "pipeline_report_compile_host.py",
                    "Pipeline Report CompileHost owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_pipeline_report_compile_host_owner_split",
                    "pipeline_report_compile_host.py",
                    "CompileHost final Report diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_compile_host.py",
                    "Pipeline Report CompileHost owner",
                    "CompileHost final Report diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_pipeline_report_compile_host_owner_split",
                    "pipeline_report_compile_host.py",
                    "Pipeline Report CompileHost owner",
                ],
            },
            "Current export/plugin docs do not reflect Pipeline Report CompileHost owner split",
        )

    def test_current_export_plan_reflects_validate_compile_host_semantics_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_validate_compile_host_semantics_owner_split",
                    "pipeline_report_validate_compile_host_semantics.py",
                    "Validate CompileHost semantics owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_validate_compile_host_semantics_owner_split",
                    "pipeline_report_validate_compile_host_semantics.py",
                    "Validate CompileHost semantics owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_validate_compile_host_semantics_owner_split",
                    "pipeline_report_validate_compile_host_semantics.py",
                    "Validate CompileHost identity semantics",
                ],
                "export tooling docs": [
                    "pipeline_report_validate_compile_host_semantics.py",
                    "Validate CompileHost semantics owner",
                    "Validate CompileHost identity semantics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_validate_compile_host_semantics_owner_split",
                    "pipeline_report_validate_compile_host_semantics.py",
                    "Validate CompileHost semantics owner",
                ],
            },
            "Current export/plugin docs do not reflect Validate CompileHost semantics owner split",
        )

    def test_current_export_plan_reflects_validate_compile_host_command_semantics_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_validate_compile_host_command_semantics_owner_split",
                    "pipeline_report_validate_compile_host_command_semantics.py",
                    "Validate CompileHost command semantics owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_validate_compile_host_command_semantics_owner_split",
                    "pipeline_report_validate_compile_host_command_semantics.py",
                    "Validate CompileHost command semantics owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_validate_compile_host_command_semantics_owner_split",
                    "pipeline_report_validate_compile_host_command_semantics.py",
                    "Cargo command semantic diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_validate_compile_host_command_semantics.py",
                    "Validate CompileHost command semantics owner",
                    "Cargo command semantic diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_validate_compile_host_command_semantics_owner_split",
                    "pipeline_report_validate_compile_host_command_semantics.py",
                    "Validate CompileHost command semantics owner",
                ],
            },
            "Current plugin docs do not reflect Validate CompileHost command semantics owner split",
        )

    def test_current_export_plan_reflects_compile_host_plan_command_semantics_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_compile_host_plan_command_semantics_owner_split",
                    "compile_host_plan_command_semantics.py",
                    "CompileHost plan command semantics owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_compile_host_plan_command_semantics_owner_split",
                    "compile_host_plan_command_semantics.py",
                    "CompileHost plan command semantics owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_compile_host_plan_command_semantics_owner_split",
                    "compile_host_plan_command_semantics.py",
                    "plan-side Cargo command semantic diagnostics",
                ],
                "export tooling docs": [
                    "compile_host_plan_command_semantics.py",
                    "CompileHost plan command semantics owner",
                    "plan-side Cargo command semantic diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_compile_host_plan_command_semantics_owner_split",
                    "compile_host_plan_command_semantics.py",
                    "CompileHost plan command semantics owner",
                ],
            },
            "Current plugin docs do not reflect CompileHost plan command semantics owner split",
        )


if __name__ == "__main__":
    unittest.main()
