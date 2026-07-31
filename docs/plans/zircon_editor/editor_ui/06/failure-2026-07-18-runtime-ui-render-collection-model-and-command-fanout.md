---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-render-collection-model-and-command-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/render/collection_rows
  - zircon_runtime/src/ui/surface/render/command_palette.rs
  - zircon_runtime/src/ui/surface/render/notification_center.rs
  - zircon_runtime/src/ui/surface/render/popup_menu.rs
  - zircon_runtime/src/ui/surface/render/popup_options.rs
  - zircon_runtime/src/ui/surface/render/segmented_controls.rs
  - zircon_runtime/src/ui/surface/render/sliders.rs
  - dev/slint/internal/core/model.rs
  - dev/slint/internal/core/model/adapters.rs
tests:
  - 100k-row visible-only render projection test
  - command palette filter mapping scale test
  - tree-depth and slider-tick command budget test
---

# Runtime UI collection render全量模型重建与command fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface render collections/popup/palette/notification/decorations
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI02提供visible range；Render17提供instance/batch contract。
- 交接原因：typed collection model、filter/selection和component decoration预算属于EditorUI06。

## 失败现象与复现证据

PERF-MVP-290/291：每帧从TOML/UiValue重建rows/options/sets并可能绘制全部clip外rows；palette filtered ids O(F*N)，notification limit在全parse之后；tree depth、slider ticks和row数可无界生成command。本轮只止损文本clone、set线性扫、popup逐row定位和ASCII normalize。

## 最低共享层根因

PERF266-269的typed generation model没有贯穿到render；render自行恢复模型、可见范围与装饰primitive。

## 架构修复验收

- render消费generation-owned visible row handles、filtered mapping和selection/disabled sets；stable parse=0。
- visible+overscan之外不生成row command；notification/popup/palette有viewport或hard cap。
- tree guides和slider ticks使用bounded aggregated/instanced primitive，恶意metadata不造成无界command。
- 1k/10k/100k rows与depth/ticks 1/100/10k记录parse/clone/set probes/commands/bytes和CPU p95；语义/Cargo/产品trace通过。

## 禁止临时方案

- 不得只reserve Vec或提高hard cap而保留全量模型重建。
- 不得在render另建一份与component reducer不同步的model cache。

## 修复结果与回传

Open state: `等待EditorUI06回传visible generation model、filter mapping、decoration budget与规模证据`。
