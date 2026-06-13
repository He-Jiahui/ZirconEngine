---
related_code:
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/mod.rs
implementation_files:
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/mod.rs
plan_sources:
  - user: 2026-06-12 implement docs/plans/zircon_plugins plugin architecture code
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
tests:
  - cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo test -p zircon_runtime_interface --lib abi_v3_layout_is_stable --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-abi-v3-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never
  - cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after earlier non-assertion target/process/render compile blockers)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe hot_reload_failure_rolls_back_to_snapshot --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after Cargo invocation exited -1 with warning-only output)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe failed_registration_revoked_via_ownership --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
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
- `ZrHostBridgeApiV1` reserves the pre-resolved plugin bridge call path: interface slot, method slot, byte input payload, and borrowed output buffer. Registration scopes return `UnsupportedVersion` for this domain; `NativeHostBridgeCallScope` connects the callback to a frozen bridge table plus a method-slot dispatch map.
- `ZrHostDiagnosticsApiV1` provides diagnostic message and metric callbacks.

The ECS system registration DTO is `ZrSystemRegistrationV1`. It carries an ABI version, size, system id, stage, stable order tie-breaker, set names, before/after anchor names, an optional native invoke callback, and plugin-owned `user_data`. The host adapter translates this into a conservative native system registration without exposing a `World` pointer across the C boundary: the boxed ABI node uses `NativeDynamicAccess`, whose `SystemParam` initialization calls `SystemParamAccess::add_conservative_world_access()` so the schedule conflict graph treats the callback as a world writer until a typed native access ABI exists.

`ZrComponentDescV1` carries component type id, display name, schema bytes, and storage kind. `ZrEventTypeId` is an ABI value for event namespace/name/stable hash. `ZrByteBufferRef` is a borrowed output buffer reference used by drain and snapshot callbacks; owned buffers still use `ZrOwnedByteBuffer`.

`ZrStatusCode::BridgeNotEnabled` is the stable ABI status for a weak bridge target that exists but is disabled or temporarily unavailable. It maps to raw code `7`, leaving `NotFound` for absent slots and `UnsupportedVersion` for host tables that expose the bridge domain before runtime behavior is wired.

`ZrPluginStateSnapshotApiV1` provides the ABI-level save/restore table for hot reload. It records ABI version and size like other stable tables, then holds optional `save` and `restore` callbacks. Runtime-side live host code stores snapshots as owned Rust bytes before unloading or replacing a plugin handle, validates the replacement schema before restore, and can restore the saved snapshot into the old handle when replacement restore fails.

`zircon_runtime::plugin::native_plugin_loader::NativeHostApiV3RegistrationScope` is the first registration consumer for the public host table. It exposes the ECS system/component registration callbacks to native plugins and routes those calls into `RuntimeExtensionRegistry` with plugin-module ownership preserved. The adapter derives component catalog ownership by removing the `.runtime` module suffix, which keeps dotted package ids such as `net.rpc` intact. ABI system callbacks are intentionally scheduled through conservative access rather than the empty `()` parameter, so parallel execution cannot place a native callback beside another world access until the callback declares a narrower typed access set. Asset, event, bridge, and spawn command entries are intentionally present in the stable registration table before their runtime endpoints are enabled, so plugins can detect `UnsupportedVersion` instead of relying on missing symbols.

`zircon_runtime::plugin::native_plugin_loader::NativeHostBridgeCallScope` is the first bridge-call consumer for the public host table. It uses `ZrHostBridgeApiV1::call` to map `(interface_slot, method_slot)` through `FrozenBridgeTable` status and a registered native method function. Absent slots and missing methods return `NotFound`, disabled installed interfaces return `BridgeNotEnabled`, registration handles return `UnsupportedVersion`, and invalid non-empty null payloads return `InvalidArgument`. Enabled and not-enabled call counters are recorded through the existing bridge diagnostics surface before the callback returns.

All new v3 structs are `repr(C)` and have focused layout tests in `plugin_api_contracts.rs`. The tests assert symbol spelling, size fields, domain-table offsets, pointer-dense domain table sizes, empty constructors, bridge status-code mapping, and snapshot/buffer-ref plain-data layout. `abi_v3_layout_is_stable` is the plan-facing alias that groups the M5-T1 layout invariants into one focused Cargo filter without replacing the narrower tests. `abi_safety_contracts.rs` is the cross-table guard that keeps `ZrHostApiV3`, its five domain sub-tables, `ZrPluginStateSnapshotApiV1`, and `ZrPluginApiV1` in the shared `#[repr(C)]` function-table inventory alongside the dynamic runtime API table.

Fresh 2026-06-13 bridge ABI evidence: `cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-0613 --message-format short --color never -- --nocapture` passed 5 tests, including the bridge domain layout and `BridgeNotEnabled` status-code mapping. After adding the M5-T1 plan-facing alias, `cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-contracts-0613b --message-format short --color never -- --test-threads=1 --nocapture` passed 6 tests, including `abi_v3_layout_is_stable`. `cargo test -p zircon_runtime_interface --lib abi_safety_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-0613 --message-format short --color never -- --nocapture` passed 7 tests, including the updated function-table inventory. The expected panic subtest in `repr_c_guard_fails_on_missing_local_attribute` prints its panic message while still exiting successfully. Runtime host-adapter code was format-checked for the registration and bridge-call scopes; focused runtime Cargo execution for bridge-call behavior is not yet claimed because unrelated runtime test lanes were active in the shared checkout.

M5-T1 alias evidence: after adding `abi_v3_layout_is_stable`, `cargo test -p zircon_runtime_interface --lib abi_v3_layout_is_stable --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-abi-v3-0613 --message-format short --color never -- --test-threads=1 --nocapture` passed 1 focused test. The broader `plugin_api_contracts` suite was rerun afterward with `cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-interface-contracts-0613b --message-format short --color never -- --test-threads=1 --nocapture`; it passed all 6 contract tests, so the alias now has both focused and full-suite evidence.

M5-T2 native host-adapter evidence: `native_system_enters_schedule_as_conservative_node` now builds the ABI-registered native system and asserts the resulting schedule node reports conservative world access plus a world conflict against an otherwise empty access set. `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` passed. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never` passed with existing warning noise. The first focused Cargo attempts hit stale target-dir dep-info output, lib-test compile/link timeout, process `-1` exits without Rust errors, and one unrelated render-owned lib-test compile blocker; the later focused `cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test, so the M5-T2 assertion now has Cargo evidence.

M5-T3 snapshot evidence: `hot_reload_failure_rolls_back_to_snapshot` now saves a stateful runtime plugin snapshot, rejects a replacement plugin with a mismatched schema, and restores the saved bytes into the old handle. `failed_registration_revoked_via_ownership` binds the same owner-tracked registration rollback surface from the runtime registry side. `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs` passed. The Cargo invocation for `hot_reload_failure_rolls_back_to_snapshot` exited `-1` with warning-only output, then the warmed `zircon_runtime` lib-test binary passed both `hot_reload_failure_rolls_back_to_snapshot` and `failed_registration_revoked_via_ownership` directly.
