# Runtime74 Binding Reload Transaction

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M6
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-binding-reload-transaction.md","docs/zircon_runtime/ui/template/asset/hot_reload_executor.md","zircon_runtime_interface/src/ui/component/value.rs","zircon_runtime_interface/src/ui/template/mod.rs","zircon_runtime_interface/src/ui/template/document.rs","zircon_runtime_interface/src/ui/template/asset/mod.rs","zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs","zircon_runtime/src/ui/surface/surface.rs","zircon_runtime/src/ui/template/mod.rs","zircon_runtime/src/ui/template/asset/binding_reload_transaction.rs","zircon_runtime/src/ui/template/asset/compiler/binding_program.rs","zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs","zircon_runtime/src/ui/template/asset/compiler/node_expander.rs","zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs","zircon_runtime/src/ui/template/asset/hot_reload_executor.rs","zircon_runtime/src/ui/template/asset/mod.rs","zircon_runtime/src/ui/template/asset/surface_index.rs","zircon_runtime/src/ui/tests/asset_binding.rs","zircon_runtime/src/ui/tests/asset_binding/compiled_program.rs","zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs","zircon_runtime/src/ui/tests/asset_surface_index.rs","zircon_runtime/src/ui/tests/asset_surface_index/binding_ownership.rs","zircon_runtime/src/ui/tests/asset_surface_index/binding_ownership_performance.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P1-048`
- Delivery state: implementation complete; grouped coordinator validation pending

## Problem

The existing two-phase surface executor could prepare all replacements and migrate component state,
but the compiled IR retained only the root asset id. Imported node and binding ownership disappeared
during expansion, the surface index could not replace generation-qualified ownership at publish,
and the rebuild receipt did not prove that old binding handles were no longer accepted.

## Scope Delivered

- Expanded template nodes retain a source asset plus an aligned source asset for each binding.
  Component and prototype expansion preserve component-owned bindings while assigning instance
  bindings to the caller asset.
- `UiCompiledAssetId` interns ownership in compiled IR. Each `UiCompiledNodeBindings` and
  `UiCompiledBinding` carries a dense owner id; legacy serialized programs without ownership fields
  fall back to the root asset through serde defaults.
- `UiAssetSurfaceIndex` owns forward and reverse compiled ownership for source assets, dense node
  ids, and generation-qualified binding handles. Re-recording a program removes the prior
  generation without deleting independent resource registrations for the same surface.
- `UiBindingReloadTransaction` validates replacement IR, stable root asset identity, and generation
  uniqueness before shared mutation. The existing executor remains the only prepare/publish owner.
- A valid compiled generation cannot be replaced by the default invalid program. Missing root
  identity, valid-to-invalid replacement, and non-finite compiled values fail before publication.
- Publication drops the retired surface, installs the new program ownership, and emits a
  `UiBindingQuiescenceReceipt` with old/new generation, retired/published binding counts,
  migrated/reset state counts, stale-handle rejection, and terminal quiescence.
- A same-program publication explicitly reports `old_generation_retired = false`; its still-current
  handles are not mislabeled stale or quiescent.
- The existing stable state key remains `(component, control_id)` with unique
  `(component, node_path)` fallback. Ambiguous, missing, or changed identities reset fail closed.

## Reference Evidence

- Bevy `dev/bevy/crates/bevy_asset/src/event.rs` separates Modified, Removed, and last-strong-handle
  Unused events; `dev/bevy/crates/bevy_asset/src/handle.rs` sends an explicit drop event from the
  final strong handle. Zircon keeps its generation-qualified value handles and makes retirement
  observable in a receipt rather than introducing reference-counted binding handles.
- Slint `dev/slint/internal/live-preview/live_component.rs` builds a replacement first, preserves
  the existing window, publishes only after creation succeeds, and then restores remembered
  properties/callbacks. Zircon follows the same prepare-before-publish order while migrating typed
  retained component state before publication.
- Godot `dev/godot/core/io/resource.cpp` and `dev/godot/core/io/resource_loader.cpp` preserve cached
  resource identity while copying replacement state. Zircon preserves `UiTreeId`/root asset
  identity but replaces immutable binding IR by generation, allowing stale handles to fail closed.

## Regression Contract

- Imported component nodes and bindings must retain their source asset; caller-authored bindings
  mounted on a component root must remain owned by the caller asset.
- A second prepare failure must preserve all last-known-good surfaces, cache contents, ownership
  edges, and generations.
- A successful generation change must reject the old handle, publish exactly one replacement
  handle for the asset, and report terminal quiescence.
- The existing 1,000-entry stable-state gate must report the same migration counts in both the
  surface rebuild receipt and binding quiescence receipt.
- The import regression must compile actual root/widget prototypes, retain caller/widget binding
  ownership, and preserve independently registered resource assets when ownership is replaced.

## Performance Contract

`compiled_binding_ownership_lookup_p95_beats_program_scan` prepares 4,096 bindings with 16 bindings
owned by the changed asset, then runs 21 alternating sample pairs with 128 lookups per sample. The
legacy path scans all 4,096 bindings per lookup; the indexed path walks the 16 exact targets and
performs zero full-program scans. Each query and result crosses an equal per-lookup `black_box`
barrier so release optimization cannot hoist the invariant work or bias only the scan path.

External validation must independently enforce 21 raw samples per side, 11 legacy-first and 10
indexed-first pairs, nearest-rank P95, the exact workload constants, and indexed P95 at least 50%
lower than scan P95. The child validator SHA-256 is
`BD5397EACB51F8ACD235A4CCF93945E746D8864E756A28E9B302625AA8F1A84D`; the 84-task / 56-Cargo-group /
18-performance-row super-batch SHA-256 is
`92059C051B5E6B4341AB2B93242CE201E3F408A88251F313CDA021D8E846A9CC`. Measured values remain
pending coordinator execution. No Cargo, behavior, or performance pass is claimed.

## Remaining Scope

This transaction retires value-owned binding programs and their surface-index ownership. External
route callbacks, model-provider subscriptions, asynchronous commands, and Editor/gameplay
asset-watch integration retain their separate Runtime74/Runtime11A/Runtime64 owners and require
their own lease/quiescence receipts.
