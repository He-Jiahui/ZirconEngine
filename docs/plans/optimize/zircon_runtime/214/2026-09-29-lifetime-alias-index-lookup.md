---
title: Runtime214 lifetime alias parent lookup
category: zircon_runtime
report_id: Runtime214-lifetime-alias-index-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: lookup_complexity_improved_product_gate_pending
---

# Runtime214: index texture-view alias parents in lifetime compilation

## Change

`RenderGraphBuilder::resource_lifetimes` already builds a resource-to-declaration index. It previously scanned every resource declaration for each live pass access to discover a texture-view alias parent. It now resolves the declaration through that index and reads its `texture_view_alias` field. `resource_declarations()` copies the same alias field from the builder's resource table, so the parent lifetime is extended for the same accesses as before.

For this lookup alone, the index build is O(resources) and the per-access parent lookup is expected O(1), replacing O(resources x accesses) repeated scans with expected O(resources + accesses) work. This is a structural complexity improvement, not a measured latency result. The source file contains unrelated preexisting render-graph changes; this candidate is only the lookup replacement at `resource_lifetimes`.

## Evidence and remaining gate

- Current whole-file SHA-256: `zircon_runtime/src/render_graph/builder/compile.rs` = `f2c3bdaa9d2b7309e14ced643a9e0891c02d1e6342efce2b9a3762a86a98b53f`.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed. No Cargo command ran for this slice.
- The existing `graph_declared_texture_view_alias_keeps_its_logical_lifetime_without_a_physical_slot` regression covers the parent-lifetime behavior, but has not run against this current source in the managed batch.
- No Release graph-compile p50/p95/p99, allocation, RSS, fixed-scene product comparison, or explicit performance ceiling has been measured. Runtime214 G14 stays open.

Run the existing alias regression in the grouped Runtime library batch after the frozen v11 request has a terminal receipt. Do not count this source-level improvement as product performance acceptance.
