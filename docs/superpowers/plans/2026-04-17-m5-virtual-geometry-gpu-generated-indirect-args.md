---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/virtual_geometry_indirect_args_gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/virtual_geometry_indirect_args_gpu_resources/virtual_geometry_indirect_args_gpu_resources.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/mod.rs
plan_sources:
  - user: 2026-04-17 continue the next M5 chain without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-shared-indirect-args-buffer.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-slot-aware-indirect-compaction.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics virtual_geometry_prepare_gpu_generated_indirect_args_follow_visibility_owned_segment_span --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_render --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU-Generated Indirect Args

**Goal:** 把 `Virtual Geometry` fallback raster 的 shared indirect args buffer 再推进一层，不再由 CPU 直接打包 `IndexedIndirectArgs[]`，而是由 compute pass 根据 visibility-owned `cluster_start_ordinal / cluster_span_count / cluster_total_count / state` 生成 GPU indirect args。

**Non-Goal:** 本轮仍然不实现真正的 visibility-owned unified indirect buffer asset、multi-draw indirect、cluster raster、page residency manager 或 Nanite-like split/merge raster。

## Delivered Slice

- `SceneRendererCore` 新增 `VirtualGeometryIndirectArgsGpuResources`，在 mesh 提交层缓存专用 bind group layout 与 compute pipeline。
- `build_mesh_draws(...)` 现在会在 `Virtual Geometry` 开启时先把每条 draw 变成 `VirtualGeometryIndirectArgsInput`：
  - full-mesh fallback 路径编码成一个覆盖全 mesh 的 resident input
  - cluster fallback 路径编码成由 prepare segment 决定的 `start_ordinal / span / total_count / state`
- `build_shared_indirect_args_buffer(...)` 不再把 `PendingMeshDraw.first_index + draw_index_count` 直接写入 CPU buffer。
  它现在会：
  - 上传 `VirtualGeometryIndirectArgsInput[]`
  - dispatch `virtual_geometry_indirect_args.wgsl`
  - 生成带 `INDIRECT | STORAGE | COPY_SRC` usage 的 shared args buffer
- base/prepass/deferred 三条 render path 仍旧消费同一个 shared indirect args buffer + per-draw offset，因此对上层 prepare/visibility contract 没有新增 API 泄漏。
- `SceneRenderer` 新增 crate-local 测试 readback helper，用来直接验证 GPU 生成的 indirect args，而不是只通过像素结果间接推断。

## Why This Slice Exists

- shared indirect args buffer 已经把 args 资源从 per-draw buffer 收口到 frame-shared buffer，但 args 内容本身仍然是 CPU-side 直接编码。
- 这会让“visibility-owned segment -> indirect submission”仍旧停在 renderer CPU glue，而不是开始向真正 GPU-produced submission contract 迁移。
- 把 cluster segment 转成 GPU input，再由 compute 生成 args 后，后续继续推进到真正的 visibility-owned unified buffer、GPU compaction 或 multi-draw indirect 时，替换边界会更干净。

## Validation Summary

- `virtual_geometry_prepare_gpu_generated_indirect_args_follow_visibility_owned_segment_span`
  - 直接读回 GPU 生成的 indirect args，证明 `cluster_span_count=1` 生成 `(first_index=0, index_count=3)`，而 `cluster_span_count=2` 生成 `(first_index=0, index_count=6)`
- `virtual_geometry_prepare_render`
  - 证明 GPU-generated args 没有破坏 resident/pending coverage、slot tint、visible-cluster routing、shared buffer reuse 与 slot-aware compaction
- `cargo test -p zircon_graphics virtual_geometry --locked`
  - 证明 indirect args compute 路径与 runtime host、GPU uploader、page-table snapshot、prepare compaction 仍能一起工作
- `cargo test -p zircon_graphics --lib --locked` 与 `validate-matrix.ps1 -Package zircon_graphics`
  - 证明 `zircon_graphics` 其余 M4/M5 行为层没有被这次提交链改动回归

## Remaining Route

- 真正的 visibility-owned unified indirect buffer，而不是 renderer build step 里最后一次收口
- GPU-generated indirect compaction / multi-draw indirect
- cluster streaming / hierarchy refine 更深层 split-merge 与 cluster raster consumption
- 与 occlusion、BVH、Hybrid GI scene representation 的更深联合执行层
