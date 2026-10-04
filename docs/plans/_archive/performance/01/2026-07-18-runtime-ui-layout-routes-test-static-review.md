---
related_code:
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_manager.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
tests:
  - 6 built-in fixture/window-pump integration tests reviewed
  - resize-before-pointer geometry barrier present
  - 8 surface_frame calls, full report copies and debug JSON materialization present
  - current-source Cargo allocation/stage/route counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI layout routes测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs` 1/1个tracked Rust文件、630行、6个测试。范围覆盖QuestLog/Inventory/PauseMenu builtin fixtures、Taffy/Zircon route report、surface frame/render/hit/pointer authority，以及window-pump resize、ordered batch和失败index语义。

## MVP语义门禁

Inventory明确锁定virtualized list走Zircon fallback；QuestLog锁定native Taffy与Overlay fallback。resize后同batch pointer必须在派发前看到新geometry，再次确认PERF-MVP-314不能简单把所有batch rebuild延后到末尾；geometry event必须是typed barrier，纯move/render-only事件才合并。

## PERF-MVP-263/278/280：诊断与frame全物化

本文件8次调用owned`surface_frame()`、4处显式clone完整layout report，并各执行一次full debug snapshot和JSON序列化。route report测试又从完整`selections`多轮filter计数、构建BTreeMap校验reason。测试小规模成本可接受，但产品默认frame/diagnostics若同样执行会放大每帧clone/selection/String/serialization；稳定帧应只借用generation handle，完整selection/debug JSON必须按需。

## PERF-MVP-293/314：默认route诊断与逐event rebuild

window pump测试逐结果检查owned notes，pointer helper比较完整route/stack/path；这是诊断语义门禁，不是默认热路径预算。batch仍逐event调用入口，resize barrier正确但纯move/analog storm没有coalescing计数。EditorUI01/Runtime12需记录normalize/route/diagnostic bytes和每domain rebuild次数。

## 验收要求

对三fixture稳定300 frames、1/100/1k event batch记录frame/report/debug/route clone bytes、selection scans、layout/arranged/hit/render rebuild及CPU p95。diagnostics off完整notes/selection/JSON build=0；stable frame payload clone=0；resize后首pointer命中新geometry，纯move burst rebuild常数有界。current-source Cargo与F4产品trace完成前，本文件留在`pending.md`。
