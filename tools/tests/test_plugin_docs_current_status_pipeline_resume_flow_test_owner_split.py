import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusPipelineResumeFlowTestOwnerSplitTests(unittest.TestCase):
    def test_current_status_records_pipeline_resume_flow_test_owner_split(self) -> None:
        repo_root = Path(__file__).resolve().parents[2]
        status_id = (
            "plugins_09_e1_pipeline_resume_platform_bundle_handoff_test_owner_split"
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
        }
        required_phrases = [
            status_id,
            "tools/zircon_export/tests/test_pipeline_resume_flow.py=557",
            "tools/zircon_export/tests/test_pipeline_resume_platform_bundle_handoff.py=422",
            "tools/tests/test_zircon_export_pipeline_resume_flow_test_owner_boundaries.py=110",
            "test_platform_bundle_handoff_tests_live_in_dedicated_owner",
            "test_pipeline_resume_flow_test_owners_stay_under_line_budgets",
            "test_pipeline_platform_bundle_uses_compile_host_report_host",
            "test_stage_platform_bundle_uses_native_dynamic_report_plugins",
            "focused owner/tests 34/34",
            "current-status docs 309/309",
            "py_compile",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record pipeline resume flow owner split:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
