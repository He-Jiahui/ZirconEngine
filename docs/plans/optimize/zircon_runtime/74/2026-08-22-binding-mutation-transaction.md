# Runtime74 Binding Mutation Transaction And Selective Snapshot

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-binding-mutation-transaction.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_transaction.rs","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/input/effect/transaction.rs","zircon_runtime/src/ui/surface/mod.rs","zircon_runtime/src/ui/surface/mutation_snapshot.rs","zircon_runtime/src/ui/tests/binding_transaction.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/mod.rs","zircon_runtime_interface/src/ui/binding/mod.rs","zircon_runtime_interface/src/ui/binding/model/mod.rs","zircon_runtime_interface/src/ui/binding/model/mutation_receipt.rs","zircon_runtime_interface/src/ui/binding/model/update.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source items: `RTB-P1-022`, `RTB-P1-045`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The atomic binding target executor prepared every target before mutation but cloned the complete
`UiSurface` for each target-bearing event. That copy included arranged frames, hit-test and render
artifacts, text caches, compiled bindings, and node-pool state that target mutation cannot modify.
Commit also replaced the whole surface, while reports had no explicit transaction outcome.

## Scope Delivered

- `UiBindingMutationTransaction` captures the invalidation generation and target count, applies to
  the authoritative surface, and produces a typed committed or rolled-back receipt.
- `UiSurfaceMutationSnapshot` captures only five writable domain groups. Its tree group owns the
  retained tree, runtime style index, pending invalidation, and dirty-node index; the other groups
  own focus, input, component state, and navigation.
- Any target rejection restores all five groups before suppressing the component event. A
  `UiTreeError` also restores state before propagation. Successful execution discards the snapshot
  and never clones or replaces the complete surface.
- The existing input-effect transaction now consumes the same selective snapshot owner, removing
  its duplicate rollback implementation without changing its write-set policy.
- `UiBindingUpdateReport` carries an optional `UiBindingMutationReceipt` with base generation,
  prepared target count, applied target count, and terminal outcome.

## Performance Contract

`binding_mutation_transaction_snapshot_p95_beats_whole_surface_clone` runs 21 paired samples and
alternates which implementation executes first. Each sample performs 64 copies over a retained
surface with 4,096 arranged-frame nodes. The release marker emits both raw sample arrays and their
nearest-rank P95 values. The external validator must independently recompute P95 and enforce:

- optimized P95 is at least 25% lower than legacy whole-surface clone P95;
- `staged_surface_clones=0`;
- `snapshot_domain_groups=5`.

Measured values remain pending until the coordinator validation copy completes. No performance pass
or Cargo pass is claimed in this record yet.

## Regression Contract

- A direct transaction test mutates tree/invalidation/dirty, focus, input, component-state, and
  navigation ownership, rolls back, and compares the entire surface with its pre-transaction state.
- The existing five-target success path must emit a committed receipt with five applied targets.
- A commit-stage type rejection after an earlier class mutation must restore the class, suppress the
  event, preserve visibility, and emit a rolled-back receipt with zero applied targets.

## Remaining Scope

This slice does not implement model subscriptions, frame safe-point batching, two-way model writes,
command lifecycle, or granular per-node undo journals. The selective snapshot still clones the
retained tree and runtime-style owner because property and pseudo-state mutation can affect popup
descendants and style subtrees; replacing that safety boundary requires a lower-level mutation
journal with equivalent rollback evidence.
