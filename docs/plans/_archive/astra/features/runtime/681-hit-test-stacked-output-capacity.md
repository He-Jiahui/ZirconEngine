---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-10-default-scroll-candidate-scratch.md
related_code:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/tree/hit_test/query_scratch.rs
tests:
  - zircon_runtime/src/ui/tree/hit_test.rs
  - tools/tests/test_runtime_ui_surface_hit_query_scratch_contract.py
---

# Hit-Test Stacked Output Capacity

The Runtime hit-test query now reserves the known output lower bound before
materializing the stacked node result. A zero-radius query uses the selected
cell's entry count, while a radius query uses the deduplicated candidate count
already collected by the retained query scratch. Radius-hit scratch remains
borrowed and reused; ordering, clipping, input-policy filtering, top-hit
selection, and fallback behavior are unchanged.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / P0-6 hit-grid query | Reserve cell-entry/candidate lower bounds for stacked hit output | implemented_pending_validation | Embedded source regression and the existing hit-query scratch contract pass. The latest combined focused Runtime/Editor batch passed `132/132` in `0.143s`; managed Rust and product allocation/latency evidence remain pending. |

## Complexity Boundary

The change removes geometric growth for the common cell/candidate-sized output
without assuming that every candidate survives clipping or input-policy
filtering. The retained scratch still owns candidate dedupe and radius-hit
sorting, so this does not alter query complexity or claim a fixed product CPU,
allocation, RSS, input-to-present, or p50/p95/p99 improvement.

## Source Snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/tree/hit_test.rs` | `4C8CEA11F93CAB1F2BD0BE261D8FD09D682ADB8F30861952BF40572C71481803` |

## Managed Gate

The current Cargo submissions were rejected before ticket creation: the first
hit the ownership-overlay gate and the scoped retry hit the external
`E:\\Git\\zr_vm` dirty-worktree gate. The accepted static ticket
`7684931d60fb4a02bcc7fb5c0eb61879` covers only the Runtime text contract pair,
not this production path. No coordinator status was polled. This record stays
`implemented_pending_validation` until an owner-attributed multi-task Windows
Release run compiles the hit-test regression and reports current-source
allocation/time plus p50/p95/p99 evidence.
