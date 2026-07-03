import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class ZuiDocsSuffixPlanScopeGuardTests(unittest.TestCase):
    def test_workbench_shell_plan_design_declares_zui_only_layout_authority(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root
            / "docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md"
        ).read_text(encoding="utf-8")
        design_section = _section(
            plan_text,
            "## 3. 设计",
            "## 4. 接口与数据结构草案",
        )

        stale_phrases = [
            "`.ui.toml` 为当前权威",
            "`.v2.ui.toml` 为当前权威",
            "`.ui.toml` 作为当前壳层",
            "`.v2.ui.toml` 作为当前壳层",
        ]
        required_phrases = [
            "Workbench shell layout 描述只以 `.zui` 为当前权威",
            "`.ui.toml` / `.v2.ui.toml` 后缀已退役",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in design_section:
                failures.append(f"stale workbench shell layout authority: {phrase}")
        for phrase in required_phrases:
            if phrase not in design_section:
                failures.append(
                    f"missing workbench shell zui-only authority fact: {phrase}"
                )

        if failures:
            self.fail(
                "Workbench shell runtime UI plan design does not state current .zui-only layout authority:\n"
                + "\n".join(failures)
            )

    def test_ui_asset_management_plan_uses_zui_for_current_asset_scope(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root / "docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md"
        ).read_text(encoding="utf-8")
        current_goal = _section(
            plan_text,
            "## 1. 目标",
            "## 2. 现状（按代码核实修正）",
        )

        stale_phrases = [
            "`.zui` 单组件、`.v2.ui.toml` 页面模板",
            "`.v2.ui.toml` 页面模板",
        ]
        required_phrases = [
            "`.zui` UI 文档",
            "component / view / style / theme_tokens",
            "`.ui.toml` / `.v2.ui.toml` 后缀已退役",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_goal:
                failures.append(f"stale UI asset management scope: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_goal:
                failures.append(f"missing UI asset management zui scope fact: {phrase}")

        if failures:
            self.fail(
                "UI asset management plan current goal still references retired UI suffixes:\n"
                + "\n".join(failures)
            )

    def test_style_theme_plan_token_scan_targets_zui_documents_only(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root
            / "docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md"
        ).read_text(encoding="utf-8")
        milestone_table = _section(
            plan_text,
            "## 7. 里程碑切片化",
            "## 8. 测试矩阵（代表性用例）",
        )

        stale_phrases = [
            "扫 `.zui`/`*.v2.ui.toml`/`paint_template_nodes` 源",
        ]
        required_phrases = [
            "扫 `.zui` UI 文档与 `paint_template_nodes` 源",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in milestone_table:
                failures.append(f"stale token scan target: {phrase}")
        for phrase in required_phrases:
            if phrase not in milestone_table:
                failures.append(f"missing zui-only token scan target: {phrase}")

        if failures:
            self.fail(
                "Style/theme plan token scan still targets retired UI suffixes:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
