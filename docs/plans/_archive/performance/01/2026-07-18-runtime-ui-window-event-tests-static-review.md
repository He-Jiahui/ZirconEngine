---
related_code:
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
tests:
  - 7 tracked Rust files and 22 test definitions statically reviewed
  - resize-before-pointer geometry dependency explicitly covered
  - current-source Cargo and burst diagnostics counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI window/ABI event tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/runtime_ui_window_event_routes/**` 7/7个tracked Rust文件、1,655行、22个测试。累计UI tracked source从477/783增至484/783。测试覆盖normalized/platform/ABI pointer、wheel、touch、cursor leave、keyboard、gamepad button/axis、window lifecycle、batch顺序与错误index。

静态扫描只有1处clone，但有20组diagnostic notes读取、9组route steps读取与3组显式rebuild语义。用例全部是功能/顺序断言，没有事件规模、诊断字节、route visits或rebuild计数预算。

## 对PERF-MVP-293/314的证据

`runtime_ui_manager_runtime_event_batch_rebuilds_before_followup_pointer_input`明确要求同一ABI batch中的viewport resize先完成layout，随后pointer hit-test才可路由到新位置。这证明PERF-MVP-314必须采用typed geometry barrier，而不能把所有事件无条件推迟到batch末尾统一rebuild。

另一方面，keyboard/gamepad/pointer/window测试默认断言完整`route_trace`、`route_steps`和多条String notes；当前测试没有opt-in capture模式，因而会把PERF-MVP-293计划删除的release默认重诊断误当作不可变产品契约。修复时应把语义断言迁移到显式diagnostic capture测试，默认路径只断言轻量route summary与最终state。

## 验收与责任计划

EditorUI01联动Runtime12实现统一event barrier/coalescing authority，并将full diagnostics变为有entry/byte/age预算的显式capture。1/100/1,000 ABI/platform事件分别记录adapter calls、route visits、note/step bytes、layout/render/hit rebuild和queue age；resize→pointer、accepted prefix/error index、pointer/capture/touch边沿保持等价。现有输入handoff已补本目录，current-source Cargo与F4产品trace完成前继续pending。
