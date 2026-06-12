---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_compatible_transient_texture_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_keeps_incompatible_texture_slot_resources_separate
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_reuses_entries_across_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_stale_entries_after_keep_frames
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_creates_dense_transients_and_skips_sparse_reservations
  - cargo test -p zircon_runtime --lib materialization_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# RenderGraph Execution Resources

`RenderGraphExecutionResources` is the WGPU-side resource table used by scene renderer graph executors. The compiled `RenderGraph` remains RHI-neutral and exposes logical resource declarations, lifetimes, and the transient allocation plan; this module turns the live lifetimes into concrete `wgpu::TextureView`, `wgpu::Texture`, and `wgpu::Buffer` objects while keeping executor lookup keyed by stable graph resource names.

## Logical Names And Backing Resources

Executors still ask for resources by logical names such as `scene-color`, `screen-space-reflection-depth-pyramid`, or `light-list`. Internally, the execution resource table now has an additional mapping from logical names to physical backing names. That lets two non-overlapping graph resources share one owned WGPU backing while preserving the existing executor API:

- `imported_texture_views` maps every logical texture name to the view an executor uses.
- `owned_textures` and `owned_texture_descs` store physical WGPU texture backings.
- `owned_texture_backings` maps logical texture names to the physical backing key.
- `buffers` stores physical WGPU buffer backings.
- `buffer_backings` maps logical buffer names to the physical backing key.

This keeps `require_texture_view(...)`, `require_buffer(...)`, and `owned_texture(...)` stable for callers while allowing the materialization step to use the compiled transient plan.

## Transient Slot Materialization

`materialize_transient_resources(...)` now consumes `CompiledRenderGraph::transient_allocation_plan()` instead of allocating every dense logical resource independently. Texture and buffer allocations are grouped by graph slot:

- Dense texture slots create one WGPU texture backing when every logical texture in the slot has compatible dimensions, mip levels, array depth, sample count, format, dimension, and residency. Texture usage is the union of all logical usages in the slot.
- If a texture slot contains incompatible logical descriptors, the module falls back to one WGPU texture per logical resource. This is required because WGPU texture views cannot safely represent a smaller or differently shaped logical attachment over a larger unrelated texture.
- Dense buffer slots create one WGPU buffer backing with the maximum required size and the union of all logical buffer usages in the slot.
- Sparse texture reservations stay unbacked by dense WGPU resources. They remain visible through graph lifetimes and stats only.

The RenderGraph allocation plan is therefore the neutral aliasing contract, while this module enforces the stricter WGPU object-compatibility rules at execution time.

## Cross-Frame Pool

`SceneRendererCore` owns a `TransientResourcePool` for WGPU physical resources. A render starts the pool frame before graph materialization, materializes logical graph resources through the pool, submits the command encoder, then releases all owned graph backings into the pool and ends the pool frame. Pool keys include the WGPU-relevant descriptor shape and usage bits, so a texture or buffer is reused only when the next frame requests a compatible backing. Stale entries are evicted after `TRANSIENT_RESOURCE_POOL_KEEP_FRAMES` pool frames.

This preserves the existing per-pass resolver contract while adding the RDG-style distinction between logical graph resources and reusable physical resources. The current implementation still binds all live resources for the frame up front; it does not do pass-boundary acquire/release inside a command encoder.

The pool publishes `RenderGraphTransientPoolReport` through `RenderGraphExecutionResourceReport`. Runtime diagnostics record created, reused, retained, and evicted texture/buffer counts under `render.graph.execution.transient_pool.*`, so frame captures and automated diagnostics can distinguish first-frame allocation churn from later-frame reuse.

## SSR Mip Aliases

The screen-space reflection coarse pyramid resources remain view aliases into their parent pyramid mip levels. Parent textures may now be direct logical backings or slot-backed textures. `owned_texture_mip_view(...)` resolves through the logical-to-physical backing map before creating the mip view, so SSR aliases continue to work without requiring a separate owned texture for the coarse logical resource.

## Validation State

Five tests cover the new materialization behavior: compatible non-overlapping textures share one owned WGPU backing, incompatible textures that share a neutral graph slot are kept separate, compatible non-overlapping buffers share one WGPU backing with the maximum size and unioned usage, the transient pool reuses entries across frames, and stale pool entries are evicted. The runtime diagnostics contract also asserts the `render.graph.execution.transient_pool.*` count series when the lib-test crate can compile.

`cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed on 2026-06-12 after the pool diagnostics bridge with existing warnings only. Focused lib-test commands currently do not reach their filtered tests because unrelated `zircon_runtime` lib-test compile errors exist in `zircon_runtime/src/ui/tests/runtime_input_manager.rs` and `zircon_runtime/src/ui/tests/style_mapping.rs`; an earlier materialization test attempt was also blocked by the dirty `zircon_runtime/src/scene/tests/ecs_schedule.rs` test source.
