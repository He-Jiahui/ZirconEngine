---
related_code:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_app/src/entry/engine_entry.rs
implementation_files:
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs
  - zircon_app/src/entry/engine_entry.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/core/runtime/tests/plugin.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
---

# Runtime Module Lifecycle Observer

Core owns only the neutral `RuntimeModuleLifecycleObserver` contract and one optional observer slot. It reports successful module activation and pre-unload deactivation by module name; it does not know plugin catalogs, bridge tables, provider package ids, plugin events, or plugin lifecycle result types.

`RuntimePluginBridgeLifecycleState` remains plugin-owned and implements the neutral observer. The adapter resolves a runtime module name to its provider package, restores bridge exports after activation, and may return `RuntimeModuleLifecycleBlock` when strong dependents reject deactivation. Core maps that neutral block to `CoreError::RuntimeModuleLifecycleBlocked` and restores the module's prior lifecycle without unloading its services.

`zircon_app` is the composition owner. Registration-aware bootstrap retains the concrete plugin lifecycle state in `BuiltinEngineEntry`, erases it behind `Arc<dyn RuntimeModuleLifecycleObserver>`, and installs the observer before module activation. Explicit provider activate/disable/deactivate/reload operations remain methods of `RuntimePluginBridgeLifecycleState`; CoreRuntime no longer exposes plugin-specific lifecycle facades or concrete state accessors.

The hard cut intentionally removes `install_plugin_bridge_lifecycle_state`, `plugin_bridge_lifecycle_state`, `apply_plugin_bridge_lifecycle_event`, the provider lifecycle convenience methods, and `clear_plugin_bridge_lifecycle_state`. No alias or compatibility wrapper remains.

## Validation

- `test_core_runtime_observes_modules_without_owning_plugin_lifecycle` locks the neutral owner and rejects concrete lifecycle names in core runtime state and APIs.
- `core_runtime_module_deactivation_drives_plugin_bridge_lifecycle` covers deactivation and reactivation through the neutral observer.
- `core_runtime_module_deactivation_rejects_strong_bridge_dependents_before_unload` covers neutral blocking and module-state rollback.
- The final Frameworks05 production dependency audit records `core→plugin = 0`. Managed Windows Runtime `core-min`/default checks passed, and the focused module-deactivation tests passed 2/2, covering both lifecycle propagation and strong-dependent rejection before unload.
