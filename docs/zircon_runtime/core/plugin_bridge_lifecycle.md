---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/core/runtime/tests/plugin.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/core/runtime/tests/plugin.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
plan_sources:
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - zircon_runtime/src/core/runtime/tests/plugin.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
---

# Plugin Bridge Lifecycle In CoreRuntime

`CoreRuntime` now has an optional plugin bridge lifecycle state slot. The slot stores `RuntimePluginBridgeLifecycleState`, which already owns the plugin catalog, final extension report, and frozen bridge table used by weak bridge handles.

The public runtime and handle APIs are:

- `install_plugin_bridge_lifecycle_state(state)` stores the active bridge lifecycle state.
- `plugin_bridge_lifecycle_state()` returns a cloned state snapshot for diagnostics or bridge handle resolution.
- `apply_plugin_bridge_lifecycle_event(event)` applies a `RuntimePluginBridgeLifecycleEvent` through the installed state and returns `Some(RuntimePluginBridgeLifecycleOutcome)`; it returns `None` when no bridge lifecycle state is installed.
- `activate_plugin_bridge_provider_at_frame_boundary(...)`, `disable_plugin_bridge_provider_at_frame_boundary(...)`, and `deactivate_plugin_bridge_provider_at_frame_boundary(...)` construct the standard provider lifecycle events for callers that already know the package id.
- `plugin_bridge_provider_package_id_for_runtime_module(...)` maps a runtime module name back to the package id recorded in the installed lifecycle catalog.
- `clear_plugin_bridge_lifecycle_state()` removes the state during teardown or replacement.

This is the CoreRuntime-side call surface for M3 provider lifecycle integration. Actual plugin enable/disable code can construct a provider lifecycle event and call CoreRuntime instead of passing around catalog/registry/table triples. Provider reload is implemented on `RuntimePluginBridgeLifecycleState` and native live-host reload paths because it needs a replacement `RuntimeExtensionRegistry`; linked runtime module activation/deactivation continues to use the CoreRuntime facade above.

`zircon_app` registration-aware bootstrap now builds a `RuntimePluginBridgeLifecycleState` from the selected linked runtime plugin registrations and installs it into `CoreRuntime` before module activation. This covers startup-time provider availability for linked Rust/runtime plugin reports.

`CoreHandle::activate_module(...)` and `deactivate_module(...)` now participate in provider lifecycle routing for linked runtime plugins. When a bridge lifecycle state is installed, the module name is resolved back to a provider package id through the lifecycle catalog. Activation restores the provider exports for that package's runtime module owners from the final `RuntimeExtensionRegistry` before flipping the bridge rows enabled. Deactivation first checks the normal service unload blockers, then applies the provider deactivation event before clearing module services. If a strong bridge dependent blocks deactivation, the module lifecycle is restored to its previous state, no services are unloaded, and `CoreError::PluginBridgeLifecycleBlocked(...)` carries the stable bridge diagnostic.

## Validation

`core_runtime_applies_plugin_bridge_lifecycle_events` covers the empty-state `None` path, installing lifecycle state into `CoreRuntime`, applying provider disable through the runtime facade, observing `BridgeError::NotEnabled` through an existing weak bridge, applying provider activation through `CoreHandle`, and clearing the state.

`runtime_plugin_bootstrap_installs_bridge_lifecycle_state` covers linked runtime plugin bootstrap installing lifecycle state into `CoreRuntime`, resolving a bridge row from that state, and applying a provider disable event through the installed state.

`core_runtime_module_deactivation_drives_plugin_bridge_lifecycle` covers linked runtime module deactivation clearing the bridge provider and returning weak calls to `BridgeError::NotEnabled`, followed by module activation restoring the provider from the final registry. `core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` covers strong bridge dependents blocking module deactivation before unload and leaving the module `Running`.

Fresh static validation for this slice passed: `rustfmt --edition 2021 --check zircon_runtime/src/core/runtime/runtime.rs zircon_runtime/src/core/runtime/handle/runtime_extensions.rs zircon_runtime/src/core/runtime/state/runtime_inner.rs zircon_runtime/src/core/runtime/tests/plugin.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs zircon_runtime/src/plugin/runtime_plugin/mod.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs zircon_app/src/entry/builtin_modules.rs zircon_app/src/entry/engine_entry.rs zircon_app/src/entry/tests/profile_bootstrap.rs zircon_app/src/entry/runtime_library/runtime_session.rs`, direct whitespace/conflict scans, and `git diff --check` over the touched core/bridge/app source and docs paths. `cargo test -p zircon_app --lib runtime_plugin_bootstrap_installs_bridge_lifecycle_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-entry-0613 --message-format short --color never -- --test-threads=1 --nocapture` passed once before the test fixture manifest declaration cleanup; post-cleanup reruns timed out in runtime compilation while unrelated Cargo lanes were active, so no final post-cleanup Cargo pass is claimed.

Fresh 2026-06-13 linked runtime module lifecycle validation: `rustfmt --edition 2021 --check` passed for `error.rs`, `activation.rs`, `runtime_extensions.rs`, `runtime.rs`, `core/runtime/tests/plugin.rs`, `bridge/table.rs`, `extension_registry/access.rs`, and the bridge lifecycle state files. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-runtime-lifecycle-coremin-0613 --message-format short --color never` passed with existing warning noise. The focused `cargo test -p zircon_runtime --lib core_runtime_module_deactivation --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-runtime-lifecycle-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` target was attempted twice and timed out during lib-test compilation; the matching cargo/rustc processes were stopped. Later direct execution of the warmed `zircon_runtime` lib-test binary passed `core_runtime_module_deactivation` (2 focused tests) and `core_runtime_applies_plugin_bridge_lifecycle_events` (1 focused test).

Fresh 2026-06-13 provider reload note: `RuntimePluginBridgeLifecycleState::reload_provider_at_frame_boundary(...)` and `RuntimePluginBridgeLifecycleEvent::reload_provider(...)` now cover replacement-registry reloads outside the linked-module CoreRuntime activation/deactivation path. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never` passed with existing warning noise, and `bridge_lifecycle_state_owns_frozen_table_for_provider_events` passed with the reload assertion under the same target directory.
