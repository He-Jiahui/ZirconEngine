---
title: Runtime214 same-pass reader ledger deduplication
category: zircon_runtime
report_id: Runtime214-reader-ledger-dedup-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
implementation_status: source_candidate_pending_validation
validation_status: focused_static_checks_passed_managed_validation_pending
performance_status: duplicate_work_removed_product_gate_pending
---

# Runtime214: keep one reader per pass in each access history

## Change

Render Graph dependency inference records a pass ID for every read access in each touched resource-scope history. Overlapping read bindings within one pass are legal, so a pass could appear repeatedly in the same `readers_since_last_write` vector. A later write iterated every entry to add write-after-read dependencies, while `DependencyAdjacency` discarded the duplicate edges.

`ResourceAccessHistory::record_reader_pass` now appends only when the last stored reader differs from the current pass. Inference processes each pass's accesses contiguously, and a write clears the history's reader list. Therefore one history retains at most one entry for each reader pass between writes, while distinct reader passes and scope projection remain unchanged. The change reduces retained IDs and later membership probes for repeated same-pass reads; it does not establish a product timing improvement. The large tracker remains a cohesive access-scope module for this bounded helper change, so no unrelated module split was made.

## Evidence and remaining gate

- Current whole-file SHA-256: `zircon_runtime/src/render_graph/builder/access_scope_tracker.rs` = `9b4441e6b8a96af47e665f91ba9234e49206c05d63fdb4b131804ee74eca86ad`; `zircon_runtime/src/render_graph/builder/resource_dependency_inference.rs` = `bbba60e6532f7b090fc081cf76085397296ebb1df3d8453183b131cde3d35510`. Both files contain unrelated preexisting Render Graph work; this candidate is the reader append change.
- `rustfmt --edition 2021 --check` and scoped `git diff --check` passed. No Cargo command ran for this slice.
- Existing `builder_allows_overlapping_read_scopes_for_parent_and_texture_view_alias` covers legal same-pass overlapping reads. The grouped Runtime library batch must run current-source dependency and alias regressions. The narrow internal dedup does not change the compiled adjacency's unique-edge contract.
- No Release compile p50/p95/p99, allocator/RSS, fixed-scene product comparison, or explicit performance ceiling was measured. Runtime214 G14 remains open.

The next grouped managed Runtime library validation should include this source after the frozen v11 request has a terminal receipt. An offline draft is not passing validation.
