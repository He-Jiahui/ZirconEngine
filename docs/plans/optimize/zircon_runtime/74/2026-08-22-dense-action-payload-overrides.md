# Runtime74 Dense Action Payload Override Handoff

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M1
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-action-payload-program.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-dense-action-payload-overrides.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-015`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Atomic action-payload targets already resolved their values before mutation, but the handoff used a
`BTreeMap<String, UiValue>`. Preparation allocated and copied every field name, then invocation
construction looked it up by string and cloned the resolved value a second time. The compiled
program already owns a dense `UiPropertyId` for the same field.

## Scope Delivered

- Prepared overrides are keyed by `UiPropertyId`; target validation remains the authority that the
  ID exists in the installed compiled binding program.
- Invocation construction consumes the prepared override map and removes each value by dense ID.
  The resolved `UiValue` moves directly into the final public string-keyed payload.
- Final invocation payload shape and lexical field ordering remain unchanged. Unknown or stale
  compiled handles and unavailable payload expressions continue to fail closed.
- Target reporting still owns its own resolved value because it records previous/new values after
  commit. This slice removes only the redundant handoff copy, not report evidence.

## Performance Contract

`dense_action_payload_override_handoff_p95_beats_string_clone_handoff` runs 21 alternating sample
pairs. Each sample creates 256 action invocations with 16 overridden fields and 1,024-byte string
values. Input maps are prepared outside the timed region. The legacy side performs string lookup
and clones 16 values per invocation; the optimized side removes dense-ID entries and moves all 16
values. External validation must independently recompute nearest-rank P95 and enforce:

- optimized P95 is at least 20% lower than the string-keyed clone baseline;
- handoff value clones fall from 16 per invocation to 0, a 100% reduction;
- sample count is exactly 21 and order alternates 11 legacy-first / 10 optimized-first.

Measured values remain pending coordinator execution; no Cargo or performance pass is claimed.

## Regression Contract

- A two-field target test authors targets in reverse field order and requires both assigned values
  to publish under their original public field names.
- Existing compiled-payload, atomic transaction, stale endpoint, default interaction, and pointer
  event tests continue through the same invocation owner.

## Remaining Scope

The final public invocation remains `BTreeMap<String, UiValue>`, so field-name allocation at the
runtime/editor dispatch boundary is unchanged. Removing that allocation requires a typed public
payload representation and downstream editor/gameplay consumer migration.
