---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/refine_visible_cluster_frontier.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry unified indirect ownership downshift or wider split-merge policy
  - user: 2026-04-18 continue M5
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-merge-hysteresis.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_resident_child_page_one_frame_when_frontier_merges_back_to_parent
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident
doc_type: milestone-detail
---

# M5 Virtual Geometry Merge-Back Child Hysteresis

**Goal:** 把 `Virtual Geometry` 的 split-merge policy 再扩一层，让 frontier 从 resident children 回退到 coarse parent 的当帧里，仍然 resident 的 child page 也会被额外保护一帧，而不是立刻进入 `evictable_pages`。

**Non-Goal:** 本轮仍然不实现完整的 residency manager 策略树、budget oversubscription merge hold、GPU-driven occlusion 或更深层 cluster raster hierarchy。

## Delivered Slice

- `build_virtual_geometry_plan(...)` 新增了 `merge_back_child_hold_protected_pages`：
  - page 当前仍 resident
  - 该 child cluster 上一帧确实处于 visible frontier
  - 当前帧这个 child 已经不再 visible
  - 但它的 `parent_cluster_id` 已经重新进入当前帧 visible frontier
- 满足这些条件时，child page 在 merge-back 当帧不会被写进 `evictable_pages`。
- 这层保护只持续一个过渡帧：
  - merge-back 当帧保护 resident child page
  - 下一帧如果 frontier 仍稳定停在 parent，child page 才重新进入 evictable 集

## Why This Slice Exists

- 之前的 `split hysteresis + merge-side parent hold` 已经解决了：
  - children 刚上传完成时不要立刻替掉 parent
  - 真正 split 到 children 的落地帧上，coarse parent 不要立刻被回收
- 但反向路径仍然不稳定：
  - 一旦因为 residency/budget 变化从 children 回退到 parent
  - 仍 resident 的 child page 会在同一帧马上被判定成 `evictable`
- 这会让 split-merge frontier 只在 coarse-parent 侧有缓冲，而 child side 没有任何回弹窗口。
- 本轮补上 merge-back child hold 后，split/merge 两侧都拥有最小的一帧稳定层，能更干净地承接后续 deeper refine / residency cascade。

## Validation Summary

- `visibility_context_holds_resident_child_page_one_frame_when_frontier_merges_back_to_parent`
  - 证明从 resident children 回退到 parent 的第一帧里，仍 resident 的 child page 不会立刻进 `evictable_pages`
- `visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident`
  - 证明新增 child-side hysteresis 没有破坏原有 coarse-parent split/merge hold 行为

## Remaining Route

- 把 split-merge hysteresis 继续推进到更完整的 hierarchy policy，而不是只停在 one-frame parent/child holds
- 把这层 frontier stability 继续接进更深的 cluster streaming、residency manager 与 cluster raster consumption 路线
- 继续朝统一 visibility-owned unified indirect / deeper cluster raster / Nanite-like execution 推进
