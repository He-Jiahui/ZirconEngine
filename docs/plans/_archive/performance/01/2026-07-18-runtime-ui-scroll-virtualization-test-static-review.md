---
related_code:
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 3 functional tests reviewed on 4/6-row fixtures
  - virtual-window and dirty-domain semantics present
  - position/hide/measure visit counters and 100k-row continuous-scroll test pending
  - current-source Cargo and workbench product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI scroll virtualization测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/scroll_virtualization.rs` 1/1个tracked Rust文件、224行、3个测试。测试覆盖fixed-extent virtual window、offset/viewport/content变化的visible-range判定，以及非虚拟scroll对layout/hit/render/input dirty domain的语义。

## PERF-MVP-262：结果虚拟化不等于工作虚拟化

历史测试`virtualized_list_only_materializes_visible_window`创建6行并断言窗口外frame保持default，证明最终arranged结果正确；它没有证明布局只访问可见行。该旧名现已retired并hard cut为`retained_virtual_list_only_arranges_visible_window`。产品`arrange_scrollable_children`仍先对全部children执行`child_positions`，随后全量enumerate；每个offscreen child调用`hide_subtree_layout`递归清零。measure阶段也先递归测量所有children。固定extent单步scroll因此仍随总行数增长，新测试名也只可作为geometry验收，不能作为实例数或复杂度验收。

## 保留的语义门禁

offset变化只有跨visible window时才设置`visible_range_changed`，viewport/content变化必须触发range invalidation；非虚拟scroll则保持full-window dirty而不伪造visible-range delta。这些断言应在indexed range generation改造后保留，并扩展focus、accessibility、capture与nested scroll。

## 验收缺口与责任

EditorUI02继续负责fixed extent算术position、variable extent prefix/Fenwick或分块索引，以及进入/离开窗口edge delta；EditorUI01和Text09消费同一range/measure generation。1k/10k/100k rows、visible 10/50、连续10k scroll必须记录measure/position/slot/layout/hide visits和CPU p95，证明offscreen visited=0且per-step不随total rows增长。

current-source Cargo、规模counter与MVP Hierarchy/Asset Browser滚动trace完成前，本文件留在`pending.md`，不进入`review.md`。
