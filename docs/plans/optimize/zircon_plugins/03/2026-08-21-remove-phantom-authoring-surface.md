# Plugins03 phantom authoring surface removal record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md`
- Finding: `PLUGIN-DESKTOP-WINDOW-P0-004`
- Status: `validation_pending`

## Scope

- Remove the Native Window source registration for two Editor-core-owned views, their duplicate commands/menu items, the plugin drawer, and the nonexistent `plugins://native_window_hosting/editor/authoring.zui` template.
- Delete the obsolete extension identifiers and the editor-support dependency used only by that registration path.
- Keep package metadata, the experimental capability declaration, and the existing distribution contract unchanged.

## Contract

- `native_window_hosting` does not publish Editor authoring contributions until it owns a materializable resource bundle or a probeable native-window provider.
- Workbench and Prefab window surfaces remain owned by the Editor core; the plugin does not duplicate their view or command identities.
- Source registration and the existing dist projection now both publish zero Editor extension contributions instead of disagreeing about a nonexistent source-only surface.

## Performance Gate

- The release workload creates 1,000 source registration reports.
- The previous path allocated and registered 8 contributions per report: 2 views, 2 commands, 2 menu items, 1 drawer, and 1 UI template. The current path publishes zero, reducing phantom contribution work from 8,000 to 0 (100%).
- The previous path also queued 1,000 resolutions of a physically absent template URI; the current path queues zero (100% reduction).
- The release marker is `PERF-MVP-PLUGINS03-NO-PHANTOM-AUTHORING`.

## Validation

- The package regression asserts successful capability/package registration with zero views, drawers, UI templates, menu items, or commands.
- The deterministic scale regression emits and locks the 1,000-registration contribution and template-resolution reductions.
- Cargo compilation, all package tests, and release marker validation are queued in the multi-task Plugins aggregate; no standalone Cargo run or measured pass is claimed here.

## Remaining Plan Work

- This slice closes the nonexistent authoring-resource branch of `PLUGIN-DESKTOP-WINDOW-P0-004`.
- A real native-window backend/provider, provider-qualified capability health, source/dist executable contribution parity, and package enable/disable lifecycle remain separate Plugins03 findings.
- Desktop Export operation/profile authority is unchanged and remains `PLUGIN-DESKTOP-WINDOW-P0-003` work.
