import unittest
from tools.tests.plugin_status_document import StatusDocumentPath as Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


def _tail_section(text: str, start: str) -> str:
    return text[text.index(start) :]


class PluginDocsCurrentStatusZirconBuildStagedZuiDocumentGateTests(
    unittest.TestCase
):
    def test_current_status_records_zircon_build_staged_zui_document_gate(self):
        repo_root = Path(__file__).resolve().parents[2]
        status_id = "plugins_13_m5_t1_zircon_build_staged_zui_document_gate"

        plan_13_text = (
            repo_root / "docs/plans/zircon_plugins/13-standalone-plugin-build.md"
        ).read_text(encoding="utf-8")
        plan_13_status = _tail_section(
            plan_13_text,
            "## 9. 审查和验收记录",
        )
        plan_09_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        plan_09_status = _section(
            plan_09_text,
            "## 状态与产出记录",
            "## 5. 里程碑与任务分解",
        )
        standalone_text = (
            repo_root / "docs/zircon_plugins/plugin-standalone-build.md"
        ).read_text(encoding="utf-8")
        standalone_status = _tail_section(
            standalone_text,
            "## 9. 当前落地状态",
        )
        export_tool_text = (
            repo_root / "docs/cli-and-tooling/zircon-export-tool.md"
        ).read_text(encoding="utf-8")
        export_tool_status = export_tool_text
        plan_11_text = (
            repo_root
            / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
        ).read_text(encoding="utf-8")
        plan_11_status = _tail_section(plan_11_text, "## 7. 状态与产出记录")
        structure_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        review_text = (
            repo_root / "docs/plans/engine-code-review-findings-2026-06.md"
        ).read_text(encoding="utf-8")
        session_text = (
            repo_root
            / ".codex/sessions/20260628-0317-zui-migration-validation.md"
        ).read_text(encoding="utf-8")

        sections = {
            "Plugins 13 status": plan_13_status,
            "Plugins 09 status": plan_09_status,
            "standalone current status": standalone_status,
            "export tool docs": export_tool_status,
            "Plan 11 status": plan_11_status,
            "structure convention": structure_text,
            "review findings": review_text,
            "active session": session_text,
        }
        required_phrases = [
            status_id,
            "zircon_build_zui_assets.py",
            "validate_staged_engine_asset_suffix",
            "validate_plugin_distribution_zui_asset",
            "test_zircon_build_rejects_staged_zui_document_kind_drift",
            "test_staged_zui_asset_checks_live_in_zui_asset_owner",
            "staged engine asset",
            "matched .zui asset",
            "asset.kind",
            "component, style, theme_tokens, view",
        ]

        failures: list[str] = []
        for section_name, section in sections.items():
            for phrase in required_phrases:
                if phrase not in section:
                    failures.append(f"{section_name}: missing {phrase}")

        if failures:
            self.fail(
                "Current plugin docs do not record zircon_build staged .zui "
                "document validation gate:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
