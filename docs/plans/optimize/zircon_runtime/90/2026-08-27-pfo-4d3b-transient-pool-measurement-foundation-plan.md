# PFO-4d3b RDG Transient Pool Measurement Foundation

Status: `source_implemented_static_checks_passed_dynamic_capture_and_optimization_pending`

Date: 2026-08-27

## Scope

This slice adds the minimum product-observable evidence needed before changing the RDG transient
resource pool algorithm. It does not change resource identity, allocation reuse, submission
retirement, stale eviction, budget eviction, or container selection.

## Structural Review

The current pool has four unmeasured CPU work classes:

1. `collect_completed_submissions` queries one completion status per pending texture or buffer;
2. stale eviction visits every free allocation at frame end;
3. budget accounting visits every allocation that survives stale eviction;
4. an over-budget pool materializes and sorts every surviving allocation as an LRU candidate.

These paths make per-resource completion coalescing, age buckets, retained-byte ledgers, and a bounded
eviction queue plausible future optimizations. Static call counts are not bottleneck evidence, so no
such optimization is authorized until product captures quantify the work and CPU cost.

## Measurement Contract

- add profiler CPU scopes for completion collection and frame-end pool maintenance;
- report texture/buffer completion status query counts;
- report texture/buffer stale scan counts;
- report texture/buffer budget accounting counts;
- report texture/buffer budget sort candidate counts;
- publish every count through the existing render diagnostics store;
- preserve existing report construction compatibility by defaulting new fields to zero;
- add source and diagnostic contracts without running the deferred Cargo/WGPU acceptance queue.

## Required Dynamic Evidence Before Optimization

Capture at least 300 steady-state frames for representative deferred, forward, shadow-heavy, and
resize/device-recovery workloads. Record p50/p95/p99 CPU scope time, all eight work counters, retained
bytes, eviction counts, frame time, allocator/RSS, and submission count. Only then select an
algorithmic change and compare the same captures before and after.

RenderDoc, WGPU execution, screenshots, GPU timestamps, power, and product performance receipts are
not produced by this source slice and remain mandatory milestone acceptance work.

## Source Implementation Evidence

- `RenderGraphTransientPoolReport` now carries all eight work counters and keeps existing constructor
  call sites compatible through zero defaults plus one focused builder;
- the pool increments completion query counters at the existing status lookup, stale scan counters at
  the existing bucket visit, budget accounting counters from the existing fold, and sort candidate
  counters from the existing candidate vector; no additional pool traversal was introduced;
- the existing diagnostics store publishes all eight series with resource, pool, maintenance, and
  submission tags; the focused diagnostics fixture exercises non-zero values for every series;
- profiler scopes cover completion collection and frame-end maintenance without changing allocation,
  retirement, or eviction behavior;
- focused `rustfmt --check` and scoped `git diff --check` pass. The production diagnostics owner is
  618 lines, its focused test owner is 759 lines, the graph report owner is 660 lines, and the pool
  owner is 753 lines, all below the repository-local 800-line contract checked for these files;
- Cargo, WGPU, screenshots, RenderDoc, 300-frame profiling, allocator/RSS, GPU timing, VRAM, and power
  remain deferred. No optimization or performance claim is accepted from this source evidence.
