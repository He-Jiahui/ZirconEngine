import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPlatformBundleTemplateTestSplitTests(
    unittest.TestCase
):
    def test_current_status_records_platform_bundle_template_test_split(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_09_e1_platform_bundle_template_resolution_test_owner_split"

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
        }
        required_phrases = [
            status_id,
            "tools/zircon_export/tests/test_templates.py=877",
            "tools/zircon_export/tests/test_platform_bundle_template_resolution.py=650",
            "test_linux_template_materializes_directory_layout",
            "test_template_root_reports_missing_profile_match",
            "test_export_template_validation_keeps_manifest_contract_tests",
            "python -m unittest tools.zircon_export.tests.test_templates tools.zircon_export.tests.test_platform_bundle_template_resolution",
            "34/34",
            "python -m unittest tools.tests.test_zircon_export_template_test_owner_boundaries",
            "3/3",
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
                "template test split:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
