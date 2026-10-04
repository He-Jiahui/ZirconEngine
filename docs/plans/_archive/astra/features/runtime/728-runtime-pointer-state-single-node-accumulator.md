---
related_code:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-25-ui-input-paint-style-static-candidates.md
related_records:
  - docs/plans/astra/features/runtime/672-single-node-pseudo-state-fast-path.md
  - docs/plans/astra/features/runtime/700-runtime-ui-route-sharing-hover-completion.md
  - docs/plans/astra/features/runtime/726-runtime-hit-route-parent-index-reuse.md
  - docs/plans/astra/features/runtime/727-runtime-hit-grid-entry-capacity.md
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
tests:
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py
---

# Runtime pointer-state single-node accumulator

The pointer component-state path now keeps its common one-node transition in an `Option` instead
of immediately allocating a `BTreeSet`. Repeated updates for the same node remain deduplicated;
the accumulator promotes to the existing ordered set only when a second distinct node changes.
The finalizer dispatches a single node through the existing descendant-aware fast path and keeps
the previous deterministic batch subtree path for multi-node transitions. The batch entry point
also recognizes a one-entry set from other callers before allocating descendant/root scratch.
When true multi-node batching is required, the minimal-root output now reserves the changed-node
upper bound before walking ancestors, avoiding repeated vector growth while preserving the same
sorted-root semantics.
Hover, press, focus, error, and fail-closed behavior are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A / Editor01 deferred pointer-state candidate | Avoid the one-entry ordered-set and batch-root bookkeeping on ordinary single-node pointer transitions; reserve the bounded multi-node root output and preserve ordered batching once multiple distinct nodes are involved. | RED/GREEN source and accumulator regressions, batched Runtime/Editor source contracts, scoped Rustfmt and diff checks; managed Release allocation/latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

For a single changed node, the pointer transition bookkeeping is constant-time with no ordered-set
allocation and the existing `mark_component_state_render_dirty` path performs the one
descendant-selector probe. A second distinct node creates one `BTreeSet` and retains the prior
sorted multi-node algorithm; a one-entry set passed by another caller now early-returns before the
batch scratch sets and root walk. Multi-node ordering and minimal-root coverage remain unchanged.
The accumulator is transaction-local; it does not introduce another runtime state authority.

## Local evidence

- The new source contract was first run RED against the unconditional `BTreeSet::new()` path,
  then passed after the accumulator and direct single-node finalizer were installed.
- The in-file accumulator regression checks duplicate single-node updates stay in the scalar slot
  and that promotion preserves ordered IDs. The focused owner-structure contract passes `3/3`.
- The focused owner-structure/route/visibility/activity/hit-grid batch passes `32/32` in one
  invocation. The refreshed non-tooling Runtime/Editor performance-plus-pressure loader covers
  `347` modules and passes `1333/1333` tests in `6.334s`; after the adjacent Runtime662/664
  contract guard repair and root-capacity check, the final combined loader covers `348` modules
  and passes `1338/1338` tests in `5.388s`.
- Scoped production Rustfmt, `git diff --check`, and plan-record integrity checks pass for this
  slice. The current source snapshot SHA-256 is
  `B6D371E48548178BC8B95180D6E786339FB53DCA229ACAF02361BF72F159478F`
  (`state_invalidation.rs`); the focused Python contract hash is
  `F66E556B68A52CE515A32647A2ED705427E6737ECC7E053D1C7453AFD0FC66AE`. The adjacent text
  contract hash is `1972E55C497EF20070A0A86FDAF482490CB6DB60182BE5FDB5576E2F0AAE24AD`.

This is source/contract evidence, not managed Cargo execution or product CPU, allocator, RSS, or
p50/p95/p99 acceptance evidence.

## Managed acceptance gate

This slice joins the deferred owner-attributed Runtime/Editor admission recorded in `696`. No new
coordinator request or status query is issued for this local change. Keep the row pending until a
managed Windows Release run measures ordinary pointer-state allocation and input-to-damage
latency, including the no-descendant-selector single-node case and the descendant-selector batch
fallback.
