---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/render_graph/dump.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/dump.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_creates_dense_transients_and_skips_sparse_reservations
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_compatible_transient_texture_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_receives_incompatible_texture_resources_in_separate_graph_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_overrides_preimported_terminal_aa_input_with_owned_transient
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_exposes_owned_texture_mip_views
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_ssr_reflection_coarse_pyramid_to_parent_mip_view
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_allocates_ssr_reflection_coarse_resource_when_parent_has_no_coarse_mip
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs::graph_transient_allocation_plan_reports_slot_reserved_bytes
doc_type: module-detail
---

# Transient Materialization

`transient_materialization.rs` owns the execution-side lowering from the compiled render graph transient allocation plan to concrete WGPU resource backings. The compiled graph remains RHI-neutral and now pre-buckets dense transient lifetimes by descriptor hash before assigning bucket-local slots. This module is the WGPU-specific bridge that groups live logical lifetimes by `(bucket_key_hash, slot)`, verifies the descriptors are compatible for a shared backing, acquires pooled or freshly created WGPU objects, and binds logical resource names back into `RenderGraphExecutionResources`.

`materialization.rs` remains the orchestration layer. It calls this module for texture and buffer slot materialization, then attaches any derived view aliases such as SSR reflection-pyramid mip resources. WGPU descriptor conversion and object creation helpers still live in `materialization.rs`, because they are also used by `TransientResourcePool`.

## Texture Slots

Texture allocations are grouped by `CompiledRenderGraph::transient_allocation_plan()`. The graph planner buckets textures by width, height, depth, mip levels, sample count, format, dimension, residency, and usage bits before interval coloring; slot indices are therefore bucket-local and the dump includes `bucket_key_hash` for every transient slot row. Execution materialization uses the same `(bucket_key_hash, slot)` pair as its grouping key and names physical backings as `rg-transient-texture-bucket-<hash>-slot-<slot>`, so two descriptor buckets may both use slot `0` without sharing a WGPU texture. A slot gets one WGPU texture backing only when every live logical texture in the bucket-local slot is compatible for a shared WGPU object:

- width, height, depth, mip levels, sample count, format, dimension, and residency must match.
- usage bits are already part of the graph bucket key and remain the usage used for the shared backing.
- sparse reservations remain unbacked by dense WGPU textures.
- SSR coarse reflection-pyramid resources are skipped during slot allocation when they can be exposed as a parent mip view.

If an inconsistent graph plan ever presents incompatible texture descriptors inside the same bucket-local slot, each logical texture receives its own WGPU texture as a defensive fallback. Normal descriptor separation belongs to the graph allocation plan, not this execution module.

`PostProcessGraphResourceNames::FINAL_COMPOSITED` is the explicit preimport exception: terminal AA input may be temporarily imported as a frame-target alias before graph materialization, but the live transient graph resource must replace that alias with an owned transient backing.

## Buffer Slots

Buffer allocations are grouped by graph descriptor bucket plus bucket-local transient slot with one WGPU buffer backing per `(bucket_key_hash, slot)`. The graph planner buckets buffers by size and usage before interval coloring, so every logical buffer in a slot has the same descriptor shape. Runtime backing labels use `rg-transient-buffer-bucket-<hash>-slot-<slot>`, and each logical buffer name is then mapped to the slot backing through `RenderGraphExecutionResources::bind_buffer(...)`.

## SSR Mip Aliases

`ssr_pyramid_mip_alias(...)` and `ssr_pyramid_mip_alias_for_lifetimes(...)` define the small set of post-process logical resources that are view aliases of a parent texture mip. The alias helpers live here because they are part of transient materialization policy, while `RenderGraphExecutionResources::resource_alias_report()` reuses the same helper to report `parent:mipN` physical aliases.

## Validation State

Materialization source-contract tests now assert both same-bucket aliasing and cross-bucket separation: compatible non-overlapping textures and buffers share one bucketed backing label, while incompatible texture descriptors materialize to distinct bucketed labels even when their bucket-local slot index is the same. The 2026-06-24 RenderGraph materialization test owner split keeps `graphics/scene/scene_renderer/graph_execution/materialization.rs` as the production WGPU descriptor/materialization owner and moves those tests into `graphics/scene/scene_renderer/graph_execution/materialization/tests.rs`; guard `runtime_15_render_graph_materialization_tests_are_child_owner_split` locks that boundary under `render_plan01_materialization_tests_owner_split_static_passed_cargo_deferred_active_compile_lane`. The RenderGraph resources transient aliasing tests owner split moves allocation-plan focused graph tests to `zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs` and locks the path with `runtime_15_render_graph_resources_transient_aliasing_tests_are_child_owner` under `render_graph_resources_transient_aliasing_tests_owner_split_static_passed_cargo_deferred_implementation_cadence`. Focused lib-test execution remains deferred while active compile lanes are present; the current slice only claims scoped rustfmt/static/line-count/docs-anchor/whitespace/diff-check evidence.
