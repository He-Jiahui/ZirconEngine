---
related_code:
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
  - zircon_runtime/src/core/framework/bridge/diagnostics.rs
  - zircon_runtime/src/core/framework/bridge/strong.rs
  - zircon_runtime/src/plugin/bridge.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/bridge/weak.rs
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
implementation_files:
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
  - zircon_runtime/src/core/framework/bridge/diagnostics.rs
  - zircon_runtime/src/core/framework/bridge/strong.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/01/fixed-2026-07-13-core-contract-reverse-dependencies.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
---

# Neutral Bridge Contracts

`core/framework/bridge` owns the bridge vocabulary that can be shared without importing plugin registration or lifecycle policy. The canonical types are `PluginInterface`, `BridgeError`, `InterfaceSlot`, `BridgeInterfaceStatus`, `BridgeOwnerTransitionMode`, `BridgeDiagnosticsSnapshot`, `StrongBridge`, and `BridgeInvocationTable`.

`BridgeInvocationTable` is the narrow runtime-call seam. It resolves an interface id to a stable dense slot, reports the slot status, and records enabled/not-enabled calls. It does not expose package manifests, plugin module ids, provider registration, lifecycle reports, or the concrete frozen table.

Ownership is intentionally split:

- `core/framework/bridge` owns neutral identifiers, status/error values, diagnostics snapshots, the direct strong wrapper, and the invocation interface.
- `plugin/bridge/table.rs` owns `FrozenBridgeTable`, provider entries, plugin module ownership, lifecycle mutations, and report DTOs; it implements `BridgeInvocationTable`.
- `plugin/bridge/weak.rs` owns `WeakBridge` and `BridgeGuard` because their cached provider resolution is coupled to `FrozenBridgeTable` generations.
- `script/vm/host/bridge_host_module.rs` depends only on `BridgeInvocationTable` and neutral descriptors; it must not import the plugin facade or package-manifest types.

There are no compatibility re-exports for the types moved out of the plugin facade. Consumers import the canonical neutral paths directly. The Frameworks05 dependency audit and `test_script_bridge_host_does_not_depend_on_plugin_manifests` guard enforce this boundary.

## Validation

The 2026-07-13 hard-cut slice passed the focused script bridge owner guard as part of the Frameworks05 19/19 suite. The final production-only audit is 2,290 references / 72 edges with `script -> plugin = 0` and every tracked forbidden direction at zero. Managed Windows Runtime `core-min`/default checks and the Plugin SDK `--tests` check passed; the fixed handoff records the complete upward validation set.
