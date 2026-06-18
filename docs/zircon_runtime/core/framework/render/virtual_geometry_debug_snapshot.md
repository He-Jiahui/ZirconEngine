---
related_code:
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/encoding.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/sources.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/tests.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/bvh_visualization.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cpu_reference.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/cull_input.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/execution.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/node_and_cluster_cull.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/snapshot.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/sources.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/assets-and-rendering/virtual-geometry-nanite-foundation.md
tests:
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
- `snapshot.rs` owns the top-level `RenderVirtualGeometryDebugSnapshot` aggregation object. It composes the child owner DTOs instead of redeclaring them.
- `sources.rs` owns the lightweight provenance enums used by snapshot fields.

`encoding.rs` keeps the shared `u32::MAX` optional-value decoder local to this family. It is intentionally private so other render DTO families do not grow a hidden dependency on the Virtual Geometry debug snapshot packing policy.

## Public Surface

The public API remains the `core::framework::render` re-export set. Callers should continue to import `RenderVirtualGeometryDebugSnapshot`, `RenderVirtualGeometryCullInputSnapshot`, `RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot`, and related DTOs from the render framework namespace. The child modules are implementation owners, not a new public import surface.

## Runtime 07 Owner Gate

This split removes the former 1495-line mixed debug snapshot file from the large-file hotspot set. The Runtime 07 owner-budget guard records the current large-file evidence as `large_file_hotspot_count = 38`, `runtime-framework-render = 3`, `runtime-other = 14`, and `large_file_unclassified_hotspot_count = 0`.

The status anchor for this slice is `virtual_geometry_debug_snapshot_owner_split_static_passed_cargo_deferred`. Static validation owns the folder split and public-surface guard; package-level Virtual Geometry integration and Runtime 07 extract/ecs_query/profiling/FPS Cargo gates remain deferred under the implementation-first cadence.

Status tables use `virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs` as the compact owner anchor for the split.
