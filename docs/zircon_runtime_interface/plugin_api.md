---
related_code:
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
implementation_files:
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
plan_sources:
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
tests:
  - cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo test -p zircon_runtime_interface function_table_structs_are_all_repr_c --locked --message-format short
  - cargo check -p zircon_runtime_interface --lib --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short
doc_type: module-detail
---

# Plugin API

`zircon_runtime_interface::plugin_api` owns the stable ABI DTOs shared by the host and native/runtime plugins. The existing v1 plugin entry contract remains available, and the Runtime Plugin Interface v2 plan now adds `ZR_PLUGIN_ENTRY_SYMBOL_V3` plus `ZrHostApiV3` as the stable host-facing v3 table.

`ZrHostApiV3` is split by domain instead of exposing one flat callback list:

- `ZrHostEcsApiV1` provides `register_system`, `register_component`, and `spawn_command`.
- `ZrHostAssetApiV1` provides a byte-payload asset request callback.
- `ZrHostEventApiV1` provides event `emit` and `drain`.
- `ZrHostDiagnosticsApiV1` provides diagnostic message and metric callbacks.

The ECS system registration DTO is `ZrSystemRegistrationV1`. It carries an ABI version, size, system id, stage, stable order tie-breaker, set names, before/after anchor names, an optional native invoke callback, and plugin-owned `user_data`. The host adapter can translate this into a conservative native system registration without exposing a `World` pointer across the C boundary.

`ZrComponentDescV1` carries component type id, display name, schema bytes, and storage kind. `ZrEventTypeId` is an ABI value for event namespace/name/stable hash. `ZrByteBufferRef` is a borrowed output buffer reference used by drain and snapshot callbacks; owned buffers still use `ZrOwnedByteBuffer`.

`ZrPluginStateSnapshotApiV1` provides the ABI-level save/restore table for hot reload. It records ABI version and size like other stable tables, then holds optional `save` and `restore` callbacks. Runtime-side live host code stores snapshots as owned Rust bytes before unloading or replacing a plugin handle.

`zircon_runtime::plugin::native_plugin_loader::NativeHostApiV3RegistrationScope` is the first runtime consumer for the public host table. It exposes the ECS system/component registration callbacks to native plugins and routes those calls into `RuntimeExtensionRegistry` with plugin-module ownership preserved. The adapter derives component catalog ownership by removing the `.runtime` module suffix, which keeps dotted package ids such as `net.rpc` intact. Asset, event, and spawn command entries are intentionally present in the stable table before their runtime endpoints are enabled, so plugins can detect `UnsupportedVersion` instead of relying on missing symbols.

All new v3 structs are `repr(C)` and have focused layout tests in `plugin_api_contracts.rs`. The tests assert symbol spelling, size fields, domain-table offsets, pointer-dense domain table sizes, empty constructors, and snapshot/buffer-ref plain-data layout. `abi_safety_contracts.rs` is the cross-table guard that keeps `ZrHostApiV3`, its four domain sub-tables, `ZrPluginStateSnapshotApiV1`, and `ZrPluginApiV1` in the shared `#[repr(C)]` function-table inventory alongside the dynamic runtime API table.
