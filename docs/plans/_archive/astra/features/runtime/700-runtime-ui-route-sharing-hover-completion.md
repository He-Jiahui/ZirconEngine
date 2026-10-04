---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200/2026-08-31-borrowed-dispatch-route-sharing.md
  - docs/plans/optimize/zircon_runtime/200/2026-08-31-pointer-hover-hot-paths.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/705-hit-route-publication-scratch-reuse.md
  - docs/plans/astra/features/runtime/748-runtime-hover-diff-membership-scratch.md
---

# Runtime UI Route Sharing And Hover Hot-Path Completion

The current Runtime UI input path already contains the Runtime200 bounded
optimizations: pointer and navigation dispatch borrow route/candidate data,
share the inline-first `UiDispatchVisitedNodeSet`, and promote to a hash set
only for deep routes; hover updates reuse the retained path, keep a linear
small-path branch, and now retain one surface-owned membership table for dense
paths. Route order,
capture, bubbling, focus, passthrough, and wire projections are unchanged.

## Plan completion list

| Area | Implementation | Evidence | Status |
| --- | --- | --- | --- |
| Runtime200 borrowed dispatch | Borrowed route/candidate projections and shared inline-first visited-node set. | Route-sharing source contract and pressure model passed; navigation and pointer ownership contracts passed. | implemented_pending_validation |
| Runtime200 pointer hover | Iterator-based retained hover path plus bounded linear/hash diff branches; Runtime748 moves dense-path membership into surface scratch. | Hover-diff, hover-path, and pressure contracts passed; Runtime748's focused source batch passes `13/13`. | implemented_pending_validation |

## Batched local evidence

- Route-sharing source contract: `7/7`.
- Route-sharing pressure model: `7/7`.
- Hover-diff source contract: `4/4`.
- Hover pressure model: `5/5`.
- Hover-path source contract: `3/3`.
- Navigation dispatch ownership: `8/8`.
- Pointer dispatch ownership: `7/7`.
- Aggregate: `41/41` focused Runtime200 contract/pressure tests passed.

These are deterministic source/pressure-model results. They do not replace
managed Rust execution or product input-latency p50/p95/p99 measurements.

## Managed acceptance gate

The shared Runtime200 Release batch remains an asynchronous coordinator
responsibility. Current managed admission is deferred by the external
`E:\\Git\\zr_vm` dirty-worktree gate recorded in 696; no new ticket was
submitted and no coordinator state was polled in this slice. Keep the row at
`implemented_pending_validation` until owner-attributed Windows behavior tests
and the ignored Release benchmark provide current-source latency/allocation
evidence.

Tooling changes remain deferred by request.
