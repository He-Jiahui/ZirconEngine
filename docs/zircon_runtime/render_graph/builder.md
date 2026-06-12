---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/error.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/error.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/render_graph/tests/resources.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
tests:
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_duplicate_resource_names
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_tracks_transient_lifetimes_and_resource_edges
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_transient_read_without_producer
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_write_after_write_without_dependency
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_records_attachment_clear_load_store_ops
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_records_storage_writes_without_attachment_ops
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_transient_attachment_load_without_producer
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_read_after_discarded_transient_attachment_store
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_culls_unused_resource_writer_but_keeps_external_output_chain
  - zircon_runtime/src/render_graph/tests/ordering.rs::compile_exposes_inferred_resource_dependencies_on_compiled_passes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::metadata_context_resolves_pass_resource_handles
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::resource_registry_validates_declaration_kind_before_name_lookup
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compile_options_fallback_async_compute_passes_to_graphics_queue
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_rejects_storage_write_mode_on_read_access
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_compute_workload_metadata
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_sparse_texture_reservations_without_dense_transient_slot
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_transient_allocation_plan_reports_slot_reserved_bytes
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_transient_allocation_bytes
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_preserves_compute_workload_from_feature_descriptor
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_rejects_compute_workload_on_non_compute_queue
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_audits_planned_compute_workloads_against_dispatches
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_compatible_transient_texture_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_keeps_incompatible_texture_slot_resources_separate
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_reuses_entries_across_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_stale_entries_after_keep_frames
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::headless_wgpu_server_falls_back_async_compute_passes_to_graphics
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - cargo test -p zircon_runtime --lib --locked render_graph --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib graph_records_storage_writes_without_attachment_ops --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_options_fallback_async_compute_passes_to_graphics_queue --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib pipeline_compile_rejects_storage_write_mode_on_read_access --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compute_workload --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# RenderGraph Builder

`RenderGraphBuilder` is the runtime frame-graph authoring surface used by SRP compilation and renderer execution. It owns graph texture and buffer handles, imported external targets, pass dependencies, pass resource accesses, attachment load/store operations, culling decisions, queue lane assignment, resource declaration rows, and resource lifetime metadata.

## Resource Ownership

Every graph resource name is unique within a compiled frame graph. Duplicate names across graph textures, graph buffers, and external imports are rejected before pass ordering is derived. This keeps RenderDoc labels, lifetime spans, transient aliasing, and future history slot names unambiguous.

The RG-M1 handle table uses `RgTextureHandle`, `RgBufferHandle`, `ExternalResource`, and the sum type `RenderGraphResource`. `RenderGraphBuilder::create_texture(...)` and `create_buffer(...)` allocate stable logical handles before any physical WGPU resource exists, while `import_external_resource(...)` registers imported roots. `CompiledRenderGraph::resource_declarations()` preserves every declared resource row, including resources whose only writers were later culled, and `RenderGraphResourceLifetime.resource` carries the same logical handle for every live resource interval. `CompiledRenderGraph::resource_declaration(...)`, `resource_declaration_by_name(...)`, `resource_lifetime(...)`, and `resource_lifetime_by_name(...)` are the narrow lookup surface for resolver work: declarations answer "was this graph resource authored?", while lifetimes answer "did it survive culling and need execution-time backing?". This follows the Unreal RDG split between logical handles and later pooled resource resolution without exposing WGPU objects through the graph API.

Compiled pass resource access rows remain the current execution bridge: they preserve stable resource names, resource kind, access mode, and attachment ops so existing executors can continue querying execution-record counters by name. `RenderPassResourceResolver` is now attached to real `RenderPassExecutionContext` instances during staged execution; it binds the compiled graph plus current pass id so executors can resolve pass-declared `RenderGraphResource` handles back to declaration rows and live lifetimes. `RenderPassExecutionContext::attachment_ops_for_write(...)`, `reads_texture(...)`, and `reads_transient_texture(...)` now prefer that resolver when present and fall back to the stored pass-resource rows only for manually built metadata contexts. `RenderGraphExecutionResources::require_texture_view_for_declaration(...)` and `require_buffer_for_declaration(...)` consume declaration rows and validate texture/buffer kind before name lookup. Product postprocess resource checks now ask the resolver for pass-scoped read declarations before requiring texture views.

The WGPU execution resource table still exposes logical names to executors, but it now keeps a separate logical-name to physical-backing map. This allows compatible non-overlapping transient textures and buffers from `CompiledRenderGraph::transient_allocation_plan()` to share one owned WGPU backing while preserving the existing name-based executor API. Texture sharing is stricter than the neutral graph byte plan: WGPU backings are shared only when dimensions, mip levels, array depth, sample count, format, dimension, and residency match; usage is unioned. Incompatible texture descriptors fall back to one texture per logical resource, so the graph can reserve one abstract slot without binding an unsafe WGPU view at execution time.

External resources remain imported roots. Reads from external resources do not require an in-graph producer, while transient reads still require an ordered producer. Writes to external resources keep the output chain live even when culling is enabled.

## Pass Validation

The builder validates explicit dependency cycles, transient read-before-produce, and write-after-write hazards without an ordering dependency. Read-after-write dependencies are inferred after manual dependency order is known, so passes can be authored in declaration order without losing deterministic execution.

Compiled graph metadata exposes each pass resource access by stable resource name and kind. Resource lifetimes include the original descriptor, first pass index, last pass index, and whether the resource was imported.

## Attachment Operations

Texture and external target writes carry explicit attachment operations through `RenderGraphAttachmentOps`. The default transient texture write is `Clear + Store`, matching the safe first-use behavior expected by depth prepass, gbuffer, opaque scene color, and scratch targets. External writes default to `Load + Store`, because imported targets may already contain swapchain, UI, or previous composition contents. Callers can use `write_texture_with_ops(...)` or `write_external_with_ops(...)` when a pass needs explicit `Load`, `Clear`, `Store`, or `Discard` semantics.

Validation is intentionally conservative:

- a transient attachment cannot use `Load` before any ordered producer has stored content,
- a transient attachment stored with `Discard` cannot be read or loaded by a later pass,
- read-after-write dependencies are still inferred only after manual dependency order is known,
- write-after-write still requires an explicit ordering dependency.

Compiled pass metadata preserves the attachment ops beside the resource access. SceneRenderer executors can therefore derive WGPU `LoadOp` / `StoreOp` decisions from graph data instead of hard-coding each pass forever.

## Storage Writes

Compute and non-attachment passes use storage write metadata instead of pretending their outputs are render attachments. `RenderFeatureResourceWriteMode` splits `Attachment` writes from `Storage` writes at the SRP descriptor layer. `RenderGraphBuilder::write_storage_texture(...)` and `write_storage_external(...)` record ordinary write edges with no `RenderGraphAttachmentOps`, so dependency ordering, culling, queue counts, resource lifetimes, and debug evidence still work while load/store validation stays scoped to attachment writes.

The SRP compiler preserves that split when it lowers `RenderFeaturePassDescriptor` resources into the graph. Attachment writes still default to clear/store for first transient writes and load/store for later or external writes. Storage writes reject attachment ops and read resources reject storage write mode. The built-in `ao.ssao-evaluate` pass now writes `ambient-occlusion` through `write_storage_external(...)`, keeping its async-compute declaration and graphics-queue fallback evidence without assigning invalid attachment load/store semantics to the SSAO output.

## Sparse Texture Reservations

Sparse texture resources use the ordinary `TextureDesc` shape plus `TextureResidency::SparseReserved`. RenderGraph preserves that descriptor in `RenderGraphResourceLifetime`, exposes `RenderGraphResourceLifetime::is_sparse_reserved_texture()`, and counts such lifetimes in `CompiledRenderGraphStats.sparse_texture_lifetime_count`. This is a reservation contract only: no page table, residency allocator, tile upload path, WGPU sparse object, or runtime provider state is implied by the graph descriptor.

Transient allocation planning keeps sparse reservations out of the dense transient texture aliasing pool. `CompiledRenderGraphTransientAllocationPlan.texture_slot_count` still counts only dense transient texture slots, while `sparse_texture_slot_count` records the number of sparse transient reservations that a later residency manager must own explicitly. That keeps virtual texture resources visible to graph validation and diagnostics without pretending they can alias with ordinary render targets or scratch textures.

The allocation plan is byte-aware. Each dense transient allocation records its descriptor-derived `size_bytes`; each texture or buffer slot records the maximum byte requirement of all non-overlapping resources assigned to that slot; and the plan exposes `dense_texture_bytes_reserved`, `dense_buffer_bytes_reserved`, and `total_dense_bytes_reserved()`. Sparse reservations remain excluded from dense slots but still contribute to `sparse_texture_virtual_bytes`, giving residency planning a virtual footprint without claiming dense backing memory. The size estimate uses the same RHI-neutral `BufferDesc.size_bytes` and `TextureDesc::checked_storage_size_bytes()` inputs as the headless WGPU transient allocator stats.

`update_base_stats(...)` copies those byte totals into `RenderStats` as `last_graph_transient_texture_bytes_reserved`, `last_graph_transient_buffer_bytes_reserved`, `last_graph_transient_dense_bytes_reserved`, and `last_graph_sparse_texture_virtual_bytes`. Runtime diagnostics mirror them with `bytes` units under `render.graph.transient_texture_bytes_reserved`, `render.graph.transient_buffer_bytes_reserved`, `render.graph.transient_dense_bytes_reserved`, and `render.graph.sparse_texture_virtual_bytes`. These rows remain graph planning evidence; the execution resource report is the WGPU-side evidence for how many physical owned textures, external views, and buffers were actually materialized after descriptor compatibility checks.

## Compute Workload Metadata

Compute workload metadata is a planned graph contract, not a backend object. `RenderGraphComputeWorkload` carries a neutral pipeline label, non-zero workgroup size, and dispatch extent (`Viewport`, `ClusterGrid`, or `Fixed`). SRP feature descriptors attach it with `with_compute_workload(...)`; `RenderPipelineAsset::compile(...)` validates that the pass still declares `QueueLane::AsyncCompute`, then copies the workload onto `CompiledRenderPass.compute_workload`.

This gives pipeline activation, graph review, and plan-vs-execution diagnostics a stable place to see expected compute work before `SceneRenderer` resolves concrete WGPU resources. Actual execution evidence remains separate in `RenderGraphComputeDispatchRecord`, which is produced by renderer executors only after they record a concrete compute pass. Dispatch records now carry the renderer-private pipeline label, workgroup size, dispatch group count, and storage-write resource names; `RenderGraphExecutionRecord` derives expected dispatch groups from the compiled workload (`Viewport`, `ClusterGrid`, or `Fixed`) plus the current frame dispatch context, then compares pipeline label, workgroup size, and dispatch groups against those concrete records. The audit stores planned and actual dispatch groups so mismatched extents can be diagnosed without reading renderer-private WGPU objects, and it counts matched, missing, mismatched, and unexpected compute work.

The audit stays backend-neutral. `RenderStats` exposes only counts such as `last_graph_compute_planned_workload_count`, `last_graph_compute_matched_workload_count`, `last_graph_compute_missing_dispatch_count`, `last_graph_compute_workload_mismatch_count`, and `last_graph_compute_unexpected_dispatch_count`; `DiagnosticStore` mirrors them under `render.graph.compute_*_workload_count`, `render.graph.compute_missing_dispatch_count`, `render.graph.compute_workload_mismatch_count`, and `render.graph.compute_unexpected_dispatch_count`. WGPU pipelines, bind groups, buffers, and texture handles remain renderer-private.

## History Preparation

History resources must use distinct names for previous, current, and output slots before they are represented in the graph. The unique-name rule prevents a feature from accidentally declaring one physical graph resource as both imported history input and writable history output. The later scene renderer registry should map those slots onto concrete backing textures after resize, camera-cut, and motion-validity checks.

Focused RenderGraph validation on 2026-06-02 passed with 22 tests, 0 failures, using `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. The latest validation included attachment clear/load/store metadata, load-before-producer rejection, and read-after-discard rejection.

The 2026-06-03 M8 storage-write slice used the same target dir. `graph_records_storage_writes_without_attachment_ops`, `compile_options_fallback_async_compute_passes_to_graphics_queue`, and `pipeline_compile_rejects_storage_write_mode_on_read_access` passed, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only.

The 2026-06-03 M8 workload-audit slice reused `E:\cargo-targets\zircon-render-main-chain`. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only. `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never compute_workload` passed 5 filtered tests, covering graph metadata preservation, pipeline compile validation, and execution-record workload audit status. `headless_wgpu_server_falls_back_async_compute_passes_to_graphics` and `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` also passed, proving the matched workload count reaches `RenderStats` and runtime diagnostics. The follow-up dispatch-extent audit extends the execution-record tests so viewport, cluster-grid, and fixed dispatch plans preserve planned/actual dispatch-group evidence and report `DispatchExtentMismatch` when a renderer records the wrong group count.

The 2026-06-04 byte-aware transient allocation slice extended `CompiledRenderGraphTransientAllocationPlan` with per-resource byte size, per-slot reserved byte size, dense texture/buffer byte totals, and sparse virtual texture bytes. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only. Focused `cargo test -p zircon_runtime --lib render_graph::tests::resources::graph_transient_allocation_plan_reports_slot_reserved_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` initially timed out while the Windows lib-test binary was compiling and linking; after the compile lane drained and produced `zircon_runtime-b34ee8d8fc52f1fd.exe`, the warmed rerun passed 1 test, 0 failed, 2680 filtered, with existing warnings only.

The follow-up diagnostics bridge preserves those planned byte totals through `RenderStats` and `DiagnosticStore` without exposing backend allocations. Focused validation target: `cargo test -p zircon_runtime --lib render_framework_stats_report_transient_allocation_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`.

The 2026-06-12 WGPU materialization slice consumes the neutral transient allocation plan inside `RenderGraphExecutionResources`: compatible dense texture slots share one WGPU texture backing, incompatible texture slots fall back to per-logical-resource textures, and buffer slots share one WGPU buffer backing with max size and unioned usage. `SceneRendererCore` now owns `TransientResourcePool`, so submitted frame resources are released into descriptor-keyed texture/buffer pools and reused on later frames. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only. Focused lib-test commands were blocked before running the filtered tests by unrelated compile errors in `zircon_runtime/src/ui/tests/runtime_input_manager.rs` and `zircon_runtime/src/ui/tests/style_mapping.rs`; an earlier materialization attempt was also blocked by dirty `zircon_runtime/src/scene/tests/ecs_schedule.rs` test source.

The same slice now exposes `RenderGraphTransientPoolReport` through `RenderGraphExecutionResourceReport`, and runtime diagnostics mirror pool allocation behavior under `render.graph.execution.transient_pool.*`. Those diagnostics report texture/buffer creation, reuse, retained pool entries, and stale-entry eviction separately from the execution resource counts, keeping graph planning bytes, physical WGPU resources, and cross-frame reuse evidence distinct.
