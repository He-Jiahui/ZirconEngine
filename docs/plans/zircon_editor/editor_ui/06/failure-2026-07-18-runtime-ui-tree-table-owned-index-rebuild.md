---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-tree-table-owned-index-rebuild
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/component/state_reducer/tree_view.rs
  - zircon_runtime/src/ui/component/state_reducer/tree_view/editing.rs
  - zircon_runtime/src/ui/component/state_reducer/table.rs
  - zircon_runtime/src/ui/component/state_reducer/keyboard.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/IItemsSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h
tests:
  - 100k-node tree navigation and edit lookup scale test
  - 100k-row typed sort main-thread budget test
  - row add remove reparent generation update test
---

# Runtime UI TreeView与DataGrid按事件重建owned索引

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：tree/table/keyboard reducer产品路径
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 联动责任：EditorUI02消费visible range；EditorUI01消费focus/navigation contract。
- 交接原因：TreeView/DataGrid model、selection、sorting与editing语义均是EditorUI06组件行为权威。

## 失败现象与复现证据

PERF-MVP-267：TreeView每select/expand/nav/edit递归重建id Vec，Vec线性去重最坏O(N²)，begin edit又第二遍找label。PERF-MVP-268：DataGrid在UI线程完整排序rows，原常见字符串比较分配；column resize原先clone全部width map。本轮已直接消除后两类局部clone，但large sort、field scan与tree root仍open。

## 最低共享层根因

通用nested `UiValue`没有generation-owned linearized tree、id/parent/label/row index或typed column comparator；每个事件临时派生完整视图。

## 架构修复验收

- tree model持有linearized visible rows、id→row/parent/label、expanded/selected/disabled sets与adjacency；row delta增量patch。
- nav/select/edit/expand按index/set操作，stable parse/id clone/dedupe=0；range选择O(range)。
- table按schema持有typed comparator与field index；大sort/filter在task/model层发布generation-tagged row permutation，UI线程不越budget。
- 1k/10k/100k规模记录visits、clone bytes、dedupe probes、compare alloc、main/worker CPU与age；duplicate/reparent/rename/server sort/NaN/null/stable order及Cargo通过。

## 禁止临时方案

- 不得每个event lazily缓存一次后仍因alias字段无条件失效。
- 不得后台排序直接借用可变UI state；请求和结果必须带model generation并可丢弃stale结果。

## 修复结果与回传

Open state: `table两项局部clone已止损；等待EditorUI06回传tree/table generation model、typed comparator、worker budget与规模证据`。
