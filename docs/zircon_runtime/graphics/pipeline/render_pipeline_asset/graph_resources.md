---
related_code:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/anti_alias.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/bloom.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/construct.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/anti_alias.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/bloom.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 continue Plan 01 required external texture import and materialization modularization
tests:
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_preserves_report_only_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_preserves_report_only_external_buffer_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/typed_optional_external_tests.rs::compile_rejects_conflicting_report_only_external_texture_and_buffer_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_forward_plus_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_deferred_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_preserves_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_rejects_conflicting_required_external_texture_and_buffer_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs::compile_describes_hzb_as_half_power_of_two_mip_chain
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-typed-optional-external-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-required-external-texture-0617 --message-format short --color never
doc_type: module-detail
---

# Pipeline Graph Resources

`render_pipeline_asset/graph_resources.rs` owns the SRP descriptor to RenderGraph resource plan step. `compile.rs` owns high-level pipeline compilation orchestration; `descriptor_filtering.rs` owns post-process descriptor filtering/routing; `resource_descriptors.rs` owns transient texture/buffer descriptor sizing; `pass_authoring.rs` owns pass creation and resource access lowering. This module owns the smaller question: for every resource name mentioned by enabled `RenderFeaturePassDescriptor` rows, decide whether the compiled graph should create a transient texture, create a transient buffer, or import an external resource.

## Resource Plan

`pipeline_graph_resources(...)` scans all enabled feature descriptors and folds each pass resource declaration into one `PipelineGraphResourcePlan` per resource name. The plan stores:

- `kind`: the final graph resource kind. A resource with no writes becomes `External`, because the graph cannot own a transient resource that is only read from outside the compiled pass chain.
- `external_binding`: the execution-side contract for external imports. Transient texture and buffer plans reset this to `report_only`; external plans preserve required/report-only texture or buffer metadata.

This keeps resource ownership decisions deterministic before the builder allocates `RgTextureHandle`, `RgBufferHandle`, or `ExternalResource` handles.

## External Binding Merge

External binding merge is intentionally strict. Legacy report-only unknown imports can coexist with a later typed declaration, but one resource name cannot be both an external texture and an external buffer. That conflict returns a compile error before `RenderGraphBuilder` receives the resource, so compiled graph declarations and lifetimes never contain ambiguous external resource types. Required typed declarations upgrade the requirement bit; report-only typed declarations preserve texture or buffer ownership intent without making the missing import a hard validation failure.

The feature descriptor helpers that feed this module are:

- `read_external_texture(...)`
- `read_external_buffer(...)`
- `write_external_texture(...)`
- `write_external_buffer(...)`
- `write_storage_external_texture(...)`
- `write_storage_external_buffer(...)`
- `write_external_texture_with_ops(...)`
- `read_required_external_buffer(...)`
- `write_required_external_buffer(...)`
- `read_required_external_texture(...)`
- `write_required_external_texture(...)`
- `write_required_external_texture_with_ops(...)`
- `write_required_storage_external_texture(...)`

The older untyped `read_external(...)`, `write_external(...)`, `write_storage_external(...)`, and `write_external_with_ops(...)` helpers remain available for legacy report-only unknown resources and focused compatibility fixtures, but production feature descriptors should use the typed helpers unless the resource type is deliberately unknown.

The first production required external consumers are HZB occlusion's required buffer set and the shadow atlas required texture. The built-in `shadow-atlas` pass writes `PostProcessGraphResourceNames::SHADOW_ATLAS` through `write_required_external_texture_with_ops(...)`, and forward plus deferred receiver passes read the same external as a required texture. Pipeline lowering merges those declarations into one compiled external lifetime with `RenderGraphExternalResourceBinding::required_texture()`.

Optional/report-only external consumers now also carry typed metadata. Built-in frame-target, history, post-process, SSAO, TAA, UI, anti-alias, debug-overlay, deferred-geometry, and deferred-lighting descriptors use texture or buffer helper variants for external resources such as `FINAL_COLOR`, `VIEWPORT_OUTPUT`, `TAA_HISTORY_*`, `AMBIENT_OCCLUSION`, and `EXPOSURE_PREVIOUS`. As of the Plan 09 post-process viewport-origin slice, `BLOOM` and `GLOBAL_ILLUMINATION` are graph-owned post-process textures rather than prebound fixed frame externals, so selected-camera split viewport execution can keep those intermediates local before terminal writeback. First-party particles, Hybrid GI, and Virtual Geometry plugin descriptors use the same typed report-only helpers for their external GPU buffers or history textures.

## Validation

Source-contract tests in `compile_tests.rs` verify that a required external texture descriptor produces a compiled external lifetime with `RenderGraphExternalResourceBinding::required_texture()`, and that a same-name external texture/buffer conflict is rejected during pipeline compilation. `shadow_atlas_required_external_tests.rs` keeps the production default Forward+ and Deferred graph checks out of production `compile.rs` while proving the real `SHADOW_ATLAS` writer/reader chain compiles to a required external texture. The slice also passed the scoped `zircon_runtime` `core-min` Cargo check listed in the header; focused tests remain authored but deferred to the milestone testing stage.

`typed_optional_external_tests.rs` keeps the optional/report-only texture and buffer source contracts out of `compile.rs`. It asserts that typed optional helpers preserve `report_only_texture()` and `report_only_buffer()` bindings on compiled external lifetimes and that same-name optional texture/buffer declarations are rejected. The 2026-06-17 typed optional external slice passed the scoped `zircon_runtime` `core-min` check listed in the header. The first-party plugin package check was attempted with `--locked` for particles, Hybrid GI, and Virtual Geometry, but Cargo stopped before compilation because `zircon_plugins/Cargo.lock` would need an update; that lockfile was left untouched in the dirty worktree.

`resource_descriptors.rs` keeps the transient descriptor sizing contracts covered by the compile tests for HZB/SSR mip-chain resources and Color LUT 3D texture resources. `descriptor_filtering.rs` keeps post-process stack selection and resource rerouting out of graph resource planning, so this module always receives the final logical descriptor set. `pass_authoring.rs` consumes this plan and maps each pass resource declaration to the matching builder call. These extractions are behavior-preserving and reduce `compile.rs` to orchestration, frame-extract-dependent particle descriptor insertion, validation, and metadata collection.
