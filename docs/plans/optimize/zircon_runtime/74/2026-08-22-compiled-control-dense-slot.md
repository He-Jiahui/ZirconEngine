# Runtime74 Compiled Control Dense Slot

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M4
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-compiled-control-dense-slot.md","docs/zircon_runtime/ui/surface/binding_targets.md","zircon_runtime/src/ui/surface/binding_targets.rs","zircon_runtime/src/ui/surface/control_index.rs","zircon_runtime/src/ui/surface/surface.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-044`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

Compiled `ControlProperty` expressions retained a dense `UiCompiledControlId`, but evaluation
decoded that ID back to a control name and queried the string-keyed `UiSurfaceControlIndex` on every
access. Large control tables therefore paid a logarithmic string comparison path after compilation
had already established the control identity.

## Scope Delivered

- `UiCompiledBindingProgram` exposes its control count and ordered control names without cloning.
- Program installation derives a generation-qualified dense control slot table from the existing
  uniqueness index. Duplicate controls resolve to `None`, and tracked insert, remove, or rename
  mutations refresh affected slots incrementally.
- A generation mismatch or absent deserialized cache rebuilds the dense slots lazily. A final
  node/control-name check rejects a stale positive entry.
- Compiled `ControlProperty` evaluation resolves the node by `UiCompiledControlId` and performs no
  runtime string-index lookup before reading the property.
- Regression coverage locks duplicate introduction/removal and compiled generation changes.
- The child validator is integrated into the Runtime74 superbatch, which now contains 62 tasks in
  31 Cargo groups and fourteen independent performance rows.

## Performance Contract

`compiled_control_dense_slot_p95_beats_string_index_lookup` runs 21 alternating sample pairs. Each
sample performs 8,192 lookups over 2,048 unique controls. The legacy side queries the string-keyed
index for every lookup; the optimized side reads the installed dense slot.

The release marker emits both raw sample arrays and nearest-rank P95 values. External validation
must independently enforce:

- exactly 21 samples per side and 11 legacy-first / 10 optimized-first pairs;
- 8,192 legacy string-index lookups and zero optimized string-index lookups per sample;
- optimized nearest-rank P95 at least 25% lower than the legacy lookup.

Measured values remain pending coordinator execution. No Cargo or performance pass is claimed.

## Remaining Scope

The public component-event envelope still carries owned binding and control strings. Binding mode,
model subscription, frame safe-point batching, and typed command admission remain separate
Runtime74 items.
