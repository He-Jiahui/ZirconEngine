# Plugins03 phantom authoring surface removal record

- Date: 2026-08-21
- Owner plan: `docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md`
- Finding: `PLUGIN-DESKTOP-WINDOW-P0-004`
- Status: `static_validation_complete`; managed release Cargo batch queued

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

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`;
  ownership apply `82dfdbab6bc444c582224ab9284307a6`, fingerprint
  `13514f0c9245f53af8b16e987803b8cb341b7af901d4817464325a247fbed13f`.
- Current `plugin.rs` SHA-256:
  `F4722C493CF94D9E1E29207CC5C1138066B5AE9C65F4646F8467794E92AC3A94`;
  formatted `lib.rs` and `tests.rs` SHA-256 are
  `E1AB7F79E95A2AF0842D065CD419AE3BA5B1C66F33B5C08F7AC653CBC87844DA`
  and `0B9F3CC6853EE60131518770611188D4EB675545DC9350E1F270CEAC2606F8D2`.
- Unified deterministic model manifest SHA-256:
  `17A9DACBE245A8562CD994DCC61423E4061BB1A7F264B281EAC9F9FF4AB85719`.
  It records phantom contributions `8,000 -> 0` and missing-template
  resolutions `1,000 -> 0`, both exact `100%` reductions.
- Focused source/model/validator contract passed locally `10/10`; managed
  static ticket `b1ba67ff1d2d43c98abfa21c4b14dee8` is queued.
- Cross-package Windows release ticket
  `66d397b6a4454a68b53bb295a5c4fe78` runs the full feature-enabled editor lib
  suite and marker in the same validation copy as Plugins01 and Plugins09.

## Validation

- The package regression asserts successful capability/package registration with zero views, drawers, UI templates, menu items, or commands.
- The deterministic scale regression emits and locks the 1,000-registration contribution and template-resolution reductions.
- Cargo compilation, all package tests, and release marker validation are
  queued in one multi-task Plugins aggregate; no standalone Cargo run or
  measured pass is claimed here.

## Remaining Plan Work

- This slice closes the nonexistent authoring-resource branch of `PLUGIN-DESKTOP-WINDOW-P0-004`.
- A real native-window backend/provider, provider-qualified capability health, source/dist executable contribution parity, and package enable/disable lifecycle remain separate Plugins03 findings.
- Desktop Export operation/profile authority is unchanged and remains `PLUGIN-DESKTOP-WINDOW-P0-003` work.
