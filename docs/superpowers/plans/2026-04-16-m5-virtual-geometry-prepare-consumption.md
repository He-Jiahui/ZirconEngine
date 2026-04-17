---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
- zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
- zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/types/mod.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/mod.rs
- zircon_graphics/src/scene/scene_renderer/mesh/mesh_draw/mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs
  - zircon_graphics/src/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
- zircon_graphics/src/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene.rs
plan_sources:
  - user: 2026-04-16 continue next step after M5 baseline completion
  - user: 2026-04-17 Virtual Geometry next step should enter cluster streaming or indirect raster consumption
  - user: 2026-04-17 Virtual Geometry should continue from refine/uploader baseline into real cluster-id consumption instead of only entity-level hints
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-gpu-uploader-readback.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_streaming_state_changes_fallback_raster_output --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_streaming_state_changes_fallback_raster_coverage --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_visible_cluster_ids_change_fallback_raster_region --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_draw_segments_override_extract_cluster_ordinals_for_fallback_raster --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare Consumption Plan

**Goal:** 让 `virtual-geometry-prepare` 第一次真正消费 viewport 级 `VirtualGeometryRuntimeState`，把 page-table / pending-request / visible-cluster 组合成 frame-local prepare snapshot，并让当前 mesh fallback renderer 在 `VirtualGeometry` feature 显式开启时 honor 这个结果。

**Non-Goal:** 本轮仍然不实现真正的 indirect raster、Nanite meshlet decode、page streaming I/O 或深层 split-merge / hysteresis cluster policy。

## Delivered Slice

- `VirtualGeometryRuntimeState::build_prepare_frame(...)` 现在会把 runtime host 与当前可见 cluster 合成为 `VirtualGeometryPrepareFrame`。
- `build_virtual_geometry_plan(...)` 现在会在 visibility/preprocess 阶段为每个可见 cluster 计算稳定的 `cluster_ordinal / cluster_count`，并且这个 ordinal 固定从 entity 的完整 extract cluster 集导出，而不是只从本帧 frontier 导出。
- `EditorOrRuntimeFrame` 新增内部 `virtual_geometry_prepare` 槽位，以及 `with_virtual_geometry_prepare(...)` builder。
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 之前：
  - 从 viewport 记录里克隆并复用已有 Virtual Geometry runtime host
  - 用当前 extract + visibility page plan 更新它
  - 生成 frame-local prepare snapshot
  - 把 snapshot 连同 render extract 一起交给 renderer
- `build_mesh_draws(...)` 现在会在 `VirtualGeometry` feature 显式启用时：
  - 使用 prepare snapshot 的 `visible_entities` 过滤当前 fallback mesh draw 集
  - 直接消费 prepare snapshot 提供的 `cluster_draw_segments`，不再从 `extract.geometry.virtual_geometry.clusters` 反推 cluster ordinal
  - 同时生成 `MeshDraw.first_index + draw_index_count`，并把这些范围继续编码成 renderer-local indirect args，把 cluster-id 对应的 index range 真正压进 base/prepass/deferred draw submission
  - 让 resident cluster 走完整 cluster slice，pending upload cluster 则走 ghosted / partial coverage slice；不同 visible cluster id 现在会把 fallback 画到不同区域，而不只是换 tint

## Prepare Snapshot Contract

- `VirtualGeometryPrepareFrame`
  - `visible_entities`
  - `visible_clusters`
  - `cluster_draw_segments`
  - `resident_pages`
  - `pending_page_requests`
  - `evictable_pages`
- `VirtualGeometryPrepareCluster`
  - `entity`
  - `cluster_id`
  - `page_id`
  - `lod_level`
  - `resident_slot`
  - `state: Resident | PendingUpload | Missing`
- `VirtualGeometryPrepareDrawSegment`
  - `entity`
  - `cluster_id`
  - `cluster_ordinal`
  - `cluster_count`
  - `lod_level`
  - `state: Resident | PendingUpload | Missing`
- `VirtualGeometryPreparePage`
  - `page_id`
  - `slot`
  - `size_bytes`
- `VirtualGeometryPrepareRequest`
  - `page_id`
  - `size_bytes`
  - `generation`

## Runtime Rules

- `build_prepare_frame(...)` 会：
  - 按 runtime host 当前 resident slot 导出 `resident_pages`
  - 按 pending request 队列导出 `pending_page_requests`
  - 仅把 `Resident` 或 `PendingUpload` cluster 的 entity 纳入 `visible_entities`
  - 仅为 `Resident` 或 `PendingUpload` cluster 生成 `cluster_draw_segments`
  - 把完全 missing 的 page/cluster 继续暴露在 `visible_clusters` 里，但不会进入 fallback draw 白名单
- renderer fallback 消费规则现在分成两层：
  - `visible_entities` 决定 entity 是否还能进入当前 mesh fallback draw 列表
  - `cluster_draw_segments` 决定哪些 cluster slice 会被提交，以及每个 slice 的 `first_index + draw_index_count + tint`
  - `Resident / PendingUpload / Missing` 现在不只影响 entity 级亮度，而会影响 cluster slice 的实际 index range coverage，因此 cluster frontier/streaming 状态会直接反映到当前离屏输出和空间分布
- prepass / base scene / deferred geometry 三条 mesh draw path 现在都会 honor `MeshDraw.first_index + draw_index_count`，并在 `VirtualGeometry` feature 开启时把这些范围继续走 renderer-local `draw_indexed_indirect(...)`，因此同一条 prepare snapshot 会在所有主要 raster stage 上消费相同的 cluster fallback slice
- renderer 不允许再从 `RenderVirtualGeometryExtract` 重建 fallback cluster slice；`cluster_draw_segments` 是 prepare 对 renderer 的唯一 segment 合同
- submit 路径会在 render 前使用 `last_generation + 1` 作为当前 prepare planning 的 generation 基线，从而让 prepare snapshot 与本帧 runtime host 状态一致，而不是只在 render 完成后再更新 host。

## Why This Slice Exists

- M5 baseline 之前已经有 capability slot、page planning、CPU runtime host 和 façade stats，但 `virtual-geometry-prepare` 仍然只是 compile-time pass 名字。
- 这一轮的目标不是伪造 Nanite raster，而是让 runtime host 真正进入 frame/runtime path，并让后续 GPU uploader / indirect / streaming/refine 有一个可以继续替换的已消费边界。
- 现在 cluster streaming 不再只存在于 prepare/runtime 数据结构里；resident 和 pending cluster 的差异已经能通过 fallback raster 的颜色、覆盖面积与 cluster-id 对应的空间区域被直接观测到。

## Validation Summary

- `virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters`
  - 证明 runtime host 能把 resident/pending/missing cluster 和 page/request 状态稳定压成 prepare snapshot
- `virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities`
  - 证明当 `VirtualGeometry` feature 显式开启且 prepare snapshot 只允许特定 entity 时，当前 fallback mesh path 会改变最终离屏输出
- `virtual_geometry_prepare_streaming_state_changes_fallback_raster_output`
  - 证明相同 entity 在 `PendingUpload` 与 `Resident` cluster 状态下会得到不同的 fallback raster 输出，说明 cluster streaming 状态已经被 renderer 真正消费
- `virtual_geometry_prepare_streaming_state_changes_fallback_raster_coverage`
  - 证明同一 entity 在 `PendingUpload` 与 `Resident` cluster 状态下会覆盖不同数量的像素，说明 prepare snapshot 已经进入真实 draw submission，而不再只是 tint 提示
- `virtual_geometry_prepare_visible_cluster_ids_change_fallback_raster_region`
  - 证明不同 `visible_cluster_id` 会把同一 entity 的 fallback raster 压到不同屏幕区域，说明 prepare 已经开始真正消费 cluster frontier，而不再只是 entity 级统计 hint
- `virtual_geometry_prepare_draw_segments_override_extract_cluster_ordinals_for_fallback_raster`
  - 证明 renderer 实际消费的是 prepare 提供的 `cluster_draw_segments`，而不是从 extract 侧按 cluster id 重新反推 ordinal；即使 prepare 显式覆盖 segment ordinal，离屏输出也会跟着 prepare 走
- `virtual_geometry_prepare_segments_submit_indirect_raster_draws_when_feature_enabled`
  - 证明当 `VirtualGeometry` feature 开启且 prepare segment 存在时，renderer 会提交真实的 indirect raster draw，而不是继续只走 direct indexed fallback

## Remaining Route

- 更深层的 cluster streaming / residency manager / page I/O 耦合
- split-merge / hysteresis / SSE driven refinement policy
- GPU indirect args / occlusion / BVH update 与 Virtual Geometry prepare 的更深层耦合
- 用真实 cluster/meshlet raster / indirect consumption 替换当前 fallback `first_index + draw_index_count + tint` 基线
