import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPlatformBundleTemplateReportSchemaTestSplitTests(
    unittest.TestCase
):
    def test_current_status_records_platform_bundle_template_report_schema_test_split(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_13_m5_t1_platform_bundle_template_report_schema_test_owner_split"
        )

        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 09 status": _tail_section(
                plan_09_text, "## 状态与产出记录"
            ),
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_schema.py=280",
            "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_files.py=271",
            "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_identity.py=389",
            "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_semantics.py=404",
            "tools/zircon_export/tests/platform_bundle_template_report_helpers.py=54",
            "test_report_rejects_template_report_manifest_invalid_toml",
            "test_report_rejects_template_report_manifest_host_artifact_mismatch",
            "test_report_rejects_template_report_missing_profile_membership",
            "test_template_report_schema_root_keeps_shape_tests",
            "python -m unittest tools.zircon_export.tests.test_pipeline_report_platform_bundle_template_report_schema tools.zircon_export.tests.test_pipeline_report_platform_bundle_template_manifest_files tools.zircon_export.tests.test_pipeline_report_platform_bundle_template_manifest_identity tools.zircon_export.tests.test_pipeline_report_platform_bundle_template_report_semantics",
            "31/31",
            "python -m unittest tools.tests.test_zircon_export_platform_bundle_template_report_schema_test_owner_boundaries",
            "5/5",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin status docs do not record the PlatformBundle "
                "template report schema test split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
