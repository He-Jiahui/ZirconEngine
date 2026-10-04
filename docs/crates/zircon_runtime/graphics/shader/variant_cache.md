---
related_code:
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/execution_budget.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/module_validation_cache.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/wgpu_validation.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - tools/zircon_build.py
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/mod.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/execution_budget.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/module_validation_cache.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm/wgpu_validation.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/asset_inventory.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/material_sources.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/module_dependencies.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - tools/zircon_build.py
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
plan_sources:
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs::render_shader_variant_prewarm_writes_disk_entries
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs::render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root
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

`ShaderVariantCacheDisk` stores entries under the cache root as `v1/<hash-prefix>/<hash>.wgsl.zst` plus a sibling `<hash>.meta`. The default writable root is `<project>/.zircon/cache/shader_variants`, and `ZR_SHADER_CACHE_DIR` can override it. A relative override is resolved from the selected project's physical root; an absolute override remains an explicit external cache location. Runtime lookup can also read staged prewarm roots such as `<project>/cache/shader_variants` without writing to them. The hash is `blake3(canonical_string + include/source hashes)`, so changes to the material key or any participating source content produce a different entry instead of reusing stale WGSL.

The `.wgsl.zst` file contains compressed WGSL. The `.meta` JSON stores the schema version, hash, canonical key, template revision, naga version, wgpu version, and creation time. Writes use a temporary file followed by rename; if another writer already produced the final path, the current writer treats that as a benign race. Corrupt metadata, schema mismatches, key mismatches, decompression errors, or invalid UTF-8 are reported as cache errors and the entry files are removed so the next lookup becomes a normal miss.

## Mesh Integration

`MeshPipelineCache` constructs a `ShaderVariantCacheDisk` at startup and uses it when the base mesh shader module is first created. The current source hash is derived from the WGSL selected by `ResourceStreamer` or the built-in fallback. Disk hits reuse cached WGSL, misses compile from the current source and write the cache entry, and disk errors fall back to the current source while recording the error count.

The live WGPU render-pipeline maps still key by the complete `PipelineKey`. That preserves blend, depth, cutoff, and texture-presence distinctions until the later Plan 08 owner moves all mesh pass variants to `ShaderVariantKey` as the final cache key.

## Diagnostics

`ShaderVariantMissReport` is a neutral frame DTO exposed through `RenderStats.last_shader_variant_miss_report`. The mesh variant registry counts per-frame variant requests and memory hits, while the disk cache path records disk hits, compile misses, writes, and errors. `render_stats_store/shader_variant.rs` mirrors those counters into `DiagnosticStore` paths under `render.shader_variant.*`.

## Prewarm

`ShaderVariantPrewarmManifest`, `ShaderVariantPrewarmSource`, and `ShaderVariantPrewarmRequest` are neutral framework DTOs for offline cache population. Schema 2 stores WGSL source, include/source hashes, template revision, and naga/wgpu version strings once in a content-addressed source table; a request carries the final `ShaderVariantKey`, optional pipeline state, and its source id. Repeated resolution builds one borrowed O(1) source-id index without duplicating WGSL. These schema-2 DTOs reject unknown fields, so legacy request-level or manifest-level inline shader payloads fail at deserialization instead of bypassing source-table validation. `validate_integrity()` and `ShaderVariantPrewarmExecutionBudget::validate()` expose typed errors for invalid source identity and invalid serial-worker bounds; only the report boundary renders those failures as text. `graphics::shader::variant_cache::prewarm_shader_variants_to_disk(...)` resolves that id through the validated table, writes the same disk layout used by runtime misses, and returns `ShaderVariantPrewarmReport` with requested, written, failed, execution-budget, and per-entry failure data.

Manifest-level preflight failures are stored in `ShaderVariantPrewarmReport.preflight_error`. They do not invent a variant index or affect per-variant counts, but the command-line wrapper still returns its failure exit code.

`zircon_runtime::dynamic_api::prewarm_shader_variants(...)` is the Rust-side headless entry point. The first implemented producer is `builtin_fallback_shader_prewarm_manifest()`, which emits the base forward fallback mesh variant using the same key/source/hash contract as `MeshPipelineCache::ensure_pipeline(...)`.

The physical owner boundary keeps `prewarm.rs` as the thin entry route and `prewarm/worker.rs` as the
single production execution owner. Primary unit tests live in `prewarm/tests.rs`; combined module and
pipeline validation tests remain in `prewarm/tests/combined_validation_tests.rs`. Structure guards
read these owners independently so a production assertion cannot be satisfied accidentally by test
text. Status: `runtime_07_15_shader_prewarm_test_owner_and_source_guard_routing_static_passed_cargo_deferred`.

When WGPU module validation is enabled, the headless prewarm entry points cache both pass and failure outcomes once per source id for the current batch. Render-pipeline validation remains request-scoped because pipeline state is part of its contract.

`zircon_shader_prewarm` is the command-line wrapper used by `tools/zircon_build.py --prewarm-shaders`. It can read an explicit manifest JSON, add the built-in fallback manifest, and write cache entries into the staged payload at `ZirconEngine/cache/shader_variants` with a sibling JSON report. Runtime mesh cache lookup now checks the writable runtime cache first and then that staged prewarm cache.

For asset roots, the wrapper persists a bounded inventory snapshot under `<cache-root>/asset_inventories`. File length and modification time only decide whether that snapshot can be reused; source and variant identity remain content-derived. The compact-index fast path also refuses a symlink or Windows reparse point before it can classify an asset tree as unchanged. Schema 4 writes a compact index beside the hydrated payload, so an unchanged warm run with only default local dimensions checks compact file and directory stats and skips asset projection without deserializing cached WGSL or metadata bodies. External quality, geometry, permutation, resource-registry, or registry-export inputs conservatively hydrate and rebuild as required. A changed source is projected through the SCC-condensed include graph's reverse-dependency closure. External include content hashes are interned once per graph batch; graph edges retain only scalar table indices, avoiding a content-hash allocation for every high-fanout import. Snapshot writes stream to a temporary file and rename it only after a complete write. A nested cache root is excluded before scanning and is part of the snapshot scan-shape identity, so cache writes cannot invalidate or contaminate the asset inventory. A cache root equal to an asset root is rejected because it cannot safely distinguish generated entries from source assets.

The custom-id fallback contract uses that same lookup chain for plugin-range
shader dimensions. `render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root`
writes a `GeometrySourceId(4)` / `ShadingModelId(16)` request into a staged
`cache/shader_variants` root, requires a miss from the empty runtime root, and
then requires `ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])`
to hit the same canonical key without creating or writing the runtime root.
Status: `render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred`.

The current prewarm scope covers built-in fallback permutations, manifest-driven writes, and asset-root material manifests assembled from the bounded inventory. Optional headless WGPU module and render-pipeline validation consume the same manifest source table. The final product gate remains a real second startup with zero runtime compilation misses against a rendered scene.
