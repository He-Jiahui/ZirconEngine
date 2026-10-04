---
related_code:
  - zircon_runtime/src/ui/tests/widget_radio_behavior.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/input
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 15 radio/range/scrollbar behavior tests reviewed
  - pointer keyboard accessibility and disabled parity present
  - 100k-move structure lookup/metadata parse/dirty-stage counters pending
  - current-source Cargo and F4 control interaction trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI radio/range/scrollbar测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{widget_radio_behavior,widget_range_navigation,widget_scrollbar_behavior}.rs`，共3/3个tracked Rust文件、1,093行、15个测试。范围覆盖radio pointer/keyboard/group/a11y、range Home/End/drag metrics，以及scrollbar track/thumb/capture/disabled/a11y ScrollTo。

## PERF-MVP-284：小控件fixture没有热路径预算

radio fixture只有2个options，无法测量产品选择时递归扫描group descendants并逐sibling更新的成本；range只有一次move，无法测量每move从metadata/state读取和解析min/max/step/value；scrollbar只有单target，无法测量String target全tree查找。EditorUI06应在tree/component generation编译group membership、range scalar和scroll target node index，pointer handler只读取typed context。

## PERF-MVP-262/315：scroll drag失效放大

thumb move更新scroll offset后binding report包含Layout+Render+Input；当前virtual layout仍全children position/hide，input dirty还会触发arranged rebuild。连续drag必须记录每move的layout/arranged/hit/render/input visits，结合indexed virtual range与typed property effect让工作随visible/changed range而非total nodes增长。

## Accessibility与诊断

radio/scrollbar各调用full accessibility snapshot，属于语义门禁但没有changed-node delta；drag测试逐event断言完整diagnostics/component event/binding update。PERF-MVP-256/257/293/294的snapshot/diagnostic/payload预算仍需产品规模证据。

## 验收要求

1/1k/10k group controls/scroll targets、连续100k select/drag/key/a11y actions记录tree/metadata probes、parse、property transactions、binding/diagnostic bytes、dirty stage visits与CPU p95。stable generation structure lookup近O(1)、per-move parse=0、transaction=1；virtual scroll不随total rows增长。current-source Cargo与F4 Inspector/Hierarchy control trace完成前，3/3留在`pending.md`。
