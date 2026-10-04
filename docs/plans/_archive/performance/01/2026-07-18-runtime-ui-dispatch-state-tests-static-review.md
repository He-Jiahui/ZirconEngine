---
related_code:
  - zircon_runtime/src/ui/tests/runtime_dispatch_effect_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_drag_drop_component_state.rs
  - zircon_runtime/src/ui/tests/runtime_loading_component_state.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/component
  - zircon_runtime/src/ui/v2
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 7 effect/drag/loading state tests reviewed
  - applied/rejected effect index and v2 pseudo-style parity present
  - effect payload clone rule-probe transaction and 120-Hz drag counters pending
  - current-source Cargo and F4 drag/loading product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI dispatch/component state测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{runtime_dispatch_effect_matrix,runtime_drag_drop_component_state,runtime_loading_component_state}.rs`，共3/3个tracked Rust文件、1,272行、7个测试。范围覆盖17类dispatch effect中的主要apply/reject合同、drag session/source/target flags、loading property与v2 pseudo-style/painter投影。

## PERF-MVP-294：effect多份所有权

matrix测试一次应用8个effects或拒绝15个effects，要求保持effect index、完整effect值、reason、host request和component event。产品当前把相同String/Vec/text/payload在reply、applied/rejected、host request与diagnostics之间多份保存；测试只比语义，没有payload clone bytes、effect storage count、reason format或batch CPU预算。

## PERF-MVP-275/265：typed flags仍投影通用attributes

drag/loading测试证明`UiComponentFlags`变化会同步为`dragging/drop_hovered/active_drag_target/loading` runtime attributes，再驱动v2 pseudo style和painter；小树只有2个目标和4条rules。运行态样式仍可能遍历目标子树、扫描全部pseudo rules并重建attributes/style/tokens maps。EditorUI04应让compiled selector index消费interned state bitset/typed delta，EditorUI06一次事务更新canonical flag，不把TOML map作为第二authority。

## 验收要求

effects 1/100/1k、payload 0/1KiB/1MiB、1/1k/10k nodes/rules、60/120 Hz连续100k drag updates记录payload/String clone bytes、effect copies、reason formats、rule probes/map clones、transactions、dirty nodes和CPU p95。每effect payload owner有明确上限，drag update只访问source/old/new target与candidate rules，transaction=1。current-source Cargo与F4 drag/loading产品trace/像素完成前，3/3留在`pending.md`。
