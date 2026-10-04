---
related_code:
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - 5 tracked Rust files and 18 test definitions statically reviewed
  - focus dirty-domain, modal trap, restore, tab, and directional semantics covered
  - history bounds and large-tree candidate counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI focus/navigation tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/focus_navigation.rs`与`tests/focus_navigation/**` 5/5个tracked Rust文件、1,080行、18个测试。累计UI tracked source从493/783增至498/783。覆盖autofocus、focus-visible、property disable、modal/popup trap与restore、tab/group order和directional override。

## 性能结论

focus state变化明确断言只标记render，unchanged/rejected property mutation不产生focus change，这是正确的增量契约。另一方面，测试直接断言`focus.changes`、`focused_inputs`与`modal_restore_stack`累积内容，生产`surface/focus.rs`也对changes/focused_inputs持续push；没有entry/byte/age/clear测试。tab/directional用例只有2到5个候选，也没有暴露每键全树candidate重建与排序的node visits。

unbounded history继续回链既有focus/input history handoff；每键候选重建回链PERF-MVP-252，typed route/history与modal stack预算联动PERF-MVP-293/297。1/100/10,000 focusable nodes和连续1M keyboard/text/IME事件需记录tree visits、sort、route/history clone bytes、entries/age、dirty nodes与CPU p95；stable focus/no-op mutation不增长history，modal restore有depth hard limit，focus change只render-dirty。current-source Cargo与F4 keyboard/gamepad产品trace完成前保持pending。
