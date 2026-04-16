---
related_code:
  - zircon_scene/src/components.rs
  - zircon_scene/src/render_extract.rs
  - zircon_scene/src/lib.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/declarations/mod.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_scene/src/components.rs
  - zircon_scene/src/lib.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/visibility/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history.rs
  - zircon_graphics/src/visibility/declarations/mod.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_cluster.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_page_upload_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
plan_sources:
  - user: 2026-04-16 continue next step after M5 capability-slot slice
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/tests.rs
  - cargo test -p zircon_graphics visibility_context_builds_virtual_geometry_visibility_feedback_and_page_plan --locked
  - cargo test -p zircon_graphics visibility_context_with_history_tracks_virtual_geometry_requested_pages --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics render_server_bridge --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_tracks_page_table_and_request_sink --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_deduplicates_requests_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_scene --locked
  - cargo test -p zircon_render_server --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Preprocess Plan

**Goal:** 把 `Virtual Geometry` 从 “只有 opt-in feature/pass 名称” 推进到真正的统一前处理基线：extract 能描述 cluster/page，visibility 能输出可见 cluster、page upload 计划和 feedback，render-server stats 能暴露当前 runtime 的 Virtual Geometry 计划规模。

**Non-Goal:** 本轮不实现 page table、GPU residency manager、cluster streaming backend、hierarchical cluster traversal、meshlet decode、Nanite raster 或任何真实 GPU 虚拟几何执行器。

## Delivered Slice

- `RenderVirtualGeometryExtract` 现在包含：
  - `cluster_budget`
  - `page_budget`
  - `clusters`
  - `pages`
- `RenderVirtualGeometryCluster` 定义了：
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `bounds_center`
  - `bounds_radius`
  - `screen_space_error`
- `RenderVirtualGeometryPage` 定义了：
  - `page_id`
  - `resident`
  - `size_bytes`

## Visibility Contract

- `VisibilityContext` 新增：
  - `virtual_geometry_visible_clusters`
  - `virtual_geometry_page_upload_plan`
  - `virtual_geometry_feedback`
- `VisibilityVirtualGeometryPageUploadPlan` 明确拆成：
  - `resident_pages`
  - `requested_pages`
  - `dirty_requested_pages`
  - `evictable_pages`
- `VisibilityVirtualGeometryFeedback` 明确暴露：
  - `visible_cluster_ids`
  - `requested_pages`
  - `evictable_pages`
- `VisibilityHistorySnapshot` 现在保留 `virtual_geometry_requested_pages`，用于跨帧 dirty request 规划

## Rules

- cluster visibility 不只跟随 entity 级 mesh visibility；还会对 cluster 自己的 bounds 再做一次 frustum test
- visible cluster 排序固定为：
  - `screen_space_error` 高优先
  - `lod_level` 低优先级值优先
  - `cluster_id` 作为稳定 tie-break
- `cluster_budget` 会截断本帧真正进入 Virtual Geometry 可见集的 cluster
- `page_budget` 会截断本帧真正发起请求的 missing page
- `dirty_requested_pages` 只记录相对上一帧新出现的 page request
- `evictable_pages` 只来自当前 resident 但本帧未被任何 visible cluster 使用的页

## Render Server Stats

- `RenderStats` 新增：
  - `last_virtual_geometry_visible_cluster_count`
  - `last_virtual_geometry_requested_page_count`
  - `last_virtual_geometry_dirty_page_count`
- 当前这些计数只会在 `VirtualGeometry` feature 真正进入有效 compiled pipeline 时写入
- 由于 headless `wgpu` 仍被 capability gate 关闭，所以当前 façade 回归里这些值会保持 `0`
- 后续 CPU runtime host slice 进一步补上了：
  - `last_virtual_geometry_page_table_entry_count`
  - `last_virtual_geometry_resident_page_count`
  - `last_virtual_geometry_pending_request_count`
  - 详见 [2026-04-16-m5-virtual-geometry-runtime-host.md](./2026-04-16-m5-virtual-geometry-runtime-host.md)

## Validation Summary

- `visibility_context_builds_virtual_geometry_visibility_feedback_and_page_plan`
  - 证明 cluster-level frustum filtering、生效预算、resident/requested/evictable 分离都按预期工作
- `visibility_context_with_history_tracks_virtual_geometry_requested_pages`
  - 证明跨帧 dirty page request 只记录新增页
- `headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features`
  - 证明 capability gate 仍然会让 Virtual Geometry feature 和对应 stats 在当前 `wgpu` 基线上保持关闭/归零

## Remaining Virtual Geometry Route

- cluster hierarchy / parent-child refinement
- GPU uploader / feedback consumer
- Virtual Geometry pass 真正消费可见 cluster/page/runtime-host 计划
- 与 indirect draw / instance upload / BVH/occlusion 的更深层联动
