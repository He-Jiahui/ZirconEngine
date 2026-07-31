---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-default-control-table-tree-reprojection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/range.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/scrollbar.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_reparent.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/tree_view_support.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/IItemsSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h
  - dev/slint/internal/core/model/adapters.rs
tests:
  - 100k-control pointer move structure-discovery counter
  - 100k-row table sort and resize transaction test
  - 100k-node tree select edit reparent scale test
---

# Runtime UI基础控件与table/tree按事件全量重投影

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface default interactions与event routing 18/18
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI01拥有pointer intent/route；EditorUI02拥有virtual visible window。
- 交接原因：基础控件typed behavior、table schema和tree model是EditorUI06组件行为authority。

## 失败现象与复现证据

PERF-MVP-284至287：scrollbar move全树解析target，radio/range重复结构扫描和scalar解析；table resize/sort全量投影并多事务；tree每事件重建id、递归多轮flatten/remove/insert且按branch clone source，selection/edit/reorder再写5至9个属性。本轮已让产品table常见string-like comparator借用`&str`，但主成本仍open。

## 最低共享层根因

default interaction没有generation-owned typed control context、table schema或linearized tree；通用TOML/`UiValue`在每次事件中临时恢复静态结构和索引，结果再通过多个alias property逐项写回。

## 架构修复验收

- generation内编译control owner/target/group/range scalar、table field/comparator与tree id/row/parent/label/disabled/expanded索引。
- pointer move不全树扫描或重parse静态metadata；resize/selection/edit/reparent一次typed transaction提交。
- 大表sort/filter在有界task/model层发布generation-tagged permutation；stale result可丢弃且记录age。
- tree reparent先索引定位，再只patch source/target affected ranges；不得按失败branch clone subtree。
- 1k/10k/100k规模记录visits、clone/alloc bytes、transactions、main/worker CPU与age；disabled/cycle/duplicate/mixed/null/NaN/alias及Cargo/产品trace通过。

## 禁止临时方案

- 不得为每种控件建立独立、无失效契约的缓存。
- 不得把全量sort/reparent简单移到worker后允许无界排队；请求、预算、generation和取消必须闭环。
- 不得只减少alias数量而继续多事务维护多个内部authority。

## 修复结果与回传

Open state: `常见table scalar comparator分配已止损；等待EditorUI06回传typed control/table/tree generation model、single transaction与规模证据`。
