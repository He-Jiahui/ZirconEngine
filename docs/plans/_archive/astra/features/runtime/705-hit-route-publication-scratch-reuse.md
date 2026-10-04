---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-08-31-borrowed-dispatch-route-sharing.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/700-runtime-ui-route-sharing-hover-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Runtime Hit-Route Publication Scratch Reuse

`build_route_nodes` now keeps one traversal scratch buffer for the entire
publication.  The old loop created a fresh `Vec<usize>` for every unresolved
tree component; the new loop clears and reuses that buffer while retaining the
same iterative cycle/missing-parent rejection and route ordering.  This removes
per-component temporary heap allocation from multi-root and disconnected-tree
publication without changing the shared route-table or copy-on-write boundary.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Runtime200 hit-route publication | Reuse one chain scratch `Vec` across route components; invalidate through a borrowed slice. | Route-index, route-pressure, and dispatch-route source contracts pass; Rust formatting/diff checks pass. | implemented_pending_validation |

## Batched local evidence

- Runtime hit-route and route-sharing contract batch: `16/16` passed.
- The existing deep-chain and missing-parent/cycle regressions remain in the
  production test module, and `failed_component_does_not_poison_reused_traversal_scratch`
  covers a failed component followed by a valid root.
- A source-bound guard confirms one scratch declaration outside the component
  loop, `chain.clear()` at each unresolved component, and no per-component
  `Vec` declaration.
- Current source snapshot SHA-256:
  `898597EE385675C5122325A0B02164083FE310BB422FBEB4D3B658CBD208D919`.
- Scoped Rustfmt parse/check and `git diff --check` pass; only the repository's
  existing LF/CRLF notices are reported.

This is source/contract evidence, not managed Cargo execution or product
latency/allocation p50/p95/p99 evidence.

## Managed acceptance gate

The owner-attributed Windows Release gate remains asynchronous and currently
deferred by the external `E:\\Git\\zr_vm` dirty-worktree admission recorded in
696. No coordinator state was polled in this slice. Keep this row at
`implemented_pending_validation` until the batched managed Runtime UI tests and
Release allocation/latency measurements validate the current source.

Tooling changes remain deferred by request.
