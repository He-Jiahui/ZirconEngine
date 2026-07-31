---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/render_graph/dump.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
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
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_preserves_imported_persistent_texture_without_pool_backing
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_exposes_owned_texture_mip_views
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_aliases_ssr_reflection_coarse_pyramid_to_parent_mip_view
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs::materialization_allocates_ssr_reflection_coarse_resource_when_parent_has_no_coarse_mip
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs::graph_transient_allocation_plan_reports_slot_reserved_bytes
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs::graph_transient_allocation_plan_bypasses_persistent_textures
  - zircon_runtime/src/render_graph/tests/resources/transient_aliasing.rs::graph_readback_lifetimes_extend_to_graph_end_and_do_not_alias
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_graph_materialization_tests.rs::runtime_15_render_graph_materialization_requires_transient_pool
doc_type: module-detail
---

# Transient Materialization

`transient_materialization.rs` owns the execution-side lowering from the compiled render graph transient allocation plan to concrete WGPU resource backings. The compiled graph remains RHI-neutral and now owns the descriptor-bucketed, interval-colored allocation plan produced during `CompiledRenderGraph` construction. `transient_allocation_plan()` returns a stable reference; frame materialization and graph dumps no longer rerun bucket grouping, interval coloring, sorting, or slot-reservation aggregation. This module is the WGPU-specific bridge that groups live logical lifetimes by `(bucket_key_hash, slot)`, verifies the descriptors are compatible for a shared backing, acquires every WGPU object through `TransientResourcePool`, and binds logical resource names back into `RenderGraphExecutionResources`. A pool miss creates a backing through the pool owner; there is no direct-allocation fallback in materialization.

`materialization.rs` remains the orchestration layer. It requires `&mut TransientResourcePool`, calls this module for texture and buffer slot materialization, then attaches any derived view aliases such as SSR reflection-pyramid mip resources. WGPU descriptor conversion and object creation helpers still live in `materialization.rs`, but only `TransientResourcePool` invokes them for a pool miss.

`CompiledRenderGraph` builds a `HashMap<RenderGraphResource, usize>` over its live lifetimes once and each transient allocation row retains the typed `RenderGraphResource`. Texture and buffer lowering resolve allocations through that compiled index, so materialization allocates no frame-local lifetime-name map. Allocation names remain available for dump and diagnostics, and slot groups remain `BTreeMap<TransientMaterializationSlotKey, _>` so deterministic physical-slot traversal and backing labels are unchanged.

Persistent resources are cull roots but not transient-pool allocations; their renderer-owned backing must be imported before materialization. Readback resources remain pool-backed, but compilation extends their logical lifetime through the terminal graph pass because CPU extraction happens after graph recording. Multiple readback outputs therefore cannot interval-alias each other before post-graph extraction, while their physical WGPU objects may still return to the pool after submission and extraction complete.

`TransientResourcePool::begin_frame` is a constant-time counter reset and does not precompute entry counts or retained bytes that `end_frame` would overwrite. `end_frame` performs one combined retained-count/byte scan per resource class before budget eviction. Under-budget pools return immediately without allocating an eviction candidate list. Over-budget pools collect candidates once, sort by `(last_used_frame, bucket_key, original_index)`, select the oldest prefix while decrementing both retained count and bytes, then remove selected per-bucket indices in descending order. This preserves oldest-first and deterministic tie behavior while reducing repeated oldest-entry selection from worst-case O(N²) to O(N log N); the final retained count and byte totals flow directly into `RenderGraphTransientPoolReport`, so `end_frame` no longer scans both pools again for entry counts. `transient_resource_pool_materialization_budget_eviction_orders_candidates_once` covers cross-bucket age ties, non-contiguous same-bucket removal, count/byte totals, and a second tighter-budget pass without WGPU setup.

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

The RG-M2 hard-cut guard `runtime_15_render_graph_materialization_requires_transient_pool` additionally rejects the removed test-only `RenderGraphExecutionResources::materialize_transient_resources(...)` entry point and any restored `Option<&mut TransientResourcePool>` plumbing. Materialization, IBL compute/readback, compiled-scene binder, and resolver fixtures now begin an explicit pool and call `materialize_transient_resources_with_pool(...)`, so tests exercise the same allocation-accounting boundary as production frames.

RG-M2 product acceptance reuses `render_product_post_full_chain_all_effects_on` and the ignored `export_render_graph_transient_cache_full_chain_wgpu_png` exporter. Both call one shared assertion over `RenderGraphExecutionAliasReport`: the deferred/post full chain must report `texture_backing_count() < texture_logical_count()`, not merely a nonzero pool reuse counter. The exporter writes the real framebuffer to `docs/tests/runtime/render/plan01_render_graph_transient_cache_full_chain_wgpu_20260718.png`. The paired DX12 capture is reserved as `docs/tests/runtime/render/plan01_render_graph_transient_cache_full_chain_dx12_renderdoc_20260718_capture.rdc`; it uses `D:\Tools\renderdoc\renderdoccmd.exe`, `WGPU_BACKEND=dx12`, and the submit-scoped `ZR_RENDERDOC_CAPTURE_NEXT=1` hook. The PNG, RDC, replay, and visual comparison remain pending until the managed WGPU gate executes.
