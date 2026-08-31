---
related_code:
  - zircon_editor/src/ui/retained_host/hierarchy_pointer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01/failure-2026-07-17-retained-asset-pointer-full-surface-rebuild.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
  - dev/godot/scene/gui/tree.cpp
tests:
  - existing hierarchy bridge and retained-host pointer integration suites
  - current-source Windows focused Cargo and 10k-scene-row scroll/move storm pending
doc_type: implementation-evidence
status: superseded_by_2026-08-23_current_source_review
---

# Editor Retained Hierarchy Pointer 逐文件性能静态审查（2026-07-17）

> Superseded on 2026-08-23 by
> `2026-08-23-editor-retained-hierarchy-typed-row-receipt-hard-cutover-architecture-review.md`.
> Current source no longer rebuilds logical rows on scroll: it uses O(1) arithmetic row routing and
> O(V) visible-row paint. The O(N) findings below describe the 2026-07-17 source generation and must
> not be used as current acceptance evidence.

## 范围与覆盖

`zircon_editor/src/ui/retained_host/hierarchy_pointer` 当前共 **20** 个 Rust 文件，已按状态、布局、surface 构建、事件分发与滚动处理逐文件阅读 **20/20**。动态 Cargo、10k-scene-row scroll/move storm 与 route parity 尚未完成，因此继续留在 `pending.md`。

## 主要结论

- `sync()` 的 layout/state equality fast path正确，上层本轮也已改为复用 committed `Arc<[SceneEntry]>` 且 unchanged size 不重建 layout；这只消除了 app callback 的冗余 projection。
- `handle_scroll()` 在 offset 改变后仍调用 `rebuild_surface()`，重新创建 root/viewport/全部 scene row nodes、格式化 path、注册 dispatcher callback、clone `node_id` route、构建 route map 并执行 `surface.rebuild()`。
- `UiScrollableBoxConfig.virtualization` 为 `None`，因此每个有效滚轮步的构建与分配成本随完整场景节点数线性增长，而不是受 viewport 行数约束。
- move/click 共享 typed route 并保持 deterministic index order；优化不能用异步旧 generation 或 hash iteration 破坏 node identity 与命中顺序。

## 责任计划与验收

PERF-MVP-109 与 EditorUI01 handoff 已扩展到 hierarchy：资产行与场景行必须复用统一 stable row identity、viewport+overscan materialization、增量 scroll transform/hit-grid 和 state-only move dispatch。EditorUI08 只在 hierarchy generation/size 变化时同步 bridge。验收覆盖 1/100/10000 scene rows、1k scroll/move、resize、clamp、hover/click、selected/expanded projection、stale generation 与 deterministic route parity。
