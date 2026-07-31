---
related_code:
  - zircon_runtime/src/ui/tests/surface_slot_dirty_domains.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/arranged_tree.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - 4 slot dirty-domain tests reviewed
  - no-op revision and z-order no-layout semantics present
  - 4 dirty rebuilds and 15 surface_frame publications reviewed
  - large-tree changed-range counters and current-source Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface slot dirty测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/surface_slot_dirty_domains.rs` 1/1个tracked Rust文件、519行、4个测试。fixture覆盖Overlay/Canvas z-order与Free/Canvas placement mutation，执行4次dirty rebuild和15次`surface_frame()`发布/读取。

## 正向增量门禁

相同slot值返回false、revision不增长且dirty保持空；z-order改变只标hit+render，不标layout；placement改变才标layout+hit+render并传播到parent/child。这些语义应在slot edge authority与incremental arranged tree改造后保留，防止把所有slot变化重新粗化为full layout。

## PERF-MVP-277/281：精确flag后仍全量stage

两个z-order测试只有3个nodes，但报告明确`arranged_rebuilt/hit_grid_rebuilt/render_rebuilt=true`。当前`rebuild_dirty`对任一hit dirty重建整份arranged tree，后续hit/render也是全量；测试只断言dirty node count=1，未记录各stage visited/reused范围，不能证明工作随changed slot edge增长。EditorUI02/08需用generation-owned node/slot index和changed draw/hit range patch这些stage。

## PERF-MVP-278：测试消费深复制API

本文件15次调用`surface_frame()`，同一placement测试在连续断言中多次重新深clone tree/render/hit/report。测试侧可在后续整理为一次borrow/Arc snapshot，但根因是产品API仍返回完整owned frame；EditorUI08必须发布immutable generation handle，stable access payload clone=0。

## 验收要求

1/100/1k/10k nodes及slot edges、单次z-order/placement/no-op mutation记录slot probes、dirty propagation、arranged/hit/render visits、frame clone bytes与CPU p95。no-op所有stage=0；z-order只patchaffected order/range；placement只访问layout boundary及changed geometry。current-source Cargo、workbench popup/canvas产品trace和像素/命中完成前，本文件留在`pending.md`。
