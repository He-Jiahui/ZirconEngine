# Runtime74 Atomic Binding Target Executor Core

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-atomic-binding-target-executor.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/mod.rs","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/surface/event_routing.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P0-001` reusable transaction core
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

`UiBindingTargetAssignment` survived validation and template compilation, but pointer component
dispatch only built an optional action invocation. Property, class, visibility, enabled, and action
payload targets were silently ignored, so accepted assets could report runtime binding capability
without applying their declared effects.

## Scope Delivered

- Pointer component events now enter one target-execution boundary after default interactions and
  focus event collection, before dispatch publication.
- Every expression and target shape is prepared before mutation. Invalid expressions, unresolved
  values, wrong boolean kinds, unnamed targets, or missing payload fields reject and suppress only
  the affected binding event.
- A target-bearing binding executes through `UiBindingMutationTransaction`. Property, class,
  visibility, enabled, and action-payload updates publish together only after all mutations
  succeed; rejection restores the five captured write-domain groups before publication. This
  supersedes the original whole-surface staging implementation; see
  `2026-08-22-binding-mutation-transaction.md`.
- Property, visibility, and enabled targets reuse the authoritative property mutation path. Class
  targets invoke the existing runtime style owner and report style/layout/hit-test/render/text/input
  dirtiness. Payload targets override the current action invocation without mutating the asset.
- Reports retain binding identity, previous/new values, update status, and existing dirty-domain
  receipts. Target order is the serialized assignment order.
- Candidate bindings are scanned once into a node/event/id index. Only target-bearing bindings are
  cloned, and a completely unassigned batch returns without draining events or cloning a surface.

## Deterministic Performance Evidence

The scale gate dispatches one click across 1,000 matching bindings that contain no targets:

- candidate bindings and index scans: `1,000` / `1,000`;
- target-bearing binding clones: `0`;
- staged surface clones: `0`;
- binding reports: `0`;
- retained component events: `1,000`.

The runtime marker is
`PERF-RUNTIME74-BINDING-TARGET candidate_bindings=1000 binding_index_scans=1000 target_bindings=0 target_binding_clones=0 staged_surface_clones=0 binding_reports=0`.
This is deterministic operation-count evidence for the empty-target hot path; no wall-clock speedup
is claimed.

## TDD And Validation State

- `pointer_binding_targets_commit_atomically_and_override_action_payload` covers all five target
  kinds, action payload replacement, mutation reports, committed receipt, and final retained state.
- `pointer_binding_target_commit_rejection_rolls_back_prior_target` covers a commit-stage rejection
  after an earlier target mutation and requires a rolled-back receipt with restored state.
- `pointer_binding_target_prepare_failure_rolls_back_and_suppresses_event` covers multi-target
  prepare failure, event suppression, and preservation of the prior state.
- `pointer_binding_target_fast_path_skips_staging_for_one_thousand_unassigned_bindings` locks the
  linear index scan and zero-clone/staging fast path.
- Focused Cargo tests and grouped external validation are pending. No Cargo pass is claimed.

## Remaining Scope

This does not close parent `RTB-P0-001`. Cooked binding instructions, typed/generation-qualified
endpoints, component-instance-relative control lookup, safe-point batching, complete dirty domains,
non-pointer event sources, and shared Editor/gameplay product integration remain open. The current
delivery supplies a reusable atomic target mutation core on the existing pointer component path.
