---
related_code:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance/state_encoding.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
implementation_files:
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/lib.rs
  - zircon_plugins/zr_vm_language/runtime/src/module.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance/state_encoding.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
plan_sources:
  - docs/plans/zircon_plugins/08-zr-vm.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/state_encoding.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/woc_lifecycle.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/tests.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - cargo test -p zircon_plugin_zr_vm_language_runtime --locked
doc_type: module-detail
---

# ZrVM Project Backend Ownership Boundary

## Current Owner

The concrete `zr_vm:project` backend is owned by
`zircon_plugins/zr_vm_language/runtime`. The plugin owns the native binding
dependency, package compilation, host-module registration, call-site lowering,
runtime/session serialization, lifecycle exports, and concrete backend tests.

`zircon_runtime` owns only the VM-neutral contracts used by every language
plugin: backend-family registration, host-export discovery, plugin host context,
selected-backend policy, hot reload coordination, and lifecycle dispatch. The
Runtime crate has no ZrVM binding dependency, concrete backend module, feature
forwarder, or compatibility re-export.

## Registration Flow

The application enables the first-party ZrVM language runtime plugin. Its module
descriptor installs the concrete backend family into Runtime's neutral registry.
`VmPluginManager` then selects the registered family by manifest backend name and
drives packages through the shared lifecycle contract.

Host exports remain Runtime-owned capabilities. Before native host callbacks are
registered, the plugin snapshots those exports into `ScriptCallTable` and captures
pre-resolved call sites. Callback execution therefore validates the captured call
site without re-resolving module and function names.

With `backend-zr-vm`, an embedding host that owns an SDK project session can use
`build_zr_vm_native_host_modules(&HostExportRegistry, &CapabilitySet)` from the
language plugin. This builds modules from the same Runtime descriptors, value
converters, and captured call sites as the managed plugin backend. The caller
registers them on its SDK runtime and retains the registrations until its
sessions are dropped. Supplied capabilities are enforced on every call; creating
the modules does not grant capabilities or provide plugin lifecycle/reflection
extensions. WOC's standalone adapter can therefore consume `zr.zircon.math`
without duplicating Runtime's scalar algorithms.

## Lifecycle State Encoding

The backend reads the optional `stateEncoding() -> string` export once during
package loading. An absent export or `vm-state-json-v3` selects the existing
JSON lifecycle: `saveState` returns a serialized `VmStateBlob`, and
`restoreState` receives the same JSON representation. Unknown encodings prevent
loading rather than choosing a format from the returned value.

`vm-state-bytes-v1` selects an opaque binary lifecycle. `saveState` must return
`Array<uint>` containing values in 0..=255, and `restoreState(Array<uint>)` must
return integer zero. The backend wraps the bytes in a schema-v3 `VmStateBlob`
with an empty type table. Reflected or non-v3 blobs/schemas are rejected; the
package owns payload validation and migration. The encoding is fixed for the
instance, so restore-before-save and manager hot reload use the same contract.

WOC uses this binary format for its canonical WOS world state. Existing JSON
plugins retain their current lifecycle contract.

## Native Build Input

The external ZrVM sys crate can build its existing CMake native targets from the
complete enclosing source tree when `ZR_VM_RUST_BINDING_LIB_DIR` is absent.
The build writes only below Cargo's `OUT_DIR`, uses one native worker, and does
not recursively invoke Cargo. Managed validation must pin the full external
source tree, including populated submodules, before starting the Cargo job.

An explicit `ZR_VM_RUST_BINDING_LIB_DIR` still selects a separately built SDK;
its shared libraries must be available to the process loader. Automatic source
builds require matching host and target triples and an installed native CMake
toolchain. Cargo test/run can load generated libraries from `OUT_DIR`; product
deployment must package the native libraries through its normal artifact path.

## Lock Policy

The concrete serialization lock belongs to
`zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs`. Package loading
and instance lifecycle/export calls acquire that plugin-owned lock before touching
the native runtime or session. Poisoned locks recover their inner guard instead of
panicking, and the plugin-owned unit test covers recovery after poisoning.

Runtime 15 no longer mirrors this implementation as a Runtime lock-policy row.
Runtime's lock-poison guards cover only Runtime-owned neutral registries and the
selected-backend manager. Concrete ZrVM lock behavior is validated in the plugin
crate where the implementation lives.

## Validation

On 2026-07-14, the owner hard cut passed the plugin default-feature suite with
18 tests and its documentation tests. Runtime's core-min scene build also compiled
past all removed ZrVM owners; its only failure was an unrelated scene reflection
number-type mismatch. Source guards confirm callback call sites are resolved before
registration and that no concrete ZrVM backend implementation remains under
`zircon_runtime`.
