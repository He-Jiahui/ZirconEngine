---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/plugin/bridge/import.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register/bridge_registration.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs
tests:
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report/validation/interfaces.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/ai/runtime/src/tests/integration_tasks.rs
plan_sources:
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: module-detail
---

# Script behavior bridge

## Purpose

`ScriptBehaviorBridge` is the neutral callback contract between behavior consumers and the script subsystem. Consumers depend on the framework trait, `ScriptBehaviorCallbackRef`, and a lifecycle-aware `BridgeImport`; they never resolve or retain `VmPluginManager`.

Callback identities are provider-qualified as `<package>::<node-id>`. The package selects one active VM slot and the node id selects that slot's current behavior registration. This prevents same-id callbacks from aliasing across packages and lets hot reload advance to the new slot generation without exposing VM ownership to AI.

## Registration and ownership

The ZrVM runtime plugin is the linked Rust provider of `script.behavior.v1`. It registers the typed export through `RuntimePluginRegistrationBuilder`, declares the interface id in `plugin.toml`, and installs a `First`-stage binding system. It intentionally declares no NativeDynamic bridge method until a byte-level `ScriptHostValue` protocol exists, so package metadata cannot advertise an unbound or signature-inaccurate native method. The public `VmScriptBehaviorBridge` owns only a `Weak<VmPluginManager>` and generation-aware callback cache, so the bridge cannot prolong manager lifetime. Rebinding a different manager clears cached handles.

The AI runtime plugin declares an optional interface dependency on ZrVM and obtains `BridgeImport<dyn ScriptBehaviorBridge>` through the plugin SDK module builder. An import is recorded while contributions are assembled, but it is bound only after all runtime contributions have been merged and the central registry has finalized its cached `FrozenBridgeTable`. This guarantees that consumers and lifecycle control use the same frozen table rather than a private snapshot.

## Lifecycle behavior

`BridgeImport::call` resolves through `WeakBridge` on every invocation. Before finalization or without a provider it returns `BridgeError::Absent`. Disabling the provider is observed as `NotEnabled`; reloading changes the provider generation and subsequent calls reach the replacement. The import also observes the same diagnostics state as direct table users. Owner revocation unbinds removed imports, rebuilds the authoritative table after registrations are removed, and rebinds surviving imports, so no consumer remains attached to a stale pre-revocation table.

The VM provider rejects ambiguous active package names. Callback cache entries are accepted only while both slot and generation still match; a hot reload therefore forces a fresh resolution. ScriptTask maps missing, unavailable, or invalid bridge results to typed AI diagnostics instead of using a bare-id fallback or concrete-manager escape hatch.

## Validation

Manifest validation checks both directions: every exported interface must appear in `provides_interfaces`, and every registered import must have a matching `dependencies.interfaces` declaration. Runtime tests cover final-merge binding, disable, reload, diagnostics sharing and owner revocation. VM tests cover same node ids in different slots, duplicate active package names and generation refresh. AI tests cover provider-qualified HostHandle invocation and unavailable-provider behavior.
