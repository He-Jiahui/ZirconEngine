---
related_code:
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
implementation_files:
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/core/framework/bridge/mod.rs
  - zircon_runtime/src/core/framework/bridge/interface_slot.rs
plan_sources:
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
  - tools/tests/test_frameworks_05_layer_direction.py
---

# VM Bridge Host Module

`bridge_host_module.rs` adapts the plugin bridge table to the existing VM host export system. It does not introduce a second VM call path; instead, it registers a regular script host module named `zr.zircon.bridge`.

`ScriptBridgeMethodDescriptor` is the metadata row for one script-visible bridge method. It stores the script function name, bridge interface id, method slot, return value kind, optional parameters, required capabilities, documentation, and a Rust callback. `register_bridge_host_module(...)` resolves the descriptor interface id through the neutral `BridgeInvocationTable` contract once during registration, then stores the dense `InterfaceSlot` in the generated host callback.

The script host does not read plugin package manifests. Plugin/native owners may derive their own descriptors from manifests, but the script boundary accepts only neutral `ScriptBridgeMethodDescriptor` values. This removes the former `script -> plugin` facade dependency and keeps package registration policy out of VM dispatch. Method-specific capabilities remain explicit on each descriptor and are merged with the base `bridge.call` capability.

The generated callback checks bridge status before dispatch:

- absent slots report a script host error,
- disabled or provider-cleared rows report "not enabled" and record a bridge not-enabled diagnostic counter,
- enabled rows record an enabled-call diagnostic counter and invoke the descriptor callback with `ScriptBridgeCall`.

This gives VM backends a stable pre-resolved bridge host surface. `HostExportRegistry::script_call_table()` now snapshots registered host export callbacks into dense `ScriptCallSite` rows, and the real `zr_vm` backend resolves those rows while registering native functions. Runtime callbacks therefore dispatch through the pre-resolved call site instead of looking up module and function names again.

## Validation

`bridge_host_module_dispatches_vm_calls_through_resolved_bridge_slots` covers successful descriptor registration, `bridge.call` capability exposure, script host call dispatch, argument forwarding, and dense slot/method-slot delivery.

`bridge_host_module_reports_disabled_bridge_to_vm_callers` covers disabled provider handling through the script host export path.

`script_call_table_pre_resolves_host_export_callbacks` covers dense call site creation, id-based dispatch, capability validation, and host call context preservation.

`zr_vm_real_backend_uses_script_call_table_for_host_callbacks` guards the real backend path so VM native callbacks continue to use `ScriptCallSite` rather than `HostExportRegistry::call_with_capabilities(...)`.

`bridge_performance_baseline_vm_bridge_callbacks_capture_resolved_slot`, `bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup`, and `bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites` keep the same contract in the bridge performance baseline: bridge host registration may resolve interface ids, and real backend registration may resolve module/function names, but runtime callbacks must dispatch by captured slot or captured call site.

Fresh 2026-07-13 validation for the hard cut: rustfmt parsing/formatting passed for the touched bridge and script files; `test_script_bridge_host_does_not_depend_on_plugin_manifests` passed; the production dependency audit reports `script -> plugin = 0` and 24 total remaining forbidden references. Cargo validation is not claimed for this slice because shared Windows Cargo lanes remain active.
