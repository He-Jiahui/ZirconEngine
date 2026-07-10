import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path

from tools.tests.plugin_docs_current_status_export_template_cook_assets_support import (
    assert_required_phrases,
    load_export_template_cook_assets_sections,
)


class PluginDocsCurrentStatusCookAssetsOwnerSplitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_export_template_cook_assets_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_cook_assets_report_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_cook_assets_report_owner_split",
                    "pipeline_report_cook_assets_manifest_io.py",
                    "pipeline_report_cook_assets_pack_handoff.py",
                    "CookAssets report owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_cook_assets_report_owner_split",
                    "pipeline_report_cook_assets_manifest_io.py",
                    "pipeline_report_cook_assets_pack_handoff.py",
                    "CookAssets report owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_cook_assets_report_owner_split",
                    "pipeline_report_cook_assets_manifest_io.py",
                    "pipeline_report_cook_assets_pack_handoff.py",
                    "CookAssets manifest IO owner",
                    "CookAssets Pack handoff owner",
                ],
                "export tooling docs": [
                    "pipeline_report_cook_assets_manifest_io.py",
                    "pipeline_report_cook_assets_pack_handoff.py",
                    "CookAssets report owner",
                    "CookAssets Pack handoff owner",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_cook_assets_report_owner_split",
                    "pipeline_report_cook_assets_manifest_io.py",
                    "pipeline_report_cook_assets_pack_handoff.py",
                    "CookAssets report owner",
                ],
            },
            "Current export/plugin docs do not reflect CookAssets report owner split",
        )

    def test_current_export_plan_reflects_cook_assets_manifest_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_cook_assets_manifest_owner_split",
                    "cook_assets_manifest.py",
                    "CookAssets asset manifest owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_cook_assets_manifest_owner_split",
                    "cook_assets_manifest.py",
                    "CookAssets asset manifest owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_cook_assets_manifest_owner_split",
                    "cook_assets_manifest.py",
                    "asset manifest diagnostics",
                ],
                "export tooling docs": [
                    "cook_assets_manifest.py",
                    "CookAssets asset manifest owner",
                    "asset manifest diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_cook_assets_manifest_owner_split",
                    "cook_assets_manifest.py",
                    "CookAssets asset manifest owner",
                ],
            },
            "Current export/plugin docs do not reflect CookAssets manifest owner split",
        )

    def test_current_export_plan_reflects_cook_assets_pack_trim_closure_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_cook_assets_pack_trim_closure_owner_split",
                    "pipeline_report_cook_assets_pack_trim_closure.py",
                    "pipeline_report_cook_assets_trim_evidence.py",
                    "CookAssets Pack trim closure owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_cook_assets_pack_trim_closure_owner_split",
                    "pipeline_report_cook_assets_pack_trim_closure.py",
                    "pipeline_report_cook_assets_trim_evidence.py",
                    "CookAssets Pack trim closure owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_cook_assets_pack_trim_closure_owner_split",
                    "pipeline_report_cook_assets_pack_trim_closure.py",
                    "pipeline_report_cook_assets_trim_evidence.py",
                    "trim-closure reconstruction/source-byte diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_cook_assets_pack_trim_closure.py",
                    "pipeline_report_cook_assets_trim_evidence.py",
                    "CookAssets Pack trim closure owner",
                    "trim-closure reconstruction/source-byte diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_cook_assets_pack_trim_closure_owner_split",
                    "pipeline_report_cook_assets_pack_trim_closure.py",
                    "pipeline_report_cook_assets_trim_evidence.py",
                    "CookAssets Pack trim closure owner",
                ],
            },
            "Current export/plugin docs do not reflect CookAssets Pack trim closure owner split",
        )

    def test_current_export_plan_reflects_cook_assets_project_fallback_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_cook_assets_project_fallback_owner_split",
                    "cook_assets_project_fallback.py",
                    "CookAssets project fallback owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_cook_assets_project_fallback_owner_split",
                    "cook_assets_project_fallback.py",
                    "CookAssets project fallback owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_cook_assets_project_fallback_owner_split",
                    "cook_assets_project_fallback.py",
                    "project manifest fallback/res:// direct-reference closure",
                ],
                "export tooling docs": [
                    "cook_assets_project_fallback.py",
                    "CookAssets project fallback owner",
                    "project manifest fallback/res:// direct-reference closure",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_cook_assets_project_fallback_owner_split",
                    "cook_assets_project_fallback.py",
                    "CookAssets project fallback owner",
                ],
            },
            "Current export/plugin docs do not reflect CookAssets project fallback owner split",
        )


if __name__ == "__main__":
    unittest.main()
