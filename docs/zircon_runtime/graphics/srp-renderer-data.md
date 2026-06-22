---
related_code:
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_reference.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_report.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_source.rs
  - zircon_runtime/src/graphics/tests/renderer_data_feature_names.rs
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs
  - zircon_runtime/src/graphics/tests/renderer_data_material_shader.rs
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs
  - zircon_runtime/src/graphics/tests/renderer_data_version.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/mod.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/texture_transform.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/layouts/create_material_texture_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/material_texture_set.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_geometry.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_pipeline/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/shaders/normal_prepass.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/shadow_map.wgsl
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_with_asset_context.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/pipeline/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/builtin_render_feature.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/advanced_slots.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature/requires_explicit_opt_in.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/dispatch/descriptor_for.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/advanced_slot.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_reference.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_report.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/mod.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/debug_overlay.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_scene_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/preview_sky_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_with_asset_context.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/pipeline/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-06-08 implement ZirconEngine WGPU render main-chain material texture transform slice
  - user: 2026-05-18 continue SRP RendererData workflow with zshader/zmaterial contract validation
  - user: 2026-05-25 continue material shader renderer functionality
  - docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
tests:
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::renderer_data_document_uses_builtin_feature_authoring_name_contract
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::renderer_data_document_uses_render_pass_stage_authoring_name_contract
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::renderer_data_document_rejects_legacy_aggregate_stage_names
  - zircon_runtime/src/graphics/tests/renderer_data_compile_report.rs
  - zircon_runtime/src/graphics/tests/renderer_data_compile_report.rs::render_pipeline_compile_report_groups_diagnostics_by_feature_material_and_shader
  - zircon_runtime/src/graphics/tests/renderer_data_compile_report.rs::render_pipeline_compile_report_groups_diagnostics_by_severity
  - zircon_runtime/src/graphics/tests/renderer_data_compile_report.rs::renderer_feature_contract_diagnostic_exposes_canonical_sources
  - zircon_runtime/src/graphics/tests/renderer_data_compile_report.rs::renderer_feature_contract_diagnostic_exposes_canonical_severity
  - zircon_runtime/src/graphics/tests/renderer_data_feature_names.rs
  - zircon_runtime/src/graphics/tests/renderer_data_feature_names.rs::renderer_data_document_rejects_feature_name_aliases_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_feature_names.rs::renderer_data_document_accepts_canonical_feature_name_source_pair
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs::renderer_data_document_rejects_empty_local_config_keys_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs::renderer_data_document_rejects_padded_local_config_keys_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs::renderer_asset_projection_rejects_empty_local_config_keys
  - zircon_runtime/src/graphics/tests/renderer_data_local_config.rs::renderer_asset_projection_rejects_padded_local_config_keys
  - zircon_runtime/src/graphics/tests/renderer_data_material_shader.rs
  - zircon_runtime/src/graphics/tests/renderer_data_material_shader.rs::asset_aware_compile_uses_material_shader_for_material_only_contract_diagnostics
  - zircon_runtime/src/graphics/tests/renderer_data_material_shader.rs::asset_aware_compile_reports_material_owned_shader_readiness_diagnostics
  - zircon_runtime/src/graphics/tests/renderer_data_material_shader.rs::asset_aware_compile_reports_material_owned_shader_missing
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs::renderer_data_document_rejects_empty_renderer_names_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs::renderer_data_document_rejects_padded_renderer_names_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs::renderer_asset_projection_rejects_empty_renderer_names
  - zircon_runtime/src/graphics/tests/renderer_data_names.rs::renderer_asset_projection_rejects_padded_renderer_names
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs::renderer_asset_projects_to_renderer_data_document_with_authoring_names
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs::renderer_asset_projection_rejects_non_renderer_data_stage
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs::renderer_asset_projection_rejects_plugin_feature_sources
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs::renderer_asset_projection_rejects_descriptor_overrides
  - zircon_runtime/src/graphics/tests/renderer_data_projection.rs::renderer_asset_projection_rejects_runtime_only_capability_requirements
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs::renderer_data_document_rejects_empty_quality_gate_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs::renderer_data_document_rejects_padded_quality_gate_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs::renderer_data_document_rejects_padded_quality_gate_names
  - zircon_runtime/src/graphics/tests/renderer_data_quality_gate.rs::renderer_asset_projection_preserves_cross_feature_quality_gate
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_duplicate_required_entry_points
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_duplicate_expected_properties
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_duplicate_expected_texture_slots
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_empty_required_entry_points
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_blank_expected_properties
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_empty_expected_texture_slots
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_padded_required_entry_points
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_padded_expected_properties
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_padded_expected_texture_slots
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_required_entry_points_without_shader_reference
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_expected_properties_without_shader_reference
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_data_document_rejects_expected_texture_slots_without_shader_reference
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_asset_projection_rejects_duplicate_required_entry_points
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_asset_projection_rejects_empty_expected_properties
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_asset_projection_rejects_padded_expected_texture_slots
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_asset_projection_rejects_contract_references_without_shader_reference
  - zircon_runtime/src/graphics/tests/renderer_data_references.rs::renderer_asset_projection_accepts_contract_references_with_shader_reference
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs::renderer_data_document_rejects_empty_stage_lists_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs::renderer_data_document_rejects_empty_feature_lists_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs::renderer_asset_projection_rejects_empty_stage_lists
  - zircon_runtime/src/graphics/tests/renderer_data_required_lists.rs::renderer_asset_projection_rejects_empty_feature_lists
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs::renderer_data_document_rejects_duplicate_stages_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs::renderer_data_document_rejects_duplicate_features_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs::renderer_asset_projection_rejects_duplicate_stages
  - zircon_runtime/src/graphics/tests/renderer_data_uniqueness.rs::renderer_asset_projection_rejects_duplicate_features
  - zircon_runtime/src/graphics/tests/renderer_data_version.rs
  - zircon_runtime/src/graphics/tests/renderer_data_version.rs::renderer_data_document_rejects_future_versions_before_runtime_projection
  - zircon_runtime/src/graphics/tests/renderer_data_version.rs::renderer_data_document_uses_current_version_when_field_is_omitted
  - zircon_runtime/src/graphics/tests/advanced_followup_slots.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile.rs
  - zircon_runtime/src/graphics/tests/mod.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_rejects_empty_descriptor_extract_section_names
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_rejects_duplicate_history_bindings_in_one_descriptor
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_assigns_attachment_ops_from_resource_write_order
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::depth_prepass_executor_requires_prepass_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::preview_sky_executor_requires_preview_renderer_context_instead_of_nooping
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::screen_space_ui_executor_uses_graph_attachment_ops_for_viewport_output
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests.rs::overlay_executor_requires_overlay_context_instead_of_nooping
  - cargo test -p zircon_runtime --lib render_pass_executor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::default_deferred_pipeline_compiles_expected_stage_order_and_passes
  - zircon_runtime/src/graphics/tests/project_render.rs::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs::tests::renderer_material_layout_diagnostics_accept_current_renderer_abi
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs::tests::renderer_material_layout_diagnostics_validate_skinning_and_texture_bindings
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs::tests::standard_material_uniform_packs_per_slot_texture_transforms
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_samples_standard_pbr_texture_set
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/shader_source.rs::tests::deferred_geometry_shader_writes_sampled_material_gbuffer_channels
  - zircon_runtime/src/graphics/scene/scene_renderer/prepass/normal_prepass_shader_source/normal_prepass_shader_source.rs::tests::normal_prepass_shader_samples_material_normal_map_into_scene_normal
  - cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::asset_aware_compile_reports_shader_payload_readiness_gaps
doc_type: module-detail
---

# SRP RendererData Documents

`RendererDataDocument` is the TOML-facing data surface for Unity SRP-style renderer data assets. It describes the renderer name, ordered `RenderPassStage` list, and a list of renderer features without changing the existing render graph execution path.

The runtime ownership boundary stays narrow: `zircon_runtime::graphics::pipeline` owns RendererData structure, feature references, asset-aware compile reports, and non-fatal SRP diagnostics, while `.zshader` and `.zmaterial` truth remains in `zircon_runtime::asset`.

## Runtime Data Model

`RendererDataDocument` parses and serializes a document with `version`, `name`, `stages`, and `features`. Missing `version` defaults to the current `RENDERER_DATA_DOCUMENT_VERSION`, but conversion to `RendererAsset` now rejects any explicit unsupported version before stage or feature parsing. The renderer `name` must also be present in canonical form: empty, whitespace-only, or leading/trailing-whitespace names are rejected in both forward TOML conversion and reverse runtime-asset projection. The stage and feature lists must each contain at least one entry before runtime projection or reverse export, so an empty authoring document cannot be treated as a valid renderer shell. This keeps future document shapes and ambiguous renderer identities from being partially interpreted by an older runtime. `RendererFeatureDocument` stores authoring-oriented fields for a feature: canonical `name`, built-in `source`, `enabled`, optional `quality_gate`, optional shader/material `AssetReference`s, required shader entry points, expected material properties, expected texture slots, and string `local_config`. A quality gate is a compile-option gate over a canonical built-in feature name; it may intentionally refer to a different feature than `source`, but authored strings must be non-empty and already trimmed before parsing. Local config values stay opaque strings, but their keys are authoring identifiers and must be non-empty and already trimmed before the config is copied into or out of `RendererFeatureAsset`.

The document converts into the existing `RendererAsset` and `RendererFeatureAsset` runtime structures. Conversion validates strings against known built-in feature names and the current renderer stage names instead of silently accepting aliases. Built-in feature strings now go through `BuiltinRenderFeature::authoring_name()` / `from_authoring_name(...)`, so the enum owns the single authoring vocabulary used by renderer data, quality gates, and follow-up slot tests. A feature document's `name` must match that canonical built-in `source` value because `RendererFeatureAsset` has no separate display-label or authoring-label field; mismatches are rejected before runtime projection instead of being silently overwritten during reverse projection. Feature contract reference lists also reject duplicate, empty, or whitespace-padded required entry points, expected material properties, and expected texture slots before they reach compile-report diagnostics, and any non-empty shader-contract expectation list requires a shader `AssetReference`; otherwise asset-aware compile would have no shader asset to inspect and would silently skip the expectation. The same checks run before reverse projection writes runtime references back to authoring data. Stage strings now go through `RenderPassStage::authoring_name()` / `from_renderer_data_authoring_name(...)`, with `RenderPassStage::RENDERER_DATA_AUTHORING_STAGES` defining the subset accepted by RendererData documents. After parsing, RendererData rejects duplicate stages and duplicate feature sources before creating a runtime `RendererAsset`, mirroring the downstream renderer validation while keeping authoring errors at the TOML boundary. Empty or whitespace-padded quality gates are rejected before built-in feature parsing so accidental TOML padding cannot silently become a different compile-option gate. This preserves hard-cutover behavior and keeps misspelled or ambiguous authoring data visible before graph compile.

The reverse projection is intentionally narrower and explicit. `RendererDataDocument::from_renderer_asset(...)` can write a runtime `RendererAsset` back to TOML-facing data only when the renderer asset name is canonical, every stage is in `RENDERER_DATA_AUTHORING_STAGES`, and every feature is a built-in `RendererFeatureAsset` expressible by the current document fields. It writes canonical stage and feature authoring names from the same central contracts, preserves enabled state, optional quality gates, local config, and shader/material contract references, and uses the built-in source name as the feature document `name` because `RendererFeatureAsset` does not store a separate authoring label. Runtime-only or invalid authoring state is rejected instead of silently dropped: empty or padded renderer names, empty stage or feature lists, plugin feature sources, descriptor overrides, ad-hoc `RendererFeatureAsset.capability_requirements`, duplicate stages, duplicate built-in features, duplicate/empty/padded feature reference names, shader-contract expectation lists without a shader reference, empty or padded local config keys, and internal aggregate stages such as `Opaque` / `Transparent` produce `RendererDataDocumentError` values.

## Feature Contract References

`RendererFeatureAssetReferences` is stored on every `RendererFeatureAsset`. It carries optional shader/material references plus the required entry/property/texture-slot names that M2 will resolve against imported `ShaderAsset` and `MaterialAsset` contracts. RendererData treats each reference list as a set of authoring expectations: a feature may require several entry points, properties, or texture slots, but the same name may not be repeated inside one list, and empty, whitespace-only, or leading/trailing-whitespace names are rejected before the runtime asset is created or before a runtime asset is exported back to a document. Non-empty `required_entry_points`, `expected_properties`, or `expected_texture_slots` also require `shader` to be present because they are feature-shader expectations. A material-only reference remains valid: the `.zmaterial` owns its shader reference, and asset-aware compile now uses that material-owned shader to report material shader-contract diagnostics when no feature shader override is authored. RendererData intentionally does not infer asset kind from URI extension because project assets can use compound resource locators; asset loading and `.zshader` / `.zmaterial` truth remain in the asset system.

The `RendererFeatureAsset` constructors default these references to empty values, so existing default pipelines and plugin feature descriptors keep their current graph behavior. Builder helpers add contract references fluently for tests and future asset importers: `with_shader_reference`, `with_material_reference`, `with_required_entry_point`, `with_expected_property`, and `with_expected_texture_slot`.

## Stage And Feature Names

M1 accepts exact built-in feature names such as `Mesh`, `Sprite`, `PostProcess`, `Ui`, `DebugOverlay`, `AntiAlias`, `Bloom`, `ColorGrading`, and `HistoryResolve`. It also supports the other currently declared built-ins so existing pipeline feature vocabulary remains complete. These names are the canonical `BuiltinRenderFeature::authoring_name()` values; RendererData parsing no longer keeps a separate hand-written feature-name match table. The same canonical value must appear in both a feature's `name` and `source` fields until the runtime asset model stores a distinct authoring label. Advanced descriptor-first slots such as `SkinnedMesh`, `MeshLod`, `ReflectionProbes`, `BakedLighting`, `SparseTexture`, `Particle`, `Terrain`, `Tree`, `Decal`, `Projector`, `Halo`, `LensFlare`, `Trail`, `Billboard`, `Tilemap`, `TextShaping`, `Skybox`, `Cubemap`, `Texture2dArray`, `NormalMap`, `Mipmap`, and `ColorSpace` can be named as renderer-data `source` and `quality_gate` values before their dedicated renderer plans land. The descriptor-only catalog owns their exact descriptor names, neutral extract sections, and sparse-texture capability metadata; parsing those names does not imply a graph pass or executor exists. For `ReflectionProbes`, `BakedLighting`, and `Particle`, built-in renderer-data names reserve authoring vocabulary only; executable runtime flags still require plugin feature descriptors named `reflection_probes`, `baked_lighting`, and `particle`.

Stage names are accepted only in the current explicit RendererData vocabulary: `DepthPrepass`, `Shadow`, `Deferred`, `AmbientOcclusion`, `Lighting`, `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `Opaque3d`, `AlphaMask3d`, `Transparent3d`, `PostProcess`, `Ui`, `Overlay`, and `Debug`. These are the `authoring_name()` values of `RenderPassStage::RENDERER_DATA_AUTHORING_STAGES`; RendererData parsing no longer keeps a separate hand-written stage-name match table. A stage may appear only once in the ordered document list, and a built-in feature source may appear only once in the feature list. Legacy aggregate stages such as `Opaque` and `Transparent` remain part of the enum for internal compile validation but are intentionally not accepted by the document parser for this milestone.

## Validation Scope

The focused M1 tests cover TOML roundtrip, conversion to `RendererAsset`, disabled feature preservation, shader/material reference preservation, and unknown stage/feature errors. The follow-up `renderer_data_projection` tests cover `RendererAsset` back-projection through central authoring names plus rejection of internal stages, plugin sources, descriptor overrides, and runtime-only capability requirements. The `renderer_data_version` tests cover the document version gate and the omitted-version default. The `renderer_data_feature_names` tests cover canonical feature `name` / `source` pairs and reject display-label aliases before runtime projection. The `renderer_data_names` tests cover empty and padded renderer asset names in both forward TOML conversion and runtime-asset back-projection. The `renderer_data_required_lists` tests cover empty stage and feature list rejection in both forward TOML conversion and runtime-asset back-projection. The `renderer_data_local_config` tests cover empty and padded local config keys in both forward TOML conversion and runtime-asset back-projection. The `renderer_data_quality_gate` tests cover empty and padded quality gates before runtime projection and preserve cross-feature quality gates during runtime-asset back-projection. The `renderer_data_uniqueness` tests cover duplicate stage and duplicate feature rejection in both forward TOML conversion and runtime-asset back-projection. The `renderer_data_references` tests cover duplicate, empty, whitespace-padded, and shader-reference-dependent required entry point, expected property, and expected texture-slot rejection in both forward TOML conversion and runtime-asset back-projection. The `renderer_data_material_shader` tests cover material-only feature references resolving the material-owned shader for shader-contract diagnostics, preserving that shader reference on the resulting material validation rows, keeping material-local validation rows shaderless, forwarding material-owned shader readiness diagnostics as `ShaderValidation`, and reporting `MaterialShaderMissing` with both the material and shader references when that shader is missing. The `renderer_data_compile_report` tests cover compile-report grouping by feature, `.zmaterial`, `.zshader`, and diagnostic source, plus the canonical `RendererFeatureContractDiagnostic::source()` classification used by editor projections. The milestone testing stage is responsible for running the focused `renderer_data_asset` / `renderer_data_compile_report` / `renderer_data_material_shader` / `renderer_data_projection` / `renderer_data_version` / `renderer_data_feature_names` / `renderer_data_local_config` / `renderer_data_quality_gate` / `renderer_data_names` / `renderer_data_required_lists` / `renderer_data_uniqueness` / `renderer_data_references` tests and a scoped `zircon_runtime` library check before this area is considered complete.

M1 runtime data validation passed on 2026-05-20 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` ran 6 focused tests, 6 passed, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` completed successfully. Both commands emitted only the pre-existing `zircon_runtime/src/scene/world/query.rs::entity_ids_matching_query_archetypes` dead-code warning outside the SRP RendererData files.

## Asset-Aware Compile Reports

`RenderPipelineAsset::compile_with_asset_context(...)` validates the normal graph path first by delegating to `compile_with_options(...)`. Descriptor, stage, phase, resource, and core-pipeline errors therefore remain hard compile failures. Shader/material authoring mismatches are gathered afterward into `RenderPipelineCompileReport::diagnostics` and do not prevent a `CompiledRenderPipeline` from being returned. The report also exposes read-only `diagnostics_by_feature()`, `diagnostics_by_material()`, `diagnostics_by_shader()`, `diagnostics_by_source()`, and `diagnostics_by_severity()` groupings so runtime tools, asset management, and editor projections can consume one canonical interpretation of diagnostic ownership, repair-source classification, and triage severity.

The compile context is abstracted as `RenderPipelineAssetContext`, which can load `ShaderAsset` and `MaterialAsset` by `AssetReference`. This keeps `graphics::pipeline` independent of `ProjectAssetManager` internals and lets tests use a small in-memory context.

M2 diagnostics cover missing feature shader/material assets, missing material-owned shaders, feature shader versus material shader mismatch, required shader entry points, expected shader properties, expected shader texture slots, existing material-local validation errors, stored material validation diagnostics, material shader-contract diagnostics, and shader payload readiness diagnostics. When a feature has a material reference but no explicit feature shader reference, asset-aware compile loads the `MaterialAsset.shader` reference and uses that shader for material contract validation and shader readiness reporting; if that material-owned shader is missing, the compile report emits `MaterialShaderMissing` with the material reference and the shader reference it owns. Explicit feature shader references still emit `ShaderMissing` when their shader asset cannot be resolved. Material-shader mismatch, material validation, and stored material diagnostic rows carry the owning material reference. Material validation rows also carry the shader reference when they were produced by shader-contract validation, while material-local validation rows remain shaderless. Shader diagnostics remain shader-owned and are wrapped as `ShaderValidation`. `RendererFeatureContractDiagnostic` centralizes the material/shader ownership, `RenderMaterialDiagnosticSource`, and severity accessors used by compile-report and editor groupings, including de-duplicating shader references when a material validation error and its shader-contract source point at the same `.zshader`, grouping dependency-resolution/schema/texture/WGSL-capture rows by source, grouping structural repair rows as `Error`, and grouping stored material/shader validation strings as `Warning`.

The shader side now consumes `ShaderAsset::readiness_report()` instead of forwarding only `shader.validation_diagnostics`. RendererData therefore reports asset-owned shader readiness gaps before GPU preparation: missing runtime WGSL for non-WGSL sources without emitted WGSL, invalid entry-point stage tokens, empty or duplicate shader definition names, and copied shader validation diagnostics. This is still a compile-report diagnostic surface only. It does not compose WGSL imports, create shader modules, specialize typed shader definitions, allocate bind group layouts, or prewarm renderer pipelines.

M2 asset-aware compile validation passed on 2026-05-20 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` ran 10 focused tests, 10 passed after review added material-local validation diagnostics to the SRP report; `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never` ran 39 focused tests, 39 passed; `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never` ran 75 runtime lib tests plus 1 matching integration test, all passed; and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` completed successfully. All commands emitted only the pre-existing `entity_ids_matching_query_archetypes` dead-code warning outside this SRP lane.

## Product Pipeline Placement

RendererData complements the product render pipeline instead of replacing it. Runtime product profiles still choose which renderer product is active, and `RenderPipelineAsset::compile_with_options(...)` remains the hard graph compiler for descriptors, pass stages, phase ordering, resource IO, and core-pipeline requirements. RendererData supplies an authoring-facing document and feature contract reference layer that can be converted into the same `RendererAsset` and `RendererFeatureAsset` declarations consumed by those existing compilers.

The asset-aware compile path is therefore a reporting layer around the graph compiler. It resolves `.zshader` and `.zmaterial` references after hard graph validation, records authoring diagnostics in `RenderPipelineCompileReport`, and leaves the compiled pipeline usable when only shader/material contract mismatches are present. This matches the SRP intent from Unity renderer data assets while deliberately diverging from Unity's runtime resource creation and render-pass invalidation hooks for this milestone.

## Graph Resource Descriptors

The render-main-chain M2 slice moves compiled graph resources away from placeholder 1x1 descriptors. `RenderPipelineAsset::compile_with_options(...)` now derives transient texture and buffer descriptors from the submitted `RenderFrameExtract`: headless target or explicit viewport size determines the graph extent, camera HDR selects `Rgba16Float` for scene color-style resources, and camera MSAA samples propagate into non-shadow transient attachments.

This remains an SRP compile-time contract rather than direct GPU allocation. Concrete WGPU texture ownership still belongs to the renderer/resource registry, but the compiled graph can now expose realistic width, height, format, sample-count, usage, and storage/copy intent for validation, scheduling, and later executor cutover.

Descriptor validation also covers the SRP feature metadata that controls extraction and temporal resources. Feature descriptors reject empty or duplicate required extract sections, empty pass executors, conflicting resource kinds, explicit external/transient name collisions, and duplicate history slot bindings inside a single feature descriptor. Separate features may still merge history access for the same slot during pipeline compile.

The built-in `HistoryResolve` feature now declares temporal scene-color resources by role: it reads `scene-color` plus external `history.previous.scene-color`, and writes external `postprocess.history-resolved`. `history.current.scene-color` is reserved for the renderer-owned texture updated by the history-copy step after the frame. Pipeline compile tests explicitly reject the old single `history-scene-color` graph resource, because a temporal pass must not hide previous input and current output behind one name.

SRP compile now assigns graph attachment operations from resource write order. The first transient texture producer receives `Clear + Store`; later producers for the same transient texture receive `Load + Store`; imported external writes use `Load + Store` by default. Feature pass descriptors can explicitly declare write ops when the pass owns target initialization, for example Deferred preview sky clears imported `final-color` before later passes read it as the background. This lets transparent mesh, sprite, postprocess, UI, overlay, and preview-sky executors consume graph metadata for WGPU load/store decisions instead of carrying private pass-name rules. The UI and debug-overlay descriptors both write the external `viewport-output` resource, so the compiled graph names the final view target independently from post-process `final-color`.

The mesh, UI, overlay, Deferred, and post-process descriptors now mirror the current WGPU shader reality. Forward mesh descriptors start the `DepthPrepass` stage with `preview-sky` writing `scene-color` and `scene-depth`; Deferred geometry starts the same stage with `preview-sky` writing imported `final-color` and `scene-depth`. `depth-prepass` then writes `scene-depth` and `gbuffer-normal`; `mesh.depth-prepass` / `deferred.depth-prepass` are concrete graph executors rather than registered no-ops. SSAO, clustered lighting, and bloom preparation are separate concrete executors: `ao.ssao-evaluate` writes external `ambient-occlusion`, `lighting.clustered-cull` writes the `light-list` buffer, and `post.bloom-extract` writes external `bloom-texture`. `post.stack` now consumes graph-bound scene color, AO, bloom, final color, global illumination, and light-list resources for final post-process composition only. `overlay.gizmo` reads `scene-depth` and writes external `viewport-output`, matching the concrete overlay renderer's depth-tested line/icon passes. `deferred.gbuffer` reads `scene-depth` and writes `gbuffer-albedo` plus `gbuffer-material`; alpha-mask geometry is included in the non-transparent G-buffer input rather than modeled as a separate scene-color pass. `lighting.deferred` reads `gbuffer-albedo`, `gbuffer-normal`, `gbuffer-material`, and external `final-color` as the preview/background input, then writes `scene-color`. The executor registry still validates compiled SRP executor ids, but concrete built-in executor bodies now live in post-process and scene child modules so RendererData-driven pass expansion does not require growing the registry owner itself. `RenderPassExecutionContext` remains the SRP metadata/resource-access context; its renderer-side GPU payload and concrete draw/dispatch bridges live in `render_pass_execution_context/gpu.rs`. The renderer material ABI now expects group 2 bindings 0-9 for base-color, normal, metallic-roughness, occlusion, and emissive texture/sampler pairs plus group 3 binding 0 for the material uniform; the standard uniform keeps scalar channels in `data0/1` and packs the five standard texture UV transforms in `data2..data6`. Forward, deferred, normal-prepass, shadow alpha-mask, and motion-vector alpha-discard WGSL apply those transforms before sampling. Renderer-data authoring that declares non-empty custom shader layouts is diagnosed against that ABI before custom shader execution, but RendererData does not yet provide WGSL reflection or custom-shader execution of renderer-owned standard material state.

Focused SRP compile validation on 2026-06-02 passed with 43 `pipeline_compile` tests, including the extract-section, duplicate-history-binding, extract-derived descriptor, history-slot-split, attachment-op write-order, preview-sky pass ordering/clear-load behavior, truthful Deferred resource, overlay depth/viewport-output, and SSAO/cluster/bloom external frame-resource regressions added for the render-main-chain slices. The latest run used `cargo test -p zircon_runtime --lib --locked pipeline_compile --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`.

The remaining gaps are explicit: no GPU prewarm, no shader variant compilation, no WGPU pipeline specialization, no mutable editor authoring surface, no ShaderGraph or VFX graph, and no real GPU preview. Editor work after this runtime milestone should consume the `RendererAsset` and diagnostic rows as read-only projection data.

Final scoped acceptance on 2026-05-20 also ran `cargo fmt --all --check`, `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --color never`. The editor tests passed 8 focused material-editor tests including RendererData projection coverage. Workspace and plugin-wide green were not claimed because optional broad expansion commands were not run in this closeout.
