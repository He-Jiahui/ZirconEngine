---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/scene/scene_renderer/mesh.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_scene.rs
plan_sources:
  - user: 2026-04-16 continue next step after M5 baseline completion
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare Consumption Plan

**Goal:** 让 `virtual-geometry-prepare` 第一次真正消费 viewport 级 `VirtualGeometryRuntimeState`，把 page-table / pending-request / visible-cluster 组合成 frame-local prepare snapshot，并让当前 mesh fallback renderer 在 `VirtualGeometry` feature 显式开启时 honor 这个结果。

**Non-Goal:** 本轮仍然不实现 GPU page upload、page streaming I/O、feedback readback、cluster hierarchy refine、Nanite raster 或任何真实 meshlet decode。

## Delivered Slice

- `VirtualGeometryRuntimeState::build_prepare_frame(...)` 现在会把 runtime host 与当前可见 cluster 合成为 `VirtualGeometryPrepareFrame`。
- `EditorOrRuntimeFrame` 新增内部 `virtual_geometry_prepare` 槽位，以及 `with_virtual_geometry_prepare(...)` builder。
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 之前：
  - 从 viewport 记录里克隆并复用已有 Virtual Geometry runtime host
  - 用当前 extract + visibility page plan 更新它
  - 生成 frame-local prepare snapshot
  - 把 snapshot 连同 render extract 一起交给 renderer
- `build_mesh_draws(...)` 现在会在 `VirtualGeometry` feature 显式启用时，使用 prepare snapshot 的 `visible_entities` 过滤当前 fallback mesh draw 集。

## Prepare Snapshot Contract

- `VirtualGeometryPrepareFrame`
  - `visible_entities`
  - `visible_clusters`
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
  - 把完全 missing 的 page/cluster 继续暴露在 `visible_clusters` 里，但不会进入 fallback draw 白名单
- submit 路径会在 render 前使用 `last_generation + 1` 作为当前 prepare planning 的 generation 基线，从而让 prepare snapshot 与本帧 runtime host 状态一致，而不是只在 render 完成后再更新 host。

## Why This Slice Exists

- M5 baseline 之前已经有 capability slot、page planning、CPU runtime host 和 façade stats，但 `virtual-geometry-prepare` 仍然只是 compile-time pass 名字。
- 这一轮的目标不是伪造 GPU uploader，而是让 runtime host 真正进入 frame/runtime path，并让后续 GPU uploader / indirect / streaming/refine 有一个可以继续替换的已消费边界。

## Validation Summary

- `virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters`
  - 证明 runtime host 能把 resident/pending/missing cluster 和 page/request 状态稳定压成 prepare snapshot
- `virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities`
  - 证明当 `VirtualGeometry` feature 显式开启且 prepare snapshot 只允许特定 entity 时，当前 fallback mesh path 会改变最终离屏输出

## Remaining Route

- GPU uploader / page request sink backend
- page residency feedback consumer / readback
- cluster hierarchy refine / split-merge / SSE driven refinement
- indirect draw / occlusion / BVH update 与 Virtual Geometry prepare 的更深层耦合
- 真正替换当前 mesh fallback，而不是只做 prepare-driven fallback filtering
