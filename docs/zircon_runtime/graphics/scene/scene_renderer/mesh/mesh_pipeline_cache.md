---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_motion_vector_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# Mesh Pipeline Cache

`MeshPipelineCache` owns built-in mesh WGPU pipeline state for the scene renderer. It keeps shader modules, forward base mesh pipelines, motion-vector mesh pipelines, and the pass/pipeline variant registry used by the MD-M1 mesh command layer.

## Variant Registry

`MeshPipelineVariantRegistry` assigns stable `MeshPipelineVariantId` values for cache-backed pipeline variants. Its key is `MeshPassPipelineKind` plus the complete `PipelineKey`, so the same material pipeline used by the base pass and motion-vector pass resolves to different ids. Repeated requests for the same pass and pipeline shape reuse the same id.

Variant id `0` is intentionally left outside the registry for fixed pass-owned pipelines, including the normal depth prepass pipeline and shadow-map pipelines. Registry ids start at `1`, which lets command sorting and replay distinguish cache-backed variants from fixed pipelines without relying on hash-derived identities.

## Integration

`MeshPipelineCache` implements `MeshPipelineVariantResolver`. `SceneRendererCore::render_compiled_scene` and overlay mesh recording pass the live cache into `build_mesh_pass_command_buffers`, while unit tests pass a pure `MeshPipelineVariantRegistry`. `MeshPassBuildContext` forwards processor requests to that resolver when processors create `MeshDrawCommand` values.

Concrete WGPU pipeline lookup now has variant-id bridge methods for cache-backed mesh pipelines. Forward/base mesh recording uses `ensure_pipeline_for_variant(...)`; motion-vector recording uses `ensure_motion_vector_pipeline_for_variant(...)`. Both methods look up the registry key for the variant id, verify the expected `MeshPassPipelineKind`, then delegate to the existing `ensure_pipeline(...)` or `ensure_motion_vector_pipeline(...)` construction path.

This slice does not move fixed depth prepass or shadow-map pipelines into `MeshPipelineCache`; those pass-owned pipelines keep variant id `0`. Static command caching remains the next cache consumer.
