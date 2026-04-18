---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry unified indirect / deeper cluster raster / residency-manager cascade
  - user: 2026-04-18 列出后续所有 tasks，把它们作为 todo，然后继续深入
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-page-table-residency-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-merge-back-child-hysteresis.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_requests_nonresident_ancestor_page_and_holds_descendants_when_frontier_collapses_multiple_levels
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo test -p zircon_graphics --offline --locked render_server_bridge
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Multi-Level Frontier Collapse Cascade

**Goal:** 把 `Virtual Geometry` 的 split-merge / residency cascade 再推进一层：当上一帧 frontier 已经下探到 resident descendants，而当前帧因为中间 ancestor page 掉 resident 导致 frontier 多级塌回 coarse parent 时，visibility planning 不再只会“退回 parent + 立刻回收 descendants”，而是会优先请求缺失的 ancestor page，并保护上一帧活跃的 resident descendants 一拍。

## Delivered Slice

- `build_virtual_geometry_plan(...)` 现在新增了一条 hierarchy cascade 请求路径：
  - 对上一帧还活跃、这一帧被隐藏的 visible descendant clusters
  - 沿 `parent_cluster_id` chain 向上找
  - 如果在某个当前可见 ancestor 之前遇到了 nonresident ancestor page
  - 就把这条最高层缺失 ancestor page 放进优先 `requested_pages`
- `requested_pages` 不再只来自当前 `streaming_target_clusters` 的直接 page 去重排序；现在会先消化这批 hierarchy-cascade ancestor requests，再消费常规 page priority 请求。
- `merge_back_child_hold_protected_pages` 也从 direct-parent 扩展成了 “任一当前可见 ancestor”：
  - 只要上一帧活跃 descendant 这一帧因为 multi-level collapse 被隐藏
  - 且它的 ancestor chain 最终回到了当前 visible frontier
  - 它对应的 resident page 就不会在 collapse 首帧直接落进 `evictable_pages`

## Why This Slice Exists

- 之前的一层 merge-back hysteresis 只覆盖 direct child -> parent：
  - parent visible
  - child previous visible
  - child 本帧 hidden
- 但更深 hierarchy 下会出现更糟的情况：
  - 上一帧 frontier 已经在 grandchild / deeper descendant
  - 本帧中间 ancestor page 掉 resident
  - frontier 直接塌回更粗的 ancestor
- 这时旧逻辑有两个空洞：
  - 不会回补丢失的 nonresident ancestor page
  - 会把上一帧活跃的 resident descendants 直接扔进 `evictable_pages`
- 结果就是 deeper hierarchy 既不能快速恢复，也会在 frontier collapse 首帧就把 descendant residency 热状态冲掉。

## Validation Summary

- `visibility_context_requests_nonresident_ancestor_page_and_holds_descendants_when_frontier_collapses_multiple_levels`
  - 证明 multi-level collapse 现在会优先请求缺失 ancestor page，并把上一帧活跃的 resident descendants 从第一轮 `evictable_pages` 里保护出去。
- `visibility`
  - 证明新的 cascade request + descendant hold 没有破坏现有 parent/child hysteresis、page priority、requested-page history 和 Hybrid GI visibility coverage。
- `virtual_geometry`
  - 证明这条更宽的 split-merge / residency cascade 没有破坏 runtime prepare、GPU uploader readback、shared indirect buffer 与 page-owned cluster raster 主链。
- `render_server_bridge`
  - 证明 render-server 统计链与 VG capability path 仍然稳定。

## Remaining Route

- 把这条 multi-level frontier cascade 继续推进到 runtime/readback authority，而不是继续只停在 visibility planning。
- 继续推进 unified indirect ownership 下沉，让 runtime / prepare / renderer / readback 对 cluster/page ownership 的真值进一步收敛。
- 继续推进 deeper cluster raster consumption / residency-manager cascade，把 split-merge hysteresis 与 slot/page truth 对齐到更完整的 page hierarchy。
