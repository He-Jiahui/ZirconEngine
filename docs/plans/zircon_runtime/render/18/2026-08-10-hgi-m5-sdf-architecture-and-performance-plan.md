---
related_code:
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer
plan_sources:
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenMeshSDFCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenTracingUtils.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GlobalDistanceField.cpp
  - dev/LumenInUE5.5.4WithComputeShader/Res/Shader/ScreenProbeGather/TraceVoxels.hlsl
tests:
  - focused Rust unit and source-contract tests
  - coordinator-managed Windows WGPU readback and product PNG
  - coordinator-managed RenderDoc DX12 capture
doc_type: implementation-plan
status: source_implementation_complete_validation_pending
---

# HGI M5 SDF architecture and performance plan

## Decision

M5 is implemented as three independent owners: imported Mesh SDF assets, a persistent Global SDF clipmap, and a capability-graph trace scheduler. Existing HGI voxel and Radiance Cache clipmaps remain fallback and lighting-cache owners; neither is reused as Global SDF storage. Hardware RT remains an optional Render Plan 20 candidate and is not an MVP dependency.

The first implementation slice is correctness infrastructure, not a performance claim. Dynamic GPU timing, resource-byte totals, WGPU product pixels, and RenderDoc evidence require a coordinator-managed current-source run and remain pending. No historical screenshot or timing is accepted as evidence for this slice.

## Current-source findings

1. `RenderMeshBounds` stores imported local bounds, but runtime-prepare collectors cannot query the already prepared mesh/model bounds. HGI Card and voxel occupancy currently approximate every object with `max(abs(scale)) * 0.5`, ignoring imported size, local center, rotation, and non-uniform scale.
2. `RuntimePrepareMaterialCaptureSeed` exposes emissive data but drops the already resolved `cast_shadows` value. M5 flags therefore cannot combine instance and material shadow policy without reaching through renderer-private state.
3. HGI has no Mesh SDF asset, object table, Global SDF page table, or software-SDF trace source. The current `voxel_scene_state` is a coarse 4x4x4 fallback representation and must not be renamed into Global SDF.
4. `voxel_clipmap_debug.rs` repeats mesh/cell overlap traversal for occupancy, radiance, and dominant-object products. Its static work is proportional to the sum of overlapped cells for every traversal and currently starts from inaccurate bounds. This is a profiler target, not yet a measured bottleneck.
5. `scene_representation/representation.rs` is already 819 lines. M5 behavior must use folder-backed owners instead of extending that root.

## Reference mapping

| Zircon owner | Unreal/Lumen reference | Adopted contract |
| --- | --- | --- |
| Mesh SDF object table | `FDistanceFieldSceneData` and `LumenMeshSDFCulling.cpp` | Stable object identity, conservative world bounds, explicit flags, and bounded object-to-grid influence lists |
| Mesh SDF asset | distance-field atlas/object buffers | Versioned imported/cooked payload; runtime upload only, never synchronous first-frame baking |
| Global SDF | `GlobalDistanceField.cpp` and Lumen tracing parameters | Camera-relative clipmaps, page-aligned snapping, dirty regions, page residency, deterministic budgets |
| Trace scheduler | `FLumenMeshSDFGridParameters` and screen-probe tracing | Screen and world domains are separate from intersection backend and lighting source; misses remain typed fallbacks |
| Lightweight fallback | local compute-shader replica `TraceVoxels.hlsl` | Keep voxel fallback available on core WebGPU when Mesh/Global SDF or RT is unavailable |

## Ownership and data flow

```text
asset import/cook
  -> MeshAsset / ModelPrimitiveAsset versioned Mesh SDF payload
  -> ResourceStreamer prepared geometry + SDF upload
  -> runtime-prepare Mesh SDF object table (bounds + flags + asset revision)
  -> Global SDF dirty-page scheduler and object influence lists
  -> software world trace
  -> capability graph selects lighting source
  -> existing resolve/composite source ledger
```

Mesh flags are resolved once per runtime-prepare frame from authoritative inputs:

- `visible`: renderer enabled and not `ShadowsOnly`;
- `movable`: scene mobility is dynamic;
- `casts_shadow`: renderer shadow mode and resolved material `cast_shadows` both allow it;
- `emissive`: finite positive emissive constant or an emissive texture is present;
- `indirect_while_hidden`: enabled `ShadowsOnly` geometry that still casts shadows.

## Milestones

1. M5-S1: add exact transformed bounds, cached prepared-resource bounds, material shadow capture, Mesh SDF object flags, deterministic ordering, and conservative clipmap influence culling.
2. M5-S2: add a schema-versioned Mesh SDF payload and importer cook request; validate dimensions, voxel count, finite distance range, source hash, and byte budget. Runtime consumes only a ready payload and records a typed missing/invalid fallback.
3. M5-S3: add independent Global SDF clipmap/page state with page-aligned camera snapping, dirty region projection, deterministic residency/eviction, streaming invalidation, GPU page resources, and initialized-page generation.
4. M5-S4: add the three-axis capability graph: trace domain, intersection backend, and lighting source. Every result records source, distance, confidence, fallback reason, and cost counters.
5. M5-S5: register Render Plan 20 RT selection as an optional world-intersection backend without owning acceleration structures or platform policy.

## Performance gates

The algorithmic targets are explicit before tuning:

- object preparation: `O(N)` over scene mesh instances, with local bounds cached per prepared resource revision;
- influence build: `O(N * C)` for the MVP clipmap count, with fixed small `C`, then bounded per-page object lists;
- dirty updates: proportional to changed object/page intersections, not total clipmap voxels;
- tracing: bounded steps and bounded candidate objects per cell/page; no unbounded all-object loop per ray;
- readback: diagnostic-only and capped by the shared frame ring.

## 2026-08-11 trace-work boundedness review

Static review found that the current probe-trace shader treats the scene card/voxel seed list as a
global trace-tile list and scans the complete list for every resident or completed probe. Each
voxel fallback tile then scans the bounded voxel-cell descriptor array. The resulting upper bound
is `O(P * T * D)` for probe count `P`, scene seed count `T`, and voxel descriptor count `D`; this
violates the milestone contract even before runtime profiling can rank it as a measured bottleneck.

The local Lumen reference does not use scene objects as an unbounded outer tile axis.
`UpdateRadianceCache/GenerateProbeTraceTiles.hlsl` owns tiles by `ProbeTraceIndex` and emits a
fixed `TraceTileResolution * TraceTileResolution` count for that probe; the replicated default
resolution is 4, or 16 tiles. Zircon therefore keeps its existing lightweight scene seeds but
selects at most 16 deterministic seeds per probe in the shader. This makes trace work
`O(P * min(T, 16) * D)` without changing resource ownership, backend order, ray-quality mapping,
or the diagnostic ABI. The cap is an acceptance-bound repair derived from the reference layout,
not a measured tuning result.

Coordinator profiling must still compare the old and bounded paths on the same current-source
scene and adapter. Until GPU timestamps, output pixels, fallback counters, PNG, and RDC agree, this
record makes no frame-time or power claim and does not justify further workgroup, ray-count, or
quality tuning.

Coordinator evidence must record pre/post CPU preparation time, per-pass GPU timestamps, object/page counts, dirty/uploaded pages, candidate-list truncation, resource bytes, frame time, and PNG pixel deltas on the same scene and adapter. Improvements are reported only after those measurements; energy impact is expressed as GPU-time and bandwidth reduction unless platform power telemetry is available. No percentage or power claim is inferred from the reference engines.

## Acceptance status

- Architecture review: complete for M5-S1 through M5-S5 ownership and data flow.
- Source implementation: complete for M5-S1 through M5-S5. It includes exact prepared-geometry
  bounds/revisions, versioned Mesh SDF cook and artifact invalidation, generation-tagged Global
  SDF pages, typed trace provenance/fallbacks, bounded probe trace seed work, and Render Plan 20
  capability routing without RT resource ownership.
- Structure and static review: complete. Production and test owners in the current HGI/resolve
  scope are folder-backed and below the 800-line convention threshold; two independent static
  reviews found no remaining Critical or Important issue after forward repairs.
- Current-plan failure: the neutral scene-prepare sideband has its source-level architectural
  repair and remains `open` in
  `failure-2026-08-11-hgi-m5-scene-prepare-neutral-sideband.md` until the managed runtime gate
  supplies current-source evidence.
- Runtime acceptance: pending coordinator-managed Windows WGPU readback, product PNG under
  `docs/tests/runtime/render`, RenderDoc capture, and performance measurements. No source-only
  review is acceptance evidence.
- Focused static contracts: complete for the current source; managed Rust/WGPU execution remains
  pending.
- WGPU PNG/readback, stats snapshot, RenderDoc `.rdc`, and quantified performance comparison: pending coordinator execution.
- Accepted milestone closeout: open.
