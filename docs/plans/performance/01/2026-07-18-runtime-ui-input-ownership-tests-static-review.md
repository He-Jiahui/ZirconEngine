---
related_code:
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
tests:
  - 7 tracked Rust files and 17 test definitions statically reviewed
  - source guard batch queued through shared Cargo CPU reservation
  - million-event ownership/state/diagnostics budgets pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI input ownership tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/runtime_input_ownership.rs`与`tests/runtime_input_ownership/**` 7/7个tracked Rust文件、1,212行、17个测试。累计UI tracked source从498/783增至505/783。覆盖input method、owner validation、drag/drop、high precision pointer、analog repeat suppression、route trace、popup与tooltip。

根文件`input_hot_paths_avoid_eager_capture_trace_and_effect_index_allocations`是本轮8项源码守卫的聚合入口，防止非terminal capture map复制、specialized trace先建后丢、effect index remap map、keyboard token/constraint String、IME layout/style clone与capture owner临时Vec回归；精确Cargo测试已在共享CPU FIFO等待。

## 性能结论

行为测试确认重复analog值在routing前被抑制，多pointer/capture/high-precision/drag session与stale owner保持隔离；但route trace测试显式要求popup stack、preview/bubble/focus路径多份owned Vec/String，drag payload在event/effect/state/result间的owner数量未计数，analog control map与popup/tooltip/drag state也没有entry/byte/age上限或1M事件长会话。

这些根因继续回链PERF-MVP-293/294/297和EditorUI01/Runtime12：release默认route trace/full notes为零分配，effect/payload authoritative owner=1，typed/interned control id与popup/timer/drag/capture state有hard budget，move/analog可coalesce但边沿不丢。1/100/10,000 owners与1M events记录clone/alloc bytes、state entries/age、route visits和CPU p95；current-source Cargo与F4产品trace完成前保持pending。
