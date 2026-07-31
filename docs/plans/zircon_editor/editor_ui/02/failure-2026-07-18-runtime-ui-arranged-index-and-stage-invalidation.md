---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-arranged-index-and-stage-invalidation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/interaction_gate.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetHeap.h
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - 10k-node arranged node-slot-ancestor probe counter
  - one-percent dirty stage visited-node test
  - stable-generation zero-arranged-hit-render rebuild test
---

# Runtime UI arranged无索引且dirty下游全量重建

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：surface core当前批21文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`
- 联动责任：EditorUI01消费indexed focus/hit path；EditorUI08拥有published frame generation。
- 交接原因：arranged/layout/slot index与stage invalidation由EditorUI02定稿。

## 失败现象与复现证据

PERF-MVP-277/281：arranged build重复祖先与全slots走查，`UiArrangedTree::get`线性；incremental layout后仍全量arranged/hit/render。局部止损已把dirty flags/count合并为一次树扫描，但主要stage工作未变。

## 最低共享层根因

arranged artifact缺node/slot dense index与inherited effective-state cache，dirty transaction也没有携带changed nodes/ranges跨越layout→arranged→hit→render边界。

## 架构修复验收

- node id→dense index、parent/child/slot直接查询；一次DFS计算clip/visibility/input/disabled，draw order独立。
- changed ranges按layout boundary增量patch arranged/hit/render，stable generation所有stage visits=0。
- 1/100/1k/10k nodes记录node/slot/ancestor probes、visited/reused/rebuilt/damage和CPU p95；单叶/1% dirty不随N全扫。
- z/paint/canvas/clip/focus/hit/像素与serde/Cargo通过。

## 禁止临时方案

- 不得只给某个consumer加私有HashMap，留下arranged owner与其他consumer继续线性查找。
- 不得把全量stage移动到worker后无限积压；changed set、budget和frame publish必须闭环。

## 修复结果与回传

Open state: `等待EditorUI02回传generation-owned arranged index和跨stage changed-range证据`。
