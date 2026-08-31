# Runtime74 Single-Binding Event Payload Move

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-single-binding-event-payload-move.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-045`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Pointer component-event dispatch accepted an owned `UiComponentEvent`, but the per-binding helper
borrowed and cloned that value for every matching binding. The dominant one-binding path therefore
allocated and copied large typed payloads after the caller had already transferred ownership.

## Scope Delivered

- Matching bindings remain in compiled or authored order and are exposed through a peekable
  iterator.
- Non-final bindings receive the minimum required clone; the final or only binding takes the
  retained original payload.
- The public serialized component-event envelope is unchanged.
- Regression coverage locks heap-allocation identity for one binding and locks clone-then-move
  behavior for two bindings.
- A release-only benchmark compares the previous clone-on-every-match path with the owned handoff.
- The benchmark child is integrated into the Runtime74 superbatch, which now contains 62 tasks in
  31 Cargo groups and fourteen independent performance rows. The existing component-event group
  owns the behavior regression, so the release child does not rerun it.
- External validator SHA-256: `589591D2B080F722F9E2973AA33AC75AA57E5E24135680AB8F9E82CBCAC4A24C`.
- Superbatch validator SHA-256: `2E2EEF2CDB8F300197B65EBCF1B8F8521A6742AEB99357DFE25EE8B179DD5245`.

## Performance Contract

`single_binding_event_payload_move_p95_beats_clone` runs 21 alternating sample pairs. Each sample
emits 2,048 one-binding events whose keyboard-text payload is 4,096 bytes. Workload allocation is
outside the timed region; the timed region includes event-envelope construction and the legacy
payload clone or optimized payload move.

The release marker emits both raw sample arrays and nearest-rank P95 values. External validation
must independently enforce:

- exactly 21 samples per side and 11 legacy-first / 10 optimized-first pairs;
- 2,048 legacy payload clones and zero optimized payload clones per sample;
- optimized nearest-rank P95 at least 20% lower than the legacy path.

Measured values remain pending coordinator execution. No Cargo, behavior, or performance pass is
claimed.

## Validation Note

The preceding 61-task grouped ticket reached the Cargo workspace loader but failed before
compilation because its isolated copy omitted the external `E:/Git/zr_vm` path dependency. The next
grouped ticket must materialize that external dependency closure; this is not recorded as a source
test failure.

## Remaining Scope

Multi-binding dispatch still requires one payload value per published envelope, so `N - 1` clones
remain necessary for `N` matches. Binding modes, model subscriptions, frame safe-point batching,
and typed command admission remain separate Runtime74 items.
