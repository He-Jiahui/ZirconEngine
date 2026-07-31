---
related_code:
  - zircon_runtime_interface/src/ui/dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/bevy/crates/bevy_input/src/mouse.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/tests/ui_dispatch_error_contracts.rs
  - current-source Windows dispatch/input tests pending
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Runtime interface UI dispatch clean subset 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/dispatch/**`当前 **18/19** 个clean Rust文件、**1,316** 行已逐文件阅读；外部dirty `input/reply.rs`未读取、未吸收，目录整体仍pending。已反查runtime pointer/navigation dispatch、surface input结果构造及interface合同。本轮未修改Rust源码。

## 性能结论

- interface pointer/navigation context必须owned整份route；产品因此先clone route进result、复制候选Vec，再为每个命中node/phase clone含多组path Vec的route。归 **PERF-MVP-254**：single route generation由result持有或共享，handler context只借用route+node/phase，候选直接遍历已有ordered slices。
- `UiInputDispatchResult`同时持完整event/reply、route trace/steps/notes、applied/rejected effect副本、host requests、component events与binding reports；产品多个入口还显式`event.clone()`或`reply.clone()`。归 **PERF-MVP-293/294**：release默认只发布compact outcome/effect index，full event/route/diagnostic受capture entries+bytes+age预算，effect payload只有一个owner。
- pointer/navigation result又同时拥有route、invocations、passthrough/damage、component events与binding reports；`UiPointerComponentEvent::new()`复制tree id、control/binding identity并构造wide envelope。归 **PERF-MVP-265/278**：normal event使用generation-scoped tree/node/control/binding handles与single changed receipt，serde/authoring边界才物化wide DTO。
- window/surface metadata及physical/logical key、analog control、popup/tooltip/option/toast id均为per-event String，继续回链 **PERF-MVP-297/426**；高频move/analog必须在Runtime12 barrier batch内合并，不能靠结果队列承受风暴。
- `UiInputMethodSurroundingText`有4,000-byte硬上限和UTF-8边界校验，是正向基线；composition rectangles仍是无上限Vec且request拥有完整surrounding String，继续回链 **PERF-MVP-296**。u64 user/device/pointer/session/sequence、Copy pointer effect/range与boxed drag payload也是正向基线。
- UE `FReply`以handled state加共享widget/drag operation引用表达capture/focus/drag请求；Bevy把mouse motion/scroll作为帧内accumulated resource。Zircon采用shared identity、compact receipt及frame accumulator原则，同时保留自身serde/ABI合同。

## 动态验收

1. route depth 1/16/64、handlers 1/4/32、连续1M pointer/navigation events：记录route/event/reply clone bytes、context/result Vec allocations、candidate visits、owners与p95；handler数不增加route bytes，normal route owner=1。
2. effects/diagnostics 1/10/1k、payload 0/1 KiB/1 MiB：记录effect/event owners、full diagnostics bytes、format、capture dropped/age与RSS；release full trace alloc=0，payload owner=1。
3. 125/500/1000 Hz move/analog与IME composition rects 1/100/10k：记录String/Vec bytes、coalesced/dropped、queue age、main-thread p95；edge事件保序且所有入口有硬预算。
4. current interface合同、runtime pointer/navigation/surface dispatch tests与F4 pointer/keyboard/IME产品trace通过。

current-source Cargo、规模counter、F4产品trace及dirty `input/reply.rs`独立审查未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
