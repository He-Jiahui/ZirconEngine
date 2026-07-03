import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_platform_bundle_support import (
    assert_required_phrases,
    load_platform_bundle_sections,
)


class PluginDocsCurrentStatusPlatformBundleReportOwnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = load_platform_bundle_sections(
            Path(__file__).resolve().parents[2]
        )

    def test_current_export_plan_reflects_platform_bundle_stage_handoff_report_owner_split(
        self,
    ):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_stage_handoff_report_owner_split",
                    "pipeline_report_platform_bundle_stage_handoff.py",
                    "PlatformBundle stage handoff report owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_platform_bundle_stage_handoff_report_owner_split",
                    "pipeline_report_platform_bundle_stage_handoff.py",
                    "PlatformBundle stage handoff report owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_stage_handoff_report_owner_split",
                    "pipeline_report_platform_bundle_stage_handoff.py",
                    "Host/Pack/Delta cross-stage handoff diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_platform_bundle_stage_handoff.py",
                    "PlatformBundle stage handoff report owner",
                    "Host/Pack/Delta cross-stage handoff diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_stage_handoff_report_owner_split",
                    "pipeline_report_platform_bundle_stage_handoff.py",
                    "PlatformBundle stage handoff report owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle stage handoff report owner split",
        )

    def test_current_export_plan_reflects_platform_bundle_file_evidence_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    "plugins_13_m5_t1_platform_bundle_file_evidence_owner_split",
                    "pipeline_report_platform_bundle_file_evidence.py",
                    "PlatformBundle file evidence owner",
                ],
                "13 standalone status": [
                    "plugins_13_m5_t1_platform_bundle_file_evidence_owner_split",
                    "pipeline_report_platform_bundle_file_evidence.py",
                    "PlatformBundle file evidence owner",
                ],
                "standalone current contract": [
                    "plugins_13_m5_t1_platform_bundle_file_evidence_owner_split",
                    "pipeline_report_platform_bundle_file_evidence.py",
                    "manifest/path/hash/output file diagnostics",
                ],
                "export tooling docs": [
                    "pipeline_report_platform_bundle_file_evidence.py",
                    "PlatformBundle file evidence owner",
                    "manifest/path/hash/output file diagnostics",
                ],
                "active session notes": [
                    "plugins_13_m5_t1_platform_bundle_file_evidence_owner_split",
                    "pipeline_report_platform_bundle_file_evidence.py",
                    "PlatformBundle file evidence owner",
                ],
            },
            "Current export/plugin docs do not reflect PlatformBundle file evidence owner split",
        )

    def test_current_plugin_docs_reflect_platform_bundle_report_payload_owner_split(self):
        slug = "plugins_13_m5_t1_platform_bundle_report_payload_owner_split"
        owner_file = "platform_bundle_report_payload.py"
        assert_required_phrases(
            self,
            self.sections,
            {
                "09 export status": [
                    slug,
                    owner_file,
                    "PlatformBundle report payload owner",
                ],
                "13 standalone status": [
                    slug,
                    owner_file,
                    "PlatformBundle report payload owner",
                ],
                "standalone current contract": [
                    slug,
                    owner_file,
                    "bundle manifest and stage report payload assembly",
                ],
                "export tooling docs": [
                    owner_file,
                    "PlatformBundle report payload owner",
                    "bundle manifest and stage report payload assembly",
                ],
                "active session notes": [
                    slug,
                    owner_file,
                    "PlatformBundle report payload owner",
                ],
            },
            "Current plugin docs do not reflect PlatformBundle report payload owner split",
        )


if __name__ == "__main__":
    unittest.main()
