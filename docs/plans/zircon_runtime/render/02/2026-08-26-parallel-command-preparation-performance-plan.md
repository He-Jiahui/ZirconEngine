# Render-02 Parallel Command Preparation Performance Plan

Date: 2026-08-26

## Scope and Current Algorithm

This review covers `mesh_pass/mesh_draw_command_list/builder.rs`, specifically
`build_mesh_pass_command_buffers_from_batches_cached_parallel`.

The current algorithm has four ordered stages:

1. Collect batches and normalize by `source_draw_index`.
2. Resolve pipeline variants and cache hits/misses on the owner thread.
3. Build immutable command chunks on the shared `TaskPool` workers.
4. Merge chunks in source order, then commit cache stores on the owner thread.

This preserves the Render-17 contract: no private rendering thread pool, no
mutex-based owner mutation in workers, and serial/parallel output equivalence.
The focused regression test covers input presented in reverse source order.

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

- Use `wpr.exe` CPU sampling and `xperf.exe` analysis on the render process.
- Report p50/p95 wall time for the command-preparation call tree and the share
  attributable to source-order normalization.
- Record worker utilization, cache-hit/rebuild counts, command count, and
  allocation count if the runtime counters are available.

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

## Current Blocker

No exact current-source managed compile receipt or 600-frame WPR/xperf product
capture is attached to this slice. Scoped formatting, source-contract checks, and
workspace metadata inspection pass, but they are not dynamic performance evidence.
This document therefore makes no performance, power, or optimality claim and does
not change Render-02 milestone status.
