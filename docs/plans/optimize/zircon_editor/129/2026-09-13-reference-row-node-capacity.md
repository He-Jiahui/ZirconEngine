---
title: Editor reference-row node capacity reservation
category: zircon_editor
report_id: Editor129-reference-row-node-capacity-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor129 reference-row node capacity reservation

## Scope

The References and Used By panes append four retained template nodes for every
`AssetReferenceSnapshot`.  The refresh path removed stale dynamic rows and then
extended the backing `Vec` one four-node array at a time, so a large reference
projection paid geometric vector growth before layout and pointer indexing.

## Implementation

- Added the structural `REFERENCE_ROW_NODE_COUNT` constant (`4`).
- After the empty-state early return, `sync_asset_reference_list` reserves
  `references.len() * 4` additional node slots before appending rows.
- Prototype reset, row order, control-id numbering, unknown-kind fallback, and
  empty-list behavior are unchanged.
- Added a lower Rust regression that checks row count, capacity, order, and the
  retained prototype after a 128-row refresh.
- Added the ignored Release marker
  `EDITOR742_REFERENCE_ROW_CAPACITY_BENCH_V1`, comparing the prior geometric
  extension shape with the reserved path over 4,096 rows.

## Complexity and deterministic pressure model

For `R` references, projection remains `O(R)` and still owns the four required
prototype clones per row.  The reserved path performs one capacity admission;
the old shape grows geometrically while extending 4-node arrays.  A 4,096-row
model (16,384 appended nodes, 16 retained prototype nodes) has 11 geometric
growth steps versus one upfront capacity allocation.  This is a structural
allocation model, not a product CPU/RSS or latency measurement.

## TDD and local evidence

- The new source contract was intentionally RED before the reservation was
  added, then GREEN at `3/3`.
- The lower Rust regression and ignored benchmark are in the same source test
  module; Rustfmt and Python compilation pass.
- The refreshed one-process non-tooling Runtime/Editor
  performance-plus-pressure batch covers `555` modules and passes `2065/2065`
  tests in `24.966s`, including this contract and the existing asset/reference
  suites. This is local source/model evidence; no per-task Cargo invocation is
  required.
- The subsequent shared batch covers `556` modules and passes `2068/2068`
  tests in `29.116s`; the focused Runtime206/Runtime85 plus Editor reference
  invocation passes `37/37` in `111.865s`. The historical source snapshot
  above remains the Editor742 checkpoint before Editor743's adjacent change.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/views/asset_reference_rows.rs` | `84A98F407653B3ECDFDDABB1654609484A08ADF731A701E3E9663A51BB0E962E` |
| `tools/tests/test_editor_asset_reference_rows_capacity_performance_contract.py` | `50B4C5B9DDB7795FFD4E75B600E52C437FE29B304610B18B3149A6EFB3FC610C` |

## Managed acceptance gate

Keep this slice at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves compilation, row/prototype parity,
allocation behavior, and reference-panel p50/p95/p99. Tooling production work
remains intentionally deferred.
