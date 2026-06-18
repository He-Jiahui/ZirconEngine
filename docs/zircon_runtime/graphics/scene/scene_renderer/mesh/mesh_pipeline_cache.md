---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_motion_vector_pipeline.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - tools/zircon_build.py
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - tools/zircon_build.py
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_derives_material_shader_variant_key
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed with existing warnings)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_writes_disk_entries
  - rustfmt --edition 2021 on Plan 08 MS-M4-S1b touched files (2026-06-17 shader variant disk cache slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617 (2026-06-17 shader variant disk cache slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
doc_type: module-detail
---

# Mesh Pipeline Cache

`MeshPipelineCache` owns built-in mesh WGPU pipeline state for the scene renderer. It keeps shader modules, forward base mesh pipelines, motion-vector mesh pipelines, and the pass/pipeline variant registry used by the MD-M1 mesh command layer.

## Variant Registry

`MeshPipelineVariantRegistry` assigns stable `MeshPipelineVariantId` values for cache-backed pipeline variants. Its key is `MeshPassPipelineKind` plus the complete `PipelineKey`, so the same material pipeline used by the base pass and motion-vector pass resolves to different ids. Repeated requests for the same pass and pipeline shape reuse the same id.

Each `MeshPipelineVariantKey` also stores the Plan 08 neutral `ShaderVariantKey` derived from that `PipelineKey`. The registry maps base and TAA reactive passes to `ShaderPassType::Forward`, depth prepass to `DepthPrepass`, shadow variants to `Shadow`, and motion vectors to `Velocity`, using `wgpu-runtime` as the current platform token. This is the runtime bridge for future shader variant cache, disk cache, and prewarm work; it does not yet replace the full `PipelineKey` inside WGPU render-pipeline hash maps because blend/depth state and authored texture-presence bits still need to stay distinct.

The registry also records a per-frame `ShaderVariantMissReport`. Variant id resolution counts requests and memory hits. Base mesh shader-module creation adds disk hits, compile misses, disk writes, and disk errors when it consults `ShaderVariantCacheDisk`.

Variant id `0` is intentionally left outside the registry for fixed pass-owned pipelines, including the normal depth prepass pipeline and shadow-map pipelines. Registry ids start at `1`, which lets command sorting and replay distinguish cache-backed variants from fixed pipelines without relying on hash-derived identities.

## Integration

`MeshPipelineCache` implements `MeshPipelineVariantResolver`. `SceneRendererCore::render_compiled_scene` and overlay mesh recording pass the live cache into `build_mesh_pass_command_buffers`, while unit tests pass a pure `MeshPipelineVariantRegistry`. `MeshPassBuildContext` forwards processor requests to that resolver when processors create `MeshDrawCommand` values.

Concrete WGPU pipeline lookup now has variant-id bridge methods for cache-backed mesh pipelines. Forward/base mesh recording uses `ensure_pipeline_for_variant(...)`; motion-vector recording uses `ensure_motion_vector_pipeline_for_variant(...)`. Both methods look up the registry key for the variant id, verify the expected `MeshPassPipelineKind`, then delegate to the existing `ensure_pipeline(...)` or `ensure_motion_vector_pipeline(...)` construction path.

`ensure_pipeline(...)` now checks the shader variant disk cache before creating the base mesh shader module. It hashes the selected WGSL source with blake3, combines that hash with the derived `ShaderVariantKey`, and stores compressed WGSL plus metadata under `.zircon-cache/shader_variants` or `ZR_SHADER_CACHE_DIR`. It also has a read-only staged fallback root at `cache/shader_variants`, which is where the build prewarm tool writes packaged cache entries. Disk corruption falls back to the current source and increments the report error count.

`zircon_shader_prewarm` can prepopulate the built-in base forward fallback mesh variant into that staged root. This does not yet replace the live WGPU pipeline map key or cover Velocity/TAA/deferred/template variants; it gives `MeshPipelineCache` a real packaged cache handoff path for the first base mesh shader module.

This slice does not move fixed depth prepass or shadow-map pipelines into `MeshPipelineCache`; those pass-owned pipelines keep variant id `0`. Static command caching remains the next cache consumer.
