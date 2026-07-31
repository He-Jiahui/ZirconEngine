---
related_code:
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/dispatch
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 9 pointer/keyboard/popup/tooltip semantic tests reviewed
  - typed widget behavior and runtime disabled gate present
  - route/effect/binding allocation and popup-state long-session counters pending
  - current-source Cargo and F4 product interaction trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI pointer/popup测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{pointer_click_semantics,popup_tooltip_state}.rs`，共2/2个tracked Rust文件、544行、9个测试。范围覆盖double click、typed Button/Toggle widget contract、pointer/keyboard disabled gate、binding report，以及popup open/toggle和tooltip arm/show/cancel。

## 正向MVP门禁

行为由`UiWidgetContract`而非component name猜测，disabled runtime state同时阻断pointer与keyboard mutation；这些语义应在PERF-MVP-265 typed atomic patch后保留。popup toggle清除匹配stack entry、tooltip只保留单一pending/visible state，是有界状态设计的基础。

## PERF-MVP-265/293/294：热事件缺少成本门禁

测试只派发单次或少量事件，断言完整route/component event/binding report/host request；没有记录route/path/diagnostic Strings、effect payload、binding updates、property transactions或clone bytes。默认完整diagnostics与多份effect ownership仍由EditorUI01处理，Button/Toggle状态一次动作单事务由EditorUI06处理。

## PERF-MVP-297：popup长会话未验收

popup测试最大stack深度1，仅验证同id toggle；没有不同popup连续open、owner shutdown、focus/window close、stale owner或容量/bytes/age。popup/tooltip/drag/capture状态必须按owner lifecycle关闭并有hard cap，不能依赖测试结束释放。

## 验收要求

1/1k/10k controls、连续100k pointer/keyboard events与popup depth 1/10/1k记录route/effect/binding/property transactions、String/Vec clone bytes、state entries/age和CPU p95。diagnostics off完整notes=0，widget action transaction=1，popup/tooltip owner close后entry=0且预算不越界。current-source Cargo与F4按钮/menu/tooltip产品trace完成前，2/2留在`pending.md`。
