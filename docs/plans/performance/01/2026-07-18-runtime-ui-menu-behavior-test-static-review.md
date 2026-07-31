---
related_code:
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/input
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 12 menu/popup behavior tests reviewed
  - pointer keyboard Escape backdrop disabled and nested-topmost parity present
  - owner-lifecycle popup depth/bytes/age and event allocation counters pending
  - current-source Cargo and F4 menu product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI menu behavior测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/widget_menu_behavior.rs` 1/1个tracked Rust文件、549行、12个测试。范围覆盖MenuItem pointer/keyboard activation、Escape/backdrop dismiss policy、disabled popup owner、nested topmost close和inside-empty-space保持打开。

## 正向ownership门禁

nearest/topmost popup关闭而不误关parent、disabled owner阻止dismiss、无item binding仍可关闭popup，这些语义应在popup stack索引/owner lifecycle改造后保留。typed open property和binding source也应由PERF-MVP-265的单事务patch更新。

## PERF-MVP-297：depth 2不是长会话验收

测试最大popup depth 2，只在同一surface内构造并结束；没有不同owner持续open、重复id、owner detach/window close/focus loss、stale entry、容量/bytes/age或shutdown清理。每次Escape/outside click的ancestor查找、popup stack scan、component event/binding report和String payload也没有规模计数。长时间编辑器menu/tooltip使用仍可能积累或反复扫描owned state。

## 验收要求

popup depth 1/10/1k、连续100k open/toggle/dismiss、1/1k owners/windows记录ancestor/stack probes、property transactions、binding/effect clone bytes、entries/bytes/age与CPU p95。topmost lookup近O(1)或O(depth)有界；owner detach/window close后相关entries=0；全局hard cap不越界。current-source Cargo与F4主菜单/context menu产品trace完成前，本文件留在`pending.md`。
