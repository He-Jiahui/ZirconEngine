---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
related_code:
  - zircon_runtime/src/ui/surface/focus/modal_scope.rs
tests:
  - zircon_runtime/src/ui/surface/focus/modal_scope.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
---

# Runtime Modal Restore Scratch Streaming

## Scope

Closing a modal focus scope previously enumerated every restore-stack entry,
materialized the matching indices in a temporary `Vec<usize>`, removed the
closing entry, and then adjusted each stored index before applying the splice.
The restore target is resolved from the candidate entry and the current tree;
it does not depend on the position of another restore-stack entry. The close
path now removes the target state first and streams the surviving entries in
their original order, resolving and updating each eligible dependent directly.

The existing splice semantics remain intact: every dependent restore edge that
targets the closing scope receives the closed state when restoration is
enabled, or is cleared when it is disabled. Nested and visual-z-order-different
modal behavior remains owned by the existing behavior suite. This is a local
scratch-allocation reduction, not completion of the planned generation-owned
focus graph or cross-surface modal authority.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime11A / RUI-673 | Stream modal restore-stack splice candidates after removal instead of allocating dependent index scratch | implemented_pending_validation | The focused Rust source regression rejects the temporary index collection and requires post-removal sequential resolution. Scoped Rustfmt, scoped diff check, a four-invariant source probe, and the combined Runtime/Editor static-contract batch passed. Managed Runtime Cargo and Windows Release allocation/time evidence remain pending. |

## Static Evidence

- `modal_restore_splice_streams_remaining_entries` requires removal before
  iteration, sequential access to the remaining stack, target resolution before
  the mutable write, and no `dependent_indices` or collected `Vec` in the
  helper.
- Existing modal focus behavior tests cover nested scopes, out-of-order close,
  disabled restore, and path-based restore. They are staged for the managed
  Rust batch and are not claimed as locally run.
- `rustfmt --edition 2021 --check --config skip_children=true` passed for
  `modal_scope.rs`. Scoped `git diff --check` passed; Git reported only the
  repository's existing line-ending notice.
- The direct source probe passed all four invariants.
- The combined Runtime/Editor static-contract batch passed `95/95` in
  `0.090s`.

## Complexity Boundary

For a close with `S` surviving restore states, both versions retain `O(S)`
target checks. The prior version also allocated and populated an index vector
for every matching state, then performed index correction. The new version
performs the same ordered checks and splice writes without index scratch. It
does not remove the tree walks used to resolve path-based restore targets.

## Source Snapshot

- `modal_scope.rs`: `6B862E9CF31731E993D2B18F0572F394211F636DD2132EE71D287ABA1D6DA3A2`

## Managed Gate

No direct Cargo command or coordinator status query was run for this slice.
The next immutable multi-task Runtime/Editor validation input must compile the
focused modal behavior and source-regression tests together with the existing
batch, then collect a release allocation/time comparison. Until that batch
succeeds, this record remains `implemented_pending_validation` and makes no
product performance or p50/p95/p99 claim.
