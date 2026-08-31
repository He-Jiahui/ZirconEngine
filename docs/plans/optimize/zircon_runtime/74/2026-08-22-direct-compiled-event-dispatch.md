# Runtime74 Direct Compiled Event Dispatch

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-binding-event-index.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-direct-compiled-event-dispatch.md","docs/zircon_runtime/ui/surface/binding_targets.md","docs/zircon_runtime/ui/surface/default_interactions.md","tools/cargo-zircon/src/main.rs","zircon_runtime/src/ui/surface/surface/compiled_binding_event_index.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-045`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The derived event index already rejected a mismatched compiled-program generation, but every hit
still returned to authored node metadata. Pointer delivery loaded the source slot, checked the raw
event, resolved the same dense handle, and compared the raw binding name. Typed default delivery
also read the authored `component_event`. This duplicated work and let post-install authored drift
change a compiled dispatch decision.

## Scope Delivered

- `UiCompiledBinding` retains the optional typed `UiComponentEventKind`; serde defaulting keeps
  older artifacts readable and maps the absent field to no typed selector.
- The derived event index copies the typed selector beside its source slot and dense handle while
  preserving compiler order and program generation.
- Pointer delivery obtains the public binding identity from the compiled string table and publishes
  through the indexed handle directly.
- Default interaction delivery filters by the indexed typed selector and publishes through the same
  direct handle. It no longer reads authored binding id, event, or component-event identity on the
  valid compiled path.
- Surfaces without a valid compiled index retain the authored scan and existing compatibility
  behavior.

## Regression Contract

- The compiler must cook `OpenPopup` into the compiled binding and the derived index must retain it.
- Two sparse Click bindings must preserve authored order.
- Mutating the retained authored binding id and event after program installation must not alter the
  compiled event identity or suppress its dispatch.
- The existing component-event batch runs the three dispatch regressions together. The new child
  adds two debug groups for the runner and compiled-artifact contracts plus one release performance
  group, so no behavior test receives its own Cargo process.
- `cargo-zircon validation-batch` is a restricted coordinator entry point: it accepts exactly one
  `zircon-validation-*.ps1` script, fixes `SourceRoot` to the materialized checkout, and propagates
  the child exit code. Nested Cargo groups use a `validation-batch` child of the coordinator target
  directory so the waiting parent runner cannot retain the same target lock. Its argument contract
  is covered in the same grouped child validator.

## Performance Contract

`direct_compiled_event_handle_p95_beats_index_revalidation` runs 21 alternating sample pairs. Each
sample performs 4,096 lookups and consumes 128 indexed matches per lookup. The legacy side resolves
the authored source slot, validates the handle, and compares the binding name for every match. The
optimized side consumes the generation-pinned dense handle directly.

External validation must independently enforce:

- exactly 21 raw samples per side with 11 legacy-first and 10 optimized-first pairs;
- handle revalidations per lookup fall from 128 to 0;
- binding-name comparisons per lookup fall from 128 to 0;
- nearest-rank optimized P95 is at least 50% lower than the revalidated baseline.

The standalone validator is
`.codex/state/session-coordinator/cargo-runs/zircon-validation-runtime74-direct-compiled-event-handle.ps1`
with SHA-256 `739E6B1686CE45232E7402D8F4E47DEA44B7BDFF56C480205C2F25583393FD5A`.
It is pinned by the 63-task / 34-Cargo-group / 15-performance-row Runtime74 super-batch with
SHA-256 `751B04E167FE72995BF9811634C290FE389C56188797FE35E7A9C0738FB3C03C`.

Measured values remain pending coordinator execution. No Cargo or performance pass is claimed.

## Validation History

Ticket `2c86300c5d18429aad01897d3d4e2f34` covered the previous 62-task source snapshot but failed before
workspace loading, compilation, or tests. Cargo received the alias configuration with literal quote
characters and reported `no such command: runtime74-superbatch` (exit 101). This is infrastructure
evidence only: zero behavior tests and zero performance rows ran, so it is not acceptance evidence
for either the prior snapshot or this slice.

## Remaining Scope

Public event envelopes still allocate binding/control strings. Removing those allocations requires
a typed public dispatch payload and downstream Editor/gameplay consumer migration. Binding modes,
model subscriptions, safe-point batching, and typed command admission remain separate Runtime74
items.
