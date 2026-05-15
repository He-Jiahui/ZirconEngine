---
related_code:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
implementation_files:
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
tests:
  - zircon_runtime/src/script/vm/tests.rs
  - "cargo test -p zircon_runtime script::vm: passed 2026-05-15"
  - "cargo check --workspace --locked: passed 2026-05-15"
  - "cargo test --workspace --locked: attempted 2026-05-15; stopped by no space on device"
doc_type: module-detail
---

# ZrVM Host Reflection

The VM host surface is split into two layers:

- `zircon_runtime::core::framework::script` owns neutral descriptors and values. VM backends can read `ScriptHostModuleDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostValueKind`, `ScriptHostCallContext`, and `ScriptHostResult` without depending on concrete runtime managers.
- `zircon_runtime::script::vm::host` owns registration, handle allocation, validation, capability checks, and callback dispatch.

VM code never receives Rust object pointers. Host objects are represented as `HostHandle` values, and framework-level values carry those handles as `u64` so the neutral contract does not depend on the VM subsystem.

## Registry Behavior

`HostExportRegistry` validates a module before it becomes visible:

- module, version, capability, type, function, and parameter names must be non-empty and trimmed;
- module names, type names, function names, and parameter names must not duplicate within their scope;
- function arity must be coherent with its parameter list;
- function required capabilities must be declared on the module;
- callbacks must exactly match declared function names.

Each registered module receives a `HostHandle` through the shared `HostRegistry`, using a `host.module.<module>` capability label. This keeps script-visible handles stable and lets existing handle validation continue to work.

Calls go through `call_with_capabilities`. The registry checks arity and required capabilities before building a `ScriptHostCallContext` and dispatching the callback. Backends should pass the package capability set from `VmPluginHostContext`.

## Built-In Modules

`PluginHostDriver::default()` registers first-wave built-in host modules:

- `zr.zircon.foundation`: time, log, and event helper descriptors.
- `zr.zircon.asset`: locator/status/revision query descriptors.
- `zr.zircon.scene`: default world handle and handle validity helpers.
- `zr.zircon.render`: read-only render metadata descriptors.
- `zr.zircon.math`: pure value descriptors and deterministic vector helpers.

This first wave deliberately favors stable values and handles over concrete manager references. Manager-backed behavior can replace the diagnostic placeholders once the target services expose stable trait-object access through `core::manager`.

## Package Protocol

`discover_vm_plugin_package` still supports bytecode packages. A ZrVM project package uses:

```toml
backend = "zr_vm:project"

[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "binary"
```

Project packages store no bytecode in `VmPluginPackage::bytecode`; instead they populate `VmPluginPackage::zr_vm_project` and `VmPluginPackageSource::zr_vm_project_path`.
