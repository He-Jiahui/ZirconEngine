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
  - zircon_runtime/src/plugin/bridge/table.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
implementation_files:
  - zircon_runtime/src/script/vm/host/bridge_host_module.rs
  - zircon_runtime/src/script/vm/host/script_call_table.rs
  - zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
plan_sources:
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/plans/zircon_plugins/08-zr-vm.md
tests:
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/tests/host_exports.rs
  - zircon_runtime/src/script/vm/tests/bridge_host.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
---

# VM Bridge Host Module

`bridge_host_module.rs` adapts the plugin bridge table to the existing VM host export system. It does not introduce a second VM call path; instead, it registers a regular script host module named `zr.zircon.bridge`.

`ScriptBridgeMethodDescriptor` is the metadata row for one script-visible bridge method. It stores the script function name, bridge interface id, method slot, return value kind, optional parameters, required capabilities, documentation, and a Rust callback. `register_bridge_host_module(...)` resolves the descriptor interface id against a `FrozenBridgeTable` once during registration, then stores the dense `InterfaceSlot` in the generated host callback.

Package manifests can now drive the same metadata through `provides_interfaces.methods`. `script_bridge_method_descriptors_from_manifest(...)` pairs manifest method rows with `ScriptBridgeMethodBinding` callbacks, and `register_bridge_host_module_from_manifest(...)` builds and registers the host module from that manifest-backed descriptor list. Manifest method capabilities are merged with the base `bridge.call` capability so method-specific access remains visible in the script host function descriptors.

The generated callback checks bridge status before dispatch:

- absent slots report a script host error,
- disabled or provider-cleared rows report "not enabled" and record a bridge not-enabled diagnostic counter,
- enabled rows record an enabled-call diagnostic counter and invoke the descriptor callback with `ScriptBridgeCall`.

This gives VM backends a stable pre-resolved bridge host surface. `HostExportRegistry::script_call_table()` now snapshots registered host export callbacks into dense `ScriptCallSite` rows, and the real `zr_vm` backend resolves those rows while registering native functions. Runtime callbacks therefore dispatch through the pre-resolved call site instead of looking up module and function names again.

## Validation

`bridge_host_module_dispatches_vm_calls_through_resolved_bridge_slots` covers successful descriptor registration, `bridge.call` capability exposure, script host call dispatch, argument forwarding, and dense slot/method-slot delivery.

`bridge_host_module_reports_disabled_bridge_to_vm_callers` covers disabled provider handling through the script host export path.

`bridge_host_module_registers_methods_from_package_manifest` covers manifest-backed descriptor generation, function metadata preservation, merged capabilities, and dispatch through resolved bridge slots.

`bridge_host_module_rejects_manifest_method_without_binding` covers declared manifest methods that have no VM callback binding.

`script_call_table_pre_resolves_host_export_callbacks` covers dense call site creation, id-based dispatch, capability validation, and host call context preservation.

`zr_vm_real_backend_uses_script_call_table_for_host_callbacks` guards the real backend path so VM native callbacks continue to use `ScriptCallSite` rather than `HostExportRegistry::call_with_capabilities(...)`.

`bridge_performance_baseline_vm_bridge_callbacks_capture_resolved_slot`, `bridge_performance_baseline_script_call_table_calls_dense_id_without_name_lookup`, and `bridge_performance_baseline_real_zr_vm_callbacks_capture_call_sites` keep the same contract in the bridge performance baseline: bridge host registration may resolve interface ids, and real backend registration may resolve module/function names, but runtime callbacks must dispatch by captured slot or captured call site. The independent source-structure check passed for these guards; Cargo lib-test validation is currently blocked before target execution by render-owned E0061 call-arity errors outside the VM bridge host module.

Fresh validation for this slice: `rustfmt --edition 2021 --check zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/host/script_call_table.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/vm/backend/zr_vm_project_backend/real_backend/host_modules.rs zircon_runtime/src/script/vm/tests.rs` passed. Direct whitespace/conflict scans passed, and `git diff --check -- <touched VM bridge/script-call-table paths>` returned only LF/CRLF notices. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never` passed with existing warning noise. `cargo test -p zircon_runtime --lib script_call_table_pre_resolves_host_export_callbacks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` and `cargo test -p zircon_runtime --lib zr_vm_real_backend_uses_script_call_table_for_host_callbacks --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture` each passed one focused test.
