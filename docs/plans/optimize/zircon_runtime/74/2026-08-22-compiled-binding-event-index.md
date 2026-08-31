# Runtime74 Compiled Binding Event Index

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-binding-event-index.md","docs/plans/optimize/zircon_runtime/74/2026-08-22-direct-compiled-event-dispatch.md","docs/zircon_runtime/ui/surface/binding_targets.md","docs/zircon_runtime/ui/surface/default_interactions.md","zircon_runtime/src/ui/surface/surface.rs","zircon_runtime/src/ui/surface/surface/compiled_binding_event_index.rs","zircon_runtime/src/ui/surface/surface/default_interactions.rs","zircon_runtime/src/ui/surface/surface/pointer_component_events.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/event_routing/component_events.rs","zircon_runtime/src/ui/tests/event_routing/component_events/performance.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-045`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The compiled program already mapped a node/source slot directly to a dense binding handle, but
pointer and default-interaction delivery still scanned every authored binding on the node for each
event kind. A node with many mixed bindings therefore paid `O(all bindings)` comparisons before it
could use the compiled handle.

## Scope Delivered

- `UiSurface` builds a derived `(UiNodeId, UiEventKind)` index when a compiled binding program is
  installed. Entries retain compiler/source order and carry the source slot, dense handle, and
  compiled typed component-event identity.
- Pointer component events and typed default interactions visit only indexed matches. A matching
  program generation makes the dense handle authoritative, so dispatch no longer resolves the
  authored source slot, compares the authored binding name, or revalidates the same handle.
- `UiCompiledBinding::component_event` is optional with a serde default. Older artifacts therefore
  remain readable and mean no typed component-event selector. The derived index itself remains
  unserialized; missing or stale state falls back to the previous authored scan.
- A sparse-order regression uses 64 bindings and requires two Click bindings to publish in authored
  order through the installed index.
- Compiler/index regressions retain `OpenPopup`, and a drift regression changes authored id/event
  metadata after program installation while requiring publication under the compiled identity.

## Performance Contract

`compiled_binding_event_index_p95_beats_authored_binding_scan` runs 21 alternating sample pairs.
Each sample performs 4,096 Click lookups over 256 bindings spread evenly across all 16 event kinds.
Both paths consume the same 16 matches. The legacy side probes all 256 bindings per lookup; the
indexed side probes only 16, reducing deterministic binding probes by 93% when rounded down.

The release marker emits both raw sample arrays and nearest-rank P95 values. External validation
must independently enforce:

- exactly 21 samples per side and 11 legacy-first / 10 optimized-first pairs;
- 256 legacy probes and 16 indexed probes per lookup;
- optimized nearest-rank P95 at least 50% lower than the authored scan.

Measured values remain pending coordinator execution. No Cargo or performance pass is claimed.

`direct_compiled_event_handle_p95_beats_index_revalidation` separately compares the previous
indexed-but-revalidated delivery with direct dense-handle delivery. It runs 21 alternating pairs,
4,096 lookups per sample, and 128 indexed matches per lookup. External validation must recompute
nearest-rank P95 from both raw arrays, require at least 50% lower P95, and verify that handle
revalidations and binding-name comparisons both fall from 128 per lookup to zero.

## Remaining Scope

The public event envelope still owns binding/control strings, and compiled event delivery still
constructs those strings at the runtime/editor boundary. Binding mode, model subscription,
safe-point batching, and typed command admission remain separate Runtime74 items.
