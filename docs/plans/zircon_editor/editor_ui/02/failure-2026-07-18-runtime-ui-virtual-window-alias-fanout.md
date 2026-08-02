---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-virtual-window-alias-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/virtualization.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_virtualization.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - dev/slint/internal/core/model.rs
tests:
  - 100k-row one-step virtual scroll transaction counter
  - unchanged virtual-window zero-layout-invalidation test
  - table and tree alias compatibility test
---

# Runtime UI virtual window每步18 alias事务

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：table/tree default virtual scroll
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI06拥有typed component patch与binding alias projection。
- 交接原因：canonical visible range、overscan、scroll offset与layout invalidation属于EditorUI02。

## 失败现象与复现证据

PERF-MVP-286：table/tree每次virtual window变化分别写18个count/range/overscan/scroll兼容属性，每个属性独立mutation、binding report与dirty/style判断。一格滚动触发18个同步事务，且与PERF-MVP-262的全child布局放大叠加。

## 最低共享层根因

visible range没有单一typed authority；内部state同时保存snake/camel及同义alias，layout无法区分canonical变化和边界投影。

## 架构修复验收

- EditorUI02发布generation-owned canonical visible window，单次更新start/count/overscan/offset和changed range。
- EditorUI06一次提交typed patch；alias只在binding/serde边界投影，内部authority=1。
- 1k/10k/100k rows连续10k scroll记录property calls、binding reports、dirty/layout visits与CPU p95：每step transaction=1、unchanged invalidation=0、offscreen work不随total增长。
- table/tree、fixed/variable extent、focus/accessibility、alias兼容与current-source Cargo/产品trace通过。

## 禁止临时方案

- 不得只把18次调用包进循环helper；验收要求一次事务和一次dirty union。
- 不得删除公开alias而无迁移；alias应保留在边界projection。

## 修复结果与回传

Open state: `等待EditorUI02联动EditorUI06回传canonical virtual window、single patch与布局规模证据`。
