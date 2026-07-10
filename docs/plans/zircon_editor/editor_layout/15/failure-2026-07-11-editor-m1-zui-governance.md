---
handoff_kind: failure
status: open
created_at: 2026-07-11
summary_slug: editor-m1-zui-governance
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_editor/editor_layout/15
related_code:
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance
  - zircon_editor/assets/ui/editor
  - zircon_runtime/assets/ui
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
  - cargo test -p zircon_editor --lib --locked -- --test-threads=1
---

# Editor Layout 15：Editor M1 ZUI 治理失败交接

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：Editor M1 Windows 全量失败聚类与 V2 公共契约闭环测试阶段
- 修复责任计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 交接原因：失败集中在 Editor Layout 15 所有的 production `.zui` 资产与共享 governance expectation，来源计划的 Editor kernel 不拥有这些资产规则。

## 失败现象与复现证据

Editor 架构会话的独立诊断 binary 中 `zui_asset_governance` 为 68/71。布局 owner 最新官方单线程完整门禁为 2761 passed / 133 failed / 34 ignored，且该切片自有 focused tests 全绿，因此不能把 133 项整体归因于本计划，必须在当前源码重新运行治理组并逐项建立 owner 映射。

复现命令：

```text
cargo test -p zircon_editor --lib --locked zui_asset_governance -- --test-threads=1
cargo test -p zircon_editor --lib --locked -- --test-threads=1
```

## 最低共享层根因

当前已证明的最低边界是 production authored `.zui` 与共享 governance expectation 不一致；尚未把 3 个具体失败逐项归因到资产或规则 owner。修复者必须先在当前源码取得精确失败清单，再定位最底层共享契约，不得从 133 项全量失败反推临时上层补丁。

## 架构修复验收

- 当前源码的 `zui_asset_governance` 失败逐项映射到 authored `.zui` 或共享 governance owner。
- 先修生产资产或共享规则，并为每类根因增加最低层回归。
- governance 组全绿后再向上运行完整单线程 Editor 门禁并记录精确结果。

## 禁止临时方案

- 禁止恢复旧 `.ui.toml` / `.v2.ui.toml`、旧 `kind = "layout"` 或兼容 loader 路线。
- 禁止为单个截图、单个测试或单一资产增加绕过 governance 的特殊分支。

## 修复结果与回传

- 状态：`open / 待修复`。
- 当前不声明 Editor Layout 15 或 Editor M1 的完整门禁通过。
- 修复验收后，修复者必须更新本文件、移动到 `docs/plans/zircon_editor/editor/01/`，并重命名为 `fixed-{resolved_at}-editor-m1-zui-governance.md`；Editor Layout 15 仅保留相对链接和已修复摘要。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| S15 / Editor M1 | Current-source ZUI governance 重建复验 | `未通过-70/71-单一L4槽位合同待修` | 2026-07-11 | 当前 owner binary `F:\cargo-targets\zircon-ui-state-priority-0711\debug\deps\zircon_editor-1ca47919e17744f1.exe zui_asset_governance --test-threads=1 --nocapture` 完整执行 71 项为 70 passed / 1 failed，耗时 121.08s。唯一失败 `workbench_shell::l4_surfaces_contain_no_inline_primitive_structures` 报告 `activity_drawer_window.zui` 的 `bottom_left_slot`、`bottom_right_slot`、`content_slot`、`left_bottom_slot`、`left_top_slot`、`right_bottom_slot`、`right_top_slot` 七个 `Slot` 节点不在 L4 Workbench primitive/shell/structural allowlist。Layout owner 应裁决 `Slot` 是否为合法结构节点并在共享治理规则/资产中单点修复；禁止恢复旧 asset suffix/kind 或对该文件加路径特例。 |
