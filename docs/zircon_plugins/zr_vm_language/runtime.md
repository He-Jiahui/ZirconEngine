---
related_code:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_runtime/src/builtin/runtime_modules.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/Cargo.toml
  - zircon_runtime/src/builtin/runtime_modules.rs
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
tests:
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime: passed 2026-05-15"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features real-zr-vm: passed 2026-05-15 with ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug"
doc_type: module-detail
---

# ZrVM Language Runtime Plugin

`zircon_plugin_zr_vm_language_runtime` contributes the `zr_vm` VM backend family. The backend selector for source projects is `zr_vm:project`. Its runtime module resolves `VmPluginManager` and registers `ZrVmBackendFamily` during module activation.

The plugin is optional and disabled by default in project selection. This keeps ZirconEngine buildable on machines that do not have `E:\Git\zr_vm` or the `zr_vm_rust_binding` dynamic library available.

## Build Modes

Default build:

- compiles the plugin crate and registers the backend family type;
- resolves `zr_vm:project`;
- reports `BackendUnavailable` when a package is loaded, because the native binding is not linked.

Real ZrVM build:

- enable feature `real-zr-vm`;
- set `ZR_VM_RUST_BINDING_LIB_DIR` to the directory containing the built `zr_vm_rust_binding` dynamic library;
- the crate links `zr_vm_rust_binding` from `E:\Git\zr_vm\zr_vm_rust_binding\rust`.

The real backend serializes access through a process-global mutex because the current binding tests show shared C-side runtime state.

## Host Module Translation

When `real-zr-vm` is enabled, `ZrVmBackend`:

1. Opens the discovered `.zrp` project.
2. Builds a standard `zr_vm` runtime.
3. Converts every `HostExportRegistry` module descriptor into a `zr_vm_rust_binding::ModuleBuilder`.
4. Registers native callbacks that dispatch back into `HostExportRegistry::call_with_capabilities`.
5. Compiles the project incrementally.
6. Maps optional lifecycle exports to `VmPluginInstance` methods.

The lifecycle names are optional:

- `activate()`
- `deactivate()`
- `saveState(): string`
- `restoreState(state: string)`

`saveState` and `restoreState` map to `VmStateBlob` UTF-8 bytes. Missing lifecycle exports are accepted.
