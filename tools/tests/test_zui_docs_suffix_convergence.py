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

    def test_plugin_export_plan_m6_current_progress_uses_zui_report_templates(self):
        repo_root = Path(__file__).resolve().parents[2]
        plan_text = (
            repo_root / "docs/plans/zircon_plugins/09-export-publishing.md"
        ).read_text(encoding="utf-8")
        current_progress = _section(
            plan_text,
            "#### M6 当前进度（2026-06-14）",
            "## 5. 里程碑与任务分解",
        )

        stale_phrases = [
            "当前仍不声明 `.zui` 模板",
            "主面板与三类报告现在是 `.v2.ui.toml` view 模板",
            "当前不声明 `.v2.ui.toml` 真实渲染",
        ]
        required_phrases = [
            "主面板与三类报告当前均为 `.zui` view 模板",
            "profile drawer 保持 `.zui` component 模板",
            "`.ui.toml` / `.v2.ui.toml` 后缀已退役",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_progress:
                failures.append(f"stale export wizard template authority: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_progress:
                failures.append(f"missing export wizard zui authority fact: {phrase}")

        if failures:
            self.fail(
                "Plugin export plan M6 current progress still references retired report-template suffixes:\n"
                + "\n".join(failures)
            )

    def test_editor_build_export_desktop_doc_current_templates_use_zui(self):
        repo_root = Path(__file__).resolve().parents[2]
        doc_text = (
            repo_root / "docs/zircon_plugins/editor-build-export-desktop.md"
        ).read_text(encoding="utf-8")
        current_sections = "\n".join(
            [
                _section(doc_text, "## Contributions", "## Boundary"),
                _section(doc_text, "## Boundary", "## Validation"),
            ]
        )

        stale_phrases = [
            "plugin-private `.v2.ui.toml` view templates",
            "registered `.v2.ui.toml` templates",
            "that `.v2.ui.toml` document",
            "UI templates may point at `.zui` components or `.v2.ui.toml` view templates",
            "UI templates can use `.v2.ui.toml`",
        ]
        required_phrases = [
            "plugin-private `.zui` view templates",
            "registered `.zui` templates",
            "that `.zui` document",
            "UI templates now point at `.zui` view or component templates",
            "UI templates use `.zui` documents",
            "stale `.ui.toml` and `.v2.ui.toml` documents are rejected",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_sections:
                failures.append(f"stale desktop export template suffix: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_sections:
                failures.append(f"missing desktop export zui fact: {phrase}")

        if failures:
            self.fail(
                "Desktop export plugin doc still describes current templates with retired suffix authority:\n"
                + "\n".join(failures)
            )

    def test_editor_command_workflow_component_drawer_uses_zui_document_authority(self):
        repo_root = Path(__file__).resolve().parents[2]
        workflow_text = (
            repo_root / "docs/editor-and-tooling/editor-command-workflow.md"
        ).read_text(encoding="utf-8")
        drawer_projection = _section(
            workflow_text,
            "### Component Drawer Template Projection",
            "### EditorOperation 分派",
        )

        stale_phrases = [
            "UI document must be a `.v2.ui.toml` asset",
            "registry rejects legacy `.ui.toml` drawer and template documents",
        ]
        required_phrases = [
            "UI document must be a `.zui` component asset",
            "registry rejects stale `.ui.toml` and `.v2.ui.toml` drawer/template documents",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in drawer_projection:
                failures.append(f"stale component drawer document authority: {phrase}")
        for phrase in required_phrases:
            if phrase not in drawer_projection:
                failures.append(f"missing component drawer zui authority fact: {phrase}")

        if failures:
            self.fail(
                "Editor command workflow still describes component drawer documents with retired suffix authority:\n"
                + "\n".join(failures)
            )

    def test_editor_workbench_shell_current_authority_uses_zui_host_assets(self):
        repo_root = Path(__file__).resolve().parents[2]
        shell_text = (
            repo_root / "docs/editor-and-tooling/editor-workbench-shell.md"
        ).read_text(encoding="utf-8")
        current_sections = "\n".join(
            [
                _section(shell_text, "## Purpose", "## Ownership"),
                _section(shell_text, "## Template Authority", "## Input Authority"),
                _section(
                    shell_text,
                    "## Host Contract And Painter",
                    "## Hard Cutover From Deleted Slint Host",
                ),
                _section(shell_text, "## Hard Cutover From Deleted Slint Host", "## Validation"),
                shell_text[shell_text.index("## Validation") :],
            ]
        )

        stale_phrases = [
            "consumes `.ui.toml` host assets",
            "The current shell structure comes from source-controlled `.ui.toml` assets",
            "Current root-shell frame authority is the host `.ui.toml` geometry",
            "workbench_shell.ui.toml",
            "floating_window_source.ui.toml",
            "workbench_drawer_source.ui.toml",
            "come from `.ui.toml`, shared surface projection",
            "Current code, tests, docs, and validation commands should use `retained_host`, `.ui.toml`",
            "editor boundary tests for `.ui.toml` host assets",
        ]
        required_phrases = [
            "consumes `.zui` host assets",
            "The current shell structure comes from source-controlled `.zui` assets",
            "Current root-shell frame authority is the host `.zui` geometry",
            "workbench_shell.zui",
            "floating_window_source.zui",
            "drawer source frame recompute remains owned by `workbench_drawer_source/layout.rs`",
            "come from `.zui`, shared surface projection",
            "Current code, tests, docs, and validation commands should use `retained_host`, `.zui`, and Rust-owned `host_contract` names",
            "editor boundary tests for `.zui` host assets",
        ]

        failures: list[str] = []
        for phrase in stale_phrases:
            if phrase in current_sections:
                failures.append(f"stale workbench shell host asset authority: {phrase}")
        for phrase in required_phrases:
            if phrase not in current_sections:
                failures.append(f"missing workbench shell zui authority fact: {phrase}")

        if failures:
            self.fail(
                "Editor workbench shell doc still describes current host assets with retired suffix authority:\n"
                + "\n".join(failures)
            )

if __name__ == "__main__":
    unittest.main()
