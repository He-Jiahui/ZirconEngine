---
date: 2026-08-10
related_plan: docs/plans/zircon_runtime/render/18/2026-08-10-hgi-m5-sdf-architecture-and-performance-plan.md
doc_type: structural-performance-research
status: implementation_in_progress
coordination_owner: docs/plans/zircon_runtime/render/17
---

# HGI M5 Global SDF Structural Performance Research

## Status

This is a proposed implementation plan, not an accepted performance result. The current-source M5 plan requires coordinator-managed Windows WGPU timing, a product PNG, and a RenderDoc capture. Those runs are not available in this session, so no timing, power, or percentage improvement is claimed here.

## Current Source Progress

- Implemented: named CPU phase telemetry for object collection, Mesh SDF state sync, Global SDF residency, influence-index update, and candidate packing. The total remains a saturating sum and the RenderStats reset path clears every phase. The projection path now also emits mutually exclusive cache-hit and rebuild diagnostics, so coordinator evidence can count actual rebuilds instead of inferring them from a zero sync duration. Influence telemetry now includes materializable contributor entries, typed clipmap fallback count, and retained candidate-bucket capacity bytes; contributor counts exclude pages in a clipmap-level voxel fallback because those cached buckets cannot become sampleable Global SDF pages.
- Implemented: a folder-backed stable-key Global SDF influence index, bounded per-page candidate ownership, candidate-overflow fallback, clipmap-level oversized-object fallback, and indexed GPU packing.
- Static implementation review: prepared model geometry resolves the selected referenced MeshAsset primitives before caching local bounds and revisions. Active morph targets use conservative signed delta bounds and remain a typed voxel fallback; skinning remains a typed fallback whose bounds span the active clipmaps. Regressions cover positive and negative morph weights, external Mesh dependency revision invalidation, and skinning expansion.
- Static implementation review: the production probe-trace dispatch serializes the selected capability route into the intersection-backend mask, Global SDF lighting-source axis, and typed fallback reason. Its bound 13-word diagnostics record then carries actual intersection/lighting provenance, distance, confidence, fallback reason, and texture/page/SDF/voxel cost counters through GPU readback into runtime feedback; this is source-contract evidence only, not a measured trace result.
- Implemented: trace lookup now projects sampleable Global SDF pages into a fixed four-clipmap `8^3` atlas-slot table. The shader checks the fixed clipmap count rather than walking every resident page per sphere-trace step; dirty, uninitialized, generation-mismatched, and typed-fallback pages remain unavailable sentinels. CPU regressions cover clipmap-origin projection, uninitialized-page exclusion, and typed fallback exclusion, while the trace dispatch test guards the 192-byte Rust/WGSL uniform contract.
- Second source review: completed for the M5 path. It found that contributor telemetry could count retained page buckets after a clipmap-level typed voxel fallback, although none of those pages can materialize; the metric now excludes those buckets and a focused regression covers a normal contributor alongside an oversized fallback object in the same clipmap.
- Static verification: focused Rust formatting, scoped diff checks, and source contracts confirm the indexed packing path has no remaining page-by-all-object candidate loop, no-request frames bypass packing entirely, and all phase metrics project to runtime diagnostics.
- Pending coordinator evidence: current-source Cargo, Windows WGPU readback, product PNG under `docs/tests/runtime/render/`, GPU timestamps, RenderDoc capture, and measured comparison on a fixed adapter and scene.

## Trace Lookup Audit (2026-08-11)

### Source Evidence

The indexed build path removed the former CPU page-by-all-object scan, but the trace path still has an independent algorithmic scaling defect. `GlobalSdfGpuState::create_trace_bindings` serializes every sampleable page into a compact buffer. `sample_global_sdf` in `trace_probe_tiles.wgsl` then linearly walks that complete list for every sphere-trace step to find the smallest-cell-size containing page. For `R` probe rays, `S` bounded march steps, and `P` sampleable pages, this is `O(R * S * P)` storage reads and AABB tests. The 16-step limit does not make `P` constant.

The local `TraceVoxels.hlsl` replica binds a Global Distance Field page table and page-object grid, and UE's `GlobalDistanceField.cpp` owns a persistent per-page object-grid buffer before composing its page atlas. Zircon should adopt the page-table lookup principle now, but not copy UE's larger GPU cull/compact/indirect pipeline into the MVP before measured need exists.

### Chosen Structural Repair

Replace the trace-only compact page list with a persistent, fixed-domain sparse page table:

```text
four clipmaps x 8 x 8 x 8 page coordinates
    -> 2,048 u32 atlas-slot entries (u32::MAX means unavailable)
    + four clipmap descriptors (global page-coordinate origin, page-world size)
    -> trace lookup checks exactly four clipmaps, chooses the finest valid entry
```

The CPU projection writes only `sampleable_pages`: dirty, generation-mismatched, terminal-fallback, or typed clipmap-fallback pages retain the unavailable sentinel. WGSL derives the absolute page coordinate from world position, converts it through each descriptor's origin, bounds-checks the fixed `8^3` local coordinate, then reads the atlas slot directly. This changes the lookup component to `O(R * S * C)` with fixed `C = 4`, preserves the existing completion/generation contract, and keeps capability routing/fallback selection outside the page-table owner.

The build dispatch's per-update upload buffers are a separate allocation-churn concern. They stay out of this repair: first record the new bounded lookup count, actual upload bytes, GPU timestamps, and allocator events on one fixed scene. A persistent upload-ring decision requires that evidence and must not be hidden in the page-table correctness change.

### Validation Contract

- Rust projection tests cover page-coordinate origin, dirty/fallback exclusion, and sentinel handling; the WGSL source contract covers the fixed clipmap loop and finest-cell selection.
- WGSL/source contracts prohibit a loop over `global_sdf_page_count` in `sample_global_sdf`; the only lookup loop is bounded by the fixed clipmap count.
- Coordinator-managed Windows WGPU evidence reports per-probe lookup count, SDF steps, GPU timestamps, output pixels, and actual allocation/upload behavior. No current performance percentage, power conclusion, or PNG is claimed before that run.

## Residual Stable-Frame Projection

The Global SDF page/object relationship is now indexed, but the runtime-prepare collector still cloned `scene.meshes`, re-created its material capture map, re-sampled material textures, and rebuilt every `HybridGiMeshSdfObject` before discovering that the Mesh SDF scene state had not changed. This is a separate F3-style stable-frame cost from the former page-by-all-object packing scan.

The current `RenderMeshSnapshot` carries the fields needed for a conservative first cache gate: stable instance key, transform revision and transform, model/mesh/material identities, morph weights, common flags, and `RenderMeshStaticState`. The latter is authoritative only when the transform is static and both geometry and material revisions are nonzero. Global SDF clipmap centers are page-aligned, so exact clipmap bounds remain stable for camera motion inside the current page.

The framework does **not** currently expose a scene-level geometry generation or changed/removed Mesh SDF record stream. Consequently, a cache keyed only by mesh count, resource identity, or a world handle would be unsafe: it could hide transforms, material reloads, visibility changes, or removal. The first implementation must therefore be intentionally narrow:

```text
all render meshes have authoritative static revisions
    AND exact RenderMeshSnapshot sequence is unchanged
    AND page-aligned Global SDF clipmap bounds are unchanged
        -> perform an O(N), allocation-free snapshot comparison and reuse existing
           HybridGiMeshSdfSceneState objects
        -> skip scene clone, ResourceStreamer geometry lookup,
           material capture, center texture sampling, object allocation, sort, and scene sync
otherwise
        -> rebuild the complete projection and refresh the cache snapshot
```

Dynamic, skinned, morphing, pending, or revision-zero meshes always take the rebuild path. This protects Mesh SDF deformation fallbacks and hot-reload correctness. The cache owns only its comparison snapshot; `HybridGiMeshSdfSceneState` remains the sole owner of projected objects and dirty-region computation.

The cache must also be the source of the short-lived execution snapshot passed to `execute_prepare`. The encoder currently needs scene meshes and Mesh SDF world bounds only for synchronous prepare/voxel-debug work; copying either into a second execution-input `Vec` on a cache hit creates avoidable stable-frame allocations. Store immutable `Arc<[RenderMeshSnapshot]>` and `Arc<[(stable_key, RenderMeshBounds)]>` snapshots in the comparison cache, refresh both only after a successful projection rebuild, and pass cloned `Arc`s through the execution-input DTO. This shares data without extending it past the synchronous encoder call, without creating a second mutable scene owner, and without weakening the dynamic rebuild gate.

Every encoded GPU update must have a completion observation. Apply the shared readback-ring admission check before radiance-cache prepare dispatch as well as before Global SDF dispatch. When the ring is full, defer the radiance-cache prepare rather than encoding an untracked update; once admitted, propagate enqueue failure instead of dropping it. The bootstrap revision advances only for work that can be observed, so a readback failure remains a forward bootstrap retry rather than an unbounded stream of invisible work.

The next architectural layer belongs to the render-extract owner rather than Hybrid GI: add a monotonic scene-geometry revision plus explicit changed/removed render-mesh records, then update the existing Mesh SDF scene state incrementally. That change is deliberately not folded into M5 before a measured need exists, because it changes a cross-runtime extraction contract.

### Projection Measurement Contract

The existing `cpu_mesh_object_collection_time_us` phase must include material capture and texture sampling, not merely object construction. Coordinator evidence must distinguish cached and rebuilt static frames, record the number of projection rebuilds, and compare CPU preparation, GPU timestamps, output pixels, fallback counters, and memory allocation behavior on a fixed scene. No performance result is accepted without those artifacts.

## Evidence And Problem Statement

The current Global SDF path has the intended independent owners, but not the intended influence algorithm:

1. `HybridGiGlobalSdfSceneState` owns clipmap residency, dirty generations, and sampleability in `scene_representation/global_sdf_scene_state/`.
2. `pack_global_sdf_build_inputs` in `renderer/gpu_resources/global_sdf/packing.rs` iterates every dirty page and then every Mesh SDF object to rebuild its candidate list.
3. A candidate list is capped at 32; overflow already becomes a terminal voxel fallback rather than a partial Global SDF page.
4. Current frame statistics publish total Global SDF CPU preparation time, page counts, upload bytes, and candidate overflow, but do not separate residency, influence construction, packing, and dispatch preparation time.

The inner scan is `O(P_dirty * N_objects)`. It conflicts with the M5 performance gate: `O(N * C)` influence construction with a small fixed `C`, followed by bounded per-page object lists. It can also rebuild the same object/page relation in each dirty frame, adding exactly the avoidable every-frame work called out by F3 in `docs/plans/engine-code-review-findings-2026-06.md`.

## Reference Comparison

Unreal's `GlobalDistanceField.cpp` uses a page object grid (`PageObjectGridBuffer`) rather than asking every page to scan every scene distance-field object. Its grid factor is four cells per page edge, so the page build consumes pre-bucketed object indices. `LumenMeshSDFCulling.cpp` similarly performs bounded culling, compaction, and indirect dispatch instead of an unbounded shader-side scene scan. The local Lumen compute-shader replica keeps the same separation: page tables and page/object-grid inputs are distinct from Global SDF tracing.

Zircon should preserve the principle, but not prematurely copy Unreal's full GPU culling pipeline. The current WGPU path has a single Global SDF build pass and no reusable scan/compact/indirect-object-grid infrastructure. The lowest-risk MVP is a deterministic CPU broad phase with strict bounds; the later GPU object-grid path is a separate optimization milestone only after timing proves it necessary.

## Chosen MVP Design

Add a folder-backed `global_sdf_scene_state/influence.rs` owner. It maintains a deterministic, bounded relation between stable Mesh SDF instance keys and resident Global SDF pages:

```text
Mesh SDF object deltas
    -> GlobalSdfInfluenceIndex (stable key, revision, conservative bounds)
    -> page-ready contributor list | page-terminal fallback marker
    -> dirty page build requests
    -> GPU packing maps stable keys to current object-table indices once
```

The index owns only broad-phase membership and page readiness. `packing.rs` remains the GPU ABI owner and retains validation of payload count, voxel-byte budget, and candidate-buffer layout. The tracer remains the capability/fallback owner.

### Membership Contract

- A page list contains sorted unique stable instance keys, never transient object-vector indices.
- A Mesh SDF object revision or bounds change removes its old memberships before inserting its new memberships.
- Page range calculation uses the same expanded `page_influence_bounds` contract as invalidation, including adjacent-page influence. One owner supplies this conversion so packing and dirty tracking cannot diverge at cell boundaries.
- Ready objects contribute to a page only when their full Mesh SDF payload is budgetable. Missing, invalid, active-morph, unbounded-skinning, and packing-overflow objects remain explicit fallback contributors; they must never produce an empty-but-sampleable Global SDF page.
- Each page keeps at most the existing 32 candidate keys. The 33rd key records deterministic overflow and makes the page unavailable to Global SDF tracing until the voxel fallback path services it.
- A normal object has a private fixed `GLOBAL_SDF_MAX_PAGES_PER_OBJECT` membership budget. An object exceeding it promotes the affected clipmap to a typed voxel-fallback state rather than expanding an unbounded page list. This preserves conservative tracing and gives diagnostics a visible reason for the quality reduction.

### Complexity And Allocation Contract

| Operation | Current | Proposed MVP |
| --- | --- | --- |
| Stable scene object preparation | `O(N)` clone/resource lookup/material capture/object build | `O(N)` allocation-free snapshot comparison while the narrow authoritative-static cache gate holds; otherwise `O(N)` full projection |
| Candidate construction | `O(P_dirty * N)` | `O(N * C)` when the Mesh SDF snapshot or residency changes; stable frames reuse the index |
| Dirty invalidation | dirty-region scan over residents | direct membership invalidation; bounded clipmap fallback only for exceptional oversized objects |
| GPU build work | at most 32 candidates per page | unchanged, but reads pre-bucketed candidates |
| Per-frame allocations | page-local temporary candidate vectors | no allocation in no-request steady frames; resident index buckets retain their vector capacity across rebuilds, and each actual dispatch builds one stable-key-to-object-index map |

`C` is an explicit fixed page-membership limit, not a scene-size-dependent value. The allocation contract forbids cloning the full scene or making a full object copy solely to build Global SDF candidates. It directly avoids the review finding F3 pattern.

## Measurement Plan

Before accepting the algorithm change, add phase counters to `RenderHybridGiGlobalSdfStats`:

- object synchronization microseconds;
- page residency and dirty projection microseconds;
- influence-index update microseconds;
- GPU-packing microseconds;
- index object count, contributor count, overflow-page count, clipmap-fallback count, and reusable-capacity bytes.

The coordinator must capture a fixed scene and adapter in cold and warm conditions. It records the counters above, per-pass GPU timestamps, transient/persistent resource bytes, frame time, WGPU completion readback, product pixels, and a RenderDoc capture. The comparison is valid only when candidate overflow and fallback behavior are equivalent or intentionally reported. GPU time and upload-bandwidth reduction may be used as energy proxies; no platform power claim is made without power telemetry.

## Implementation Order

1. Add the phase metrics and unit contracts without changing the current candidate result.
2. Add `influence.rs`, page-key range helpers, deterministic stable-key buckets, and change/removal tests.
3. Replace the nested packing scan with indexed candidate lookup plus one stable-key-to-current-index pass.
4. Add typed clipmap fallback for over-budget objects and regressions for missing/invalid/deforming/overflow contributors, adjacent-page influence, camera page turnover, and deterministic ordering.
5. Add the narrow authoritative-static projection cache. It must not cache dynamic or revision-zero inputs, must include page-aligned clipmap bounds, and must move material capture inside the timed rebuild branch.
6. Use the new phase counters and cached-vs-rebuilt evidence to decide whether a render-extract changed/removed-record API is justified; do not add an HGI-local delta owner without that shared contract.
7. Run static formatting and source-contract checks in this session. The coordinator then performs current-source WGPU PNG/readback, timestamps, and RenderDoc validation.
8. Conduct a second independent review before the coordinator prepares the integration candidate.

## Acceptance Gates

- No Global SDF page is marked sampleable without a completed, generation-matched GPU build.
- The source contains no page-by-all-object candidate scan in the normal packing path.
- The 32-candidate limit, upload limits, readback backpressure, and typed voxel fallbacks remain bounded and observable.
- Dynamic results and the actual image artifact are written only by the coordinator under `docs/tests/runtime/render/`; no static or historical output is substituted.
