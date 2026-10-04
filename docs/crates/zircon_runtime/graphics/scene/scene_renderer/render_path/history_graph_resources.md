---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/scene_frame_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/render_graph/types.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs::tests::history_binder_imports_enabled_live_history_externals
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs::tests::history_binder_skips_enabled_resources_absent_from_live_graph
doc_type: module-detail
---

# History Graph Resources

`bind_history_graph_resources.rs` owns the built-in scene history actual-binding step before RenderGraph materialization validation. These resources are not transient WGPU resources and are not plugin-owned externals; they are runtime-owned history textures and buffers already allocated by `SceneFrameHistoryTextures`.

The binder takes the compiled graph, the execution resource table, optional history resources, and `HistoryGraphResourceBindingFlags`. It imports only resources that are both enabled for the current frame and present in `CompiledRenderGraph::resource_lifetime_by_name(...)`. This keeps the actual WGPU binding aligned with the compiled graph instead of importing every possible history resource unconditionally.

## Bound Resources

The current binding set covers:

- TAA scene color history: `TAA_HISTORY_PREVIOUS` and `TAA_HISTORY_CURRENT`.
- Screen-space reflection previous history: `HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION`.
- HZB previous furthest history: `HISTORY_PREVIOUS_HZB_FURTHEST`.
- Hybrid GI previous lighting and temporal-metadata histories.
- Volumetric scattering previous history when its quality-qualified D3 allocation exists.
- Exposure history buffers: `EXPOSURE_PREVIOUS` and `EXPOSURE_CURRENT`.

Typed texture histories are imported with their borrowed backing texture, default view, and physical `TextureDesc`. The external access-ID materializer derives the pass-scoped WGPU view from the compiled range and intent packet instead of treating the owner view as the final lease. Exposure previous/current histories are imported as borrowed buffers with the owner-supplied 16-byte `STORAGE | COPY_SRC | COPY_DST` physical descriptor; using descriptor-less `insert_buffer` here would erase the lease metadata and is not permitted. Descriptor-less compatibility imports remain outside this exact-history path.

Exposure buffers are initialized to the default value by mapped creation. A retained camera-cut
invalidation records a pending reset on `SceneFrameHistoryTextures`; it does not write the queue. When
exposure resources are live, the compiled-frame owner appends the two reset ranges to the frame's one
`FrameBufferUpload` transaction. The history owner commits the reset intent only after backend admission
and producer-ledger recording, preserving retry semantics when graph recording or admission fails.

## Boundaries

This module is intentionally narrower than the HZB execution-owned external buffer binder. It does not make optional history externals required and does not allocate fallback history resources. Plugin-owned external buffers are handled separately by `bind_plugin_graph_resources.rs`; Hybrid GI and volumetric feature descriptors are plugin-linked, but their previous-history backings stay here because `SceneFrameHistoryTextures` owns those allocations.

Ambient-occlusion history is part of the exact binding set. Its renderer owner stores a dedicated SceneLinear `render_size`, allocates the `Rgba8Unorm` texture at that primary extent, and initializes it to 1.0 before scene submission. The SSAO descriptor publishes a Render-sized full-texture compute-sampled access; the history binder imports the actual texture, view, and physical descriptor. History availability only gates shader sampling and success-only feedback, so cold start no longer substitutes a 1x1 view or relaxes physical extent validation.

## Validation State

Source-contract tests build small graphs with exact typed history accesses and assert that enabled live externals publish physical texture identity before materialization, while enabled resources absent from the compiled graph are skipped. Current source formatting, metadata, and scoped contract checks pass. Focused managed Cargo/WGPU execution remains pending; this document does not claim dynamic validation for the latest exact-lease slices.
