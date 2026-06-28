---
related_code:
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - tools/zircon_build.py
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - tools/zircon_build.py
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
plan_sources:
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_writes_disk_entries
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs (2026-06-28 Runtime custom id staged fallback lookup contract: passed)
  - source/docs anchor scan, conflict marker scan, trailing-whitespace scan, scoped git diff --check (2026-06-28 Runtime custom id staged fallback lookup contract: passed; diff-check only reported LF/CRLF warnings)
  - rustfmt --edition 2021 on Plan 08 MS-M4-S1b touched files (2026-06-17 shader variant disk cache slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617 (2026-06-17 shader variant disk cache slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - python -m py_compile tools\zircon_build.py (2026-06-17 shader prewarm slice: passed)
  - python tools\zircon_build.py --targets runtime --out D:\zircon-shader-prewarm-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-17 shader prewarm slice: passed)
doc_type: module-detail
---

# Shader Variant Cache

`graphics::shader::variant_cache` owns renderer-side persistence for Plan 08 shader variants. The neutral `ShaderVariantKey` still lives in `core::framework::render::shader`; the cache only decides how a concrete renderer stores and reloads generated WGSL for that key.

## Disk Layout

`ShaderVariantCacheDisk` stores entries under the cache root as `v1/<hash-prefix>/<hash>.wgsl.zst` plus a sibling `<hash>.meta`. The default writable root is `<project>/.zircon-cache/shader_variants`, and `ZR_SHADER_CACHE_DIR` can override it. Runtime lookup can also read staged prewarm roots such as `<project>/cache/shader_variants` without writing to them. The hash is `blake3(canonical_string + include/source hashes)`, so changes to the material key or any participating source content produce a different entry instead of reusing stale WGSL.

The `.wgsl.zst` file contains compressed WGSL. The `.meta` JSON stores the schema version, hash, canonical key, template revision, naga version, wgpu version, and creation time. Writes use a temporary file followed by rename; if another writer already produced the final path, the current writer treats that as a benign race. Corrupt metadata, schema mismatches, key mismatches, decompression errors, or invalid UTF-8 are reported as cache errors and the entry files are removed so the next lookup becomes a normal miss.

## Mesh Integration

`MeshPipelineCache` constructs a `ShaderVariantCacheDisk` at startup and uses it when the base mesh shader module is first created. The current source hash is derived from the WGSL selected by `ResourceStreamer` or the built-in fallback. Disk hits reuse cached WGSL, misses compile from the current source and write the cache entry, and disk errors fall back to the current source while recording the error count.

The live WGPU render-pipeline maps still key by the complete `PipelineKey`. That preserves blend, depth, cutoff, and texture-presence distinctions until the later Plan 08 owner moves all mesh pass variants to `ShaderVariantKey` as the final cache key.

## Diagnostics

`ShaderVariantMissReport` is a neutral frame DTO exposed through `RenderStats.last_shader_variant_miss_report`. The mesh variant registry counts per-frame variant requests and memory hits, while the disk cache path records disk hits, compile misses, writes, and errors. `render_stats_store/shader_variant.rs` mirrors those counters into `DiagnosticStore` paths under `render.shader_variant.*`.

## Prewarm

`ShaderVariantPrewarmManifest` and `ShaderVariantPrewarmRequest` are neutral framework DTOs for offline cache population. A request carries the final `ShaderVariantKey`, WGSL source, include/source hashes, template revision, and naga/wgpu version strings. `graphics::shader::variant_cache::prewarm_shader_variants_to_disk(...)` writes those requests through the same disk layout as runtime misses and returns `ShaderVariantPrewarmReport` with requested, written, failed, and per-entry failure data.

`zircon_runtime::dynamic_api::prewarm_shader_variants(...)` is the Rust-side headless entry point. The first implemented producer is `builtin_fallback_shader_prewarm_manifest()`, which emits the base forward fallback mesh variant using the same key/source/hash contract as `MeshPipelineCache::ensure_pipeline(...)`.

`zircon_shader_prewarm` is the command-line wrapper used by `tools/zircon_build.py --prewarm-shaders`. It can read an explicit manifest JSON, add the built-in fallback manifest, and write cache entries into the staged payload at `ZirconEngine/cache/shader_variants` with a sibling JSON report. Runtime mesh cache lookup now checks the writable runtime cache first and then that staged prewarm cache.

The custom-id fallback contract uses that same lookup chain for plugin-range
shader dimensions. `render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root`
writes a `GeometrySourceId(4)` / `ShadingModelId(16)` request into a staged
`cache/shader_variants` root, requires a miss from the empty runtime root, and
then requires `ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])`
to hit the same canonical key without creating or writing the runtime root.
Status: `render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`.

The current prewarm scope covers the built-in base forward fallback mesh shader and manifest-driven writes. Velocity, TAA reactive, deferred/template variants, asset-scanned material manifests, actual device compilation, and the final "second startup compile miss = 0" product acceptance remain future slices.
