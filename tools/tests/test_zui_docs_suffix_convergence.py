import unittest
from pathlib import Path


def _section(text: str, start: str, end: str) -> str:
    start_index = text.index(start)
    end_index = text.index(end, start_index)
    return text[start_index:end_index]


class ZuiDocsSuffixConvergenceTests(unittest.TestCase):
    def test_current_zui_authority_docs_do_not_describe_legacy_suffixes_as_active(self):
        repo_root = Path(__file__).resolve().parents[2]
        stale_phrases_by_doc = {
            "docs/plans/zircon_editor/editor_ui/index.md": [
                ".zui` + `.v2.ui.toml` 双轨",
                ".zui/.ui.toml 加载",
                ".zui` 只允许单组件 profile",
            ],
            "docs/editor-and-tooling/zui-asset-governance.md": [
                "Production `.v2.ui.toml`",
                "current view/style roots",
                "deprecated `.v2.ui.toml` roots",
                "both `.v2.ui.toml` and `.zui`",
            ],
            "docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md": [
                "tree-shaped `.ui.toml`",
                "zircon_editor/assets/ui/**/*.ui.toml",
                "zircon_runtime/assets/ui/runtime/fixtures/*.ui.toml",
            ],
            "docs/editor-and-tooling/ui-asset-editor-host-session.md": [
                "watch `project_root/assets` 下的 `.ui.toml`",
                "structural frames from `.ui.toml` assets",
                "current implementation and tests must use `retained_host`, `.ui.toml`",
                "All pane shell topology now comes from `.ui.toml` assets",
                "EditorTemplateRuntimeService` now owns the high-level editor façade for `.ui.toml` parsing",
            ],
            "docs/editor-and-tooling/editor-template-compatibility-migration.md": [
                "Editor UI templates are `.ui.toml` documents",
                "loads `.ui.toml` documents",
                "backing file remains named `workbench_shell.ui.toml`",
                "workbench_shell.ui.toml",
                "workbench_drawer_source.ui.toml",
                "floating_window_source.ui.toml",
                "scene_viewport_toolbar.ui.toml",
                "asset_surface_controls.ui.toml",
                "Current production and tests must not restore `SlintUiProjection`, generated include modules, `slint_build` seams, `slint_host` owner paths, or `workbench_slint*` test names as active authorities.",
                "use `retained_host`, `RetainedUiHost*`, `.ui.toml`, and `host_contract`",
                "host template assets remain `.ui.toml` authority",
            ],
        }

        failures: list[str] = []
        for relative_path, stale_phrases in stale_phrases_by_doc.items():
            text = (repo_root / relative_path).read_text(encoding="utf-8")
            for phrase in stale_phrases:
                if phrase in text:
                    failures.append(f"{relative_path}: {phrase}")

        if failures:
            self.fail(
                "Current .zui authority docs still describe retired suffixes as active:\n"
                + "\n".join(failures)
            )

    def test_plan_11_current_status_block_reflects_zui_only_static_state(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root
            / "docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md"
        ).read_text(encoding="utf-8")
        current_authority = _section(
            plan_text,
            "> 状态：",
            "## 2. 后缀检测/加载/治理触点清单",
        )

        stale_phrases = [
            "## 1. 现状基线",
            "UI 资产当前存在**三态后缀**",
            "view/style root 仍锁死在 `.v2.ui.toml`",
            "git ls-files '*.ui.toml'` 共 89",
            "plain v1 `.ui.toml` 仅剩测试 fixture",
        ]
        required_phrases = [
            "当前收口事实",
            "production legacy UI suffix file count = 0",
            "`.zui` 是唯一 UI 资产后缀",
            "production `.zui` parse scan = 268",
            "layout metadata `.toml` indexes reference only `.zui` UI assets",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_authority:
                failures.append(f"stale active status: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_authority:
                failures.append(f"missing current fact: {phrase}")

        if failures:
            self.fail(
                "Plan 11 current status block does not match .zui-only static state:\n"
                + "\n".join(failures)
            )

    def test_structure_convention_descriptor_families_do_not_list_retired_ui_suffix(self):
        repo_root = Path(__file__).resolve().parents[2]
        convention_text = (
            repo_root / "docs/plans/engine-code-structure-convention.md"
        ).read_text(encoding="utf-8")
        resource_section = _section(
            convention_text,
            "## §5 资源 / 描述 / manifest 放置",
            "## §6 统一插件接口开发体验框架（Plugin DX）",
        )

        stale_phrases = [
            "`.zui` / `.ui.toml`",
            "`.ui.toml` / `.zmaterial`",
        ]
        required_phrases = [
            "`.zui` 是唯一 UI asset descriptor 家族",
            "`.ui.toml` / `.v2.ui.toml` 已退役",
            "typed editor layout metadata",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in resource_section:
                failures.append(f"stale descriptor family phrase: {phrase}")
        for phrase in required_phrases:
            if phrase not in resource_section:
                failures.append(f"missing descriptor family fact: {phrase}")

        if failures:
            self.fail(
                "Structure convention §5 still describes retired UI suffixes as active:\n"
                + "\n".join(failures)
            )

    def test_plugin_editor_integration_plan_uses_zui_for_current_template_authority(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root / "docs/plans/zircon_plugins/10-editor-integration.md"
        ).read_text(encoding="utf-8")
        current_authority = _section(
            plan_text,
            "## 4. AI Workbench 风格对位表",
            "## 7. 验收标准（每插件 Editor 里程碑通用）",
        )

        stale_phrases = [
            "retained host 的 `.ui.toml` 模板体系",
            "不走 .ui.toml 静态投影",
            "定制 drawer 仍走 `.ui.toml`",
        ]
        required_phrases = [
            "retained host 的 `.zui` 模板体系",
            "`.ui.toml` / `.v2.ui.toml` 后缀已退役",
            "定制 drawer 仍走 `.zui`",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_authority:
                failures.append(f"stale plugin editor template authority: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_authority:
                failures.append(f"missing plugin editor zui authority fact: {phrase}")

        if failures:
            self.fail(
                "Plugin editor integration plan current authority still references retired UI suffixes:\n"
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
