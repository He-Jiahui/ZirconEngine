---
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/pass_authoring.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/update.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-render-pipeline-and-mesh-draw.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphAllocator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphResourcePool.cpp
tests:
  - 16 of 16 current render_graph Rust files read, 5243 logical lines and 49 inline tests
  - path plus per-file-sha256 manifest fingerprint 3136462b97704d5a84bcf391e64717739aa0ac897229965948eee6898aacecde
  - 7 of 49 tests are source-shape assertions rather than behavioral or complexity gates
  - current managed Cargo, WPR/xperf, GPU timestamps and RenderDoc remain blocked by the non-runnable product baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Render graph current architecture review (2026-08-15)

## Scope and source freeze

This review covers every Rust file under `zircon_runtime/src/render_graph` and follows the product
call path through pipeline authoring, the compiled-graph cache, frame submission statistics and the
physical transient pool. The 16-file source freeze contains 5,243 logical lines and 49 tests. Its
path plus per-file SHA-256 manifest fingerprint is
`3136462b97704d5a84bcf391e64717739aa0ac897229965948eee6898aacecde`.

| File | Lines | Tests | Static review state |
|---|---:|---:|---|
| `builder.rs` | 430 | 0 | read; handle validation and authoring cost checked |
| `builder/compile.rs` | 682 | 0 | read; dependency, WAW, culling and lifetime algorithms checked |
| `dump.rs` | 439 | 1 | read; capture-only materialization checked |
| `error.rs` | 117 | 0 | read; failure contracts checked |
| `graph.rs` | 703 | 0 | read; compiled indexes, stats and transient plan checked |
| `mod.rs` | 37 | 0 | read; public surface checked |
| `store_lint.rs` | 252 | 2 | read; scan complexity and product call frequency checked |
| `tests/builder_validation.rs` | 23 | 2 | read; one source-shape false-green identified |
| `tests/compute_dispatch.rs` | 574 | 7 | read; dispatch validation coverage checked |
| `tests/culling.rs` | 202 | 11 | read; five source-shape tests identified |
| `tests/cycles.rs` | 13 | 1 | read; cycle behavior checked |
| `tests/mod.rs` | 6 | 0 | read |
| `tests/ordering.rs` | 178 | 5 | read; 129-pass correctness case has no work budget |
| `tests/resources.rs` | 746 | 15 | read; lifetime and declaration behavior checked |
| `tests/resources/transient_aliasing.rs` | 298 | 5 | read; one source-shape test and bucket semantics checked |
| `types.rs` | 543 | 0 | read; identity, descriptor and compiled DTO contracts checked |

These files are not accepted. Several are foreign-dirty, current Cargo has not executed the 49
tests, and no current product exists for WPR, GPU timestamp or RenderDoc evidence. They remain in
`pending.md`; `review.md` is intentionally unchanged.

## Corrections to earlier assumptions

The current source already contains useful work that the hard cut must retain:

- Main submission does not unconditionally compile every frame. `CompiledGraphCache` retains 16
  `Arc<CompiledRenderPipeline>` entries and reports hits, misses and evictions.
- `CompiledRenderGraphStats` is materialized once in `CompiledRenderGraph::new`; steady diagnostics
  read the value instead of rescanning all passes.
- Logical handle bounds checks are constant time, dependency lists are deterministic, pass ordering
  detects cycles, and exact-descriptor interval coloring does reuse non-overlapping logical slots.
- The downstream `TransientResourcePool` reuses exact texture/buffer descriptors across frames,
  applies 256 MiB/64 MiB budgets, evicts stale entries after eight frames and reports create/reuse,
  retained bytes and evictions.
- Dump construction is on an explicit capture path, not an unconditional frame path.

Those repairs do not solve the ownership and dependency model below.

## P0 architecture findings

### 1. The product graph is deliberately serialized before RDG sees it

`pass_authoring.rs:108-153` records `previous -> pass` for every authored pass across every stage.
Consequently an N-pass product graph has at least N-1 manual edges and exactly one ready pass at each
topological layer. A pass may retain `AsyncCompute` or `AsyncCopy` as a label, but the total chain
prevents useful queue overlap. This is the direct structural cause behind the compiled-scene
parallel recording limitations already tracked by PERF-MVP-622.

Before creating that total chain, `order_unique_resource_producers_before_readers`
(`pass_authoring.rs:207-270`) scans all passes to count writers and scans them again to find readers
for each produced resource. Its source bound is `O(P^2 * A_pass)` and it clones the full pass vector.
Post-process ordering then uses hard-coded executor-name sets (`pass_authoring.rs:272-319`). The
result is duplicated topology authority: feature code partially orders by resource, special cases
order by string identity, and the graph compiler finally orders again.

### 2. Unversioned resources force blanket WAW dependencies and prevent correct dead-write culling

`RgTextureHandle` and `RgBufferHandle` are bare builder-local `usize` values
(`types.rs:16-38`). They carry neither graph generation nor resource version. A same-kind handle from
another builder with an in-range index can therefore pass the O(1) bounds check and refer to a
different resource. More importantly, every write continues to name the same logical resource.

`compile.rs:341-380` requires every adjacent writer pair to be connected through the full manual
transitive closure; only RAW edges are inferred from the latest writer. Culling at
`compile.rs:495-548` tracks one unversioned `needed_resources` set, never consumes a satisfied write,
and preserves manual dependencies. An earlier clear/write overwritten by a later clear/write cannot
be removed independently because the WAW ordering edge itself keeps it live. This is not a local set
operation bug: correct culling needs producer-version edges and load/store semantics before WAW/WAR
ordering, culling and barriers are derived.

Compilation also converts dense handles back into owned string DTOs. It builds name maps repeatedly,
clones pass/resource/executor/compute metadata, clones declarations into lifetimes, then reconstructs
access indexes by matching names in `CompiledRenderGraph::new`. Manual WAW reachability allocates
`P * ceil(P/64) * 8` bytes and merges one word vector per dependency. At P=1,024 the matrix alone is
128 KiB; at P=4,096 it is 2 MiB. The current total chain performs 262,080 word merges at P=4,096,
before string/hash/descriptor work. These are source-model counts, not measured frame time.

### 3. A cache miss compiles the full graph while holding framework state

`compile_pipeline.rs:51-64` acquires `framework.lock_state()` and calls
`get_or_compile_with_status`; its miss closure runs pipeline compile and capability validation before
the lock is released. Shader/pipeline generation changes, feature-presence changes, resize and
dynamic resolution can therefore put full graph construction on the submission caller inside the
global framework lock.

The key in `compiled_graph_cache.rs:43-91` includes exact view and render width/height in addition to
topology-affecting state. A resize sequence or fluctuating resolution scale creates separate entries
and can churn the fixed 16-entry LRU even when pass topology is unchanged. Increasing capacity would
only retain more cloned compiled DTOs. The architectural fix is to separate stable pipeline schema
generation from frame extents/resource materialization, compile outside the state lock, and publish
an immutable generation atomically.

### 4. Logical aliasing and physical pooling are restricted to exact descriptor buckets

`graph.rs:417-610` groups lifetimes by the full texture descriptor or exact buffer size/usage before
interval coloring. Slot numbering restarts per bucket; `slot_for(name)` omits the bucket, so two
physically incompatible resources can both report slot zero. `dense_*_bytes_reserved` correctly sums
the separate bucket reservations, meaning the test that observes both resources at slot zero does
not demonstrate shared physical memory.

The downstream pool is real, but it repeats the same exact width/height/format/usage or
size/usage key. It has no fence-bearing compatibility class, oversized-buffer reuse or backend
transient-memory alias authority. Unreal's behavior does not justify blindly copying a page size or
retention count; it does justify one resource authority that chooses compatibility/alignment from
backend capability and records requested bytes, committed bytes, alias savings, create/reuse and
fences. Zircon currently has a compile-time slot plan and a separate exact-key frame pool.

### 5. Store lint is a quadratic allocating diagnostic on every submitted frame

`store_lint.rs:55-105` visits each attachment and calls helpers that rescan all prior or future
passes and their resource strings. It clones pass/resource names for every finding. The worst-case
source bound is `O(P^2 * A_pass)` per report. `update_stats/update.rs:46-53` constructs the complete
report and discards everything except `count()` on every submitted frame, including compiled-cache
hits.

With one attachment access per pass, the upper source model is P*(P-1) pass-pair scans: 65,280 at
P=256 per frame (3.92 million/s at 60 Hz) and 1,047,552 at P=1,024 per frame (62.85 million/s), before
inner access scans and string comparisons. The count belongs in compiled diagnostics, while owned
rows belong behind explicit capture/export demand. Caching the current report alone would be a
small stopgap, but it would preserve the string DTO and dual diagnostic authority; no source change
is accepted before the hard-cut owner is fixed.

### 6. Tests protect container shape but do not prove the intended architecture

Seven of 49 tests read Rust source text and assert implementation tokens. They explicitly lock in a
bitset closure, HashMap/BTreeSet choices and cached-field syntax. Behavioral tests cover validation,
cycles, culling and exact-bucket slot reuse, but none measures compile allocation/work, cache churn,
lock hold, producer-version culling, cross-graph handle rejection, async ready width, physical alias
savings or per-frame lint work. The 129-pass ordering case checks correctness only.

## Unreal source basis

The target uses these local source behaviors, not Unreal C++ type names:

- `RenderGraphBuilder.cpp:593-637` gives each builder root/transient allocators and configures
  parallel setup/execute. `RenderGraphAllocator.cpp:71-94` destroys registered objects in reverse and
  flushes one frame allocator in bulk.
- `RenderGraphBuilder.cpp:1341-1437` sets dependencies/reference counts, starts from output and
  never-cull roots, performs DFS reachability and compiles operations only for live passes. It does
  not require the author to total-order every pass.
- `RenderGraphBuilder.cpp:1846-2025` reserves exact registry sizes, schedules descriptor/resource
  setup above a resource threshold, compiles barriers, collects allocations/deallocations and
  overlaps pooled/transient allocation and view creation through setup tasks.
- `RenderGraphResourcePool.cpp:31-54,92-151` aligns buffers to power-of-two or 64 KiB page classes to
  improve reuse, checks transient fences and records last-use frame. Lines 9-18 and 183-225 expose
  pool create/release/count/bytes counters and retire unused entries after a bounded frame window.

Unreal still expresses and compiles a frame RDG; the lesson is not "cache one owned graph forever".
The reusable authorities are stable registries/schema, linear frame allocation, task setup, pooled
physical resources and fence-aware transient allocation. Zircon should retain a cheap frame graph
instance when frame-dependent passes exist while eliminating repeated owned strings, global-lock
compile and duplicate resource owners.

## Required hard-cut architecture

| Artifact | Owner and contents | Forbidden parallel truth |
|---|---|---|
| `RenderPipelineSchemaGeneration` | Render01/02; interned pass/resource/executor slots, queue constraints, capability/shader generation and topology rules | hard-coded executor ordering, per-frame owned strings and extent-only topology keys |
| `FrameRenderGraphInstance` | Render01; generation-tagged handles, versioned writes, roots and frame extents allocated from a reusable linear arena | cross-graph bare indexes, one total manual chain and heap allocation per DTO field |
| `CompiledFrameGraphPacket` | Render01/17; live producer versions, queue schedule, barriers, lifetimes and immutable execution packets | compile under framework lock, unversioned needed-resource culling and executor name lookup |
| `TransientResourceGeneration` | Render01/RHI; compatibility/alignment class, physical pool/transient allocation, fences, requested/committed/alias bytes and budget | separate exact-key plan/pool truths and bucket-less slot identity |
| `CompiledGraphDiagnostics` | Render17; compile-time lint/counts plus explicitly materialized capture rows | quadratic lint/string allocation in every frame statistics update |

Implementation order is mandatory:

1. Define generation-tagged logical handles and versioned producer edges. Compile RAW, WAR and WAW
   from resource versions and attachment load/store; remove the product-wide `previous -> pass`
   chain and executor-name topology special cases in the same hard cut.
2. Freeze `RenderPipelineSchemaGeneration`. Keep view/resource extents in the frame instance unless
   they actually change pass topology. Build schema/compiled packets outside the framework lock and
   publish by generation check; stable schema hits do no string/hash/descriptor reconstruction.
3. Move frame-local pass/access/version nodes to a reusable linear arena and use Runtime11 setup
   tasks only above a measured threshold. The result must remain deterministic with one worker.
4. Merge graph lifetime output and the physical transient pool into one RHI-facing generation with
   compatibility classes, fences and budget counters. Keep an exact-descriptor fallback where the
   backend cannot alias safely.
5. Materialize lint counts during compile and owned rows only for explicit capture. Replace the
   seven source-shape tests with behavior, complexity-counter and allocation gates; delete old graph
   DTO/cache paths with their consumers, without aliases or forwarding wrappers.

PERF-MVP-633 is the cross-owner acceptance task. Render01 owns graph/schema/resource versions,
Render02 owns pass/mesh declaration integration, Runtime11 owns bounded setup tasks, and Render17
owns counters plus WPR/GPU/RenderDoc evidence.

## Complexity and dynamic acceptance

Run 1/32/256/1,024 passes, 1/8/64 accesses per pass, 1/2/8 queue lanes/views, 0/1/10/100% culled,
720p/1080p/1440p/4K, stable/resize/dynamic-resolution/shader-reload and pool pressure at
0.5x/1x/2x budget. Record schema builds, frame instance/finalize work, ready width, edges/version
visits, string/hash/clone/allocation bytes, state-lock wait/hold, setup tasks/worker occupancy,
requested/committed/aliased/pool bytes, create/reuse/evict/fence waits, lint scans and CPU/GPU
p50/p95/p99/RSS/context switches/energy.

Hard gates are: stable schema build and owned string reconstruction = 0; compile work under framework
state lock = 0; independent async-capable passes produce ready width greater than one and measured
overlap; a dead overwritten producer is culled; cross-generation handles are rejected; frame
finalization is near `O(P+A+E)` without a full transitive matrix; stable pool create=0 after warmup;
resize rematerializes affected extents once without rebuilding unchanged topology; per-frame lint
scan/allocation=0. Pixels, pass order, barriers, load/store, history and device-loss behavior must
remain correct.

The current product build is blocked by the Render06 camera resolution-scale contract drift, so no
valid WPR/xperf, GPU timestamp, RenderDoc or power sample exists for this source freeze. This review
therefore makes no measured latency or power claim, does not edit Rust, does not enter `review.md`,
and is not a commit or WeCom milestone.
