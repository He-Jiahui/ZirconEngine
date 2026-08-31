# Runtime74 Two-Phase Template Hot Reload Core

Plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
Milestone: M0
Status: validation_pending
Files: ["docs/plans/optimize/zircon_runtime/74/2026-08-22-two-phase-template-hot-reload.md","docs/zircon_runtime/ui/template/asset/hot_reload_executor.md","zircon_runtime/src/ui/surface/component_state.rs","zircon_runtime/src/ui/surface/surface.rs","zircon_runtime/src/ui/template/asset/hot_reload_executor.rs","zircon_runtime/src/ui/template/asset/mod.rs","zircon_runtime/src/ui/template/mod.rs","zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs"]

- Date: 2026-08-22
- Owner: `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`
- Source item: `RTB-P0-005` reusable transaction core
- Delivery state: core implementation complete; grouped coordinator validation pending

## Problem

`UiAssetHotReloadExecutor` copied template rebuild targets into a success report, evicted the compile
cache, and marked old surfaces dirty without building or publishing a replacement tree. A failed
later compile therefore had no transaction boundary, while a successful-looking receipt left the old
tree, state, bindings, and callbacks active.

## Scope Delivered

- `UiAssetSurfaceRebuilder` is the host-supplied prepare boundary for active template and removed
  artifact targets. Active rebuild work without a rebuilder fails closed.
- The executor clones only targeted surfaces, prepares every replacement, verifies retained
  `UiTreeId`, migrates state, and applies dirty marking entirely on a staged map.
- Compile-cache eviction, resource invalidation, theme publication, and retained-surface replacement
  occur only after all fallible preparation succeeds. One failed surface preserves every
  last-known-good surface and the compile cache.
- Compatible component state uses a unique `(component, control_id)` identity, falling back to a
  unique `(component, node_path)` when no control id exists. Missing, changed, or duplicate
  identities reset and are counted in the per-surface receipt.
- Persistent values, validation, references, and durable flags migrate for compatible state.
  Focused, focus-visible, hovered, pressed, dragging, drop-hovered, active-drag-target, popup,
  input, focus, and navigation state reset; window state is retained.
- `UiAssetTemplateRebuildReceipt` exposes affected assets plus migrated/reset counts so callers no
  longer infer rebuild success from a copied target list.

## Deterministic Scale Evidence

The scale gate builds one retained surface with 1,000 component states, prepares a replacement with
1,000 different node ids and the same stable control identities, then publishes once:

- retained state entries: `1,000`;
- compatible states migrated: `1,000`;
- incompatible states reset: `0`;
- staged surfaces: `1`;
- shared-surface publications before the prepare barrier: `0`;
- final shared-surface publications: `1`.

The runtime marker is
`PERF-RUNTIME74-HOT-RELOAD state_entries=1000 staged_surfaces=1 precommit_publications=0 published_surfaces=1 migrated_states=1000 reset_states=0`.
This is deterministic scale/operation evidence; no wall-clock speedup is claimed.

## TDD And Validation State

- `template_hot_reload_prepare_failure_preserves_last_known_good_surfaces_and_cache` covers a
  two-surface batch whose second prepare fails after the first succeeds locally.
- `template_hot_reload_atomically_publishes_replacements_and_migrates_compatible_state` covers
  replacement publication, component-identity compatibility, reset receipts, durable state, and
  transient-state cleanup.
- `hot_reload_state_migration_rejects_duplicate_stable_keys` locks fail-closed ambiguous identity.
- `template_hot_reload_migrates_one_thousand_stable_states_in_one_publication` provides the scale
  marker and state-integrity checks.
- Scoped `rustfmt` completed. Focused Cargo tests and grouped external validation are pending; no
  Cargo pass is claimed.

## Remaining Scope

This does not close parent `RTB-P0-005`. Runtime11A/64 still own production Editor/gameplay
asset-watch integration, real compiler/rebuilder selection, dependency-index refresh, callback and
model-subscription lease retirement, generation-qualified binding/node handles, and product-level
node/param/binding/event reload acceptance. This record only delivers the reusable two-phase
surface publication and compatible-state migration core.
