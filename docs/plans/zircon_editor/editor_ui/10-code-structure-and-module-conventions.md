---
related_code:
  - zircon_editor/src/ui/activity/mod.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_viewport_panel.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row/selection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_tree_row/surface.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/tree.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/events.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor/binding.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor/designer.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor/document.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor/presentation.rs
  - zircon_editor/src/ui/host/editor_manager_asset_editor/style.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/animation_editor/session/graph.rs
  - zircon_editor/src/ui/animation_editor/session/lifecycle.rs
  - zircon_editor/src/ui/animation_editor/session/parameters.rs
  - zircon_editor/src/ui/animation_editor/session/presentation.rs
  - zircon_editor/src/ui/animation_editor/session/sequence.rs
  - zircon_editor/src/ui/animation_editor/session/state_machine.rs
  - zircon_editor/src/ui/animation_editor/session/support.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock/entries.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle/v2_projection.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector/payload_editing.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/run.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/support.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_plan.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/pipeline_execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/job.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/panel_session.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/tests/view_model.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/retained_host/app/tests/mod.rs
  - zircon_editor/src/ui/retained_host/app/tests/support.rs
  - zircon_editor/src/ui/retained_host/app/tests/command_palette.rs
  - zircon_editor/src/ui/retained_host/app/tests/child_window_focus.rs
  - zircon_editor/src/ui/retained_host/app/tests/menu_pointer.rs
  - zircon_editor/src/ui/retained_host/app/tests/projection_geometry.rs
  - zircon_editor/src/ui/retained_host/app/tests/viewport_template_bridge.rs
  - zircon_editor/src/ui/retained_host/app/tests/root_pointer_fallbacks.rs
  - zircon_editor/src/ui/retained_host/app/tests/child_window_tabs.rs
  - zircon_editor/src/ui/retained_host/ui/tests/mod.rs
  - zircon_editor/src/ui/retained_host/ui/tests/support.rs
  - zircon_editor/src/ui/retained_host/ui/tests/host_scene_projection.rs
  - zircon_editor/src/ui/retained_host/ui/tests/host_scene_projection/assertions.rs
  - zircon_editor/src/ui/retained_host/ui/tests/apply_presentation_shell.rs
  - zircon_editor/src/ui/retained_host/ui/tests/workbench_layout_frames.rs
  - zircon_editor/src/ui/retained_host/ui/tests/welcome_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/tests/scene_document_pane.rs
  - zircon_editor/src/ui/retained_host/ui/tests/floating_windows.rs
  - zircon_editor/src/ui/retained_host/ui/tests/component_showcase.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/template_runtime/mod.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/core/editor_event/service/state.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring/action_projection.rs
  - zircon_editor/src/ui/asset_editor/style/theme_authoring/merge.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/activity_rail.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/dock_header.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/status_bar.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection/tests.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation/scene_conversion.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/engine-architecture/large-file-ownership-m1.md
implementation_files:
  - docs/zircon_editor/structure/module-convention.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/editor_structure_audits/module_convention_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py
  - zircon_editor/src/tests/structure_convention/mod.rs
tests:
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib structure_convention --locked
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json
  - cargo fmt --all --check
doc_type: structure-plan
status: active
---

# 10 · Editor UI 代码结构与模块规范收束计划

> 状态：active · 范围：`zircon_editor/src`（`core/` `scene/` `ui/`）
> 上游权威：[`docs/plans/engine-code-structure-convention.md`](../../engine-code-structure-convention.md)
> 本计划承接 editor UI 计划 01–09 之外的**结构治理横切**：把 retained-host / workbench / asset-editor 的 owner 边界、超大投影文件、重复测试树收敛到规范，并以 editor `module_convention_gate` 防回归。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-editor-ui-code-structure-convergence",
  "goal": "按 owner 边界、文件预算、测试单一真源、命名与死代码纪律完成 Editor UI 结构硬切收敛。",
  "milestones": [
    {"id": "M1", "title": "UI 根层 owner 边界硬切", "depends_on": []},
    {"id": "M2", "title": "超大投影文件按责任拆分", "depends_on": ["M1"]},
    {"id": "M3", "title": "重复测试树与巨型测试收敛", "depends_on": ["M1"]},
    {"id": "M4", "title": "命名与 prelude 边界收敛", "depends_on": ["M1"]},
    {"id": "M5", "title": "投影样板与 production dead-code 清除", "depends_on": ["M2", "M3", "M4"]}
  ]
}
```

<!-- Workflow topology mirrors the existing M1-M5 execution table. -->

## 1. 目标

editor 是结构债最重的 crate（探索报告评分最低）：`ui/` 臃肿、`retained_host` 子目录无限细化、超大投影 / 转换文件、`src/tests/**` 与 `src/ui/**` 重复测试树。本计划让 editor 达到与 runtime 同口径的结构友好度。

## 2. 现状缺口（按代码实查，带路径证据）

| # | 缺口 | 规范条目 | 证据路径 |
|---|------|---------|---------|
| S1 | `ui/` 顶层单文件与目录模块混排 | R1.1 / R1.2 | `ui/activity.rs`、`ui/control.rs`（单文件）vs `ui/asset_editor/`、`ui/animation_editor/`（目录） |
| S2 | 碎片化过度（细节文件名 + 36 行 mod.rs） | R1.3 / R3.1 | `ui/retained_host/activity_rail_pointer/`（25 个 100–300 行私有模块） |
| S3 | 超大投影 / 转换文件、职责过载 | R1.3 / R1.4 | `template_bridge/workbench_projection.rs`(3619)、`workbench_host_window/chrome_template_projection.rs`(1741)、`retained_host/ui/pane_data_conversion/`（mod 1559 + projection 1126 + tests 1288） |
| S4 | 重复测试树（无单一 source of truth） | R4.3 | `src/tests/host/retained_callback_dispatch/**`(23 文件) ↔ `src/ui/retained_host/callback_dispatch/**` |
| S5 | 层级不清 / 无主名 | R2.3 | `core/editor_event/runtime/editor_event_runtime_state.rs` 已完成 `_inner` 禁名硬切；`editor_runtime_play_mode_backend.rs` 仍需继续评估 runtime 目录职责 |
| S6 | 巨型测试文件 | R4.3 | `src/tests/.../workbench_projection.rs`(3619)、`src/tests/editor_event/runtime.rs`(3591)；`retained_host/ui/tests.rs` 已拆为 `retained_host/ui/tests/{mod,support,host_scene_projection,apply_presentation_shell,workbench_layout_frames,welcome_presentation,scene_document_pane,floating_windows,component_showcase}.rs` |
| S7 | **投影函数样板复制 + 超长函数**（`to_host_contract_pane` ~228 行、`animation_template_projection` ~326 行共用 view→映射→model_rc 骨架） | R1.3 / 规范 E5 | `retained_host/ui/pane_data_conversion/mod.rs:74,245,316,468,760,1082` |
| S8 | **死代码抑制聚集，疑半成品**（component_adapter 注册 5 处 `#[allow(dead_code)]`、workbench world slot 4 处） | 规范 E6 | `ui/template_runtime/component_adapter/registry.rs:14,50,83,116,198`、`ui/workbench/state/editor_world_slot.rs:10,23,35,47` |

> S7-S8 来自 [`engine-code-review-findings-2026-06.md`](../../../engine-code-review-findings-2026-06.md)（F15/F12）。

## 3. 目标结构（收敛后形态）

- `ui/` 顶层单文件归入对应 owner 目录；每个 owner 目录 `mod.rs` 是分组精选 façade。
- 超大投影按 authoring 工作流 + template-runtime owner 拆叶子，root 留薄 façade（见 §范式）。
- 测试单一 owner：`src/ui/**` 的行为测试就近 folder-backed，`src/tests/**` 仅保留跨模块集成测试，消除双写。
- `core/editor_event/runtime/` 去 `_inner`，按职责（dispatcher / runtime / play_mode_backend）显式分名。

### 范式：超大投影文件 owner 拆分
```
workbench_projection.rs(3619)  →  workbench_projection/
                                    mod.rs            # 薄 façade
                                    layout.rs         # 区域布局投影
                                    event_routing.rs  # 事件 → host 路由投影
                                    data_conversion.rs# DTO 转换
                                    template_bridge.rs# 模板桥接
```
对齐 `large-file-ownership-m1.md`：按 ownership 拆、不按等行数切；root 不为避免改调用方而留行为。

## 4. 里程碑（任务级执行蓝本）

切片期 `cargo check -p zircon_editor --lib --locked`；里程碑末进测试。

| 里程碑 | 任务 | 改动文件（代表） | 依赖 | 验收命令 / 测试函数 |
|---|---|---|---|---|
| **M1 `ui/` owner 边界** | T1 顶层单文件归入 owner 目录 | `ui/activity.rs`、`ui/control.rs` → owner 目录 | — | `cargo check -p zircon_editor --lib`；`editor_ui_10_ui_module_owner_boundaries` |
| | T2 碎片化收口 + façade 预算 | `ui/retained_host/activity_rail_pointer/` | T1 | `editor_ui_10_facade_reexport_within_budget` |
| **M2 拆超大文件** | T1 投影桥拆 owner | `template_bridge/workbench_projection.rs`(3619) → `workbench_projection/{layout,event_routing,data_conversion,template_bridge}.rs` | — | `editor_ui_10_no_oversized_production_files` |
| | T2 chrome 投影拆 owner | `workbench_host_window/chrome_template_projection.rs`(1741) | — | 同上 |
| | T3 pane 转换拆 owner | `retained_host/ui/pane_data_conversion/**` | — | 同上 |
| **M3 测试树去重** | T1 单一 owner | `src/tests/host/**` ↔ `src/ui/retained_host/**` 去双写；拆 > 800 行测试 | — | `editor_ui_10_no_duplicate_test_trees`、`editor_ui_10_no_oversized_test_files` |
| **M4 命名 + prelude** | T1 去 `_inner` 按职责分名 | `core/editor_event/runtime/editor_event_runtime_state.rs` 等 | — | `editor_ui_10_no_banned_name_modules` |
| | T2 子系统 prelude | `ui/`、`scene/`、`core/` 各 `prelude.rs` | T1 | `editor_ui_10_prelude_within_budget` |
| **M5 投影去样板 + 死代码清除** | T1 抽 `project_nodes<T>()` 泛型 helper + 拆超长投影函数 | `retained_host/ui/pane_data_conversion/mod.rs`、`apply_presentation.rs` | M2 | `editor_ui_10_no_oversized_production_functions`（F15） |
| | T2 `#[allow(dead_code)]` sweep（核查 component_adapter 5 处是否半成品） | `ui/template_runtime/component_adapter/registry.rs`、`ui/workbench/state/editor_world_slot.rs` | — | `editor_ui_10_no_dead_code_suppression_in_production`（F12） |

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`10/2026-07-09-code-structure-and-module-conventions-output-records.md`](10/2026-07-09-code-structure-and-module-conventions-output-records.md)
- 当前结构审计：`audit_editor_structure.py --json` 的 `module_convention_gate` 已纳入 `editor_ui_10_visual_style_owner_tree_is_hard_cut_over`，用于锁定 visual-style 旧单文件 owner 删除与 folder-backed owner 树完整性。
- 2026-07-17 current source：`component_registry.rs` / `preferences.rs` 两项 root owner debt 已按 M1.T1 硬切为 folder-backed owner 树，Python audit 为 `classified-and-clear`、迁移债/root owner violations 均为 0；受管偏好 12/12（另 1 项显式截图 exporter ignored）、组件 1/1、`structure_convention` 3/3 全部 exit 0，独立复审 Critical/Important/Minor = 0/0/0，已具备向 Editor07 回传 [fixed](../editor/07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md) 的完整证据，详见 [子计划记录](10/2026-07-17-ui-root-owner-hardcut.md)。
- open 待修复：[editor-ui-plan-output-notices](10/failure-2026-07-13-editor-ui-plan-output-notices.md)
- fixed 已修复：[ui-root-owner-boundary-migration-debt](../editor/07/fixed-2026-07-17-ui-root-owner-boundary-migration-debt.md)
- 2026-07-22 audit test性能补充：PERF-MVP-561让两个`structure_convention`测试通过`OnceLock<Value>`共享一次`audit_editor_structure.py` JSON结果，不再每test重复spawn/全仓scan；源码合同2/2、直接audit `classified-and-clear`通过。current-source Cargo与统一changed-file inventory仍待，不把CI收益计作产品运行时收益。
