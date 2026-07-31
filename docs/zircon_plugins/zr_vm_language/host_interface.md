---
related_code:
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/system.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/bt_node.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/rpc.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/editor_op.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/extension_host.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
implementation_files:
  - zircon_plugins/zr_vm_language/plugin.toml
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/mod.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/system.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/bt_node.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/rpc.rs
  - zircon_plugins/zr_vm_language/runtime/src/host_interface/editor_op.rs
  - zircon_plugins/zr_vm_language/runtime/src/plugin.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/extension_host.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
plan_sources:
  - user: 2026-07-13 implement the complete engine plugin architecture plan
  - docs/plans/zircon_plugins/08-zr-vm.md
tests:
  - zircon_plugins/zr_vm_language/runtime/src/tests/host_interface.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/registration.rs
  - zircon_plugins/zr_vm_language/runtime/src/tests/real_backend.rs
  - zircon_runtime/src/script/vm/tests/host_interfaces.rs
doc_type: module-detail
---

# ZrVM Extension Registration Channels

## Purpose

The ZrVM language plugin exposes one coherent extension surface in both embedding forms. Rust-side integrations call the small helpers under `runtime/src/host_interface/`; real ZrVM packages import `zr.zircon.extensions`. Both paths delegate to the same runtime-owned `VmHostInterfaceRegistry`, so capability checks, identifier validation, dense callback compilation, hot reload, and unload behavior cannot drift.

## Script API

The native module exports:

```text
register_system(id, stage, module, function)
register_bt_node(id, display_name, module, function)
register_rpc_handler(id, payload_schema, module, function)
register_editor_operation(operation, module, function)
```

All script parameters are strings. Supported system stages are `fixed_update`, `update`, and `last`; editor operations use the `Domain.Group.Action` form. The RPC payload string is converted at the boundary into the shared `RpcPayloadSchema` plus `ReflectSchemaRequest`, not stored as a language-specific schema. Native argument values and counts are checked before the runtime registry is called. A missing capability is returned as a binding error that includes the typed host-interface diagnostic.

The native module is installed alongside the existing foundation/math/log host modules before a project package is loaded. Registration calls are expected during package activation, after the coordinator has assigned the package slot and generation.

## Rust API

The Rust helpers accept `&VmPluginHostContext`, obtain its authenticated `VmInterfaceCaller`, and forward to the matching registry method. They intentionally contain no second registration store and no concrete AI, networking, or editor dependency.

## Scheduled Systems

The runtime plugin registers exactly three static runtime-scene systems:

- `zr_vm_language.systems.fixed_update`
- `zr_vm_language.systems.update`
- `zr_vm_language.systems.last`

Each system resolves `VmPluginManager`, selects active contributions for its stage, and invokes them sequentially. The registration builder reports conservative world access, preventing the scheduler from making unsafe parallelism assumptions about VM code.

The same three identifiers are projected from `RuntimePluginDescriptor::system_anchors` into the generated root `plugin.toml`. Registration validation therefore checks declared anchors against the actual fixed dispatchers instead of relying on an undocumented runtime side effect.

## Test Coverage

Default-feature tests verify all four capability gates, active RPC/editor descriptor visibility, stale callback generation refresh, mock callback execution, conservative schedule registration, and descriptor/manifest anchor parity. The real-backend fixture imports `zr.zircon.extensions`, registers all four channels during `activate`, repeats registration through hot reload, invokes the reloaded system callback, and verifies that unload removes the published descriptors. On 2026-07-13 the current source passed 9/9 default tests and 12/12 `backend-zr-vm` tests on Windows with an isolated, offline-generated lock and `--locked --offline`; the foreign-modified main plugin lock remained untouched. The feature run also exercises unsorted manifest capabilities, generation 2 reload, callback dispatch, and descriptor cleanup. Exact evidence is recorded in the numbered Plugins 08 output record.

## Follow-up

The descriptor-to-consumer adapters remain owned by the destination plugins: AI M3 maps `VmBehaviorNodeRegistration` to `ScriptTask`, networking consumes `VmRpcHandlerRegistration`, and the editor consumes `VmEditorOperationRegistration`. This keeps the language plugin independent of those concrete implementations while preserving a single typed contract.
