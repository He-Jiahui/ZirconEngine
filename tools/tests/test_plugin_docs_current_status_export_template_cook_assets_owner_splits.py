import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_export_template_cook_assets_support import (
    assert_required_phrases,
    load_export_template_cook_assets_sections,
)


class PluginDocsCurrentStatusExportTemplateOwnerSplitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_export_template_cook_assets_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_export_template_manifest_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_export_template_manifest_owner_split",
                    "export_template_manifest.py",
                    "ExportTemplate manifest owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_export_template_manifest_owner_split",
                    "export_template_manifest.py",
                    "ExportTemplate manifest owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_export_template_manifest_owner_split",
                    "export_template_manifest.py",
                    "template manifest/path/hash helpers",
                ],
                "export tooling docs": [
                    "export_template_manifest.py",
                    "ExportTemplate manifest owner",
                    "template manifest/path/hash helpers",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_export_template_manifest_owner_split",
                    "export_template_manifest.py",
                    "ExportTemplate manifest owner",
                ],
            },
            "Current export/plugin docs do not reflect ExportTemplate manifest owner split",
        )

    def test_current_export_plan_reflects_cli_argument_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_cli_argument_owner_split",
                    "cli_arguments.py",
                    "zircon_export CLI argument owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_cli_argument_owner_split",
                    "cli_arguments.py",
                    "zircon_export CLI argument owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_cli_argument_owner_split",
                    "cli_arguments.py",
                    "pipeline CLI argument parsing/defaults",
                ],
                "export tooling docs": [
                    "cli_arguments.py",
                    "zircon_export CLI argument owner",
                    "pipeline CLI argument parsing/defaults",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_cli_argument_owner_split",
                    "cli_arguments.py",
                    "zircon_export CLI argument owner",
                ],
            },
            "Current export/plugin docs do not reflect zircon_export CLI argument owner split",
        )

    def test_current_export_plan_reflects_export_template_resolution_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_export_template_resolution_owner_split",
                    "export_template_resolution.py",
                    "ExportTemplate resolution owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_export_template_resolution_owner_split",
                    "export_template_resolution.py",
                    "ExportTemplate resolution owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_export_template_resolution_owner_split",
                    "export_template_resolution.py",
                    "template root candidate resolution",
                ],
                "export tooling docs": [
                    "export_template_resolution.py",
                    "ExportTemplate resolution owner",
                    "template root candidate resolution",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_export_template_resolution_owner_split",
                    "export_template_resolution.py",
                    "ExportTemplate resolution owner",
                ],
            },
            "Current export/plugin docs do not reflect ExportTemplate resolution owner split",
        )

    def test_current_plugin_docs_reflect_schema_string_array_owner_split(self):
        slug = "plugins_13_m5_t1_pipeline_report_schema_string_array_owner_split"
        owner_file = "pipeline_report_schema_string_array.py"
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [slug, owner_file, "schema string-array owner"],
                "13 standalone status": [
                    slug,
                    owner_file,
                    "schema string-array owner",
                ],
                "standalone current contract": [
                    slug,
                    owner_file,
                    "string-array schema diagnostics",
                ],
                "export tooling docs": [
                    owner_file,
                    "schema string-array owner",
                    "string-array schema diagnostics",
                ],
                "active session notes": [slug, owner_file, "schema string-array owner"],
            },
            "Current plugin docs do not reflect schema string-array owner split",
        )


if __name__ == "__main__":
    unittest.main()
