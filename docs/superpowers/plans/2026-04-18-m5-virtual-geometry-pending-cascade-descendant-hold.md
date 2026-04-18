---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的更深 residency-manager cascade / split-merge policy
  - user: 2026-04-18 继续任务
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-multi-level-frontier-collapse-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-merge-back-child-hysteresis.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_keeps_resident_grandchild_pages_hot_while_multi_level_cascade_request_remains_pending
  - cargo test -p zircon_graphics --offline --locked visibility
doc_type: milestone-detail
---

# M5 Virtual Geometry Pending-Cascade Descendant Hold

**Goal:** 把 `Virtual Geometry` 的 split-merge hysteresis 从“一拍 merge-back hold”继续推进到“只要 ancestor cascade request 还挂着，就继续保热更深 resident descendants”。

**Non-Goal:** 本轮不实现真正的 page hierarchy residency manager、GPU-driven page replacement 或 Nanite-like residency scoring。

## Delivered Slice

- `build_virtual_geometry_plan(...)` 现在会继续保留仍未 resident 的上一帧 `requested_pages`，前提是这条 request 仍然能沿当前 visible frontier 找到有效 hierarchy 路径。
- 在 request 继续挂起时，planning 会额外计算：
  - `requested_frontier_cluster_ids`
  - 同一 visible frontier 下、最深层的 resident hidden descendants
- 这些 deepest resident hidden descendants 会继续被排除在 `evictable_pages` 之外，直到 pending ancestor request 落地或 frontier 真的脱离这条恢复路径。

## Why This Slice Exists

- 之前的 multi-level frontier collapse 只修到了第一拍：
  - 第一次 collapse 会补请求缺失 ancestor page
  - 也会保护上一帧活跃 resident descendants 不立刻进入 `evictable_pages`
- 但第二个 collapsed frame 里，系统会把：
  - `requested_pages` 清空
  - `400/500` 这类 still-hot resident descendants 提前放回 `evictable_pages`
- 这会让 runtime host 在 ancestor page 还没真正 resident 前，就开始回收本该保热的 deeper descendants，等于把 cascade 恢复路径自己拆掉。

## Validation Summary

- `visibility_context_keeps_resident_grandchild_pages_hot_while_multi_level_cascade_request_remains_pending`
  - 证明当 ancestor page request 还在 pending 时，第二个 collapsed frame 仍会保留 `requested_pages=[300]`，并继续把 deeper resident descendants 挡在 `evictable_pages` 外。
- `visibility`
  - 证明直接 parent/child merge-back、page priority、visibility-owned draw-segment lineage 和 Hybrid GI visibility 都没有因为这条更宽 hold 回归。

## Remaining Route

- 把这层 pending-cascade hold 继续下沉到更真实的 runtime/readback authority，而不只停在 visibility planning。
- 继续把 slot reuse / page table truth / draw submission ownership 对齐成更完整的 hierarchy residency policy。
