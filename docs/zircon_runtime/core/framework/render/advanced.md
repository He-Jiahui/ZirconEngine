---
related_code:
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/profile.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/types/graphics_error.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/advanced/mod.rs
  - zircon_runtime/src/core/framework/render/advanced/feature.rs
  - zircon_runtime/src/core/framework/render/advanced/provider_report.rs
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/feature/render_feature_capability_requirement.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_summary/capability_summary.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capability_validation/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/compile_options_for_profile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/compile_options_for_profile/apply_flagship_profile_features.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepared_runtime_submission.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_new/new.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/provider_registration.rs
  - zircon_runtime/src/graphics/types/graphics_error.rs
  - zircon_runtime/src/graphics/tests/plugin_render_feature_fixtures.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/virtual_geometry/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_runtime/src/rhi/capabilities.rs
  - zircon_runtime/src/rhi_wgpu/capabilities.rs
plan_sources:
  - user: 2026-05-18 continue Render M9A advanced profile integration
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/core/framework/render/advanced/runtime_plan.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - cargo test -p zircon_runtime --lib advanced --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib capability_validation --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib capability_class_report --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib backend_caps_report_queue_classes_and_rt_support_independently --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_provider_selection --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_options_ --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_runtime_plan --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_profile_bundle_for_quality_profile --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib virtual_geometry_payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib hybrid_gi_payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib payload_source --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib resolve_enabled_features --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib collect_runtime_feedback --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked render_product_advanced --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked virtual_geometry --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked hybrid_gi --jobs 1 --message-format short --color never
  - cargo test -p zircon_app --locked --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-advanced-render-runtime-plugins" render_profile_runtime_plugins --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib advanced_render_plugin_manifests_declare_profile_capabilities --locked --jobs 1 --message-format short --color never
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib virtual_geometry_registration_contributes_render_feature_descriptor --locked --jobs 1
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_hybrid_gi_runtime --lib hybrid_gi_registration_contributes_render_feature_descriptor --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Advanced Render Profile Runtime Plan

## Purpose

`zircon_runtime::core::framework::render::advanced` owns the neutral M9A report vocabulary for advanced render products. It is deliberately framework-level: it describes whether `VirtualGeometry` and `HybridGlobalIllumination` are requested, backed by backend capability, backed by a runtime provider, and therefore ready to run.

This module does not instantiate plugin providers, select app features, or touch renderer-private VG/HGI state. Those steps are later M9A slices. The first contract slice gives those later systems one stable DTO family to report through instead of inventing ad hoc booleans.

## Feature Mapping

`AdvancedRenderFeature` currently has two variants:

- `VirtualGeometry`
- `HybridGlobalIllumination`

Each variant maps back to its `RenderProductFeature` and to its required `RenderCapabilityKind` set. The mapping stays in the neutral render framework so profile validation, provider planning, submit preparation, and stats can all talk about the same feature identity.

The current concrete advanced requirement sets are:

- Virtual Geometry: `VirtualGeometry`, `StorageBuffers`, `IndirectDraw`, and `BufferReadback`.
- Hybrid Global Illumination: `HybridGlobalIllumination`, `StorageBuffers`, and `BufferReadback`.

`AsyncCompute` is still modeled as a separate capability, but it is not required by this slice because current render graph queue fallback can run async-declared work on graphics when async compute is unavailable. Ray-query requirements remain reserved for providers that actually require ray-query or for the later Solari milestone.

## Provider Availability

`AdvancedProviderAvailability` is the input side of the runtime plan. It carries optional provider IDs for VG and HGI without owning provider objects. This is intentional: app bootstrap, plugin manifests, and native loading stay outside the neutral runtime plan.

The runtime framework now fills this DTO from selected provider registrations during `WgpuRenderFramework` construction. App bootstrap remains the only layer that may depend on first-party provider implementation crates; once it hands `RuntimePluginRegistrationReport` values to runtime, selected IDs are available through `RenderStats::advanced_provider_availability`.

Graphics bridge tests now keep two separate fixture paths: descriptor-only pluginized render features for missing-provider degradation, and descriptor-plus-no-op-provider fixtures for provider-backed submit acceptance. This prevents runtime graph tests from accidentally treating a descriptor as a runnable advanced feature when provider arbitration has not selected a provider.

## App Provider Collection And Manifests

`zircon_app` exposes `first-party-advanced-render-runtime-plugins` for linking the VG/HGI provider crates. `first_party_runtime_plugin_registrations_for_config(...)` reads `EntryConfig::render_profile` and appends transient target-scoped runtime plugin selections for `RuntimePluginId::VirtualGeometry` and `RuntimePluginId::HybridGi` when the bundle contains the matching advanced product features. `DefaultRender` therefore collects no advanced providers, while `AdvancedRender` collects both providers without requiring every project manifest to list them manually.

The VG/HGI runtime plugin descriptors and `plugin.toml` manifests now declare profile-facing capabilities:

- `runtime.render.advanced.virtual_geometry`
- `runtime.render.advanced.hybrid_gi`

The built-in runtime catalog mirrors those capability declarations and marks both as `Partial`, matching the current provider-backed but still experimental advanced-render maturity. Manifest-contribution tests compare the package manifests, runtime module descriptors, projected descriptor manifests, and catalog descriptors so the capability vocabulary cannot drift silently.

`render_product_advanced` is the product-level acceptance surface for this slice. It submits the same AdvancedRender extract with and without runtime providers: provider-backed runs must report both advanced features as effective, execute VG/HGI graph passes, and mark both payload sources as authored; descriptor-only runs must degrade with `ProviderMissing`, execute zero VG/HGI graph passes, and clear stale payload-source stats.

## Provider Arbitration

`VirtualGeometryRuntimeProviderRegistration` and `HybridGiRuntimeProviderRegistration` carry an integer priority. The default priority is `0`, and providers can call `with_priority(...)` when registration order should not decide selection.

`WgpuRenderFramework::new_with_plugin_render_extensions(...)` selects one provider per advanced feature before renderer state is created:

- duplicate provider IDs reject construction with `GraphicsError::AdvancedProviderSelection`;
- a tie at the highest priority rejects construction instead of silently picking by registration order;
- the highest-priority provider is stored in `RenderFrameworkState` and reported through `RenderStats::advanced_provider_availability`.

The arbitration layer deliberately does not collect providers from `zircon_app` or plugin manifests. It only makes framework behavior deterministic once a caller supplies registrations.

## Submit-Time Gating

Runtime submit now consumes provider availability before advanced features reach prepared sidebands:

- `compile_options_for_profile(...)` only opts in `VirtualGeometry` or `HybridGlobalIllumination` capability-gated plugin features when the quality profile requests the feature, the backend reports the capability, and the selected provider ID exists.
- `resolve_viewport_record_state(...)` builds an `AdvancedProfileRuntimePlan` from the active quality profile's advanced flags, backend capabilities, and selected provider IDs.
- `resolve_enabled_features(...)` requires both compiled feature capability metadata and ready `AdvancedProfileRuntimePlan` state before enabling VG/HGI submit sidebands.
- `FrameSubmissionContext` gates `hybrid_gi_enabled` and `virtual_geometry_enabled` through that runtime plan, so degraded features clear stale runtime state and do not carry authored or automatic sidebands forward.
- `RenderStats::last_advanced_provider_reports` records the per-feature reports from the last submit.

This is submit-local profile participation. It does not replace app-level render profile bundle selection or first-party provider collection; it ensures the runtime framework does not silently treat a requested advanced feature as enabled when the required provider is absent.

`render_framework_degrades_requested_advanced_features_without_runtime_providers` covers the negative submit path: requested VG/HGI features without runtime providers do not enter `last_effective_features`, execute zero VG/HGI graph passes, clear payload-source stats to `None`, and report `ProviderMissing` degradations. The provider-backed bridge path separately proves selected provider IDs reach stats and authored VG/HGI payloads execute with ready reports.

## Payload Source

`RenderVirtualGeometryPayloadSource` records the submit-time origin of the effective VG payload:

- `None`: VG did not submit a payload.
- `Authored`: the frame extract supplied an authored `RenderVirtualGeometryExtract`.
- `AutomaticFallback`: runtime provider extraction built the VG payload from regular mesh input because no authored VG payload was present.

`build_frame_submission_context(...)` keeps authored VG authoritative. It only asks the selected VG runtime provider for automatic extraction when the feature is enabled and no authored VG extract exists. `FrameSubmissionContext` then gates both the payload and its source through `AdvancedProfileRuntimePlan`, so a degraded or descriptor-disabled VG feature clears stale source labels as well as stale payloads.

`RenderStats::last_virtual_geometry_payload_source` mirrors the sanitized context value for diagnostics. This lets submit tests distinguish authored payload execution from automatic fallback execution without treating fallback extraction as authored scene data.

`RenderHybridGiPayloadSource` records the submit-time origin of the effective HGI payload. Current HGI submit does not have an automatic fallback extract builder equivalent to VG's mesh-derived extraction path, so the valid runtime states are:

- `None`: HGI did not submit a payload.
- `Authored`: the frame extract supplied an authored `RenderHybridGiExtract`.

`FrameSubmissionContext` gates the HGI payload source through `AdvancedProfileRuntimePlan` in the same way it gates the HGI extract, update plan, and feedback. `RenderStats::last_hybrid_gi_payload_source` therefore clears to `None` when HGI is degraded or descriptor-disabled, and it does not treat visibility-derived update planning as an authored or fallback HGI payload.

## Sideband Readback Feedback

`PreparedRuntimeSubmission` can carry neutral plugin renderer outputs produced by runtime prepare collectors. `collect_runtime_feedback(...)` now merges HGI and VG renderer readback outputs with those prepared sideband outputs in production, not only in test helpers.

For HGI, renderer and sideband outputs are appended, including scene-prepare atlas/capture/voxel payloads. For VG, page-table/completion/render-path outputs and NodeAndClusterCull page requests are appended before `VirtualGeometryGpuCompletion::from_readback_outputs(...)` and `take_node_and_cluster_cull_page_request_ids()` consume them. Particle feedback keeps its existing renderer-authoritative merge rule.

## Reports And Degradation

`AdvancedProviderReport` is emitted per advanced feature. It records:

- whether the profile requested the feature,
- the provider ID chosen or discovered for that feature,
- a status: `NotRequested`, `Ready`, or `Degraded`,
- structured degradation records.

`AdvancedRenderDegradation` currently reports two concrete reasons:

- `BackendCapabilityMissing`: the profile requested the feature but the backend capability summary does not satisfy one of the feature's required capabilities.
- `ProviderMissing`: the profile requested the feature but no provider ID was available.

If both are true, both degradations are recorded. The report does not hide provider absence behind backend absence, and it does not treat `DefaultRender` as degraded just because advanced features are absent.

## Runtime Plan

`AdvancedProfileRuntimePlan::from_profile_bundle(...)` evaluates a `RenderProfileBundle`, `RenderCapabilitySummary`, and `AdvancedProviderAvailability` into one report per advanced feature.

For `DefaultRender`, the plan reports both advanced features as `NotRequested` and produces no degradation. For `AdvancedRender`, a feature is enabled only when it was requested, all required backend capabilities are present, and a provider ID is present.

This makes the next integration step straightforward: submit-time code can accept only `Ready` reports while stats and profile diagnostics can surface every degraded report without silently pretending VG/HGI ran.

## Backend Capability Vocabulary

M9A extends the neutral backend vocabulary with concrete advanced requirements:

- `RenderCapabilityKind::StorageBuffers`
- `RenderCapabilityKind::IndirectDraw`
- `RenderCapabilityKind::BufferReadback`

`RenderCapabilitySummary` mirrors these fields as `supports_storage_buffers`, `supports_indirect_draw`, and `supports_buffer_readback`. The RHI backend caps expose the same booleans so WGPU capability projection can report them directly. The current WGPU RHI path reports all three as available for the headless baseline used by focused tests.

Capability reporting is also grouped into classes:

- `Default`: current default product capability requirements, currently screen-space anti-aliasing.
- `Advanced`: VG/HGI and their concrete supporting requirements.
- `Experimental`: acceleration structure, inline ray-query, and ray-tracing pipeline capabilities.

`RenderCapabilitySummary::capability_class_report(...)` returns satisfied and missing capabilities for one class. This lets profile diagnostics distinguish default render support from advanced and experimental gaps before later M9A/M9B slices decide whether to degrade, reject, or require explicit provider activation.

## Out Of Scope

This slice still does not implement native dynamic VG/HGI provider loading, Solari, ray-query policy, or deeper VG/HGI visual feature work. It fixes the neutral contract, app-linked first-party provider collection, plugin manifest/catalog capability declarations, framework-local provider arbitration, product-level AdvancedRender acceptance, submit-time provider gating, VG/HGI payload-source reporting, and production HGI/VG sideband readback merge.

## Validation Notes

Fresh M9A validation on 2026-05-19 used `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-render-m9a-advanced`. The product and broad runtime filters passed:

- `cargo test -p zircon_runtime --locked render_product_advanced --jobs 1 --message-format short --color never` passed: 2 tests, 0 failures.
- `cargo test -p zircon_runtime --locked virtual_geometry --jobs 1 --message-format short --color never` passed: 47 lib-filtered tests, 2 non-ignored integration tests, 4 ignored legacy automatic-VG tests, and filtered zero-test targets.
- `cargo test -p zircon_runtime --locked hybrid_gi --jobs 1 --message-format short --color never` passed: 19 lib-filtered tests plus filtered zero-test targets.
- Package-level checks passed for the touched provider crates: `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_virtual_geometry_runtime --locked --all-targets --jobs 1` and `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_hybrid_gi_runtime --locked --all-targets --jobs 1`.
- The full plugin workspace check passed after syncing shader importer `ShaderAsset` initializers to the current `texture_slots` field: `cargo check --manifest-path zircon_plugins\Cargo.toml --workspace --locked --all-targets --jobs 1`.
- The schema-sync follow-up was verified with package checks and lib tests for `zircon_plugin_asset_importer_shader_runtime` and `zircon_plugin_shader_wgsl_importer_runtime`.
