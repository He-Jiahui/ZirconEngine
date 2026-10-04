---
title: Virtual Geometry Product Topology Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/virtual_geometry/editor/src
  - zircon_plugins/virtual_geometry/runtime/src
  - zircon_plugins/virtual_geometry/dist/src
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/17-first-party-virtual-geometry-source-runtime-editor-dist-catalog-asset-cook-cluster-page-streaming-culling-raster-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/93-runtime-mesh-geometry-section-lod-instancing-skinning-morph-deformation-bounds-collision-streaming-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/ClusterDAG.cpp
  - dev/UnrealEngine/Engine/Source/Developer/NaniteBuilder/Private/Encode/NaniteEncodePageAssignment.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Rendering/NaniteStreamingManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Nanite/NaniteStreamingPageUploader.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Nanite/NaniteCullRaster.cpp
---

# Virtual Geometry Product Topology Current-Source Algorithm Performance Review

## 1. Coverage and current product truth

At repository revision `79f64878f3b9526517644c055ad3bf5cadfccd0f`, the complete plugin contains **265 Rust files / 44,656 physical lines / 41,643 non-empty lines / 1,641,840 bytes / 295 tests / 33 ignored tests**. Its ordered `workspace-relative path + NUL + raw bytes + NUL` SHA-256 is `6a04da8a30e9a93b78f8c4e90e4128e9988b7d40452b78e11f67bc8883220396`.

The non-validation set is **200 files / 15,835 physical lines / 14,497 non-empty lines / 578,665 bytes / 48 inline tests / 3 ignored tests**, with fingerprint `629052491ad2a22cb33a07e7b4060956b3c61223e53da765ca0a388a5c84d9a0`. The complementary validation set is **65 files / 28,821 physical lines / 27,146 non-empty lines / 1,063,175 bytes / 247 tests / 30 ignored tests**, with fingerprint `981677bdce5a2557902dacd5331e278df0e6c912c987222cca54b8f63518d2f4`. Both sets are statically reviewed; this still does not update the protected review ledger because execution and product capture remain pending.

The current worktree contains extensive concurrent virtual-geometry edits and file splits. They were treated as current source and preserved. No source change was made by this review.

## 2. Structural performance findings

### P0: the registered product passes validate contracts but execute no rendering work

`runtime/src/render_pass_executors.rs:155-183` routes prepare, async-compute cull, async-copy feedback, visbuffer and debug-overlay executors only to `validate_context`; `:185-229` compares metadata and returns `Ok(())`. None records a compute pass, render pass, draw, dispatch, copy or timestamp. Queue labels therefore do not create multi-threaded or asynchronous execution.

The only production collector, `runtime/src/virtual_geometry/renderer/root_output_sources/virtual_geometry_plugin_renderer_outputs.rs:44-50`, registers a buffer and returns default renderer outputs. `:53-77` creates that buffer from page request IDs already present in CPU-side prepared readback state. This is not GPU visibility feedback.

### P0: the internal GPU renderer is test-only and its named raster products are CPU record buffers

Whole-plugin call-site analysis found `VirtualGeometryGpuResources::new` and `VirtualGeometryRenderFrame::from_extract` only under node/cluster cull tests. `build_virtual_geometry_cluster_raster_draws` has no caller and passes a literal empty selection slice at `root_mesh_sources/build_virtual_geometry_cluster_raster_draws.rs:16-17`.

Even if manually wired, `root_render_passes/virtual_geometry_hardware_rasterization_pass/execute.rs:15-28` only converts CPU selection records to packed words and creates a storage buffer. It does not rasterize. `virtual_geometry_visbuffer64_pass/execute.rs:15-29` likewise encodes one `u64` per CPU-selected cluster into a linear storage buffer; it does not write per-pixel visibility/depth.

### P0: node/cluster culling is a main-thread CPU simulation followed by fresh uploads

`root_render_passes/virtual_geometry_node_and_cluster_cull_pass/execute.rs:41-149` builds global state, seeds, instance and cluster work items, clones hierarchy children, runs up to eight traversal waves, performs child decisions and produces page requests on CPU. `:150-214` then creates separate buffers for nearly every intermediate/output vector.

The hot loop repeatedly scans authored arrays: child decisions test page residency with a linear `pages.iter().any` (`child_decision.rs:191-209`); resident-parent fallback linearly searches all clusters and pages for each ancestor (`:313-353`); hierarchy lookup is another linear scan (`:302-311`). A `BTreeSet` is used for per-frame cluster and page deduplication. This produces scale closer to **O(visited clusters * (pages + ancestor depth * (clusters + pages)))** than a GPU-driven bounded work queue.

### P0: runtime extraction rebuilds and duplicates heavy debug/reference products

`runtime/src/provider.rs:26-42` loads model values, builds automatic extract, then clones the complete extract, CPU-reference instances, BVH visualization instances and resident payloads into a neutral output. The runtime state re-registers the complete extract every prepare (`:52-74`) and rebuilds tree-backed snapshots after render (`:76-127`).

`nanite/automatic_extract.rs:182` invokes model loading per mesh. It always constructs CPU-reference instances at `:254`, while only BVH visualization has an explicit debug append gate. `nanite/page_payload.rs:41-128` decodes page triangle ranges against the original model vertex/index arrays and expands indexed triangles into new vertex streams. The cooked asset is therefore not a self-contained compressed GPU cluster/page artifact; runtime retains and traverses the original mesh and several overlapping DTO projections.

### P1: residency and preparation rebuild full ordered state instead of consuming deltas

`prepare_frame/pending_page_requests.rs:27-36` sorts requests with a key whose later ranking path calls `page_descendant_ids` repeatedly (`:138-145`). Each descendant call allocates/traverses hierarchy state. Slot choice and eviction ranking scan multiple tree collections per request. Completion normalizes and reconciles a full page table through several maps, sets and vectors; reverse slot lookup is still a page-map scan.

Local preallocation and indexed-ancestor edits in the active worktree reduce isolated allocation counts, but do not change the dominant topology: full extract registration, CPU traversal, full snapshot projection, fresh GPU buffers and full readbacks remain frame work.

## 3. Unreal source constraints

- `ClusterDAG.cpp:116-206` partitions mesh adjacency into bounded clusters and uses `ParallelFor`; `:781-837` partitions cluster groups, while `:1069-1236` reduces the DAG and parallelizes group reduction. Runtime extraction is not the place to rebuild this hierarchy.
- `NaniteEncodePageAssignment.cpp:91-115` admits a cluster only while actual encoded GPU bytes and maximum cluster count fit; `:121-233` assigns bounded root/streaming pages and page ranges. A page-count budget multiplied by a nominal 4 KiB does not provide equivalent byte accounting.
- `NaniteStreamingManager.cpp:2536-2609`, `:3085-3089` and `:3305-3342` split streaming into begin/async/end phases, dispatch task-graph work and hand ready data to the uploader. The dependency/fixup state is not reconstructed as debug DTOs every frame.
- `NaniteStreamingPageUploader.cpp:137-192` owns reusable upload storage and appends actual page bytes plus dependency metadata; `:203-239` reuses/grows pooled buffers and orders installs; `:309-361` dispatches independent and parent-dependent GPU transcode, optionally on async compute.
- `NaniteCullRaster.cpp:804-1025` defines real instance and node/cluster cull compute shaders; `:4486-4662` records node/cluster and indirect cluster-cull passes; `:4670-4860` performs instance culling and indirect dispatch; `:1517-2228` defines software micropolygon and hardware vertex/mesh/pixel raster shaders.

Zircon should copy these ownership and scheduling properties, not Unreal-specific APIs or feature breadth.

## 4. Dependency-ordered optimization plan

### M0: make capability truthful and delete dead success paths

Keep virtual geometry unavailable to production until registered executors record real work. A pass that only validates metadata must not advertise cull, feedback, visbuffer or overlay output. Remove or quarantine test-only renderer trees that cannot be reached from the product entry.

### M1: create a self-contained cooked virtual-geometry artifact

Move cluster construction, hierarchy/DAG reduction, fallback selection, page packing, dependency/fixup generation and compressed GPU payload encoding to import/cook. Key the artifact by source digest, cook version, target/backend and build settings. Runtime must not require original model vertices/indices to decode resident pages.

### M2: establish byte-budgeted streaming ownership

Add a resource-authority-owned streaming manager with stable page/slot arrays, reverse slot lookup, byte budgets, root-page pinning, dependency counters, request priority, bounded IO/decompression jobs, cancellation and generation receipts. Consume deltas; do not re-register/reconcile all pages each frame.

### M3: wire a persistent GPU page pool and uploader

Allocate page table, cluster data, work queues, indirect arguments and feedback rings per device generation. Batch actual page bytes into reusable staging storage, apply dependency/fixup order, dispatch proportional workgroups and read back only compact delayed feedback/counters. No warm-frame buffer creation or full page-table readback.

### M4: replace CPU simulation with GPU-driven cull and real raster

Implement bounded instance -> node -> cluster work queues in the real render graph, with indirect dispatch/draw arguments and overflow telemetry. Feed resident cluster payloads into hardware and/or software raster paths that write depth and a per-pixel visibility target. Keep CPU traversal only as an offline/debug oracle.

### M5: make debug/editor products opt-in and generation-based

CPU reference, BVH visualization, selected-cluster lists, traversal traces and readbacks must be disabled by default and sampled into bounded generation snapshots when requested. Editor authoring must use real asset handles, cook status, residency counters and capture controls rather than cloned runtime DTOs.

### M6: qualify scale, latency and power

After a current-source executable reaches the pass, capture cold/warm scenes across cluster/page/instance scale. Report import/cook time, artifact bytes/ratio, main/render/worker CPU, allocations/bytes, upload/evict bytes, page misses, visible clusters, dispatch/draw counts, GPU pass timestamps, barriers, readback bytes, frame p50/p95/p99, RSS/VRAM, wakeups and package/GPU power. RenderDoc pixel/draw/resource inspection follows correctness; WPR/ETW supplies CPU scheduling and power evidence.

## 5. Acceptance gates

1. Registered executors record real graph work and fail closed when the required artifact/backend is unavailable.
2. Warm frames perform zero model reload, hierarchy rebuild, CPU-reference build, full extract clone, pipeline creation and per-intermediate GPU buffer allocation.
3. Streaming budgets use actual compressed/resident/upload bytes, with deterministic dependency-safe install and eviction.
4. GPU culling work is bounded, indirect and overflow-instrumented; CPU work scales with submissions and compact feedback, not total authored clusters/pages.
5. Visbuffer and depth are real per-pixel products validated against a CPU oracle and ordinary-mesh fallback.
6. Dynamic measurements come from a launchable current-source product executable and include correctness, latency distributions, memory and power.

## 6. Validation status

- Non-validation static review: complete for the captured 200-file fingerprint.
- Dedicated validation/test-source review: complete for the captured 65-file fingerprint; see `2026-08-24-virtual-geometry-validation-current-source-coverage-review.md`.
- Full Rust-source static review: **265/265 complete** for the captured composite fingerprint; product acceptance remains pending.
- Direct source optimization: deferred because concurrent edits own the same files and the next safe change is a product-wide cook/stream/render boundary, not a local loop tweak.
- Cargo execution is pending because the managed Windows validation session is unavailable. WPR/WPAExporter are installed, but WPR/ETW, GPU timestamps and power remain pending because no launchable current-source product executable exists. RenderDoc CLI is unavailable.
- No bottleneck is declared removed, no performance parity is claimed, and no milestone commit or WeCom completion message is warranted.
