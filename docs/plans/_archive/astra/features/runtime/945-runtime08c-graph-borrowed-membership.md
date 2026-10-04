---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c/2026-09-26-graph-borrowed-membership.md
related_records:
  - docs/plans/astra/features/runtime/944-runtime08c-graph-clip-aggregation-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/animation/manager/graph.rs
tests:
  - zircon_runtime/src/animation/manager/graph.rs (in-file source contract)
---

# Runtime945 · graph traversal borrowed membership keys

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime08C animation graph cycle membership | Keep traversal keys borrowed from the graph instead of allocating `String` keys for every recursive node lookup; preserve output ownership and all traversal semantics. | TDD source contract RED before the change and GREEN after it; the grouped current-source Debug batch includes the direct Output→Blend→Clip behavior regression and preserves owned output and clip weight; existing Runtime624 capacity contract retained. `RUNTIME945_GRAPH_BORROWED_MEMBERSHIP_BENCH_V1` is wired as an ignored 17-pair Release marker. Current-source local Release passes with owned p95 `430,150µs` versus borrowed p95 `165,672µs` and deterministic key constructions `1,048,576` versus `0`; managed Release/product gates remain pending. | implemented_pending_validation |

## Complexity boundary

The change removes temporary node-ID key allocations from graph traversal. It
does not alter graph lookup, recursion order, cycle handling, clip weighting,
mask target projection, or the owned `AnimationGraphEvaluation::output_node`
field.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/animation/manager/graph.rs` | `862D30B59F265B091669BB348D0E3A9EC11CD966DB1D8CB9DEAA94DBF6471CC1` |

## Validation handoff

The approved target is
`F:\\codex-targets\\zircon-engine\\runtime-graph-capacity-min-20260925`.
The grouped Cargo invocation completed the current source in the debug profile:
`12` tests discovered, `9` passed, `0` failed, and `3` Release tests ignored.
This is a local compiler/behavior receipt, not the coordinator's managed
Release result. The local Release marker batch also passed: owned-key p95
`430,150µs`, borrowed-key p95 `165,672µs`, `1,048,576` versus `0` key
constructions. Keep this record pending until the managed lane supplies its
authoritative Release allocation/p50/p95 and product evidence.
