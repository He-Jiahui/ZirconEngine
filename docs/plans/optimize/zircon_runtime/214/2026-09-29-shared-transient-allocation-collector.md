---
title: Runtime214 shared transient allocation collector
category: zircon_runtime
report_id: Runtime214-shared-transient-collector-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: intermediate_collector_removed_product_gate_pending
---

# Runtime214: collect texture and buffer allocations in one vector

## Change

`build_transient_allocation_plan` used to receive separate texture and buffer allocation vectors, then append the buffer vector to the texture vector. The bucket allocator already counts its exact successful output length. It now reserves that count in a caller-owned vector and extends the same vector for texture buckets, then buffer buckets. The separate buffer collector and its copy into the final result are removed. An explicit reserve before the old `Vec::append` would have been redundant because `append` reserves for its source length.

The source still sorts each bucket and the final allocations as before, and passes the same monotonically advancing allocation ID counter through texture then buffer phases. A mixed result may still relocate its existing texture entries when the buffer phase reserves additional capacity. This candidate does not eliminate per-bucket allocation vectors or claim zero allocation growth. On error, the partially filled local result is discarded; the changed reservation timing can affect only work before that error.

## Evidence and remaining gate

- Current whole-file SHA-256: `zircon_runtime/src/render_graph/graph/transient_allocation.rs` = `54a547a8fceacf9a409978ad40d825bf50d643a694587c27cdf07bbba85b95b7`. The same source also contains prior Runtime840 bucket-key and Runtime214 collector-capacity candidates. The earlier collector record's hash is now a snapshot, not the current file.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed. No Cargo command ran for this slice.
- Existing `graph_transient_allocation_plan_reports_slot_reserved_bytes` exercises mixed texture and buffer output and checks allocation IDs, slots, and byte reservations. It has not run against this source in the grouped Runtime library batch.
- No allocator trace, RSS, Release graph-compile p50/p95/p99, fixed-scene product comparison, or explicit ceiling was measured. Runtime214 G14 stays open.

The next grouped managed Runtime library batch should run the mixed allocation regression after the frozen v11 request has a terminal receipt. The offline draft is not passing validation.
