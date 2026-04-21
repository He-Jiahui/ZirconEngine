---
related_code:
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/automatic_extract.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/cpu_reference.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/execution_mode.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/cluster_ids_for_entity.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_count.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_words.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_render_path_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry/gpu_readback/readback/virtual_geometry_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_cluster_raster_draw.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_cluster_selection.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_virtual_geometry_cluster_selections.rs
  - zircon_runtime/src/scene/world/render.rs
implementation_files:
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_resources.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/mod.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/automatic_extract.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/cpu_reference.rs
  - zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/execution_mode.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/query_virtual_geometry_debug_snapshot/query_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/mod.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/cluster_ids_for_entity.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_count.rs
  - zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/ordering/virtual_geometry_cluster_ordinal.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_indirect_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/reset_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_gpu_readback_visbuffer64.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_hardware_rasterization_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_visbuffer64_words.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_selected_clusters.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_render_path_summary.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_order.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_hardware_rasterization_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry/gpu_readback/readback/virtual_geometry_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_cluster_raster_draw.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_cluster_selection.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_frame_with_virtual_geometry_cluster_selections.rs
  - zircon_runtime/src/scene/world/render.rs
plan_sources:
  - user: 2026-04-21 implement the M5 Nanite-like Virtual Geometry convergence plan
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/graphics/types/virtual_geometry_prepare/frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_executed_cluster_selection_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/virtual_geometry_visbuffer64_pass.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_gpu.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_nanite_cpu.rs
  - zircon_runtime/tests/virtual_geometry_extract_contract.rs
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/tests/virtual_geometry_execution_snapshot_contract.rs
  - zircon_runtime/tests/virtual_geometry_visbuffer_overlay_contract.rs
  - zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs
  - zircon_runtime/tests/virtual_geometry_stats_contract.rs
doc_type: module-detail
---

# Virtual Geometry Nanite Foundation

## What This Slice Adds

This change lands the first Zircon-native Nanite-like foundation without replacing the current M5 Virtual Geometry runtime.

The implemented scope is intentionally the lowest stable layer:

- `ModelPrimitiveAsset` can now carry an optional cooked `virtual_geometry` payload beside the legacy vertex/index mesh.
- The cooked payload has a stable, typed schema for hierarchy nodes, cluster headers, cluster page headers, raw page bytes, root page information, and debug metadata.
- A CPU reference path can traverse the hierarchy, enumerate leaves, filter by `forced_mip`, track resident pages, and bridge the selected clusters into the existing `RenderVirtualGeometryExtract` shape.
- A first execution-mode taxonomy is defined for later runtime routing:
  - `CpuDebug`
  - `CompatGpu`
  - `FlagshipGpu`

This is the N1/N2 foundation from the Nanite plan, not the full N3-N7 render path. It does not yet wire live scene extraction into `VisBuffer64`, `NodeAndClusterCull`, or hardware raster passes.

The current continuation also crosses the first concrete render-debug boundary from N0/N3: `visualize_bvh` and `visualize_visbuffer` are no longer inspection-only. They now reach the shared SRP overlay path and change the captured frame without introducing a second VG/Nanite query API.

## Unified ClusterSelection Worklist

The latest convergence step removes the last meaningful split between the prepare-owned debug worklist and the runtime-frame-owned fallback raster worklist.

`zircon_runtime/src/graphics/types/virtual_geometry_cluster_selection.rs` now defines one internal `VirtualGeometryClusterSelection` DTO that carries both classes of data at once:

- cluster identity and debug-facing fields
  - `instance_index`
  - `entity`
  - `cluster_id`
  - `cluster_ordinal`
  - cluster-local `page_id` / `lod_level`
- submission and raster-facing fields
  - `submission_index`
  - submission `page_id` / `lod_level`
  - `entity_cluster_start_ordinal`
  - `entity_cluster_span_count`
  - `entity_cluster_total_count`
  - `lineage_depth`
  - `frontier_rank`
  - `resident_slot`
  - `submission_slot`
  - `state`

`VirtualGeometryPrepareFrame::cluster_selections(...)` is now the prepare-layer authority. The old compatibility outputs are projections from that single source:

- `selected_clusters(...)`
  - projects to the public `RenderVirtualGeometrySelectedCluster` debug DTO
- `same_frame_visbuffer_debug_marks(...)`
  - projects current-frame visbuffer marks from the same selection list
- `cluster_raster_draws(...)`
  - projects compat fallback raster submissions by deduplicating submission records instead of rebuilding a second draw list from scratch

`ViewportRenderFrame` now carries `virtual_geometry_cluster_selections` instead of a pre-expanded `virtual_geometry_cluster_raster_draws` map. `build_runtime_frame.rs` snapshots the prepare-owned selection list onto the runtime frame, and `build_virtual_geometry_cluster_raster_draws.rs` derives teaching-path raster draws from that same frame-owned selection seam when prepare is absent.

This matters for the M5 plan because it establishes the first true internal `ClusterSelection` bridge for N3:

- the public debug surface still sees stable `selected_clusters` and `visbuffer_debug_marks`
- the teaching raster path still receives `VirtualGeometryClusterRasterDraw`
- future `HardwareRasterization` / `VisBuffer64` work can now consume one authoritative runtime-frame worklist instead of choosing between a debug DTO and a raster DTO

### Execution Ownership Continuity

This continuation pushes the same `instance_index` ownership one step deeper into the submission and execution chain instead of letting later stages recover it from `entity + cluster range` heuristics.

- `VirtualGeometryClusterRasterDraw` now keeps `instance_index` when `VirtualGeometryClusterSelection` projects into the compat raster DTO.
- `VirtualGeometryIndirectSegmentKey` and `VirtualGeometrySubmissionDetail` now preserve that same `instance_index`, so the shared indirect-args layout remains the authority for per-instance submission ownership rather than treating instance identity as debug-only metadata.
- `RenderVirtualGeometryExecutionSegment` now exposes `instance_index` on the public renderer-owned snapshot, so execution-facing inspection can follow the same instance lineage as `selected_clusters` and `visbuffer_debug_marks`.
- `store_last_runtime_outputs.rs` now prefers `execution_segments.instance_index` when rebuilding post-render `selected_clusters`, only falling back to extract-time lookup if the execution seam did not carry explicit instance ownership.
- `virtual_geometry_indirect_args.wgsl` now writes `instance_index` into `SubmissionAuthorityRecord`, widening the GPU authority record from 14 to 15 words so later readback can treat the shader-authored authority buffer as a real per-instance fact source instead of a draw-ref-only side channel.
- `read_indirect_segments.rs` now matches the real shared indirect segment layout at 13 words and exposes `read_last_virtual_geometry_indirect_segments_with_instances()`, so shared segment readback preserves both `submission_index` and `instance_index` instead of silently dropping them during host inspection.
- `read_indirect_authority_records.rs` now decodes that widened authority buffer into a typed `VirtualGeometryIndirectAuthorityRecord`, and `read_indirect_execution_segments.rs` can now rebuild typed execution segments either from the dedicated execution-authority buffer or from `execution indices + authority records` when the older shared segment and draw-ref buffers are gone.
- `virtual_geometry_submission_execution_order.rs` now authors explicit `RenderVirtualGeometryInstance` metadata in its fixtures, so the GPU authority/readback path is validated against true `instance_index` ownership instead of relying on entity-only coincidence.
- `RenderVirtualGeometrySubmissionRecord` now also carries `instance_index`, and `store_last_runtime_outputs.rs` backfills that field from execution-backed `original_index -> instance_index` ownership before the renderer persists the public snapshot. Host-visible submission inspection now follows the same per-instance authority chain as `execution_segments`, instead of forcing tooling to merge two debug surfaces manually.
- `RenderVirtualGeometrySubmissionEntry` now mirrors that same ownership on the public submission-order surface. `virtual_geometry_indirect_stats.rs` collects submission order as `(instance_index, entity, page_id)`, and `store_last_runtime_outputs.rs` persists it directly into the renderer-owned snapshot so hosts can inspect actual submission order per instance without joining against `submission_records`.
- The renderer test/readback helpers now expose the same continuity instead of stopping at entity/page tuples. `read_mesh_draw_submission_records.rs` adds `read_last_virtual_geometry_mesh_draw_submission_records_with_instances()`, which preserves `(instance_index, entity, page_id, submission_index, draw_ref_rank)` from stored order, execution segments, GPU authority fallback, or the final shared `submission + draw-ref + segment` fallback. `read_mesh_draw_submission_order.rs` adds the parallel `read_last_virtual_geometry_mesh_draw_submission_order_with_instances()` projection for submission-order inspection.
- `RenderVirtualGeometryVisBuffer64Entry` now gives the renderer-owned snapshot its first true `VisBuffer64` abstraction. `build_virtual_geometry_debug_snapshot.rs` and `store_last_runtime_outputs.rs` both pack execution-facing `selected_clusters` into stable 64-bit visibility entries plus a published `clear_value = 0`, so the host can inspect a real 64-bit visibility result contract before the hardware raster path exists.
- `VirtualGeometryGpuReadback` now carries that same logical `VisBuffer64` result stream whenever a VG GPU prepare/readback pass ran for the frame. `virtual_geometry_gpu_readback.rs` adds `visbuffer64_clear_value` plus typed `visbuffer64_entries`, `pending_readback/collect.rs` seeds those fields on uploader readback creation, and `store_last_runtime_outputs.rs` backfills the execution-backed entry list from the same post-render cluster subset used by the renderer-owned snapshot. `read_gpu_readback_visbuffer64.rs` adds a non-consuming last-state helper for tests and future inspection tooling, so the runtime side can read the same 64-bit visibility contract without forcing tools to query the snapshot path first or consume the stored GPU readback object. The `None` semantics for frames that never produced a VG GPU readback object remain unchanged.
- That logical stream now also lands in a real renderer-owned GPU buffer instead of existing only as DTOs. `store_last_runtime_outputs.rs` packs the final `visbuffer64_entries` into `u64` words, creates `last_virtual_geometry_visbuffer64_buffer`, and stores matching `clear_value` plus `entry_count` on `SceneRenderer`. `read_visbuffer64_words.rs` reads those packed words back even after `take_last_virtual_geometry_gpu_readback()` has consumed the CPU DTO, which fixes the first true buffer boundary needed before a later `HardwareRasterizationPass` can become the producer.
- The newest follow-up makes that buffer's provenance explicit and moves the compat producer into a named render-path seam. `RenderVirtualGeometryVisBuffer64Source` now distinguishes `Unavailable`, `RenderPathClearOnly`, `RenderPathExecutionSelections`, `SnapshotFallback`, and `GpuReadbackFallback`; `RenderVirtualGeometryDebugSnapshot`, `SceneRenderer`, and `read_visbuffer64_source.rs` preserve that source so tests can prove the packed buffer came from render-path execution ownership instead of an opaque late backfill. The executed-submission filtering, cluster deduplication, stable ordering, and `u64` buffer creation now live in `virtual_geometry_visbuffer64_pass.rs` as `VirtualGeometryVisBuffer64PassOutput`, which `render.rs` consumes directly while `virtual_geometry_indirect_stats.rs` keeps only the accounting role. `render_frame_with_pipeline.rs` now threads that explicit render-path source into `store_last_runtime_outputs.rs`, so a frame that ran the compat `VisBufferClear` path but produced zero cluster selections remains observable as `RenderPathClearOnly` instead of collapsing to `Unavailable`. This is still a compat producer, not hardware rasterization, but it is now an explicit pass boundary that can later be replaced by `VisBufferClear + HardwareRasterization` without changing the renderer-owned last-state contract.
- The latest follow-up does the same thing one stage earlier for the future raster handoff. `virtual_geometry_hardware_rasterization_pass.rs` now emits execution-backed `RenderVirtualGeometryHardwareRasterizationRecord` rows directly from the same `ClusterSelection + executed submission key` seam as `VisBuffer64`, preserving cluster identity plus the startup parameters the later raster path will need: submission page/lod, cluster span/total count, lineage depth, frontier rank, and slot ownership. `virtual_geometry_indirect_stats.rs` carries those records as an explicit pass output, and `store_last_runtime_outputs.rs` now persists them onto `RenderVirtualGeometryDebugSnapshot.hardware_rasterization_records`. This keeps the public contract fixed for `ClusterSelection -> HardwareRasterizationPass` even though the producer is still compat-side and shader rasterization has not landed yet.
- That raster-startup contract now also has explicit provenance and the same real buffer boundary as `VisBuffer64`. `RenderVirtualGeometryHardwareRasterizationSource` now distinguishes `Unavailable`, `RenderPathClearOnly`, and `RenderPathExecutionSelections`; `virtual_geometry_hardware_rasterization_pass.rs` owns the current compat producer and returns `source + record_count + buffer` directly; and `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` thread that output straight into `SceneRenderer` plus `RenderVirtualGeometryDebugSnapshot.hardware_rasterization_source`. `read_hardware_rasterization_source.rs` then exposes the renderer-owned last-state helper for tests, so clear-only frames remain observable even when snapshot assembly is absent. The pass still packs startup records into GPU-readable `u32` words, but the important change is that the buffer is now pass-owned and its provenance is no longer reconstructed later from DTO presence alone.
- The same renderer-owned seam is now visible on the framework stats surface instead of stopping at snapshot/readback inspection. `RenderStats` now carries `last_virtual_geometry_visbuffer64_source`, `last_virtual_geometry_visbuffer64_entry_count`, `last_virtual_geometry_hardware_rasterization_source`, and `last_virtual_geometry_hardware_rasterization_record_count`; `read_render_path_summary.rs` exposes the corresponding `SceneRenderer` getters; and `update_stats/virtual_geometry_stats.rs` forwards those values whenever the current frame has an effective VG extract. The important semantic choice is that stats still reset to `Unavailable`/`0` when the effective VG payload disappears, even if the underlying renderer ran a compat clear-only pass because the feature stayed enabled. That keeps `RenderStats` aligned with “effective VG workload this frame” instead of leaking renderer-local pass housekeeping into the public stats contract.
- The newest convergence slice removes the last duplicated execution filtering below those compat producers. `virtual_geometry_executed_cluster_selection_pass.rs` now computes one `VirtualGeometryExecutedClusterSelectionPassOutput` from `ClusterSelection + indirect execution draws`, locking executed submission-key filtering, `(entity, cluster_id)` deduplication, and stable ordering in one unit-tested seam. `virtual_geometry_indirect_stats.rs` executes that seam exactly once per frame, and both `virtual_geometry_visbuffer64_pass.rs` and `virtual_geometry_hardware_rasterization_pass.rs` now consume the shared ordered cluster list instead of each rebuilding the same filter/sort layer independently. That keeps current compat behavior unchanged while turning the future `NodeAndClusterCull -> HardwareRasterization -> VisBuffer64` path into a producer swap instead of another three-way logic re-alignment.
- The latest follow-up gives that same shared seam its first real renderer-owned GPU buffer boundary instead of leaving it as a transient CPU-only `Vec<VirtualGeometryClusterSelection>`. `RenderVirtualGeometrySelectedCluster` now packs to and from a compact GPU word layout; `virtual_geometry_executed_cluster_selection_pass.rs` now returns `selected_cluster_count + selected_cluster_buffer` beside the internal ordered selections; `render.rs`, `render_frame_with_pipeline.rs`, and `store_last_runtime_outputs.rs` preserve that buffer on `SceneRenderer`; and `read_selected_clusters.rs` decodes it back into typed selected-cluster records for tests. This matters because the exact execution-owned cluster identity stream can now survive even when there is no renderer-owned snapshot and no uploader readback DTO, which is the same “real buffer before real shader producer” pattern already used for `VisBuffer64` and hardware-raster startup records.

That keeps the current N3 compatibility path honest: even before real `VisBuffer64` and `NodeAndClusterCull` land, the current `ClusterSelection -> compat raster -> indirect submission -> execution snapshot` bridge no longer throws away per-instance ownership midway through the frame.

It also tightens the N3 debug fallback contract: execution-facing inspection is now resilient when host-built submission mirrors are intentionally dropped for tests, because the GPU authority buffer plus execution indices are sufficient to reconstruct segment ownership, draw-ref lineage, and per-instance execution order from shader-authored truth.

## Automatic Production Extract Synthesis

This slice now also covers the first N2-to-N5 bridge step: automatic synthesis of `RenderVirtualGeometryExtract` from cooked VG assets when the `Virtual Geometry` feature is enabled but the incoming frame extract still carries `geometry.virtual_geometry = None`.

`zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/automatic_extract.rs` adds a deterministic flattening path that:

- walks cooked `VirtualGeometryAsset` payloads attached to model primitives,
- emits every cooked cluster into the production `RenderVirtualGeometryExtract.clusters` list instead of only the CPU-selected frontier,
- remaps local cluster ids and page ids into one global id space across all instances so the current runtime page table and parent-page derivation stay authoritative,
- preserves parent-cluster lineage after remap,
- transforms cluster bounds from local mesh space into world space using the mesh instance transform, and
- seeds initial resident pages from `root_page_table` with deterministic cluster/page budgets derived from the CPU reference plus the cooked root lineage.

`zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/automatic_extract.rs` provides the renderer-side host hook that resolves model assets through the current asset manager and feeds them into the Nanite automatic-extract helper.

That automatic path now returns one internal bundle instead of only the flattened production extract:

- `extract`
  - the current runtime-facing `RenderVirtualGeometryExtract` payload
- `cpu_reference_instances`
  - per-instance CPU-reference BVH inspection for the same cooked asset lineage

That bundle keeps the production VG path and the teaching/debug BVH path sourced from one cooked-asset walk instead of rebuilding two separate traversals at the render-framework layer.

That renderer-side bridge now also consumes the prepared-model cache instead of blindly reloading `ModelAsset` from the asset manager every frame. `PreparedModel` retains an `Arc<ModelAsset>` beside the GPU resource, and `ResourceStreamer::load_model_asset(...)` now prefers that cached asset when the prepared revision still matches the current resource revision. If the asset revision changed, it falls back to the asset manager so hot-reimported cooked VG data can still refresh correctly.

The automatic Nanite extract path now also fills the new extract-side metadata:

- each cooked mesh instance contributes one `RenderVirtualGeometryInstance`
- `cluster_offset/page_offset` and `cluster_count/page_count` point into the flattened global `clusters/pages` arrays
- `source_model` is retained when synthesis starts from `RenderMeshSnapshot`
- `mesh_name/source_hint` are copied from cooked VG debug metadata
- extract-level `debug` is currently initialized from defaults, leaving room for later editor/runtime debug overrides to flow through the same contract

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs` now uses that hook before building `VisibilityContext`:

- if the feature is disabled, nothing changes;
- if the caller already authored `geometry.virtual_geometry`, that payload still wins untouched;
- if the feature is enabled and the authored payload is absent, the render framework synthesizes the VG extract from cooked model assets and feeds the supplemented extract into visibility planning and runtime submission.

This keeps the current M5 host/runtime structure intact while letting cooked Nanite-like data participate in the production VG visibility/page workflow.

## Extract Contract Growth

This slice now also upgrades the public `RenderVirtualGeometryExtract` contract so it can carry the first Nanite-style instance/debug metadata without breaking the current `clusters/pages` consumers.

`zircon_runtime/src/core/framework/render/scene_extract.rs` now defines:

- `RenderVirtualGeometryInstance`
  - one synthesized VG instance record with `entity`, optional `source_model`, `transform`, cluster/page range offsets, and optional cooked debug labels
- `RenderVirtualGeometryDebugState`
  - extract-level debug flags matching the current CPU-reference vocabulary: `forced_mip`, `freeze_cull`, `visualize_bvh`, `visualize_visbuffer`, and `print_leaf_clusters`

`RenderVirtualGeometryExtract` now carries both:

- `instances`
- `debug`

The important boundary here is compatibility:

- existing visibility/runtime/prepare code can continue reading only `cluster_budget`, `page_budget`, `clusters`, and `pages`
- new Nanite-oriented code can start consuming instance ranges and debug state from the same extract payload instead of inventing a second side channel

This is the first extract-side convergence step from the M5 plan's "instance-driven input" target. It does not yet replace the current flattened cluster/page bridge.

## Viewport Debug Override And Visibility Consumption

This continuation closes the first gap between the richer extract contract and the production visibility/runtime path.

### Host-Facing Debug Override Plumbing

`zircon_runtime/src/core/framework/render/camera.rs` now lets `SceneViewportExtractRequest` carry `virtual_geometry_debug`.

That debug override is then preserved through:

- `SceneViewportRenderPacket.virtual_geometry_debug`
- `RenderFrameExtract.geometry.virtual_geometry_debug`
- `RenderFrameExtract::to_scene_snapshot()`

`zircon_runtime/src/scene/world/render.rs` copies the request-level override into the viewport render packet, and `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs` applies that override onto the effective VG extract before `VisibilityContext` is built.

This is the first production host path that lets runtime preview and editor viewport drive Nanite-like debug controls without requiring an explicitly authored `geometry.virtual_geometry` payload.

### Instance-Aware Virtual Geometry Planning

`zircon_runtime/src/graphics/visibility/planning/build_virtual_geometry_plan/build.rs` now treats `RenderVirtualGeometryExtract.instances` as authoritative range metadata instead of ignoring it.

The current behavior is:

- if `instances` is empty, planning falls back to the legacy entity-based scan across `clusters/pages`
- if `instances` is present, planning only considers cluster/page slices covered by the visible instances' `cluster_offset/cluster_count` and `page_offset/page_count`
- `cluster_count` and `cluster_ordinal` now derive from those instance-scoped ranges instead of scanning every cluster that happens to share the entity id

This is still a compatibility bridge, not a full Nanite `HierarchyBuffer` traversal. The scope here is narrower: the public extract contract now materially changes runtime planning instead of existing as metadata-only sidecars.

### Debug-State Effects In Visibility

The first two extract-level debug flags now alter production VG planning:

- `forced_mip`
  - filters the eligible cluster set before visibility refinement, so only clusters at the forced mip participate in visibility, page-request, and draw-segment generation
- `freeze_cull`
  - reuses the previous frame's visible cluster ids and requested-page set when history exists, instead of recomputing the cluster frontier from current camera visibility

`visualize_bvh` and `visualize_visbuffer` no longer sit idle at this layer. They now feed the renderer-owned snapshot plus same-frame scene-gizmo overlays through `build_runtime_frame.rs`. `print_leaf_clusters` remains the primary inspection-only debug flag until later GPU debug passes arrive.

## Stats Observability

This continuation also promotes the first Nanite-style `instances/debug` signals into the host-visible stats surface.

`zircon_runtime/src/core/framework/render/backend_types.rs` now exposes these additional `RenderStats` fields:

- `last_virtual_geometry_instance_count`
- `last_virtual_geometry_forced_mip`
- `last_virtual_geometry_freeze_cull`
- `last_virtual_geometry_visualize_bvh`
- `last_virtual_geometry_visualize_visbuffer`
- `last_virtual_geometry_print_leaf_clusters`

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/virtual_geometry_stats.rs` fills those fields from `FrameSubmissionContext.virtual_geometry_extract`, which means the stats surface now reflects the effective authored-or-synthesized VG payload after viewport debug overrides have been applied.

This matters for the Nanite convergence plan because the richer extract contract is no longer observable only through internal visibility behavior. Runtime hosts can now query whether a frame actually ran with:

- instance-driven VG input,
- a forced mip override,
- frozen culling,
- BVH/VisBuffer debug visualization intent, or
- leaf-cluster print mode.

The same stats updater also clears those fields when the next frame no longer carries an effective VG payload, so host-side tooling does not keep stale Nanite debug state alive across non-VG frames.

## Renderer Inspection Snapshot

This continuation also adds a renderer-owned inspection surface for the richer Nanite-style debug state instead of pushing more transient fields into `RenderStats`.

`zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs` defines `RenderVirtualGeometryDebugSnapshot`, and `RenderFramework::query_virtual_geometry_debug_snapshot()` now exposes it through the public framework API.

The snapshot is assembled in `build_virtual_geometry_debug_snapshot.rs` from the effective `FrameSubmissionContext` state after authored data, automatic cooked-VG synthesis, viewport debug overrides, visibility feedback, and page-upload planning have all been applied. That keeps the query aligned with what the renderer actually used for the most recent frame instead of only reflecting author intent.

The current snapshot surface is intentionally host-oriented and minimal:

- `instances`
- `debug`
- `cpu_reference_instances`
  - one entry per automatically synthesized cooked-VG instance
  - each entry carries `instance_index`, `entity`, `mesh_name`, `source_hint`
  - `visited_nodes` records BVH node id/depth/page/mip/leaf state plus the node-local cluster ids
  - `leaf_clusters` records leaf-node cluster membership, loaded state, parent cluster id, and bounds/error metadata
  - `page_cluster_map` records the asset-local page-to-cluster mapping
  - ids remain asset-local inside this inspection surface even though the production extract remaps cluster/page ids into one global runtime id space
- `bvh_visualization_instances`
  - populated only when `debug.visualize_bvh` is enabled
  - one entry per automatically synthesized cooked-VG instance
  - each entry carries a ready-to-draw BVH node tree with `parent_node_id`, `child_node_ids`, node depth, page/mip ownership, direct node-local `cluster_ids`, subtree-selected/resident cluster ids, and node bounds/error metadata
  - this keeps BVH visualization on the same renderer-owned inspection path instead of introducing a separate Nanite BVH query
- `visible_cluster_ids`
- `selected_clusters`
  - one entry per current-frame cluster selected by the prepare-owned compatibility path at submission-build time, then re-authoritatively filtered to the real execution subset when the renderer stores last-state
  - each entry carries `instance_index`, `entity`, `cluster_id`, `cluster_ordinal`, `page_id`, `lod_level`, and the current resident/pending/missing execution state represented by the active snapshot phase
  - this gives hosts and future raster work one stable current-frame cluster worklist without reverse-engineering it from visbuffer color tags
- `visbuffer_debug_marks`
  - populated only when `debug.visualize_visbuffer` is enabled
  - now derived from `selected_clusters` during submission-build snapshot assembly when prepare-owned cluster selection truth exists
  - same-frame overlay construction first gates on `debug.visualize_visbuffer`, then prefers prepare-time `unified_indirect_draws()` / non-missing visible-cluster truth so authored-VG and automatic cooked-VG frames follow the same current-frame worklist before post-render execution backfill
  - when prepare-derived marks are unavailable, the current-frame overlay path can still fall back to the submission snapshot marks as a compatibility seam
  - the renderer-owned stored snapshot is then backfilled from actual `execution_segments` during `store_last_runtime_outputs.rs`, so missing visibility-only clusters no longer survive into the post-render inspection surface
  - each stored mark carries `instance_index`, `entity`, `cluster_id`, `page_id`, `lod_level`, execution-derived resident/pending state, and a deterministic RGBA debug tag
  - this remains explicit compatibility-path inspection truth, not a claim that real `VisBuffer64` pixels already exist
- `visbuffer64_clear_value`
  - currently published as `0`
  - defines the clear contract for the first renderer-owned `VisBuffer64` abstraction before real pixel storage exists
- `visbuffer64_entries`
  - always derived from the same `selected_clusters` worklist that drives post-render execution inspection, not gated by `debug.visualize_visbuffer`
  - each entry carries `entry_index`, `packed_value`, and the decoded `instance_index/entity/cluster_id/page_id/lod_level/state` metadata that produced that 64-bit word
  - the current compatibility pack layout uses fixed-width fields for `cluster_id`, `page_id`, `instance_index`, `lod_level`, and execution state so hosts can inspect stable 64-bit visibility results without opening another query path
  - this is still a logical visibility-entry stream, not a claim that the engine already owns a pixel-addressable Nanite `VisBuffer64` texture
- `requested_pages`
- `resident_pages`
- `dirty_requested_pages`
- `evictable_pages`
- prepare-backed page residency/request inspection:
  - `resident_page_inspections`
  - `pending_page_request_inspections`
  - `available_page_slots`
  - `evictable_page_inspections`
  - each resident/evictable page inspection carries `page_id`, `slot`, and `size_bytes`
  - each pending request inspection carries `page_id`, `size_bytes`, `generation`, `frontier_rank`, `assigned_slot`, and `recycled_page_id`
- `leaf_clusters`, but only when `print_leaf_clusters` is enabled
- render-derived execution summary:
  - `execution_segment_count`
  - `execution_page_count`
  - `execution_resident_segment_count`
  - `execution_pending_segment_count`
  - `execution_missing_segment_count`
  - `execution_repeated_draw_count`
  - `execution_indirect_offsets`
- render-derived execution segment view:
  - `execution_segments`
  - each `execution_segment` carries `entity`, `page_id`, `draw_ref_index`, best-effort submission token data, cluster ordinal/span/total counts, submission slot, execution state, lineage depth, lod level, frontier rank, and `original_index`
- render-derived submission truth:
  - `submission_order`
  - `submission_records`
  - each `submission_record` carries `entity`, `page_id`, best-effort `draw_ref_index`, `submission_index`, `draw_ref_rank`, and `original_index`

Renderer ownership is important here. `ViewportRenderFrame` now carries an internal `virtual_geometry_debug_snapshot`, `SceneRenderer` stores the last submitted copy inside its Virtual Geometry last-state, and `WgpuRenderFramework` returns that renderer-owned snapshot through the query API. That means runtime preview and editor hosts can inspect BVH/leaf-cluster intent without coupling themselves to `RenderStats` reset policy or to future GPU readback layout changes.

The execution-summary part of the snapshot is intentionally filled in two stages:

- `build_virtual_geometry_debug_snapshot.rs` still constructs the host-facing snapshot from the effective `FrameSubmissionContext`, preserving authored-or-synthesized extract state, visibility feedback, and page-upload outcomes before rendering starts.
- the same submission context now also carries the automatic cooked-VG `cpu_reference_instances` bundle, so `RenderFramework::query_virtual_geometry_debug_snapshot()` can expose per-instance BVH node visits, leaf clusters, and page maps without adding a second Virtual Geometry query API.
- that same builder now also turns `visualize_bvh` into concrete `bvh_visualization_instances` and turns `visualize_visbuffer` into concrete `visbuffer_debug_marks`, so those debug flags are no longer host-visible intent only.
- when prepare-time unified-draw truth exists, `build_virtual_geometry_debug_snapshot.rs` now seeds `visbuffer_debug_marks` from `VirtualGeometryPrepareFrame::same_frame_visbuffer_debug_marks(...)` instead of re-expanding the broader visibility frontier, so the submission-build snapshot and the same-frame overlay now share one authoritative compatibility-path source before render-time execution backfill happens.
- `zircon_runtime/src/graphics/types/virtual_geometry_cluster_selection.rs` now defines `VirtualGeometryClusterSelection`, and `VirtualGeometryPrepareFrame::cluster_selections(...)` exposes the prepare-owned current-frame cluster worklist before any visbuffer/debug or raster projection happens.
- `VirtualGeometryPrepareFrame::selected_clusters(...)`, `same_frame_visbuffer_debug_marks(...)`, and `cluster_raster_draws(...)` are now all derived views over that single `cluster_selections(...)` result instead of maintaining parallel cluster-remap or raster-remap logic.
- `build_virtual_geometry_debug_snapshot.rs` now publishes the projected public `RenderVirtualGeometrySelectedCluster` records from that same internal worklist when prepare truth exists, and its submission-build `visbuffer_debug_marks` are derived from the same selected-cluster list.
- `build_runtime_frame.rs` now re-derives same-frame visbuffer marks from the prepare-time unified draw list when that truth exists, and it keeps the overlay disabled when `visualize_visbuffer` is false so baseline frames do not accidentally inherit the compatibility marker path.
- `ViewportRenderFrame` now also carries `virtual_geometry_cluster_selections`, which is populated from `VirtualGeometryPrepareFrame::cluster_selections(...)` during runtime-frame assembly instead of snapshotting a pre-expanded raster-only map.
- `VirtualGeometryClusterRasterDraw` remains the compat fallback raster DTO, but it is now a projection from `VirtualGeometryClusterSelection` instead of a separately-owned runtime-frame seam.
- `store_last_runtime_outputs.rs` then backfills the actual render-derived execution summary, indirect offsets, and execution submission order/records into that same snapshot just before it is stored as renderer last-state.
- that same store step now also rebuilds `selected_clusters` from the actual `execution_segments` plus entity-local cluster ranges, so the stored public cluster worklist shrinks from the broader submission-build compatibility set down to the real raster-submitted subset whenever execution filtering removed clusters.
- `visbuffer_debug_marks` are then re-derived from that stored execution-backed `selected_clusters` list, so the post-render inspection surface keeps one authoritative cluster-selection truth instead of parallel backfill paths for worklists and color marks.
- when runtime submission token records and draw-ref records are both present, that same store step also merges them by `original_index` so `submission_records` can expose `draw_ref_index` without a second public query surface.
- `virtual_geometry_indirect_stats.rs` now also converts the filtered `execution_draws` list into typed `execution_segments` before `render_frame_with_pipeline.rs` stores the renderer-owned snapshot, so host tooling can inspect execution state/LOD/submission-slot data without going through test-only GPU readback helpers.
- those typed `execution_segments` now also retain `instance_index`, so post-render inspection and later raster work can keep instance ownership without reopening entity-local cluster scans.

That split keeps one query surface while avoiding a parallel API just to expose `SceneRenderer`'s post-render Virtual Geometry execution counters.

## Runtime BVH / VisBuffer Overlay Rendering

This continuation upgrades `visualize_bvh` and `visualize_visbuffer` from "host can inspect debug data through the snapshot" to "the renderer actually draws debug overlays through the normal overlay pass stack."

`zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs` is now the convergence point:

- it still builds the renderer-owned debug snapshot from `FrameSubmissionContext`
- it now also converts `FrameSubmissionContext.virtual_geometry_bvh_visualization_instances` into `RenderOverlayExtract.scene_gizmos` before `ViewportRenderFrame::from_extract(...)` snapshots the scene
- it also converts current-frame visbuffer marks back into per-cluster scene-gizmo markers using the effective production cluster bounds from `FrameSubmissionContext.virtual_geometry_extract`
- it now also snapshots the current frame's unified internal cluster-selection worklist into `ViewportRenderFrame.virtual_geometry_cluster_selections` by calling `VirtualGeometryPrepareFrame::cluster_selections(...)`, creating a runtime-frame-owned work submission seam beside the debug snapshot seam
- `VirtualGeometryPrepareFrame::same_frame_visbuffer_debug_marks(...)` now owns the authoritative same-frame visbuffer mark derivation from `unified_indirect_draws()` plus visible-cluster fallback, so `build_runtime_frame.rs` no longer reconstructs cluster ordering, state mapping, or instance lookup on its own
- the same-frame visbuffer path now respects the `visualize_visbuffer` gate, prefers prepare-owned current-frame truth over stale submission-time snapshot marks, and still preserves a compatibility fallback when prepare cannot contribute marks
- host-authored overlays are preserved; the Nanite-like debug overlays are appended instead of replacing existing `scene_gizmos`

That same runtime-frame seam now also feeds the teaching fallback raster path one layer later:

- `build_mesh_draw_build_context.rs` still prefers `prepare.visible_entities` when prepare is present, but it now falls back to the entities present in `ViewportRenderFrame.virtual_geometry_cluster_selections` when prepare is absent
- `build_virtual_geometry_cluster_raster_draws.rs` now prefers the frame-owned selection seam and projects compat raster draws from it, only falling back to recomputing from `virtual_geometry_prepare` when the new field is absent
- this keeps the current teaching raster path behavior stable while moving the actual runtime ownership boundary onto `ClusterSelection`, which is the intended bridge toward a future dedicated `ClusterSelection -> HardwareRasterization` handoff

The actual drawable representation stays deliberately simple and deterministic for this teaching/debug phase:

- each BVH node becomes an AABB-style wireframe built from the node's `bounds_center` and `bounds_radius`
- parent-child BVH relationships become connector lines between node centers
- each visible-cluster visbuffer mark becomes a lifted leader-line plus cross/wireframe marker anchored to the production cluster bounds so it survives the shared depth-tested gizmo pass on the same frame
- line colors encode the current Nanite-like state already exposed by the snapshot:
  - unselected/internal traversal context
  - selected and fully resident subtree
  - selected but partially resident subtree
  - selected but non-resident subtree
  - visbuffer mark colors still come from the deterministic per-cluster RGBA tags published by the snapshot

`SceneGizmoKind` now includes `VirtualGeometryBvh` and `VirtualGeometryVisBuffer`, but the implementation intentionally uses line-only gizmos with no icon dependency. That keeps the feature inside the existing overlay renderer instead of adding a parallel BVH/visbuffer debug pass or a second render-only debug surface.

This is still not a real `VisBuffer64` texture debug view. The important convergence point is architectural: the renderer now owns both a logical 64-bit visibility-entry stream (`visbuffer64_entries`) and the current overlay/debug compatibility view, and both are sourced from the same `ClusterSelection -> selected_clusters -> execution subset` seam through the existing SRP/runtime-frame submission path.

## Asset Data Model

`zircon_runtime/src/asset/assets/model.rs` now defines a Nanite-like cooked asset payload:

- `VirtualGeometryHierarchyNodeAsset`
  - One hierarchy node with parent link, up to four child node ids, cluster range, owning page id, mip level, and bounds/error metadata.
- `VirtualGeometryClusterHeaderAsset`
  - One cluster record with page ownership, hierarchy node ownership, LOD/mip level, parent cluster link, and the bounds/error fields needed by later culling logic.
- `VirtualGeometryClusterPageHeaderAsset`
  - Page id, byte offset, and byte size for one cluster page.
- `VirtualGeometryRootClusterRangeAsset`
  - Root-facing cluster range metadata used by the CPU reference traversal to seed the hierarchy walk deterministically.
- `VirtualGeometryDebugMetadataAsset`
  - Human-oriented labels and notes for dumps, inspection, and teaching content.
- `VirtualGeometryAsset`
  - The cooked container that groups the buffers above plus `cluster_page_data` and `root_page_table`.

`ModelPrimitiveAsset.virtual_geometry` is optional and defaults to `None`. This keeps existing mesh-only assets valid while allowing a single primitive to carry both:

- legacy triangle data for compatibility fallback, and
- a cooked Nanite-like hierarchy/page payload for the new VG path.

The importer and builtin model constructors now initialize `virtual_geometry: None`, so existing asset ingestion remains behaviorally stable until a real VG cook step populates the field.

## Project Asset Ingestion

This slice also closes the first real project-asset gap for cooked VG data: `.model.toml` files are now first-class source assets in the project importer.

`zircon_runtime/src/asset/importer/ingest/import_model.rs` parses a source `.model.toml` file directly into `ModelAsset`, and `import_from_source.rs` now routes `*.model.toml` through that path before the generic extension dispatch.

That matters because the Nanite automatic-extract bridge already resolves `ModelAsset` payloads from the project asset manager. Without a source importer for `.model.toml`, cooked VG payloads only existed as schema and tests, not as stable project content. With this step in place:

- a project can author or cook `res://models/*.model.toml` files that carry `virtual_geometry`,
- the normal asset pipeline imports them as `ResourceKind::Model`,
- `ProjectAssetManager::load_model_asset(...)` returns the cooked VG payload intact, and
- the existing Nanite fallback synthesis path can consume those project assets without introducing a parallel asset entry path.

## CPU Reference Flow

`zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/cpu_reference.rs` defines the teaching/reference path.

### Inputs

- `VirtualGeometryAsset`
- an entity id that will own the generated extract clusters
- a resident page set
- `VirtualGeometryCpuReferenceConfig`, currently centered on `VirtualGeometryDebugConfig`

### Debug Controls

The first debug surface matches the plan vocabulary even though only part of it is active today:

- `forced_mip`
- `freeze_cull`
- `visualize_bvh`
- `visualize_visbuffer`
- `print_leaf_clusters`

For this slice, `forced_mip` is the active selector. The other flags are stored as stable API surface for later passes.

### Traversal Behavior

`VirtualGeometryCpuReferenceFrame::from_asset(...)` performs:

1. Root discovery from `root_cluster_ranges` or parentless hierarchy nodes.
2. Deterministic depth-first hierarchy traversal.
3. Per-node visit recording:
   - node id
   - depth
   - page id
   - mip level
   - leaf/non-leaf state
   - cluster ids covered by the node range
4. Leaf cluster recording with:
   - node ownership
   - cluster id
   - page id
   - mip level
   - resident/non-resident status
   - parent cluster link
   - bounds/error metadata
5. Page-to-cluster mapping for debug and future residency/page tooling.

Selected clusters are currently defined as:

- resident page only, and
- mip matches `forced_mip` when that override is present.

That rule is deliberately simple. It gives Zircon a deterministic golden reference before automatic SSE-driven LOD and multi-pass BVH culling are introduced.

The CPU reference bridge now also emits one `RenderVirtualGeometryInstance` plus `RenderVirtualGeometryDebugState` when it converts into `RenderVirtualGeometryExtract`, so the teaching/reference path and the production automatic path both feed the same richer extract contract.

## Bridge To Current Virtual Geometry

`VirtualGeometryCpuReferenceFrame::to_render_extract(...)` is the compatibility bridge from the new Nanite-like data model to the current M5 VG surface.

It produces:

- `RenderVirtualGeometryCluster` entries from the CPU-selected leaf clusters
- `RenderVirtualGeometryPage` entries from the cooked page headers plus the supplied resident-page set
- the existing `cluster_budget` / `page_budget` fields expected by the current visibility/runtime pipeline

This is the key “gradual absorption” step:

- the cooked Nanite-style asset and hierarchy logic exist now,
- but they still flow into the existing `RenderVirtualGeometryExtract` contract,
- so the current M5 runtime can absorb the new data incrementally instead of being replaced wholesale.

## Execution Modes

`zircon_runtime/src/graphics/runtime/virtual_geometry/nanite/execution_mode.rs` defines the first execution-mode contract:

- `FlagshipGpu` when `RenderCapabilitySummary.virtual_geometry_supported` is true
- `CompatGpu` when the backend cannot claim flagship VG support but still exposes a usable render surface/offscreen path
- `CpuDebug` when no GPU-backed VG path should be assumed

This is only routing policy for now. The actual runtime still needs later work to switch behavior between these modes.

## Validation

This slice is locked by two focused tests:

- `zircon_runtime/src/asset/tests/assets/model.rs`
  - proves `ModelAsset` round-trips a cooked `virtual_geometry` payload through TOML
- `zircon_runtime/src/asset/tests/pipeline/manager.rs`
  - proves a source `res://models/*.model.toml` file imports through `ProjectAssetManager` as a model resource and preserves the cooked VG payload end-to-end
- `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_model_asset.rs`
  - proves current prepared model assets short-circuit fallback loading and stale prepared assets fall back to the latest asset load
- `zircon_runtime/tests/virtual_geometry_extract_contract.rs`
  - proves the public `RenderVirtualGeometryExtract` contract now carries instance ranges plus debug state and preserves them through clone/equality semantics
- `zircon_runtime/tests/virtual_geometry_visibility_debug_contract.rs`
  - proves viewport-request VG debug overrides round-trip into `RenderFrameExtract`
  - proves visibility planning now respects instance cluster/page ranges
  - proves `forced_mip` filters the production VG visibility set
  - proves `freeze_cull` preserves previous visible clusters and requested pages through history
- `zircon_runtime/tests/virtual_geometry_stats_contract.rs`
  - proves render-framework stats expose effective VG instance/debug state
  - proves those stats clear back to defaults once the effective VG payload disappears
- `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs`
  - proves the framework exposes the renderer-owned VG debug snapshot
  - proves visible-cluster ids, page residency/request state, and optional leaf-cluster output reflect the effective frame submission
  - proves the same snapshot now exposes `selected_clusters`, so host tooling can inspect the current-frame cluster worklist directly instead of reverse-engineering it from `visbuffer_debug_marks`
  - proves automatic cooked-VG synthesis also exposes per-instance CPU-reference BVH node visits, leaf clusters, and page-to-cluster maps through that same snapshot
  - proves `visualize_bvh` now exposes concrete `bvh_visualization_instances` for automatic cooked-VG assets through that same snapshot
  - proves `visualize_bvh` now also changes the captured frame through the shared overlay renderer, so BVH visualization is no longer snapshot-only
  - proves `visualize_visbuffer` now exposes concrete `visbuffer_debug_marks` for the current visible production cluster set through that same snapshot
  - proves the stored snapshot now re-filters those `visbuffer_debug_marks` through actual `execution_segments`, so missing visibility-only clusters are removed from the post-render inspection surface
  - proves `visualize_visbuffer` now also changes the captured frame through the shared overlay renderer for automatic cooked-VG content, so the current visbuffer compatibility view is no longer snapshot-only
  - proves the same snapshot now exposes prepare-backed resident slot mapping, pending request metadata, and available-slot truth without adding a second VG inspection API
  - proves render-derived execution summary counts and indirect offsets are backfilled into the same snapshot and stay aligned with `RenderStats`
  - proves typed `execution_segments` are queryable from that same snapshot and preserve resident/pending execution state plus execution-owned `instance_index`
  - proves render-derived submission order/records are queryable from that same snapshot without opening a second VG inspection API
  - proves both `submission_order` and `submission_records` now preserve execution-owned `instance_index`, so host tooling can inspect per-instance submission order without rejoining against `execution_segments`
  - proves `submission_records` can carry `draw_ref_index` when the execution submission and draw-ref channels are both available
  - proves a non-VG frame clears the last snapshot back to `None`
- `zircon_runtime/tests/virtual_geometry_execution_snapshot_contract.rs`
  - proves stored `execution_segments` now keep `instance_index` from the authoritative cluster-selection/submission seam instead of forcing later consumers to reconstruct instance ownership from the extract
  - proves the stored snapshot re-filters `selected_clusters` through actual `execution_segments`, so post-render consumers observe the same authoritative execution-backed worklist that the renderer submitted
  - proves the same store-time authoritative rebuild keeps `visbuffer_debug_marks` aligned with that execution-backed cluster subset instead of preserving missing visibility-only clusters
  - proves the same execution-backed subset now also emits stable `visbuffer64_entries` plus a published clear value, so the first `VisBuffer64` abstraction is sourced from the same authoritative post-render worklist as the rest of the execution-facing snapshot
- `zircon_runtime/tests/virtual_geometry_visbuffer_overlay_contract.rs`
  - proves same-frame `visualize_visbuffer` overlays follow the real non-missing execution subset for an explicit authored VG extract instead of resurrecting missing clusters
- `zircon_runtime/src/graphics/types/virtual_geometry_prepare/frame.rs`
  - proves the prepare layer itself exposes same-frame visbuffer marks derived from unified draw truth before the renderer-owned snapshot is backfilled from execution
- `zircon_runtime/src/graphics/types/virtual_geometry_prepare/frame.rs`
  - also proves the prepare layer exposes `selected_clusters(...)` as a prepare-owned cluster worklist derived from unified draw truth before marks are projected from it
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs`
  - proves the submission-build snapshot now prefers prepare-owned same-frame visbuffer marks when prepare has already projected the authoritative draw subset
- `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs`
  - proves the runtime-frame overlay path consumes prepare-owned same-frame visbuffer marks and still follows the prepare-time unified draw fallback when the stored snapshot is still empty
  - proves the same module-local reconstruction stays disabled when `visualize_visbuffer` is false
- `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs`
  - proves mesh-build assembly now accepts frame-owned VG cluster raster input without direct prepare access, preserving allowed-entity gating and per-entity draw ownership from the new runtime-frame seam
- `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs`
  - proves the stored renderer-owned snapshot re-authoritatively rebuilds `selected_clusters` from `execution_segments` when submission-build selection was broader than the real execution subset
- `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs`
  - proves the shared indirect-args layout preserves `instance_index` inside renderer-facing `VirtualGeometrySubmissionDetail`, so execution ownership stays sourced from the same authoritative submission plan
- `zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs`
  - proves execution segments still survive when the shared indirect segment and draw-ref buffers are dropped, as long as execution indices and GPU authority remain available
  - proves execution segments and submission records can reconstruct the same per-instance ownership from `execution indices + GPU authority` and can still recover `draw_ref_index` when the host-built execution-record mirror is gone
  - proves the helper submission-order and helper submission-record surfaces now preserve `instance_index` instead of collapsing back to entity/page-only tuples once execution-owned truth is available
- `zircon_runtime/src/graphics/tests/virtual_geometry_nanite_cpu.rs`
  - proves execution-mode selection
  - proves hierarchy traversal, page mapping, and `forced_mip` filtering
  - proves the bridge into `RenderVirtualGeometryExtract`
  - proves automatic extract synthesis remaps multi-instance cluster/page ids into a global space
  - proves world-space bounds and parent-cluster lineage survive the remap
  - proves mesh-snapshot/model-resolver synthesis only collects cooked models
  - proves explicit authored VG payload still overrides the automatic fallback

Focused validation completed for this slice:

- `cargo test -p zircon_runtime --locked asset::tests::pipeline::manager::asset_manager_imports_model_toml_with_virtual_geometry_payload -- --exact --nocapture`
- `cargo test -p zircon_runtime --locked asset::tests::assets::model::model_asset_toml_roundtrip_preserves_virtual_geometry_payload -- --exact --nocapture`
- `cargo test -p zircon_runtime --locked prepare_frame_exposes_same_frame_visbuffer_marks_from_unified_draw_truth --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked prepare_frame_exposes_cluster_selection_from_unified_draw_truth --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked debug_snapshot_prefers_prepare_owned_same_frame_visbuffer_marks_when_available --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked same_frame_visbuffer_marks_ --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked shared_indirect_args_layout_preserves_instance_index_in_submission_details --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_shared_indirect_segments_preserve_instance_index_for_submission_fallback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_segments_survive_without_shared_segment_and_draw_ref_buffers --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_segments_survive_with_execution_indices_and_gpu_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution_indices_and_gpu_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_submission_records_survive_with_execution_authority_buffer_only --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_execution_records_recover_draw_ref_indices_when_execution_index_buffer_is_gone --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked build_context_accepts_frame_owned_virtual_geometry_cluster_selections_without_prepare --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked rebuild_selected_clusters_from_execution_segments_drops_visibility_only_superset --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_renderer_mesh_draw_submission_order_tracks_visibility_owned_unified_indirect_authority --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked visbuffer64_pass_words_follow_executed_submission_keys_and_deduplicate_clusters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b hardware_rasterization_pass_records_follow_executed_submission_keys_and_preserve_startup_parameters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b executed_cluster_selection_pass_filters_deduplicates_and_sorts_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b visbuffer64_pass_words_follow_shared_executed_cluster_selection_order --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b hardware_rasterization_pass_records_follow_shared_executed_cluster_selection_order_and_preserve_startup_parameters --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_buffer_exists_without_snapshot_or_gpu_readback --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-codex-b virtual_geometry_visbuffer64_clear_only_source_exists_without_cluster_selections --lib -- --nocapture`
- `cargo test -p zircon_runtime --locked virtual_geometry_gpu_readback_exposes_execution_backed_visbuffer64_entries --lib -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_extract_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_execution_snapshot_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --test virtual_geometry_visbuffer_overlay_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_visibility_debug_contract -- --nocapture`
- `cargo test -p zircon_runtime --test virtual_geometry_stats_contract -- --nocapture`
- `cargo test -p zircon_runtime --locked --lib render_framework_ -- --nocapture`
- `cargo check -p zircon_runtime --locked --lib`
- `cargo test -p zircon_runtime --locked --lib scene_prepare_card_capture_resource_snapshot -- --nocapture`
- `cargo test -p zircon_runtime --locked --lib virtual_geometry_nanite_ -- --nocapture`
- `cargo test --workspace --locked --target-dir F:\cargo-targets\zircon-codex-a`

The latest `VisBuffer64` provenance follow-up also locks the zero-selection clear path: when the compat VG pass executes but emits no cluster writes, the renderer now preserves `RenderPathClearOnly`, keeps the published clear value, and leaves the packed-word stream empty instead of collapsing that frame to `Unavailable`.

The next N3 follow-up now also locks the first explicit `HardwareRasterizationPass` contract: the renderer-owned snapshot publishes execution-backed startup records for each rasterized cluster, and those records are sourced from a dedicated compat-side pass seam rather than being rebuilt ad hoc from later host inspection helpers. Because `E:\Git\ZirconEngine\target` ran out of space during this continuation, the focused validation for this step was moved to `F:\cargo-targets\zircon-codex-b`.

The latest continuation extends that same seam through explicit provenance plus a real GPU buffer boundary: `RenderVirtualGeometryHardwareRasterizationSource` now keeps the renderer-owned/public-snapshot contract on `Unavailable`, `RenderPathClearOnly`, or `RenderPathExecutionSelections`, while the compat pass itself constructs the startup buffer and returns `source + record_count + buffer` as one pass output. Even when there is no renderer-owned snapshot or uploader readback, the renderer still retains that hardware-rasterization startup parameter buffer and can decode it back into typed records, while the clear-only path remains observable as `RenderPathClearOnly` with an empty startup stream.

The latest continuation also closes the remaining public-stats gap on those two seams. `RenderStats` now mirrors both render-path sources plus both buffer/record counts, and `virtual_geometry_stats_contract` locks them against the renderer-owned snapshot when a real VG extract is present. On the opposite edge, a follow-up non-VG submission still resets those stats to `Unavailable` and `0`, so host tooling can distinguish “no effective VG workload” from “VG workload existed but the compat pass only cleared state this frame.”

As supporting validation work, this continuation also needed a support-only Hybrid GI readback compile repair in `gpu_readback/pending_readback/collect.rs` so the focused framework-level VG snapshot query could reach the intended red/green loop again. That change does not alter VG ownership boundaries or expand this slice into Hybrid GI feature work.

## Next Expected Layers

The next Nanite plan steps should build on this foundation instead of redefining it:

- a real VG cook pipeline that fills `virtual_geometry`
- scene extraction that sources live VG instances from cooked assets
- runtime-page integration between the CPU reference output and the existing VG residency host
- `VisBuffer64`, `NodeAndClusterCull`, and hardware raster passes
- automatic SSE-driven LOD and multi-pass hierarchy culling
