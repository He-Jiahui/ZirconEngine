# Runtime77 Atomic Input Transaction

Plan: docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/77/2026-08-22-atomic-input-transaction.md","docs/zircon_runtime/ui/surface/input.md","zircon_runtime/src/ui/surface/input/effect.rs","zircon_runtime/src/ui/surface/input/effect/transaction.rs","zircon_runtime/src/ui/tests/runtime_input_ownership.rs","zircon_runtime/src/ui/tests/runtime_input_ownership/transaction.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime77-atomic-input-review-fix-r2-bee4c707-20260822`
- Source item: `RUII-P0-001`
- Acceptance gates: `RUII-GATE-007` and `RUII-GATE-008`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`apply_dispatch_reply_core(...)` applied effects directly and independently. A valid focus or
capture prefix therefore remained committed when a later effect was rejected. Drag/drop effects
also performed session, capture, component-state, style, and dirty-state writes in multiple steps;
an error from a later step left the earlier writes visible.

The returned result mirrored the same partial state by retaining prefix applied effects, host
requests, and component events beside the rejected tail.

## Scope Delivered

- `UiInputTransaction::prepare(...)` seals the reply effect list, derives its complete mutable
  domain set, and records the base invalidation generation before effect application.
- Multi-effect replies and single drag/drop or popup composites capture only the domains they can
  write: tree/runtime-style/invalidation/dirty ids, focus, input, component state, and navigation.
  Every non-empty atomic reply captures input because any successful effect drains deferred
  focus/input-method lifecycle work, including otherwise read-only host effects.
- A rejected effect restores the captured domains before the caller regains access to `UiSurface`.
  Prefix applied effects, host requests, and component events are removed and every uncommitted
  effect is reported as transaction-aborted with the original failing reason retained.
- Successful atomic replies record the base generation and effect count in diagnostics. Aborted
  replies record the base generation and failing effect index.
- Ordinary single effects admit an empty snapshot with zero captured domains. Their target
  validation and mutation behavior is unchanged, and no retained UI tree or input-state clone is
  performed.

## Deterministic Performance Evidence

The performance contract for this correctness slice is structural rather than a timing claim:

- ordinary single-effect replies: zero captured surface domains and zero retained-tree/input clones;
- multi-effect replies whose write set contains only focus/capture input state: tree cloning occurs
  only when a focus effect can indirectly mutate tree style/dirty state;
- read-only multi-effect replies capture input only so deferred focus/input-method lifecycle work
  can be restored; they do not clone tree, focus, component state, or navigation;
- the source-level validator locks the conditional `atomic.then_some(write_set)` admission and the
  focused test locks the absence of an `input_transaction=` diagnostic on the ordinary single
  capture path.

No wall-clock improvement is claimed for `RUII-P0-001`; Runtime77 route-allocation and route-clone
timing targets remain owned by the later P1 performance tasks.

## TDD And Static Evidence

- `input_transaction_tail_rejection_rolls_back_focus_and_capture_prefix` is deterministically red
  on the previous sequential implementation: two effects remain applied and focus/capture move to
  the new owner before the missing drag target is rejected.
- `input_transaction_drag_composite_failure_restores_all_mutated_domains` injects an active drag
  whose source has disappeared. The old reducer clears the session/capture before dirty marking
  rejects the missing source; the transaction restores the complete pre-effect state.
- `input_transaction_read_only_failure_restores_deferred_input_lifecycle` locks rollback of the
  implicit input write performed after every successful effect.
- `input_transaction_popup_prefix_restores_surface_and_route_trace` locks popup-stack rollback and
  verifies the post-abort route trace is projected from restored surface state.
- `input_transaction_single_effect_hot_path_captures_no_domains` locks the common-path admission
  rule without relying on unstable wall-clock timing.
- `rustfmt +1.94.1` and scoped `git diff --check` completed for the owned Rust changes.
- Focused Cargo tests and grouped external validation are pending. No Cargo pass is claimed.

## Remaining Scope

This closes the current source partial-commit fault and drag composite rollback gates. Public
operation ids, host acknowledgement/reconciliation states, stale host-result rejection, qualified
window/user/device identity, generation-bearing public receipts, unified pointer/default/a11y
committer migration, route allocation removal, and product-scale latency gates remain open under
the later Runtime77 milestones and P1/P2 items.
