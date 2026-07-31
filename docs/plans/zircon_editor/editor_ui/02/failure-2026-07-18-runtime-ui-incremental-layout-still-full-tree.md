---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-incremental-layout-still-full-tree
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/tests/mui_responsive_layout.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
tests:
  - 10k-node stable viewport one-dirty-leaf stage visit counter
  - responsive breakpoint generation invalidation test
  - incremental arranged-tree and hit-test delta parity test
---

# Runtime UI增量布局仍夹带全树stage

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/layout` 22/22与surface产品调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI01消费增量hit/focus范围；PERF-MVP-032提供surface dirty authority。
- 交接原因：responsive、layout root、arranged tree与hit-test必须由EditorUI02的同一layout generation和changed set统一收口。

## 失败现象与复现证据

PERF-MVP-259：incremental入口在dirty root发现前执行responsive MUI多轮全node/slot扫描，root discovery再扫全tree；布局后surface全量重建arranged tree和hit-test。`visited_node_count`没有覆盖这些stage，导致性能指标低报。本轮只把geometry snapshot/compare缩到visited subtree。现有`mui_responsive_layout.rs`以约15-node fixture覆盖三组viewport语义，但没有responsive/root/arranged/hit visit counter，不能验收stable viewport成本。

## 最低共享层根因

responsive metadata、dirty roots、geometry delta、arranged tree与hit index没有共享tree/style/viewport generation和changed set；各stage只能重新扫描或重建全量projection。

## 架构修复验收

- viewport/style/tree generation维护responsive node index与compiled query，viewport generation未变时stable evaluation为0。
- layout roots直接读取dirty authority；measure/arrange产出changed geometry set，arranged tree与hit-test只patch受影响节点/边界。
- frame metrics分别记录responsive/root/measure/arrange/arranged/hit visits和时间，总访问量不得被`visited_node_count`低报。
- 100/1k/10k nodes、1 dirty leaf时各stage工作近O(changed subtree)；breakpoint、visibility、direction、slot、clip/hit order与current-source Cargo通过。

## 禁止临时方案

- 不得只重命名指标或关闭responsive处理以获得低visited数。
- 不得为每stage建立彼此不受同一generation失效约束的私有cache。

## 修复结果与回传

Open state: `等待EditorUI02回传generation-owned responsive/root索引、geometry delta、增量arranged/hit与真实stage counters`。
