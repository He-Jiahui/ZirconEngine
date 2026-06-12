---
related_code:
  - zircon_runtime/src/core/framework/bridge.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/interface_id.rs
  - zircon_runtime/src/plugin/bridge/diagnostics.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/framework/bridge.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/interface_id.rs
  - zircon_runtime/src/plugin/bridge/diagnostics.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/extension_registry/ownership.rs
  - zircon_runtime/src/plugin/runtime_plugin/lifecycle_context.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_dependencies.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/diagnostics.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/reports.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/registration/plugin.rs
plan_sources:
  - docs/plans/zircon_plugins/index.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs
---

# Plugin Bridge

The plugin bridge is the runtime-owned interface table for typed plugin-to-plugin calls. It is the 11-M1 slice of the `zircon_plugins` architecture plan: plugins can export a typed interface during extension registration, consumers can resolve strong or weak handles during finish, and the runtime keeps owner information for later unload or hot-reload work.

The bridge is intentionally separate from managers. Managers remain engine-owned global services; plugin-to-plugin synchronous calls go through bridge interfaces so dependencies, optional providers, and future hot reload behavior stay observable to the plugin system.

## Contract Layer

`zircon_runtime::core::framework::bridge::PluginInterface` is the neutral marker trait. Interface traits are defined as framework contracts and implement `PluginInterface` on the trait object, for example `impl PluginInterface for dyn PhysicsQueryInterface`.

Each interface has a stable `INTERFACE_ID` such as `physics.query.v1`. Breaking changes must use a new id. Implementations must be `Send + Sync + 'static` and must not access `World` directly; world mutation belongs in scheduled systems where `SystemParamAccess` can participate in conflict detection.

`BridgeError::Absent` is the weak-call runtime status for an interface that was not installed in the frozen bridge table. `BridgeError::NotEnabled` is reserved for an installed provider that is disabled or temporarily between generations.

## Runtime Table

`RuntimeExtensionRegistry::export_interface::<T>(owner, Arc<T>)` registers one owner-tracked export per interface id. Duplicate exports return `DuplicatePluginInterface`. `RuntimeExtensionRegistry::frozen_bridge_table()` materializes a `FrozenBridgeTable` snapshot with dense `InterfaceSlot` entries.

`FrozenBridgeTable` owns bridge entries behind shared `Arc` state, so weak bridges can keep a stable table snapshot after finish-time resolution. Each entry stores the interface id, owner module, erased provider, and generation counter. Even generations are enabled, odd generations are disabled. `interface_status(...)` reports `Absent`, `Enabled`, or `Disabled` for single-interface diagnostics, and `interface_snapshot(slot)` / `interface_snapshot_by_id(interface_id)` return one stable bridge-matrix row when the target exists. `interface_snapshots()` returns all rows: dense slot, interface id, owner module id, generation, provider-installed flag, status, and diagnostics snapshot. `interface_snapshots_owned_by(owner)` filters the same row model to one runtime module owner, which lets lifecycle code and editor diagnostics preview exactly which bridge exports a module owns without reading table internals. `set_enabled(...)`, `set_owner_enabled(...)`, `activate_owner(...)`, `deactivate_owner(...)`, `replace_provider(...)`, and `reload_provider(...)` are the current hooks for the planned activate/deactivate/hot-reload wiring. Owner-level flips let lifecycle code toggle every interface exported by one runtime module in a single bridge-table operation, while `deactivate_owner(...)` also clears provider references. The `*_with_report(...)` owner-level variants return `BridgeOwnerTransitionReport`, carrying a `BridgeOwnerTransitionMode` (`Activate`, `Disable`, or `Deactivate`), affected slots, and post-operation snapshots for lifecycle diagnostics. `BridgeOwnerTransitionReport::diagnostic()` renders a stable bridge.owner_transition text row that includes the owner slot, mode, affected count, interface ids, slot ids, generation, provider-installed state, and final status. The provider-installed flag lets diagnostics distinguish disabled-but-still-installed entries from deactivated entries whose provider reference has been cleared. Re-activation after deactivation requires providers to be installed again before `activate_owner(...)` can make weak calls succeed. Reloading a provider while enabled advances the generation by two so weak bridges refresh without changing enabled state; replacing while disabled waits for the later activation flip to publish the new provider.

## Strong And Weak Bridges

`StrongBridge<T>` is a direct `Arc<T>` wrapper with `Deref`, matching the strong-dependency hot path: after finish-time validation, calls are ordinary trait-object calls.

`WeakBridge<T>` stores an `InterfaceSlot`, a cloned frozen table, and a cached `(generation, Arc<T>)`. `call(...)` returns `BridgeError::Absent` when the interface was never installed in the frozen table and `BridgeError::NotEnabled` when the installed provider is disabled. When the generation matches the cached provider, the call path reuses the downcast provider; when the generation changes, the bridge refreshes from the table. `pin()` returns `BridgeGuard<T>` so a system body can resolve once and issue repeated calls through the same target.

`PluginFinishContext::resolve_strong::<T>()` and `resolve_weak::<T>()` are the lifecycle-facing entry points.

Linked Rust plugin registration now validates the export side of the bridge manifest contract. If a package declares `provides_interfaces`, at least one runtime module from that package must call `export_interface(...)` for that id during registration. If registration exports an interface id that the package manifest did not declare, the registration report records a diagnostic. This keeps plugin.toml from drifting away from the actual bridge table before catalog merging or future editor bridge matrices consume the data.

`RuntimePluginCatalog` now validates required bridge interface dependency closure when registration reports are merged or when plugins are registered incrementally. A dependency row with `required = true` and `interfaces = [...]` must point at a registered provider package whose manifest declares each requested interface. Optional rows remain non-blocking. Missing providers or missing interface declarations produce `bridge.strong_dependency_missing` diagnostics that include the dependency chain, for example `weather -> physics -> scene`.

The catalog also exposes `strong_bridge_dependents(provider_package_id)`. It returns the packages that declare required bridge interface dependencies on the provider, with the requested interface ids grouped per dependent. `strong_bridge_disable_blockers(provider_package_id)` projects the same data into explicit blocker rows so the planned M3 runtime path can reject disabling a strong dependency target while dependents are active. Each blocker can render a stable `bridge.strong_target_disable_blocked` diagnostic that names the provider, dependent, and required interfaces.

In debug builds, each bridge entry also records lightweight weak-bridge diagnostics. `WeakBridge::call(...)` and `WeakBridge::pin()` increment enabled-call or not-enabled counters on the target slot, and `FrozenBridgeTable::diagnostics(slot)` returns a `BridgeDiagnosticsSnapshot`. `BridgeInterfaceSnapshot` embeds the same counter snapshot so future editor and runtime diagnostic consumers can render the table without reading bridge internals. Release builds keep the same public snapshot API but compile the counters to zero so the future editor bridge matrix can be wired without adding runtime cost to release builds.

## Current Scope And Remaining Work

Completed in this slice:

- Framework bridge contract and `BridgeError`.
- Runtime plugin bridge table, dense `InterfaceSlot`, strong bridge, weak bridge, and guard.
- Explicit bridge generation parity contract: even generations are enabled and odd generations are disabled.
- Weak bridge errors distinguish absent interfaces from installed-but-disabled providers.
- Owner-tracked `export_interface(...)` registry API.
- `PluginFinishContext` strong/weak resolution helpers.
- Owner ownership/revocation accounting includes plugin interface slots.
- plugin.toml `provides_interfaces` rows, dependency `interfaces` rows, interface-only dependency rows, and package/static manifest namespace validation.
- Linked Rust plugin registration diagnostics for declared-but-unexported and exported-but-undeclared bridge interfaces.
- Catalog-level required bridge dependency closure diagnostics with dependency chains.
- Catalog-level strong bridge dependent and disable-blocker lookup for future disable rejection and editor bridge matrices.
- Owner-level bridge table generation flips for future activate/deactivate lifecycle wiring.
- Owner-level bridge table deactivate path that disables entries and clears provider references.
- Enabled-state bridge provider reload that refreshes weak bridges without caller rewiring.
- Per-interface bridge status query for diagnostics and future editor bridge matrices.
- Stable bridge interface snapshots for future diagnostics/editor bridge matrices, including owner-filtered snapshot projection and owner transition reports.
- Debug-only weak-bridge enabled/not-enabled call counters exposed as per-slot snapshots.

Still pending from the plan:

- Event dense channels and dormant subscription model.
- Runtime lifecycle integration that calls bridge activate/deactivate at frame boundaries.
- Native/VM ABI bridge calls and editor diagnostics.

## Validation Evidence

Fresh 2026-06-12 bridge-core evidence: `cargo test -p zircon_runtime --lib extension_registry_bridge --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never -- --test-threads=1` passes 6 bridge tests. `cargo test -p zircon_runtime --lib runtime_plugin_package_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never -- --test-threads=1` passed 32 package-manifest validation tests before the final export-consistency diagnostics were added. After the export-consistency diagnostics were added, fresh `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never` is currently blocked by an unrelated UI compile error in `zircon_runtime/src/ui/component/state_reducer/button.rs:8` (`UiComponentEvent` does not implement `Eq`).

The bridge dependency-closure follow-up adds `runtime_plugin_bridge_dependencies.rs` coverage for missing required interface providers, present required providers, optional missing providers, and transitive dependency-chain diagnostics. `rustfmt` passed for the touched bridge catalog and test files. Focused `cargo test -p zircon_runtime --lib runtime_plugin_bridge_dependencies --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never -- --test-threads=1` was attempted twice on 2026-06-12 but timed out during lib-test compilation while unrelated UI/render Cargo jobs were active; the bridge test process tree was stopped and no test pass is claimed for this new slice yet.

The dependent-lookup follow-up adds `RuntimePluginBridgeDependent`, `RuntimePluginBridgeDisableBlocker`, `RuntimePluginCatalog::strong_bridge_dependents(...)`, and `RuntimePluginCatalog::strong_bridge_disable_blockers(...)`. `runtime_plugin_catalog_lists_strong_bridge_dependents_for_disable_checks` covers required-only filtering, optional dependency exclusion, interface id deduplication, and deterministic ordering. `runtime_plugin_catalog_reports_strong_bridge_disable_blockers` covers the disable-blocker projection, stable `bridge.strong_target_disable_blocked` diagnostic text, and empty result for a provider with no strong dependents.

The bridge diagnostics follow-up adds debug-only enabled/not-enabled counters and `weak_bridge_records_debug_diagnostics` coverage. The same pass also adds `generation_parity_encodes_enabled_state` coverage for the bridge generation protocol. A later bridge-error follow-up changes the absent-provider test to `weak_call_returns_absent_when_target_not_installed` and reserves `NotEnabled` for installed disabled providers. `rustfmt` passed for the touched bridge files. Focused runtime test execution is deferred while unrelated runtime Cargo jobs are active in this workspace; no pass is claimed for these new tests yet.

The owner-flip follow-up adds `FrozenBridgeTable::set_owner_enabled(...)` and `bridge_table_flips_all_interfaces_owned_by_plugin_module`, covering batch generation flips for two interfaces owned by the same runtime module while another module's interface remains enabled. The activate/deactivate follow-up adds `FrozenBridgeTable::activate_owner(...)`, `FrozenBridgeTable::deactivate_owner(...)`, and `bridge_table_deactivates_owner_by_disabling_and_clearing_providers`, covering the M3 bridge-table invariant that deactivation disables entries and clears providers, disabled-state provider replacement does not publish a new generation until activation, and re-activation only reconnects weak bridges after providers are replaced; lifecycle integration remains pending.

The hot-reload follow-up adds `FrozenBridgeTable::reload_provider(...)` and `hot_reload_swaps_provider_without_caller_rewiring`, covering enabled-state provider replacement that advances generation by two and lets an existing weak bridge observe the new provider without being resolved again.

The status follow-up adds `BridgeInterfaceStatus`, `FrozenBridgeTable::interface_status(...)`, and `bridge_table_reports_interface_status_for_diagnostics`, covering absent, enabled, and disabled interface states for diagnostics.

The bridge-matrix snapshot follow-up adds `BridgeInterfaceSnapshot`, `FrozenBridgeTable::interface_snapshot(...)`, `FrozenBridgeTable::interface_snapshot_by_id(...)`, `FrozenBridgeTable::interface_snapshots()`, `FrozenBridgeTable::interface_snapshots_owned_by(...)`, `BridgeOwnerTransitionMode`, `BridgeOwnerTransitionReport`, `BridgeOwnerTransitionReport::diagnostic()`, `FrozenBridgeTable` owner transition report variants, `bridge_table_resolves_single_interface_snapshot`, `bridge_table_snapshots_interfaces_for_diagnostics_matrix`, `bridge_table_filters_interface_snapshots_by_owner`, `bridge_table_reports_owner_enabled_transition`, and `bridge_table_reports_owner_deactivation_transition`. The tests cover deterministic slot order, single-row lookup, interface ids, owner module ids, generations, provider-installed state, enabled/disabled status, debug counter projection, owner filtering, explicit transition modes, stable diagnostic text, and post-transition reports for future lifecycle diagnostics/editor consumers.
