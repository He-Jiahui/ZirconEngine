import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusNativeDynamicReportOwnerBudgetTests(unittest.TestCase):
    def test_current_status_records_native_dynamic_report_owner_budget_recovered(
        self,
    ):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_09_e1_current_status_native_dynamic_report_owner_budget_recovered"
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
        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
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
            "Plugins 09 status": _section(
                plan_09_text, "## 状态与产出记录", "## 5. 里程碑与任务分解"
            ),
            "Plugins 13 status": _tail_section(
                plan_13_text, "## 9. 审查和验收记录"
            ),
            "standalone current status": _tail_section(
                standalone_text, "## 9. 当前落地状态"
            ),
            "export tool docs": export_tool_text,
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "tools/tests/plugin_docs_current_status_native_dynamic_report_support.py",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_report_owner_boundaries.py=107",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_report_owner_splits.py=66",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_report_schema_owner_splits.py=200",
            "tools/tests/test_plugin_docs_current_status_native_dynamic_stage_report_owner_splits.py=162",
            "test_native_dynamic_report_status_tests_live_in_focused_owners",
            "test_native_dynamic_report_status_owners_stay_under_line_budgets",
            "test_current_export_plan_reflects_native_dynamic_stage_loader_manifest_owner_split",
            "test_current_export_plan_reflects_native_dynamic_build_plan_package_details_owner_split",
            "focused owner/docs 13/13",
            "current-status docs 281/281",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record NativeDynamic report owner "
                "budget recovery:\n" + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
