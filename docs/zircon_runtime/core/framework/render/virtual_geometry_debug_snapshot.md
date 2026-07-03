---
related_code:
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/encoding.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/sources.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/tests.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/extract_output.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/test_sources/virtual_geometry_imported_extract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/virtual_geometry_asset_payload_decode.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/page_payload.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/sources.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/extract_output.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_virtual_geometry_debug_snapshot.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/page_payload.rs
  - zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/nanite/automatic_extract.rs
  - zircon_plugins/virtual_geometry/runtime/src/provider.rs
plan_sources:
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/assets-and-rendering/virtual-geometry-nanite-foundation.md
tests:
  - rustfmt --edition 2021 --check on runtime VG extract sidecar/context/snapshot builder, virtual_geometry plugin nanite page payload/automatic extract/provider/imported extract test, and runtime_15_virtual_geometry_asset_payload_decode_is_wired (2026-06-29 VirtualGeometry asset payload decode: static pass; Cargo deferred)
  - cargo check -q -p zircon_plugin_virtual_geometry_runtime --lib --target-dir target\codex-plan08-vg-asset-payload-decode-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry asset payload decode: timed out after about 304s; no check result, not counted as passed)
  - rustfmt --edition 2021 --check on virtual_geometry_debug_snapshot payload DTO/re-export, production VG debug snapshot builder, GPUScene virtual-geometry ABI, mesh build resident upload owner, and runtime_15_virtual_geometry_cluster_payload_upload_is_wired (2026-06-29 VirtualGeometry cluster payload upload: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir target\codex-plan08-product-material-pass-cache-0629 --locked --jobs 1 (2026-06-29 VirtualGeometry cluster payload upload: timed out after about 304s; no check result, not counted as passed)
  - E:\cargo-targets\zircon-plan08-three-shading-model-parity-0701\debug\deps\zircon_runtime-fe15dbfd02d9864e.exe graphics::scene::scene_renderer::mesh::build_mesh_draws::build::virtual_geometry_resident_upload::tests::virtual_geometry_cluster_words_follow_resident_page_payloads --exact --test-threads=1 --nocapture (2026-07-02 VirtualGeometry cluster payload upload direct-binary WGPU backfill: passed 1/1, 5881 filtered)
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget.rs
doc_type: module-detail
---

# Virtual Geometry Debug Snapshot

`virtual_geometry_debug_snapshot.rs` is now the structural facade for the public Virtual Geometry debug snapshot DTO family. It declares child modules and re-exports the same `RenderVirtualGeometry*` names through `zircon_runtime::core::framework::render`, so existing callers keep the stable framework path while ownership is split below the facade.

The split keeps each contract family in a folder-backed owner:

- `bvh_visualization.rs` owns host-facing BVH visualization nodes and instances.
- `cpu_reference.rs` owns cooked CPU-reference traversal, page-cluster maps, mip/depth maps, and selected-cluster DTOs.
- `cull_input.rs` owns `RenderVirtualGeometryCullInputSnapshot`, debug flag packing, and cluster-selection provenance packing.
- `execution.rs` owns submission order, execution segments, hardware rasterization records, VisBuffer marks, VisBuffer64 entries, and page inspection DTOs.
- `node_and_cluster_cull.rs` owns the future NodeAndClusterCull GPU word-layout contracts: global state, instance seeds, instance/cluster/child work items, traversal records, dispatch setup, and launch worklist snapshots.
- `page_payload.rs` owns the render-side resident page payload DTOs consumed by mesh-build VirtualGeometry cluster word upload. The first-party `virtual_geometry` runtime plugin now decodes cooked asset page payloads into this DTO family before frame submission snapshots are built.
- `snapshot.rs` owns the top-level `RenderVirtualGeometryDebugSnapshot` aggregation object. It composes the child owner DTOs instead of redeclaring them.
- `sources.rs` owns the lightweight provenance enums used by snapshot fields.

`encoding.rs` keeps the shared `u32::MAX` optional-value decoder local to this family. It is intentionally private so other render DTO families do not grow a hidden dependency on the Virtual Geometry debug snapshot packing policy.

## Public Surface

The public API remains the `core::framework::render` re-export set. Callers should continue to import `RenderVirtualGeometryDebugSnapshot`, `RenderVirtualGeometryCullInputSnapshot`, `RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot`, and related DTOs from the render framework namespace. The child modules are implementation owners, not a new public import surface.

## Runtime 07 Owner Gate

This split removes the former 1495-line mixed debug snapshot file from the large-file hotspot set. The Runtime 07 owner-budget guard records the current large-file evidence as `large_file_hotspot_count = 38`, `runtime-framework-render = 3`, `runtime-other = 14`, and `large_file_unclassified_hotspot_count = 0`.

The status anchor for this slice is `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`. Static validation owns the folder split and public-surface guard; package-level Virtual Geometry integration and Runtime 07 extract/ecs_query/profiling/FPS Cargo gates remain deferred under the implementation-first cadence.

## 2026-07-03 Product Page/Cluster Readback PNG

Plan 08 VirtualGeometry page/cluster product execution now has a focused product readback PNG export under `render_plan08_virtual_geometry_page_cluster_product_readback_png_passed_renderdoc_deferred`. The ignored test `export_virtual_geometry_page_cluster_product_png` stays in `graphics/tests/render_product_mesh_cache/virtual_geometry.rs` and reuses the same automatic cooked `ModelAsset` VG path, visible Unlit material, page/cluster binding requirements, and public WGPU execution stats as `render_product_virtual_geometry_page_cluster_bindings_drive_visible_frame`.

The export wrote `docs/tests/runtime/render/runtime_render_plan08_virtual_geometry_page_cluster_product_20260703.png` through the captured product frame path. Direct generated-binary validation passed 1/1 with 6204 filtered and 7.97s; the PNG is 320x240, 2965 bytes, SHA256 `0322783567544681379085E0C944EF40DD2E6453EE4AE0CB5897F12EBBEBDDE6`. The same binary passed `runtime_15_virtual_geometry_product_draw_source_is_wired` 1/1 with 6204 filtered and 0.26s. RenderDoc/product capture, workspace/full CI, full live registry export, and broader product miss=0 remain open.

Status tables use `virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,page_payload,snapshot}.rs` as the compact owner anchor for the split.

## Plan 08 Payload Upload

`RenderVirtualGeometryPagePayload` and `RenderVirtualGeometryPagePayloadVertex` are now part of the debug snapshot family so render submission can hand resident page vertex payloads to mesh build without putting that contract into shader descriptors or GPUScene. `RenderVirtualGeometryDebugSnapshot.resident_page_payloads` is the handoff field; the production debug snapshot builder receives it from `FrameSubmissionContext::virtual_geometry_resident_page_payloads()` after the `virtual_geometry` runtime plugin decodes cooked `ZVG0` page payloads from `ModelPrimitiveAsset` source vertices and indices. The mesh-build resident upload owner converts available payload vertices into four cluster words per vertex: position, normal, tangent, and pad.

Status: `render_plan08_virtual_geometry_cluster_payload_upload_direct_binary_wgpu_passed_renderdoc_deferred`. The original static status remains recorded as `render_plan08_virtual_geometry_cluster_payload_upload_static_passed_cargo_deferred` for older generated guard binaries. The structure guard `runtime_15_virtual_geometry_cluster_payload_upload_is_wired` locks the DTO owner, re-export, production sidecar handoff, GPUScene words-per-vertex ABI, resident upload projection, docs/status anchors, and line budgets. The 2026-07-02 direct-binary WGPU backfill passed `virtual_geometry_cluster_words_follow_resident_page_payloads` 1/1 from the generated no-default `zircon_runtime` lib-test binary; Cargo-wrapper rerun, RenderDoc/product capture, default features, workspace/full CI, full live registry export, and broader product miss=0 remain open.

Status: `render_plan08_virtual_geometry_asset_payload_decode_static_passed_cargo_deferred`. The structure guard `runtime_15_virtual_geometry_asset_payload_decode_is_wired` locks the plugin decode owner, runtime extract output, frame submission context, production debug snapshot handoff, imported cooked model assertion, docs/status anchors, and file budgets. Scoped rustfmt and static anchors passed; `zircon_plugin_virtual_geometry_runtime --lib` check timed out after about 304 seconds and is not counted as passed. The dependent meshlet vertex ordinal seam now has direct-binary asset/shader evidence under `render_plan08_virtual_geometry_meshlet_vertex_ordinal_direct_binary_asset_shader_passed_renderdoc_deferred`, with the original `render_plan08_virtual_geometry_meshlet_vertex_ordinal_static_passed_cargo_deferred` retained as a historical guard anchor. Follow-up guard status `render_plan08_virtual_geometry_project_asset_manager_fixture_source_guarded_cargo_rerun_deferred` locks the ProjectAssetManager `asset_manager_imports_model_toml_with_virtual_geometry_payload` expected fixture source and records that stale binary `zircon_runtime-770562bad16f99eb.exe` from `2026-07-02 05:27:25 +08:00` still fails with the old all-zero expected payload. Remaining gates are fresh ProjectAssetManager fixture Cargo rerun, product VG draw, RenderDoc/product capture, full live registry export, and product miss=0.
