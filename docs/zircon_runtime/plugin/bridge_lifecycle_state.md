---
related_code:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs
  - zircon_runtime/src/core/runtime/tests/plugin.rs
implementation_files:
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/error.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/mod.rs
  - zircon_runtime/src/plugin/mod.rs
plan_sources:
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs
  - zircon_runtime/src/core/runtime/tests/plugin.rs
---

# Bridge Lifecycle State

`RuntimePluginBridgeLifecycleState` is the runtime-owned bundle for applying provider lifecycle events to plugin bridge rows. It keeps three values together:

- the `RuntimePluginCatalog`, which knows provider packages, runtime module names, and strong bridge dependents,
- the final `RuntimeExtensionCatalogReport`, whose registry contains the merged `InterfaceExport` rows,
- one `FrozenBridgeTable` built from that final registry.

This prevents frame-boundary callers from recomputing or mixing bridge tables. A weak bridge resolved from `state.bridge_table()` observes the same generation changes produced by `disable_provider_at_frame_boundary(...)`, `activate_provider_at_frame_boundary(...)`, `deactivate_provider_at_frame_boundary(...)`, and `reload_provider_at_frame_boundary(...)`.

`RuntimePluginBridgeLifecycleEvent` is the event-shaped entry point for CoreRuntime/plugin lifecycle wiring. It carries a provider package id plus `BridgeOwnerTransitionMode`, and `RuntimePluginBridgeLifecycleState::apply_provider_lifecycle_event(...)` dispatches it to activate, disable, deactivate, or reload. The return value is `RuntimePluginBridgeLifecycleOutcome`: `Applied(report)` for a successful transition, or `Blocked(error)` when a strong dependent prevents disable/deactivate. Both variants expose stable diagnostic text through `diagnostic()`.

Activation restores provider exports before flipping rows enabled. `RuntimePluginCatalog::activate_bridge_provider_at_frame_boundary(...)` resolves the provider package's runtime module owners, reads matching `InterfaceExport` rows from the final `RuntimeExtensionRegistry`, and calls `FrozenBridgeTable::restore_owner_exports_with_report(...)`. This lets a linked runtime module that was previously deactivated install its erased provider handles back into the frozen table when the module is activated again. Soft disable still keeps providers installed; deactivate clears providers and requires this restore path before weak calls can succeed again.

Reload replaces provider exports without disabling enabled rows. `RuntimePluginCatalog::reload_bridge_provider_at_frame_boundary(...)` maps the active registry's runtime module owner names to current owner slots, then reads replacement `InterfaceExport` rows from a replacement registry. `FrozenBridgeTable::reload_owner_exports_with_report(...)` applies those erased providers through the existing slots and reports `BridgeOwnerTransitionMode::Reload`. Enabled rows advance by two generations so existing weak bridges refresh to the replacement provider while remaining enabled; disabled rows keep the new provider installed until a later activation publishes it.

`provider_package_id_for_runtime_module(...)` maps a runtime module name such as `physics.runtime` back to the package id that owns it. `CoreRuntime` uses this to route ordinary linked runtime module `activate_module(...)` / `deactivate_module(...)` calls into provider lifecycle events without assuming every engine module name is a plugin package id.

Disable and deactivate still route through the catalog-level strong-dependent guard. If a provider has required bridge dependents, the state returns `RuntimePluginBridgeLifecycleError::StrongDependentsBlocked` and leaves the frozen table unchanged. Optional dependents can observe `BridgeError::NotEnabled` while a provider is disabled, then reconnect when the provider is activated again.

`diagnostics_summary()` exposes the bridge table aggregate for lifecycle logs: total rows, enabled/disabled rows, provider install state, and debug weak-call counters.

## Validation

`bridge_lifecycle_state_owns_frozen_table_for_provider_events` covers state construction from a catalog, final registry freezing, event-shaped optional-dependent provider disable, weak bridge `NotEnabled`, diagnostics summary updates, event-shaped re-activation through the same frozen table, reload provider event handling, generation +2 reload publication, and lifecycle diagnostic text.

`bridge_lifecycle_state_rejects_strong_provider_disable` covers strong-dependent rejection through the event-shaped state object, blocked-outcome diagnostics, and verifies the weak bridge remains enabled after the failed disable request.

`core_runtime_module_deactivation_drives_plugin_bridge_lifecycle` covers module deactivation clearing providers and module activation restoring them through the final registry. `core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` covers a strong bridge dependent blocking `deactivate_module(...)` before services are unloaded.

Fresh static validation for this slice passed after the event/outcome addition: `rustfmt --edition 2021 --check zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs zircon_runtime/src/plugin/runtime_plugin/mod.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs`, direct whitespace/conflict scans, and `git diff --check` over the touched lifecycle-state source/docs paths. A focused `cargo test -p zircon_runtime --lib bridge_lifecycle_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0612 --message-format short --color never -- --test-threads=1 --nocapture` attempt before the event/outcome addition timed out after 10 minutes without a trustworthy test result; the leftover cargo/rustc processes for that target directory were stopped. Cargo was not re-run after the event/outcome addition because an unrelated `material_keyboard_action` runtime validation lane was active.

Fresh 2026-06-13 provider-restore validation: `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-runtime-lifecycle-coremin-0613 --message-format short --color never` passed with existing warning noise after adding provider restore, runtime module package-id lookup, CoreRuntime provider lifecycle facades, and module activation/deactivation routing. The focused `core_runtime_module_deactivation` lib-test filter timed out twice during lib-test compilation, so no focused Cargo test pass is claimed for the new tests.

Fresh 2026-06-13 provider-reload validation: `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never` passed with existing warning noise after adding reload event/state/catalog wiring. `cargo test -p zircon_runtime --lib bridge_lifecycle_state_owns_frozen_table_for_provider_events --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` passed the focused lifecycle-state test, and `bridge_lifecycle_reload_replaces_provider_from_reloaded_registry` passed the catalog replacement-registry path under the same target directory.
