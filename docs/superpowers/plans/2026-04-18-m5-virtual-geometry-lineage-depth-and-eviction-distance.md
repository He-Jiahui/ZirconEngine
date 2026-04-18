---
related_code:
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/draw_segment.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/draw_segment.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry deeper cluster raster consumption 和更深 residency-manager cascade / split-merge policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline visibility -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Lineage Depth And Eviction Distance

## Goal

继续推进 `Virtual Geometry` 的两条 M5 主链：

- deeper cluster-raster consumption
- 更深的 residency-manager cascade / split-merge policy

这次切片的做法不是再加一层 renderer-side 临时推导，而是把两条 hierarchy truth 固定成统一 contract：

- visibility-owned `lineage_depth`
- runtime-owned `lineage distance`

## Delivered Slice

### 1. Visibility-Owned `lineage_depth`

- `VisibilityVirtualGeometryDrawSegment` 新增 `lineage_depth`
- `build_virtual_geometry_plan(...)` 会按 cluster parent chain 直接计算这条 depth
- runtime prepare / unified indirect projection / GPU submission segment buffer / segment readback 全部继续保留这条字段
- `virtual_geometry_indirect_args.wgsl` 已把 `lineage_depth` 接进真实 GPU indirect args 生成逻辑，不再只消费 `page_id / resident_slot / lod_level`

这意味着更深的 refined hierarchy truth 已经不是 visibility frontier 内部的临时信息，而是一直活到真实 GPU submission。

### 2. Runtime-Owned Lineage Distance Ordering

- `ordered_evictable_pages_for_target(...)` 不再只做：
  - unrelated
  - ancestor
  - descendant
- 现在会在 ancestor / descendant 内部继续按 lineage distance 反向排序：
  - 先回收更远 ancestor
  - 先回收更远 descendant

这样 runtime residency completion 在 page budget 紧张时，会优先保护更近的 active frontier page，减轻 deeper split-merge 切换时的 residency 抖动。

## Why This Slice Exists

在之前的 M5 路线里，Virtual Geometry 已经完成了：

- visibility-owned lineage segment boundary
- unified indirect authority
- GPU submission segment / draw-ref readback
- current-evictable slot recycling guard

但还剩两个明显缺口：

- deeper hierarchy truth 还没有作为稳定字段一路压到 cluster-raster consumption
- runtime eviction 虽然已经知道“是否同 lineage”，但还不知道“离 target 有多远”

本切片补上的正是这两条缺口：

- `lineage_depth` 让 hierarchy refine 深度进入真实 draw consumption
- lineage distance ordering 让 residency cascade 从二元策略推进到层级距离策略

## Validation

- `visibility_context_splits_virtual_geometry_draw_segments_across_parent_lineages_even_when_page_matches`
  - 证明 visibility 侧的 lineage-owned segment contract 现在也会固定携带 depth。
- `virtual_geometry_unified_indirect_keeps_lineage_depth_in_gpu_submission_and_indirect_args`
  - 证明 `lineage_depth` 已经进入 GPU submission segment readback 与 indirect args。
- `virtual_geometry_runtime_state_prefers_evicting_farther_target_ancestors_before_nearer_ones`
  - 证明 runtime residency 会优先回收更远 ancestor。
- `virtual_geometry_runtime_state_prefers_evicting_farther_target_descendants_before_nearer_ones_for_gpu_assignment`
  - 证明 GPU slot assignment 路径也会优先回收更远 descendant。

## Remaining Route

- 继续把 deeper cluster raster consumption 扩展到更宽的 hierarchy refine / streaming frontier 组合，而不只是当前的 depth-aware trim。
- 继续把 residency-manager cascade / split-merge policy 从当前的 lineage distance 排序推进到更完整的 frontier hysteresis policy。
- 如果后续引入真正的 visibility-owned unified indirect buffer / cluster raster submission authority，这次的 `lineage_depth + lineage distance` contract 就是继续下沉的直接基础。
