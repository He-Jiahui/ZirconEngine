---
title: Runtime62 Viewport Highlight Hash Index
category: zircon_runtime
report_id: Runtime62-viewport-highlight-hash-index-2026-08-26
date: 2026-08-31
session_id: root-runtime62-viewport-hash-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime62 Viewport Highlight Hash Index

## Scope

This slice replaces the per-viewport latest-highlight owner with `HashMap`. Editor overlay submit
and render extraction lookup now resolve viewport IDs through expected constant-time lookup.

Current baseline is `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, epoch `575`.

The store exposes no viewport iterator. Per-viewport generation rejection, latest-value
replacement, cross-viewport isolation, highlight entity ordering, and set ownership are unchanged.

## Performance Workload

The release workload fills 4,096 viewport IDs and performs 4,096 stable hits for the final ID.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered viewport lookups | 4,096 | 0 |
| Hash viewport lookups | 0 | 4,096 |
| Viewport iteration-policy changes | 0 | 0 |
| Allocations on highlight hits | 0 | 0 |

The ignored release gate runs 4 warmups followed by 17 alternating sample pairs and emits
`RUNTIME62_VIEWPORT_HIGHLIGHT_HASH_INDEX_BENCH_V1`. Acceptance requires hash lookup P95 to be at
least 30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

Validation request ID: `63af5bf8915d4210bd03b1977b5c61cf`.

- Current RED: the focused performance contract passed owner, behavior, and P95 evidence guards,
  and failed only the missing warmup guard.
- Current GREEN: `python -m unittest tools.tests.test_runtime62_viewport_highlight_hash_index_performance_contract -v`
  passes 4/4.

- `optimization_batch_20260826bx_viewport_highlight_hash_index_preserves_generation_isolation`
  covers stale-generation rejection, replacement, set ordering, and cross-viewport isolation.
- `optimization_batch_20260826bx_viewport_highlight_hash_index_has_no_ordered_iteration` locks the
  unordered owner contract.
- `optimization_batch_20260826bx_viewport_highlight_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.
- The final WeCom message must report managed ordered/hash P50 and P95 values plus the P95
  reduction, scoped to 4,096 repeated final-viewport lookups.

## Remaining Parent-plan Work

Runtime62 still owns hierarchy propagation, activation, mobility, visibility, bounds, and render
product integration. This slice only converges per-viewport editor highlight lookup.
