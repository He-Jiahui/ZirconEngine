import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_source_template_compile_host_support import (
    assert_required_phrases,
    load_source_template_compile_host_sections,
)


class PluginDocsCurrentStatusSourceTemplateOwnerSplitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_source_template_compile_host_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_source_template_generated_files_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_source_template_generated_files_owner_split",
                    "pipeline_report_source_template_generated_files.py",
                    "SourceTemplate generated files owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_source_template_generated_files_owner_split",
                    "pipeline_report_source_template_generated_files.py",
                    "SourceTemplate generated files owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_source_template_generated_files_owner_split",
                    "pipeline_report_source_template_generated_files.py",
                    "generated file diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_source_template_generated_files.py",
                    "SourceTemplate generated files owner",
                    "generated file diagnostics",
                ],
            },
            "Current export/plugin docs do not reflect SourceTemplate generated files owner split",
        )

    def test_current_export_plan_reflects_source_template_build_handoff_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_source_template_build_handoff_owner_split",
                    "pipeline_report_source_template_build_handoff.py",
                    "SourceTemplate build handoff owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_source_template_build_handoff_owner_split",
                    "pipeline_report_source_template_build_handoff.py",
                    "SourceTemplate build handoff owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_source_template_build_handoff_owner_split",
                    "pipeline_report_source_template_build_handoff.py",
                    "Validate build-plan/build-validation diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_source_template_build_handoff.py",
                    "SourceTemplate build handoff owner",
                    "Validate build-plan/build-validation diagnostics",
                ],
            },
            "Current export/plugin docs do not reflect SourceTemplate build handoff owner split",
        )

    def test_current_export_plan_reflects_source_template_generated_project_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_source_template_generated_project_owner_split",
                    "source_template_generated_project.py",
                    "SourceTemplate generated project owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_source_template_generated_project_owner_split",
                    "source_template_generated_project.py",
                    "SourceTemplate generated project owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_source_template_generated_project_owner_split",
                    "source_template_generated_project.py",
                    "generated project materialization diagnostics",
                ],
                "export tooling docs": [
                    "source_template_generated_project.py",
                    "SourceTemplate generated project owner",
                    "generated project materialization diagnostics",
                ],
            },
            "Current export/plugin docs do not reflect SourceTemplate generated project owner split",
        )

    def test_current_plugin_docs_reflect_source_template_plan_command_owner_split(
        self,
    ):
        slug = "plugins_13_m5_t1_source_template_plan_command_owner_split"
        owner_file = "source_template_plan_command.py"
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    slug,
                    owner_file,
                    "SourceTemplate plan/command owner",
                ],
                "13 standalone status": [
                    slug,
                    owner_file,
                    "SourceTemplate plan/command owner",
                ],
                "standalone current contract": [
                    slug,
                    owner_file,
                    "Validate report/build-plan handoff and command rewriting",
                ],
                "export tooling docs": [
                    owner_file,
                    "SourceTemplate plan/command owner",
                    "Validate report/build-plan handoff and command rewriting",
                ],
            },
            "Current plugin docs do not reflect SourceTemplate plan/command owner split",
        )


if __name__ == "__main__":
    unittest.main()
