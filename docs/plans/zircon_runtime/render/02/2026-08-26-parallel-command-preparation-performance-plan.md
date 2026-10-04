# Render-02 Parallel Command Preparation Performance Plan

Date: 2026-08-26

## Status

- Current algorithm and Unreal 5.5 ownership review: complete.
- Parallel preparation observability and report coverage: implemented; static and synthetic tests pass.
- Product CPU/Heap baseline: pending; no allocation, frame-time, scale, power, or speedup claim is accepted.
- Arena/ordered-input optimization: not started; the product allocation/call-tree gate remains mandatory.

## Scope and Current Algorithm

This review covers the folder-backed mesh command builder, specifically
`builder/parallel_preparation.rs` and
`build_mesh_pass_command_buffers_from_batches_cached_parallel`.

The current algorithm has four ordered stages:

1. Collect batches and normalize by `source_draw_index`.
2. Resolve pipeline variants and cache hits/misses on the owner thread.
3. Build immutable command chunks on the shared `TaskPool` workers.
4. Merge chunks in source order, then commit cache stores on the owner thread.

This preserves the Render-17 contract: no private rendering thread pool, no
mutex-based owner mutation in workers, and serial/parallel output equivalence.
The focused regression test covers input presented in reverse source order.

The parallel setup responsibility is now physically owned by
`builder/parallel_preparation.rs`, while `builder/parallel_admission.rs` owns only
admission policy and `builder.rs` retains public entries, serial orchestration,
and shared cache operations. This follows Unreal's separation between
`FParallelMeshDrawCommandPass` setup ownership and the surrounding mesh command
entry surface without pretending that Zircon already has Unreal's deferred
setup ticket. The hard cut reduced the production owners from one 782-line root
to a 459-line root and a 336-line parallel owner, both below the repository's
800-line review warning. No compatibility implementation or second task/cache
owner remains in the root.

## Current Cache-Hit Cost Boundary

The 2026-08-26 correctness repair projects cache hits through the current
`MeshBatchRef` so a prior frame's sort key, source index and GPU Scene span cannot
leak into the visible command. At the current cache ABI,
`CachedMeshDrawCommands::lookup_status` returns an `Arc<MeshDrawCommandPayload>`;
the hit path increments that shared payload reference and constructs the compact
current-view command. It no longer clones a complete resource-bearing
`MeshDrawCommand`, but the atomic reference operation and current-view projection
remain unmeasured costs.

Do not call this a cache-hit speedup from static inspection. A generation-owned
arena handle could eventually replace the remaining `Arc` operation, but only if
the product profile attributes material CPU or contention cost to it. That change
must be scheduled with the cache and extract owners so `builder.rs` does not create
a second command authority.

## Static Cost Finding

The normal `MeshDraw` entry point derives source indices from enumeration, so
it is already ordered. Its parallel route delegates to the generic batch entry
point, which must also support unordered callers. That generic parallel entry
point unconditionally collects and stable-sorts, adding an `O(n log n)`
comparison sort to the steady-state ordered path.

This is a structural observation, not a measured bottleneck. No source change
is authorized from this observation alone.

## Architecture Reassessment Against Unreal

The stronger structural risk is not the comparison sort by itself. Current
Zircon preparation allocates a capacity-six command `Vec` in every
`PreparedBatchPlan`; every nonempty worker result then allocates another command
`Vec`, and a cache miss that publishes a payload may allocate a third
`cache_stores` vector. The ordered merge moves those commands into one global
list, after which `MeshPassCommandBuffers::from_command_list` repartitions them
into ten separately allocated phase vectors and sorts each vector. Rayon uses an
indexed parallel iterator, so scheduler work is balanced internally, but that
does not remove the `O(batch_count)` heap-allocation and repeated command-move
surface created by the data model.

Unreal 5.5's `FParallelMeshDrawCommandPass` instead gives each pass one durable
`FMeshDrawCommandPassSetupTaskContext`. `DispatchPassSetup` computes
`MaxNumDraws`, reserves the pass command and temporary command arrays once on
the render thread, dispatches one asynchronous setup task, and defers
`WaitForSetupTask` until the first consumer. Draw recording is partitioned into
a bounded number of contiguous ranges derived from worker count and a minimum
draws-per-command-list threshold. The relevant authority is
`dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.h` and
`MeshDrawCommands.cpp`; the Lumen compute-shader reference does not replace this
renderer ownership contract.

If allocation/call-tree evidence confirms this surface, the next architecture
change must be a hard cut, not another local fast path:

1. Introduce one generation-owned, per-pass setup context with a pre-sized
   command arena and immutable phase ranges; do not retain one command vector
   per source batch.
2. Partition contiguous input ranges into at most the admitted worker count,
   with a measured minimum batches/commands threshold. Workers write disjoint
   range outputs or one chunk per task, never one chunk per draw.
3. Resolve cache/variant ownership before publication, then compact and seal
   phase ranges exactly once. Remove the global-list-to-ten-vectors move and
   later repeated `extend`/sort authority.
4. Add an explicit setup ticket only when the pass lifetime can overlap setup
   with independent render-thread work. The current blocking
   `parallel_map_ordered` call is not equivalent to Unreal's deferred wait and
   must not be renamed as if it were.

The target scale is one command arena with `O(command_count)` storage,
`O(worker_count)` task-local chunks at most, zero heap allocations per stable
batch, and one phase seal. Cache identity, source order, and duplicate-key
fallback remain owner-thread deterministic. These are target invariants, not
current performance claims.

## Current Measurement Surface

The cached preparation path now exposes feature-gated profiling stages for
`normalize_source_order`, `parallel_admission`, `owner_transaction`,
`worker_projection_wait`, and `ordered_merge`, followed by a distinct
`seal_phase_buffers` span for phase partitioning and sorting. The serial path has a
`prepare_cached_serial` total span and a nested `serial_prepare_and_project` stage.
The already-ordered product serial entry still bypasses normalization; generic
serial batches and the parallel dispatcher use the one normalization owner.
Counters publish batch count, worker count, parallel admission, cache hits/misses,
command rebuilds, and final command count. No scope is created inside the per-batch
worker function, so capture overhead does not scale by adding a timeline span for
every draw. Serial/fallback and parallel preparation use the same
`seal_phase_buffers` scope around phase partitioning and sorting, so that cost
remains directly comparable.

Dispatch metadata and completion results are each emitted through one
`record_counter_batch` call. This keeps each group on one recorder lock while a
capture is active; the calls and counter arrays remain feature-gated out of an
ordinary non-profiling build.

Parallel dispatch also publishes a fixed typed reason code: `0` means parallel,
`1` means single worker, `2` means fewer than two batches, and `3` means duplicate
cache identity forced the deterministic serial owner path. The ordinary product
path with no `TaskPool` remains identified by the separate `prepare_cached_serial`
scope; the builder must not infer a missing caller-owned pool from worker timing.
The `parallel_admission` span isolates worker-count/batch-count checks and the
duplicate-cache-key safety scan so that cost is not silently charged to owner
transactions. That scan receives the same shader-quality dimension used by the
subsequent cache transaction instead of reconstructing keys with an implicit
default quality.
Serial, fallback, and parallel completion all publish cache hit/miss, rebuild, and
command counts through one result-schema owner, so comparisons do not lose the
fallback side of the sample pair.

These observations make the four architectural stages distinguishable in the
runtime timeline; they do not replace WPR/xperf CPU sampling or establish that any
stage is a bottleneck.

## Measurement Before Any Sorting Change

Use one representative product scene at 1k, 10k, and 100k static draws, after
pipeline and command-cache warmup. Record 600 frames and exclude the first 120
warmup frames. For each scale, capture a baseline with the current code and a
candidate with an ordered-input fast path.

CPU evidence:

- Run CPU and Heap ETW as separate matched invocations. `-UseWpr` records the WPR
  `CPU` profile and exports PID/lifetime-filtered `PROFILE` stacks;
  `-UseWprHeap` records the WPR `Heap` profile and exports PID/lifetime-filtered
  allocation stacks sorted by total allocation. The switches are mutually
  exclusive because Heap instrumentation overhead must not contaminate the CPU
  baseline.
- `RenderExtractWprCapture.psm1` writes the ETL, xperf text export, and JSON
  receipt below the invocation trace directory. The receipt binds profile, PID,
  product lifetime range, paths, byte counts, and SHA-256 values. xperf receives
  the product `StartTime`/`ExitTime` as absolute UTC wall-clock values; it does
  not estimate an ETL-relative offset from WPR command-launch time.
- `RenderExtractSystemTraceEvidence.psm1` rejects a report when receipt profile,
  PID, lifetime range, ETL hash, or analysis hash differs from the run. Legacy
  raw ETL remains `raw_only` and cannot make CPU sampling or Heap allocations
  measured.
- Report p50/p95 wall time for the command-preparation call tree and the share
  attributable to source-order normalization.
- Record worker utilization, cache-hit/rebuild counts, command count, and
  allocation stacks/counts for plan, worker-chunk, global merge, and phase-seal
  ownership. Allocation attribution is mandatory before choosing between a
  sorting fast path and the pass-arena hard cut.

GPU evidence:

- Use the existing RenderDoc workflow only to confirm draw order and pixels;
  it is not a substitute for CPU preparation profiling.

Correctness gates:

- Serial and parallel command signatures and cache statistics remain equal.
- Duplicate cache keys retain the serial fallback.
- Render-product output remains pixel-identical and stores a real image under
  `docs/tests/runtime/render`.

Do not merge an ordered-input fast path unless the baseline attributes a
material fraction of preparation time to sorting at an MVP-relevant scale and
the post-change WPR trace removes that cost without shifting it to owner-thread
cache/variant work.

## Product Evidence Route

The accepted visual and performance route is the `zircon_runtime` product
entry, not `zircon_shader_pbr_viewer` and not a render-product test binary.
Freeze a profiling BuildSet through
`tools/mvp/Build-RenderExtractProfilingInputs.ps1`, then run that immutable
executable/library pair through `tools/mvp/Capture-RenderExtractBaseline.ps1`
with every build, trace, log, and temporary artifact rooted on `D:`, `E:`, or
`F:`. The accepted PNG is copied to `docs/tests/runtime/render` only after the
capture manifest proves the same BuildSet and the runtime log contains the
successful native-surface bind/present diagnostics plus
`runtime_product_frame_capture_written`.

RenderDoc capture uses `D:/Tools/renderdoc/renderdoccmd.exe capture` with an
explicit non-`C:` capture template and the same frozen product invocation.
`ZR_RENDERDOC_CAPTURE_FRAME_COUNT` arms the runtime-owned WGPU capture boundary;
the resulting RDC must replay successfully and end at the native swapchain
present with the expected mesh/render-graph markers. Do not pass
`--reference-cpu-presenter`: that option is an explicit degraded readback/copy
path and cannot satisfy WGPU product evidence. Historical PNG/RDC files and a
viewer-only capture also cannot satisfy the current-source gate.

The current non-validation capture infrastructure now defaults to five matched
runs with 120 warm-up and 600 measured frames and exposes the separate
`-UseWpr` and `-UseWprHeap` routes and source-bound xperf receipts. The focused
metrics owner now passes 18/18, including source-schema binding, per-attempt
fail-closed aggregation, stream isolation, and branch-specific CPU-stage
requirements for direct serial, dispatch fallback, parallel, and mixed attempts.
These synthetic tests validate evidence integrity only. No current-source WPR
baseline, allocation result, speedup, power result, RenderDoc capture, or product
PNG is claimed by this record.

The existing scale-project generator is sufficient for the requested input
census: `tools/mvp/New-RenderExtractScaleProject.ps1` accepts 1 through 100,000
primitives, streams one distinct static entity per primitive, and binds each
entity to the same immutable model/material inputs. The current renderer does
not fold these entities into one CPU instance batch: collection creates a
`PendingMeshDraw` per entity/mesh raster draw and GPU Scene registers each
stable instance with `instance_count = 1`. This proves the generated scene is a
valid 1k/10k/100k static-entity workload, but it does **not** prove an equal draw
command count because one source draw may emit Prepass, Shadow, Opaque, Velocity,
or other phase commands. Every measurement table must therefore report both
the scale manifest's `primitive_count` and the captured
`mesh_commands.batch_count`/`mesh_commands.command_count`; never relabel the
former as draw count.

`tools/mvp/RenderExtractMeshCommandMetrics.psm1` now owns two explicit coverage
records consumed by `Write-RenderExtractBaselineReport.ps1` per scenario and at
report level:

- `mesh_command_preparation` covers the serial/dispatch preparation stages,
  normalization/admission, owner transaction, worker projection/wait, ordered
  merge, phase seal, and the cache/result counter group.
- `mesh_command_parallel_dispatch` covers the parallel stages plus batch count,
  worker count, enabled state, and dispatch reason.

The report marks the corresponding record `partial` when any required counter
or actual execution-branch CPU span is missing instead of allowing the
measurement to survive only as an incidental Top-20 metric. The latest full
focused PowerShell metrics suite passes 18/18. The report owner is 796 physical
lines and the new metrics owner is 309 lines, both below the 800-line review
warning. This closes the report-contract implementation item; it is not a
product capture or a performance result.

## Current Blocker

No exact current-source managed compile receipt or 600-frame WPR/xperf product
capture is attached to this slice. Scoped formatting, source-contract checks,
and the full 18/18 focused report suite pass. These are not dynamic performance evidence.
This document therefore makes no
performance, power, or optimality claim and does not change Render-02 milestone
status.
