---
title: Runtime214 transient bucket collector capacity
category: zircon_runtime
report_id: Runtime214-transient-bucket-collector-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
benchmark_session_id: 01a0ed3b-3043-7ee3-b4e4-bb5133267ec9
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: deterministic_capacity_bound_met_product_gate_pending
---

# Runtime214: reserve the cross-bucket transient allocation collector

## Change

`allocate_transient_lifetimes_by_bucket` grouped known lifetimes and appended each bucket's compiled allocations to a zero-capacity output vector. The grouping pass now counts only lifetimes that have a bucket key and are neither imported nor persistent. That count is the successful path's exact output length because the per-bucket allocator applies the same exclusion. The final collector reserves it once before extending bucket results. Empty and fully excluded inputs still request zero capacity.

This capacity hunk leaves bucket identity and order, slot reuse, allocation IDs, interval validation, resource-size errors, and RHI materialization untouched. It adds one count branch per grouped lifetime and removes the need for this helper's final collector to grow as buckets are appended. The same source file also contains the separate Runtime840 candidate's `array_layers` and `view_formats_key` bucket-identity edits and earlier per-bucket output reservation; the whole-file hash below covers both candidates. `build_transient_allocation_plan` can still grow its combined vector when it later appends buffer allocations.

## Evidence and remaining gate

- Source-candidate snapshot SHA-256: `zircon_runtime/src/render_graph/graph/transient_allocation.rs` = `194a0e9dbecf8a0ab9f26451fa1d239ca9597f955c6b7d252a66368f30ab894a`. The later shared-collector change in `2026-09-29-shared-transient-allocation-collector.md` supersedes this whole-file currentness; this record still describes its own collector-capacity hunk.
- The ignored Release scale harness keeps the four named topologies and 16/64/256/1024-pass matrix. Transient texture widths rotate through `4/8/16`; the multi-writer graph round-robins its passes across three textures and reads all three in its final pass. Every generated graph asserts at least two distinct compiled texture bucket hashes. It reports nearest-rank p50/p95/p99 from 31 measured samples after 3 warmups, with no pass threshold. With 31 samples, p99 is the maximum measured sample. The lane has not run.
- Scale harness SHA-256: `zircon_runtime/src/render_graph/tests/scaling.rs` = `d480121d836131529f3f0026e258590e7b6f52dc2c46118de1fca014512062f2`.
- Harness validation: `rustfmt 1.8.0-stable (Rust 1.94.1 toolchain) --edition 2021 --check`, scoped `git diff --check`, and documentation trailing-whitespace checks passed. No Cargo command ran.
- Rustfmt 1.94.1 and scoped `git diff --check` passed. The adjacent Runtime840 source/model contract passed `2/2`; it is a compatibility check, not execution of this new collector path.
- The bound is exact for a successful allocation result. No allocator, CPU, RSS, render-graph compile p50/p95/p99, GPU, or same-scene product comparison has run. An error before output publication may still allocate the reserved collector; this change makes no failure-path performance claim.

Keep Runtime214 performance gate G14 open. The grouped managed Runtime library batch must execute current-source aliasing/interval tests. The updated multi-bucket Release graph-compile scale lane is supplemental compiler evidence; the full gate still needs a fixed-scene product comparison with p50/p95/p99, allocator/RSS, a hardware profile, quality setting, and an explicit ceiling.
