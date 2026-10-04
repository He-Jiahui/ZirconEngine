---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c/2026-09-25-graph-clip-aggregation-capacity.md
related_records:
  - docs/plans/astra/features/runtime/839-animation-missing-track-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/manager/graph.rs
tests:
  - zircon_runtime/src/animation/manager/graph.rs (in-file source contract)
---

# Runtime944 · graph clip aggregation lower-bound capacity

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime08C animation graph Blend aggregation | Reserve `input_count` before extending child clip results; retain nested fan-out semantics and first-seen ordering. | TDD production-section contract RED before the patch and GREEN after it; exact Rustfmt and diff-check pass. Grouped local Runtime+Editor discovery is `4,354/4,354`; the non-ignored performance-contract subset is `2,950/2,950`; the current approved Windows `core-min,animation` graph batch runs `12` tests with `9` non-ignored passes, `0` failures, and `3` Release tests ignored (the added Runtime945 behavior/marker checks are included). The local Release batch also passes Runtime08C `13,547→698µs` and Runtime624 `345,684→149,009µs` p95 markers. Managed grouped Cargo/Release allocation/p50/p95 evidence remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/manager/graph.rs` | `862D30B59F265B091669BB348D0E3A9EC11CD966DB1D8CB9DEAA94DBF6471CC1` |

The snapshot now includes the Runtime945 borrowed-membership follow-up in the
same owner file; the Runtime944 production reserve and source contract remain
unchanged.

## Complexity boundary

This is a bounded allocation reduction in the Blend aggregation temporary.
It does not change graph traversal, parameter lookup, clip weighting, mask
target collection, or output ordering. The reserve is a lower bound only:
child blends can append additional clips and `Vec` grows normally when needed.

## Managed gate

The focused Rust owner was compiled and run in an approved Windows target with
`--features core-min,animation`: `1/1` passed after an 8m57s build. The same
target then ran the complete current `performance_contract_tests` module in one
batch: `12` tests discovered, `9` non-ignored passed, `0` failed, and `3`
Release tests ignored by their existing gate conditions. These are local
current-source receipts, not the coordinator's grouped receipt. The local
Release ignored batch passes the Runtime08C marker at `13,547→698µs` p95 and
Runtime624 at `345,684→149,009µs` p95. Keep the status pending until grouped
managed Cargo, focused behavior, and authoritative Release allocation/p50/p95
evidence are attached; do not poll the coordinator from this record.
