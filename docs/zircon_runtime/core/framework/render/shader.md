---
related_code:
  - dev/bevy/crates/bevy_shader/src/lib.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline.rs
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_shader_quality.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - tools/zircon_build.py
implementation_files:
  - zircon_runtime/src/core/framework/render/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/stage.rs
  - zircon_runtime/src/core/framework/render/shader/entry_point.rs
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/shader_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_shader.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_shader_quality.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/args.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/run.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_manifest.rs
  - tools/zircon_build.py
plan_sources:
  - user: 2026-05-20 Bevy rendering completion plan continuation
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md
  - docs/superpowers/specs/2026-05-25-typed-shader-definitions-design.md
  - docs/superpowers/plans/2026-05-24-shader-readiness-report.md
  - docs/superpowers/plans/2026-05-25-typed-shader-definitions.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts
  - zircon_runtime/src/asset/tests/assets/render_product.rs::render_product_assets_shader_defs_accept_legacy_flags_and_typed_values
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs::zshader_typed_shader_definition_rows_validate_kind_and_value
  - zircon_runtime/src/asset/tests/project/zmeta.rs::project_manager_imports_compound_zshader_package_with_subassets
  - 2026-05-26 typed shader definitions: rustfmt, focused shader tests, compound zshader test, and runtime lib-test check passed on D:/cargo-targets/zircon-typed-shader-defs
  - cargo test -p zircon_runtime --lib render_product_assets_shader_defs_accept_legacy_flags_and_typed_values --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib zshader_typed_shader_definition_rows_validate_kind_and_value --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 1 passed)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs -- --test-threads=1 (2026-05-25 typed shader definitions: passed, 5 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-typed-shader-defs --message-format short --color never (2026-05-25 typed shader definitions: passed with existing warnings)
  - cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 5 passed)
  - cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1 (2026-05-25 shader readiness report: passed, 24 passed)
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo check -p zircon_runtime --lib --locked
  - zircon_runtime/src/core/framework/render/shader/geometry_source.rs::tests::render_shader_geometry_source_ids_reserve_builtin_segment
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs::tests::render_shader_variant_key_packs_dimensions_stably
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs::tests::render_shader_feature_bits_reports_named_flags
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-16 Plan 08 shader variant key contract slice: passed with existing warnings)
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs::tests::pipeline_key_derives_material_shader_variant_key
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_derives_material_shader_variant_key
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed with existing warnings)
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_treats_corrupt_entry_as_miss_after_cleanup
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_writes_disk_entries
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits
  - rustfmt --edition 2021 on Plan 08 MS-M4-S1b touched files (2026-06-17 shader variant disk cache slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617 (2026-06-17 shader variant disk cache slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/shader/variant_key.rs and cargo check -p zircon_editor --lib --locked (2026-06-17 Workbench resize splitter validation exposed shader variant key GeometrySourceId owner import drift; passed after variant_key imports GeometrySourceId from geometry_source)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned shader prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned shader prewarm slice: wrote 4/4 variants)
  - python tools\zircon_build.py --targets runtime --out D:\zircon-shader-asset-prewarm-dry-run --mode debug --prewarm-shaders --dry-run (2026-06-17 asset-scanned shader prewarm slice: command includes --asset-root ZirconEngine/assets)
  - rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned multi-pass prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned multi-pass prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned multi-pass prewarm slice: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned material-feature prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned material-feature prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned material-feature prewarm slice: wrote 40/40 variants)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned material-feature prewarm regression: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned shading-model prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned shading-model prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned shading-model prewarm slice: wrote 40/40 variants)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty (2026-06-17 asset-scanned shading-model prewarm regression: wrote 20/20 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned initial revision prewarm slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned initial revision prewarm slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned initial revision prewarm slice: wrote 40/40 variants)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs (2026-06-17 asset-scanned alpha-blend pass filtering slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 (2026-06-17 asset-scanned alpha-blend pass filtering slice: passed with existing warnings)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty (2026-06-17 asset-scanned alpha-blend pass filtering slice: wrote 40/40 variants)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_shader_quality_tiers
  - rustfmt --edition 2021 touched runtime shader-quality files (2026-06-17 runtime shader quality key wiring slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-check-0617 (2026-06-17 runtime shader quality key wiring slice: passed with existing warnings)
  - rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/{args,manifest,run}.rs (2026-06-17 quality-tier prewarm enumeration slice: passed)
  - python -m py_compile tools/zircon_build.py (2026-06-17 quality-tier prewarm enumeration slice: passed)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-quality-prewarm-check-0617 (2026-06-17 quality-tier prewarm enumeration slice: passed with existing warnings)
  - python tools\zircon_build.py --targets runtime --out D:\zircon-shader-quality-prewarm-dry-run --mode debug --prewarm-shaders --shader-quality-tier high --shader-quality-tier ultra --dry-run (2026-06-17 quality-tier prewarm enumeration slice: command includes --quality-tier high --quality-tier ultra)
  - cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-quality-prewarm-check-0617 -- --asset-root examples/vampire/assets --quality-tier high --quality-tier ultra (2026-06-17 quality-tier prewarm runtime probe: timed out during build/run; no pass claimed)
  - cargo test -p zircon_runtime --lib runtime_15_shader_prewarm_manifest_tests_are_folder_backed --no-default-features --features core-min --locked: deferred in Runtime 15 M3 shader prewarm manifest test folder split
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,new,ensure_pipeline}.rs (2026-06-17 base mesh quality-aware cache owner slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-cache-check-0617 (2026-06-17 base mesh quality-aware cache owner slice: passed with existing warnings)
doc_type: module-detail
---

# Runtime Render Shader Contracts

## Purpose

`zircon_runtime::core::framework::render::shader` owns the neutral shader contract that assets, material readiness, renderer preparation, and diagnostics can share without depending on WGPU objects or Bevy's ECS render app. It names shader stages, entry points, serialized dependencies, variant keys, and pipeline layout intent.

This module deliberately does not load files, parse WGSL imports, compile shader modules, allocate bind group layouts, or queue GPU pipelines. Asset import stays under `zircon_runtime::asset`, and concrete shader module or render pipeline creation stays under `zircon_runtime::graphics`.

## Bevy Evidence

Bevy keeps the shader asset surface separate from concrete renderer allocation. `dev/bevy/crates/bevy_shader/src/lib.rs:1-8` exposes `Shader` and `ShaderCache` as the shader crate's public surface. `dev/bevy/crates/bevy_shader/src/shader.rs:33-55` stores raw source, import path, imports, extra imports, shader defs, file dependencies, and validation policy on the shader asset. `shader.rs:85-148` constructs WGSL, GLSL, and SPIR-V shader assets, while `shader.rs:323-382` loads source files and records imported shader file handles.

`dev/bevy/crates/bevy_shader/src/shader_cache.rs:59-66` describes a cache that waits for imports and leaves renderer-specific module compilation to the render device. `shader_cache.rs:182-331` resolves imports, applies shader defs, composes the module, and reports pipelines that must be requeued when a shader changes.

The render-side precedent is `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs:190-217`, where `PipelineCache` stores queued, creating, ready, and failed pipeline states. `pipeline_cache.rs:438-446` exposes cached bind group layout creation, `pipeline_cache.rs:448-466` requeues dependent pipelines when shader assets change, and `pipeline_cache.rs:468-632` creates render or compute pipelines from shader modules and layout descriptors. `dev/bevy/crates/bevy_render/src/render_resource/bind_group_layout.rs:7-14` describes bind group layouts as the shader resource interface.

Zircon copies the boundary, not the implementation: `render::shader` is the stable DTO layer; `asset::assets::shader` projects authoring data into those DTOs; `graphics` remains the only owner of WGPU shader modules, layouts, and render pipelines.

## Product Surface

`RenderShaderStage` is the common stage vocabulary: vertex, fragment, and compute. The enum is serializable with `snake_case` names so `.zshader`, `.zmeta`, tests, and diagnostics can move stage values across asset and runtime boundaries.

`RenderShaderEntryPointDescriptor` records the public entry-point name plus its `RenderShaderStage`. Asset-side parsing accepts authoring aliases such as `vert`, `vs`, `frag`, `fs`, `comp`, and `cs`, but the framework contract only exposes canonical stage values.

`RenderShaderDependency` records a `ResourceKind` and `AssetReference`. Dependencies are explicit serialized authoring data in the current milestone; they are not inferred from WGSL import syntax by the framework layer.

`RenderShaderDefinitionValue` records Bevy-style shader definition inputs as bool, signed integer, or unsigned integer values. `From<&str>` and `From<String>` create bool-true flag definitions so legacy authoring paths and small tests can stay concise while the runtime contract is no longer string-only.

`RenderShaderVariantKey` records an optional entry point, optional stage, and typed definition list. It is a neutral key for material or pipeline specialization diagnostics and single-module compile requests, not the full material pipeline-cache key.

`GeometrySourceId` is the geometry-source dimension for the material shader variant space. Built-in ids are reserved as `0 = StaticMesh`, `1 = SkinnedMesh`, `2 = MorphedMesh`, and `3 = SkinnedMorphed`; plugin geometry sources start at `GEOMETRY_SOURCE_PLUGIN_ID_START`. This keeps VertexFactory-style geometry source selection in the framework contract without pulling WGPU vertex-buffer declarations into the neutral layer.

`GeometrySourceId` is owned by `shader/geometry_source.rs`. `shader/mod.rs` may re-export it for public callers, but internal shader submodules that need the type should import it from `super::geometry_source::GeometrySourceId`. That keeps `variant_key.rs` tied to the canonical geometry-source owner instead of relying on a facade re-export that can disappear during hard-cutover module cleanup.

`ShaderVariantKey` is the Plan 08 material pipeline variant key contract. It combines `material_shader`, `material_revision`, `geometry_source`, `shading_model`, `pass_type`, `features`, `quality`, and a backend `platform_token`. `packed_dims()` reserves stable bit segments for fast in-memory specialization dimensions: geometry bits `0..3`, shading model bits `4..11`, pass bits `12..15`, feature bits `16..47`, and quality bits `48..49`. `canonical_string()` serializes the full stable key, including material id/revision and platform token, for later disk-cache hashing and shader prewarm manifests. The type remains backend-agnostic; WGPU shader modules, render pipelines, and cache entries still belong under `graphics`. `RenderQualityProfile::shader_quality` is the runtime-facing quality source and defaults to `ShaderQualityTier::Medium`; callers can override it with `RenderQualityProfile::with_shader_quality(...)`.

`ShaderPassType`, `ShaderFeatureBits`, and `ShaderQualityTier` are the typed subdimensions of that key. Pass type covers forward, G-buffer, depth prepass, shadow, and velocity passes. Feature bits currently reserve alpha-test, receive-shadows, double-sided, LOD dither crossfade, and instanced previous-transform flags. Quality tiers are low, medium, high, and ultra. The names and bit positions are intentionally stable because future mesh pipeline cache and disk-cache code will use them as part of persisted shader variant identity.

`ShaderVariantMissReport` is the neutral diagnostic DTO for variant cache behavior. It records variant requests, memory hits, disk hits, compile misses, disk writes, and disk errors for the last frame so runtime diagnostics can verify whether prewarm and disk-cache slices actually removed runtime compiles.

`ShaderVariantPrewarmManifest`, `ShaderVariantPrewarmRequest`, and `ShaderVariantPrewarmReport` are the neutral offline-cache DTOs. The manifest version-gates a list of requests; each request carries the final `ShaderVariantKey`, WGSL source, include/source hashes, and template/compiler version strings. The report records requested, written, and failed counts plus per-variant failures. These DTOs let build tooling and headless runtime code populate `graphics::shader::variant_cache` without depending on WGPU objects. The `zircon_shader_prewarm` tool can read an authored manifest, emit the built-in fallback manifest, or scan asset roots for `.zmeta` compound shader packages, `.zshader` files, standalone `.wgsl` files, and `.zmaterial` material instances. Automatically generated built-in and asset-root requests can be expanded with repeated `--quality-tier low|medium|high|ultra` or `--quality-tier all`; no explicit tier still defaults to Medium so existing staging size stays stable. Authored manifest files keep their serialized quality keys unchanged. `tools/zircon_build.py --prewarm-shaders` forwards those tiers through its `--shader-quality-tier` option. The scan path mirrors the shader package importer by reading `.zshader` `wgsl_files` in order and combining those files into the runtime WGSL payload before writing disk-cache entries. `.zshader` entry-point stages drive static-mesh StandardPBR pass expansion: vertex+fragment sources emit Forward, GBuffer, DepthPrepass, Shadow, and Velocity; vertex-only sources emit DepthPrepass, Shadow, and Velocity; fragment-only sources emit Forward and GBuffer; compute-only sources do not enter the material-variant prewarm space. Standalone `.wgsl` sources default to the full material pass set because they do not carry serialized stage metadata. Scanned shader requests use material revision `1`, matching `ResourceManager::register_ready` initial ready resource revision, while source/include hashes remain part of the disk-cache key payload for stale-entry invalidation. `.zmaterial` files are parsed through `MaterialAsset`, joined back to scanned shader sources by shader `AssetReference` URL or resource id, and expanded into deduplicated material-dimension variants. The feature mapping matches runtime `PipelineKey`: `AlphaMode::Mask` sets `ShaderFeatureBits::ALPHA_TEST`, and `double_sided = true` sets `ShaderFeatureBits::DOUBLE_SIDED`. Built-in material lighting models also enter the prewarm key through `ShadingModelId::from_lighting_model`: PBR maps to StandardPBR, BlinnPhong maps to BlinnPhong, and Unlit maps to Unlit. `AlphaMode::Blend` material-instance requests are filtered to the Forward pass so transparent materials align with the current runtime transparent queue instead of prewarming unused G-buffer, depth, shadow, or velocity variants for that material instance. Custom lighting models still need a project shading-model registry export before prewarm can assign plugin ids safely, so unknown custom models continue to fall back to StandardPBR in this tool path.

## Runtime 15 M3 shader prewarm manifest test folder split

状态：`runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred`。

Runtime 15 R4.1/M3 的当前结构切片只移动 shader prewarm manifest 的测试 owner，不改变 manifest 读取、asset-root 扫描、quality-tier 扩展、material feature 映射或 disk-cache 写入路径。`bin/zircon_shader_prewarm/manifest.rs` 从 810 行降到 672 行，父文件只保留生产逻辑和 `#[cfg(test)] mod tests;` 挂载；原内联测试迁入 `bin/zircon_shader_prewarm/manifest/tests.rs`。

子文件保留 `shader_prewarm_asset_root_manifest_reads_compound_zshader_package`，继续覆盖 compound `.zshader`、`.zmaterial` feature bits、BlinnPhong/Unlit shading model 映射、material revision 与 alpha-blend Forward-only pass filtering。新增 `structure_convention/test_file_budget/shader_prewarm_manifest.rs::runtime_15_shader_prewarm_manifest_tests_are_folder_backed`，锁定父/子文件、moved test 不回流、1 个测试保留、800 行预算，以及 Runtime 15 计划、runtime index、结构规范、review findings、module-convention、本文档和 status-output expectations 的状态锚同步。

`RenderShaderPipelineLayoutDescriptor` records the intended shader resource interface. Each `RenderShaderBindGroupLayoutDescriptor` stores a group index, optional label, and binding rows. Each `RenderShaderBindingDescriptor` stores binding index, optional label, resource type, and stage visibility. `RenderShaderBindingResourceType` currently names uniform buffers, storage buffers, sampled textures, storage textures, and samplers. `push_constant_ranges` is intentionally a vector of labels or range descriptions rather than a WGPU-native range type because the neutral contract must remain serializable and backend-agnostic.

## Asset Projection

`ShaderAsset::runtime_wgsl_source()` is the runtime source selector. It prefers non-empty emitted `wgsl_source`, then falls back to raw `source` only when `source_language == ShaderSourceLanguage::Wgsl`. Non-WGSL source without emitted WGSL is not render-ready and must fall back or report readiness diagnostics before graphics code attempts to build a shader module.

`ShaderAsset::entry_point_descriptors()` maps serialized `ShaderEntryPointAsset` rows into canonical framework descriptors and filters invalid stage tokens. `ShaderAsset::dependencies()` maps serialized `ShaderDependencyAsset` rows into `RenderShaderDependency`. `ShaderAsset::variant_keys()` derives first-pass keys from entry point names and stage strings. `ShaderAsset::pipeline_layout_descriptor()` clones the serialized layout descriptor so render feature contracts and diagnostics can reason about bind groups without allocating WGPU layouts.

`ShaderAsset::readiness_report()` sits above the neutral render DTOs and below renderer preparation. It validates whether the asset payload has runtime WGSL, canonical entry-point stages, non-empty and non-duplicated shader definition names, and no shader-side validation diagnostics. It deliberately does not compose WGSL imports, create Naga modules, allocate WGPU shader modules, build bind group layouts, or queue pipelines; those remain shader-cache and graphics responsibilities.

`.zshader` documents are asset-layer authoring documents. They store WGSL file references, entry points, import redirects, material property schema, texture slots, and editor hints. The `.zshader` importer may perform authoring diagnostics such as WGSL capture checks, but `render::shader` stays limited to the product DTOs that the renderer and material readiness layer can consume.

## Graphics Integration

`ResourceStreamer::ensure_shader_source(...)` is the current concrete bridge. It resolves the referenced `ShaderAsset`, requires `runtime_wgsl_source()`, stores the selected WGSL in `ShaderRuntime`, and returns a material readiness fallback report when the shader is missing or cannot provide runtime WGSL. This keeps shader-source failure visible to material diagnostics instead of silently using a fallback.

The mesh renderer cache currently creates WGPU shader modules from the prepared WGSL source and caches modules by shader resource id plus revision. `PipelineKey` can now derive the neutral `ShaderVariantKey`, and `MeshPipelineVariantKey` stores that derived key beside the full `PipelineKey` with a WGPU platform token and a pass-type mapping for forward, G-buffer/depth, shadow, and velocity pass kinds. Viewport quality now flows from `RenderQualityProfile::shader_quality` through `ViewportRecordState`, `FrameSubmissionContext`, `ViewportRenderFrame`, and `MeshPassBuildContext` before `MeshPipelineVariantRegistry` writes it into `ShaderVariantKey.quality`; distinct quality tiers therefore produce distinct mesh variant ids and sort-key material bits. The base mesh render-command path now resolves pipelines by `MeshPipelineVariantId` and uses the registry-owned `ShaderVariantKey` for both shader-module cache identity and `graphics::shader::variant_cache::ShaderVariantCacheDisk` lookup/write. Disk entries are keyed by `ShaderVariantKey::canonical_string()` plus a WGSL source hash, first checking the runtime writable cache and then the staged prewarm cache produced by `zircon_shader_prewarm` / `tools/zircon_build.py --prewarm-shaders`. Build staging passes `ZirconEngine/assets` to the tool as `--asset-root`, so source packages copied into the staged runtime can contribute disk-cache entries in the same pass as the built-in fallback shader. Velocity/TAA/deferred/template variants still keep their current narrower pipeline maps until their pass-specific cache owners are moved onto the same `ShaderVariantKey` path.

## Current Limits

This module is not a full Bevy `ShaderPlugin`, `ShaderCache`, or `PipelineCache`. It does not parse WGSL imports, resolve shader include graphs, apply shader definitions to Naga composition, validate Naga modules, track dependent pipelines, deduplicate bind group layouts, or support async pipeline creation states.

Asset-level shader readiness is intentionally narrower than renderer readiness. It can report missing runtime WGSL, invalid entry-point stage tokens, duplicate or empty shader definitions, source-only versus redirected import rows, and copied validation diagnostics, but it does not decide whether a concrete device can create a module or pipeline.

The layout descriptor is serialized intent, not reflection. It does not yet derive bind groups from WGSL, validate binding type compatibility, model dynamic offsets, express texture sample types, or map push constants to backend feature gates. Future shader milestones should add those checks below the framework DTO layer so `.zshader` authoring and renderer preparation continue to share one stable contract.

Asset-root prewarm scanning is still intentionally conservative. It emits only static-mesh requests, but now covers the pass dimension from `.zshader` entry-point stages plus material-instance alpha-test, double-sided, built-in shading-model variants, alpha-blend Forward-only filtering, selected quality tiers, and initial runtime revision alignment. Runtime draw submission can carry a non-Medium `ShaderQualityTier` into `ShaderVariantKey.quality`, build staging can prewarm matching quality tiers explicitly, and the base mesh WGPU cache path now consumes that same quality-aware key. The asset-root prewarm tool does not yet enumerate non-static geometry sources, custom shading-model plugin ids, or edited-project resource revisions beyond the initial `1`. Base shader-source requests also remain conservative when no material instance narrows the pass set. Runtime `ResourceManager` revision export plus Velocity/TAA/deferred/template cache-owner migration are still needed before "second launch miss = 0" can be claimed for long-lived projects after asset edits.

## Test Coverage

`render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts` proves runtime WGSL selection, WGSL fallback source selection, non-WGSL missing-source rejection, entry-point stage projection, dependency projection, typed variant-key projection, and serialized pipeline layout projection.

`render_product_assets_shader_defs_accept_legacy_flags_and_typed_values`, `zshader_typed_shader_definition_rows_validate_kind_and_value`, and the compound `.zshader` import regression cover the typed shader-definition contract. Legacy `shader_defs = ["FEATURE"]` remains accepted as bool-true flags, while typed rows preserve bool, signed integer, and unsigned integer values through `ShaderAsset`, readiness reporting, and `RenderShaderVariantKey`.

The broader `render_product_assets` filter and `cargo check -p zircon_runtime --lib --tests --locked` remain the milestone-level compile/test gates for this surface.
