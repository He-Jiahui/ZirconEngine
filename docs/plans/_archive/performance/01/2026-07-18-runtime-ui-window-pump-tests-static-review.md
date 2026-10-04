---
related_code:
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
tests:
  - 4 tracked Rust files statically reviewed
  - dirty-domain and lifecycle semantics covered without timing assertions
  - current-source Cargo and event-burst counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI window input pump tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/runtime_window_input_pump.rs`与`tests/runtime_window_input_pump/**` 4/4个tracked Rust文件、889行。累计UI tracked source从484/783增至488/783。

测试覆盖window metrics/lifecycle、cursor hover/cancel、touch隔离、raw mouse motion、popup/tooltip dismissal与batch顺序。dirty-domain契约明确：window move保持clean，redraw request只标记render，resize/scale才标记layout+hit-test+render；closed/destroyed清hover而不伪造缺坐标pointer route。

## 性能结论

这些断言可直接保留为PERF-MVP-314的barrier分类基础：resize/scale是geometry barrier，move/activation/occlusion不是布局barrier，redraw request属于render-only合并域。当前用例仍普遍读取owned diagnostic notes、route trace、applied effects和host requests，且batch只验证顺序，不记录每域rebuild、route visits、note bytes或重复redraw合并数，继续回链PERF-MVP-293。

EditorUI01/Runtime12验收需在1/100/1,000 window/input events下记录各dirty domain transition、layout/render/hit rebuild、popup/timer scans、diagnostic bytes与queue age；连续redraw/move可合并，resize后pointer几何与focus/capture/popup边沿保持正确。current-source Cargo与F4产品window/input trace完成前保持pending。
