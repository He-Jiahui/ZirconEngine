# Render 02 Mesh Pass Command Arena Architecture And Performance Plan

## Status

- Research and current-source review: complete on 2026-08-26.
- Baseline profiling: pending; no frame-time, allocation, power, or scale claim is accepted.
- Production ABI migration: not started. The current validation lane is unavailable, so this
  report is deliberately completed before changing the central command-buffer ownership model.
- Source binding: repository HEAD `8e56165c4c789416c328898d3d8937d934b52efa` with a shared dirty
  worktree. All paths below describe the reviewed working tree, not a clean-HEAD reconstruction.

## Decision

The next structural optimization is a hard cut from producer-local command lists plus ten owned
phase vectors to two distinct lifetimes:

1. `MeshPassCommandBuildArena`: one append-only, generation-owned command vector shared by
   pre-MeshDraw extraction and residual command generation.
2. `MeshPassCommandBuffers`: a sealed read-only command vector plus fixed phase/bucket ranges.

The arena is sealed once, after the renderer knows whether the half-resolution transparency pass
exists. Sealing classifies commands, sorts once, derives contiguous ranges, and exposes borrowed
slices to replay, indirect planning, stats, and overlays. Producers must not construct finalized
phase buffers independently and merge them later.

This is not a request to wrap the current ten `Vec` fields in an array. That would retain the
allocation, partition, move, merge, and repeated-sort costs while only changing field syntax.

## Current Source Findings

The reviewed path is:

`PendingMeshDraw` -> optional pre-MeshDraw extraction -> generic `MeshDrawCommandList` ->
`MeshPassCommandBuffers::from_command_list` -> ten phase vectors -> residual generic list -> ten
more phase vectors -> `MeshPassCommandBuffers::extend` -> per-phase append and sort -> optional
half-resolution append and sort -> indirect planning/replay.

Static source facts:

| Cost center | Current behavior | Bound per invocation |
|---|---|---|
| Producer arena | each uncached, cached, parallel, and pre-Mesh path first appends into a generic `Vec<MeshDrawCommand>` | one growable allocation per non-empty producer |
| Phase partition | `from_command_list` declares ten output vectors and moves every command into one of them | `O(N)` command moves and up to `B` active-bucket allocations, `B <= 10` |
| Initial sorting | every phase list calls `sort_by_key`, including empty lists | ten sort calls per finalized producer |
| Prebuilt/residual merge | `extend` appends all ten source lists and re-sorts all ten destinations | `O(N_residual)` additional moves plus ten sort calls |
| Half-resolution fallback | moves the half-resolution list into transparent and sorts transparent again | one additional append/sort call when the graph has no dedicated pass |
| Stable full-hit frame | prebuilt commands are partitioned/sorted, an empty residual buffer is finalized, then all phase lists are re-sorted during merge | 30 sort call sites are reached before optional half-resolution fallback; active prebuilt commands are sorted twice |

The table describes call and ownership structure, not elapsed cost. Empty/single-element sorts can
be cheap, and a `Vec::new()` does not allocate until populated. WPR allocation stacks and measured
CPU samples are therefore mandatory before claiming a bottleneck has disappeared.

## Unreal 5.5 Reference Boundary

The primary reference is the local Unreal source, not an inferred engine pattern:

- `Engine/Source/Runtime/Renderer/Public/MeshPassProcessor.h:1283` restricts
  `FMeshDrawCommand` to draw-submission data and states that cached command resources must outlive
  the command.
- The same file at line 1737 uses `FDynamicMeshDrawCommandStorage` backed by `TChunkedArray` so
  command addresses do not move while visible commands hold pointers.
- `FVisibleMeshDrawCommand` at line 1748 stores the current visibility/sort payload separately from
  the submission command; `FMeshCommandOneFrameArray` at line 1839 is the one-frame visible list.
- `Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h:112` gives each
  `FParallelMeshDrawCommandPass` a pass-owned setup lifecycle and exposes command data only after
  setup completes.
- `Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp:1015` generates dynamic commands,
  applies view overrides, updates view-dependent sort keys, sorts once for that pass, then seals
  instance-culling inputs.

Zircon should preserve the same lifetime split but use Rust-safe generation ownership. The sealed
range must not borrow a cache through an unconstrained lifetime, and a raw WGPU address must never
become cache identity.

## Target ABI

```rust
struct MeshPassCommandBuildArena {
    commands: Vec<MeshDrawCommand>,
    cache_stats: MeshDrawCommandCacheStats,
}

struct MeshPassCommandRanges {
    ranges: [Range<usize>; MESH_PASS_COMMAND_BUCKET_COUNT],
}

struct MeshPassCommandBuffers {
    commands: Vec<MeshDrawCommand>,
    ranges: MeshPassCommandRanges,
    cache_stats: MeshDrawCommandCacheStats,
}
```

The exact spelling can follow the owner module, but these contracts are fixed:

- build arena is append-only and cannot expose replay slices;
- sealed buffers cannot append, repartition, or merge;
- phase accessors return slices into the one sealed command vector;
- phase classification has a fixed ten-bucket enum independent from the broader `RenderPhase`;
- half-resolution fallback classification is decided before sealing, so no post-seal merge exists;
- cache statistics accumulate in the build arena and cross the seal without recomputation;
- `MeshDrawCommandList: Clone` and `MeshPassCommandBuffers: Clone` must be removed unless a measured
  owner requires full command duplication.

The ten buckets are depth prepass, shadow, opaque, alpha mask, advanced-PBR opaque, transmission,
transparent, half-resolution transparent, velocity, and TAA reactive mask.

## Construction Algorithm

1. Create one build arena in compiled-scene render preparation.
2. Let pre-MeshDraw extraction append projected cache hits and safe rebuilt commands directly.
   Record the starting length for each draw. If a later phase requires residual fallback, truncate
   to the checkpoint; commit deferred cache stores only after all selected phases succeed.
3. Pass the same arena to residual serial or parallel command generation. Parallel workers may own
   bounded per-chunk scratch vectors, but deterministic source-order merge writes once into the
   generation arena; per-draw vectors remain forbidden.
4. Retain the persistent static cache generation after all producers finish.
5. Map half-resolution material commands to the dedicated bucket only when that graph pass exists;
   otherwise classify them directly as normal transparent commands.
6. Sort the single vector by `(bucket, sort_key, pipeline_variant_id)`. Preserve the current stable
   ordering until a test proves the key is a total deterministic order; only then evaluate an
   allocation-free unstable sort.
7. Scan once to derive the ten contiguous ranges and seal the buffer.
8. Build indirect plans, stats, replay lists, and overlay streams from sealed borrowed slices.

This MVP algorithm uses one vector and one final sort. A two-pass exact-placement algorithm is not
the first implementation: it would require command fan-out planning before materialization and
could double processor work. Revisit it only if profiling shows the one final sort dominates.

## Correctness Invariants

- Each emitted command appears in exactly one bucket and each bucket range is in bounds.
- Concatenating all ranges covers the sealed vector exactly, with no gaps or overlap.
- Existing opaque/material/transparent classifier behavior remains byte-for-byte equivalent.
- Transparent depth order and source-index tie breaking remain deterministic.
- A failed pre-MeshDraw extraction leaves no commands or cache stores from that draw in the arena.
- Cache hits project current source index, sort key, GPU Scene span, and direct draw arguments.
- Prebuilt and residual commands share one sort domain, so state bucketing cannot depend on which
  producer created a command.
- Half-resolution fallback produces the same visible transparent set without a second merge.
- Replay, indirect planning, diagnostics, and overlay consumers receive slices valid through GPU
  command recording.

## Baseline Before Implementation

Available local tools are `C:\Windows\System32\wpr.exe`,
`D:\Windows Kits\10\Windows Performance Toolkit\xperf.exe`,
`D:\Windows Kits\10\Windows Performance Toolkit\wpaexporter.exe`, and
`D:\Tools\renderdoc\renderdoccmd.exe`. Tool installation location is not an artifact location.
All ETL, exported CSV, RenderDoc captures, logs, and screenshots must be written under
`E:\zircon-profiles\render02-command-arena-20260826` or
`E:\Git\ZirconEngine\docs\tests\runtime\render`; no artifact may be written to `C:`.

Required matched scenarios after the runnable renderer lane is restored:

| Scenario | Scale | Purpose |
|---|---:|---|
| static full hit | 1k, 10k, 100k visible instances | expose repeated partition/sort/allocation work |
| 1% static dirty | 10k and 100k | verify work scales with changed entries plus visible projection |
| dynamic material diverse | 1k and 10k | exercise processor fan-out and all active buckets |
| mixed prebuilt/residual | 50/50 at 10k | expose current double finalization and merge cost |
| camera-only change and unchanged camera | 10k and 100k | separate current-view sort from reusable generation work |

Protocol: 30 warm-up frames followed by 120 settled frames, at least five runs per scenario, fixed
resolution/present mode/power profile, and p50/p95/p99 reporting. Capture CPU sampling, heap
allocation stacks, context switches/ready time, GPU timestamps, VRAM, and energy when the platform
provider exposes it. RenderDoc validates pass/event/resource/submission behavior and final pixels;
it does not prove CPU allocation or power improvement.

Add product counters before the migration so old and new paths report the same schema:

- command arena grow count and peak capacity;
- command build, partition-move, merge-move, finalize, and sort counts;
- commands visited by partition, merge, and sort;
- active bucket count and each sealed range length;
- cache-hit payload `Arc` clone count;
- command-generation and view-generation reuse count;
- CPU prepare time split by extract, processor build, seal/sort, indirect plan, and replay record.

## Acceptance Gates

- Current old-path baseline and new-path profile use identical source scenes and capture protocol.
- One sealed arena and one phase-range table own all compiled mesh commands for the view family.
- Build-to-phase partition moves are zero; prebuilt/residual merge moves and post-seal merges are
  zero; finalize count is exactly one when the artifact changes.
- Unchanged scene plus unchanged view reuses the sealed artifact with command rebuild, partition,
  merge, and sort counts all zero. Camera-only changes perform only view-owned projection/sort work.
- 1% dirty work is near-linear in changed/visible entries rather than total scene entries.
- CPU p50/p95/p99, allocation bytes/count, context-switch pressure, GPU time, and energy do not
  regress outside measurement noise; any regression requires attribution before acceptance.
- Focused/unit/property tests, headless WGPU product tests, genuine PNG comparison, and RenderDoc
  capture all pass. Text-only, synthetic image, or source-marker evidence cannot close the gate.

## Implementation Order After Baseline

1. Add counters and collect the old-path WPR/xperf/RenderDoc baseline.
2. Introduce build-arena and sealed-range types with range/classifier property tests.
3. Hard-cut processors, cache extraction, and serial/parallel builders to append into one arena.
4. Move half-resolution fallback policy before seal; delete post-build phase-list merge.
5. Hard-cut indirect, stats, graph replay, and overlay consumers to sealed range slices.
6. Remove the ten-vector representation, `from_command_list`, list `extend`, and clone surfaces.
7. Re-run the matched profile and visual protocol; only then update accepted milestone status.
8. Separately profile the payload `Arc` reference-count cost. A stable generational payload handle
   is allowed only if it beats `Arc` while preserving eviction and GPU recording lifetime safety.

## Explicitly Pending

- No production arena migration is implemented by this report.
- No WPR/xperf/RenderDoc run, genuine PNG, GPU timestamp, power sample, or scale benchmark has been
  collected because the runnable validation/artifact lane is currently unavailable.
- No claim is made that the fixed three-slot pre-MeshDraw staging or the payload split improves
  frame time until the matched baseline is complete.
