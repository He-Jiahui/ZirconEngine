---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge_performance_baseline.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime_interface/src/plugin_api.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/tests/plugin_api_contracts.rs
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
plan_sources:
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_plugins/11-plugin-call-bridge.md
  - docs/superpowers/specs/2026-05-19-native-dynamic-v3-hardening-design.md
  - docs/superpowers/plans/2026-05-20-native-dynamic-v3-hardening.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
tests:
  - cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short
  - cargo test -p zircon_runtime --lib native_hot_reload --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo test -p zircon_runtime --lib native_live_host --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture
  - cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
  - cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
  - cargo fmt --all --check
  - rustfmt --check zircon_plugins/native_dynamic_fixture/native/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - git diff --check -- zircon_plugins/native_dynamic_fixture/plugin.toml zircon_plugins/native_dynamic_fixture/native/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/mod.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never
  - cargo test -p zircon_runtime --lib native_live_host_load_report_applies_runtime_bridge_lifecycle_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_live_host_load_report_applies_runtime_bridge_lifecycle_state --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613-coremin --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_live_host_builds_bridge_call_scope_from_loaded_manifest --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_live_host_rejects_installed_bridge_bindings_without_loaded_manifest --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/plugin/bridge/table.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle_state.rs zircon_runtime/src/plugin/extension_registry/access.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_bridge_dependencies.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never
  - cargo test -p zircon_runtime --lib native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib bridge_method_bindings_parse_abi_v3_callback_table --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_live_host_auto_installs_discovered_bridge_bindings_from_load_report --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib native_host_bridge_call_scope_dispatches_registered_method --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never (2026-06-13: passed with existing warning noise)
  - cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after earlier non-assertion target/process/render compile blockers)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe hot_reload_failure_rolls_back_to_snapshot --test-threads=1 --nocapture (2026-06-13: passed 1 focused test after Cargo invocation exited -1 with warning-only output)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe failed_registration_revoked_via_ownership --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - cargo test -p zircon_runtime --lib native_loader_calls_real_fixture_descriptor_and_entries --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe native_loader_falls_back_to_v2_when_v3_descriptor_is_absent --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b\debug\deps\zircon_runtime-5d2828c2001649f6.exe declared_system_anchors_are_registered --test-threads=1 --nocapture (2026-06-13: passed 1 focused test)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-fixture-0613 --message-format short --color never (2026-06-13: stopped before compile because zircon_plugins/Cargo.lock needs update under --locked)
doc_type: module-detail
---

# Native Plugin Loader

`zircon_runtime::plugin::native_plugin_loader` owns the NativeDynamic discovery, ABI probing, behavior-call, validation, load-report, and live-host runtime boundary. It keeps C ABI data and callback pointers inside the loader and exposes owned Rust reports to the rest of runtime/editor tooling.

## Module Shape

The loader is split by responsibility so the ABI boundary does not accumulate behavior policy in one file:

- `abi_declarations.rs` owns stable `repr(C)` declarations, callback aliases, ABI version constants, descriptor symbol names, status constants, schema-version structs, and empty owned-byte-buffer constructors.
- `bridge_method_bindings.rs` owns `NativeBridgeCall`, manifest-derived `NativeBridgeMethodDescriptor`, `NativeBridgeMethodBinding`, and the `NativeBridgeMethodFn` wrapper that can invoke either an in-process Rust test callback or an ABI v3 DLL callback plus `user_data`.
- `bridge_method_abi.rs` owns ABI v3 bridge method table parsing. It validates table version, C strings, null method arrays, and missing callbacks before converting DLL-exposed rows into owned `NativeBridgeMethodBinding` values.
- `native_plugin_abi.rs` owns descriptor probing, entry symbol invocation, and conversion from raw ABI reports into owned `NativePluginDescriptor` and `NativePluginEntryReport` values.
- `native_strings.rs` owns C string reads, newline-list parsing, native symbol termination, and package-manifest TOML conversion helpers.
- `host_callbacks.rs` owns ABI v2/v3 host function table callbacks, capability negotiation, v3 host log/diagnostic capture, and callback-diagnostic draining.
- `host_api_adapter.rs` owns the public `zircon_runtime_interface::ZrHostApiV3` runtime adapter scopes. `NativeHostApiV3RegistrationScope` maps ABI system/component callbacks into `RuntimeExtensionRegistry` registrations and wraps native systems in `NativeDynamicAccess` so the ECS scheduler sees conservative world access. `NativeHostBridgeCallScope` connects the bridge domain to a `FrozenBridgeTable` and a method-slot dispatch table. `NativeHostBridgeCallScope::method_count()` exposes the rebuilt dispatch size for hot-reload reports and focused tests. Unsupported spawn/asset/event callbacks remain explicit until those host domains have real runtime endpoints.
- `behavior_calls.rs` owns copied behavior callback metadata, byte-command invocation, save/restore/unload calls, status conversion, plugin-owned byte-buffer copying, and free-callback diagnostics.
- `behavior_validation.rs` owns host-derived behavior health classification and schema/callback consistency checks.
- `loaded_native_plugin.rs` keeps the dynamic library handle alive while behavior callbacks are invoked and exposes copied behavior metadata and validation reports.
- `native_plugin_load_report.rs` aggregates discovery, descriptor, entry, callback, and validation diagnostics into package, runtime-registration, and editor-registration report surfaces.
- `native_plugin_live_host.rs` owns live runtime/editor native handles, hot reload/unload, runtime behavior descriptors, command dispatch, state snapshots, restore reports, play-mode helper composition, and bridge lifecycle/reload report re-exports including `NativePluginLiveHostBridgeReloadReport`.
- `native_plugin_live_host/bridge_lifecycle.rs` owns the runtime bridge lifecycle integration for native live-host load, unload, and hot-reload reports. It applies `RuntimePluginBridgeLifecycleEvent` to a supplied `RuntimePluginBridgeLifecycleState`, records `NativePluginLiveHostBridgeLifecycleReport`, uses activation for load, reload for hot reload, deactivation for unload, rejects strong-dependent unloads before dropping the native handle, and rolls bridge state back to active if the native unload path fails after a successful bridge deactivation.
- `native_plugin_live_host/bridge_methods.rs` owns loaded-manifest native bridge method scope construction, installed binding reuse, and provider-reload scope rebuild. `runtime_bridge_call_scope_from_loaded_manifest(...)` reads the loaded runtime package manifest, pairs `provides_interfaces.methods` with `NativeBridgeMethodBinding` callbacks, resolves method descriptors through the active bridge lifecycle table, and returns a `NativeHostBridgeCallScope`. `install_runtime_bridge_method_bindings(...)` requires a loaded runtime package manifest and validates the manifest/binding pair before storing callbacks; `runtime_bridge_call_scope_from_installed_bindings(...)` rebuilds descriptors from the current loaded manifest after reload; `reload_runtime_bridge_provider_and_scope_from_installed_bindings(...)` applies provider reload and returns the rebuilt call scope plus diagnostics; `clear_runtime_bridge_method_bindings(...)` removes the stored callback set.

`mod.rs` remains structural. It declares the child modules and re-exports only the curated public DTOs and loader/live-host types. The split intentionally did not add compatibility modules or old-path shims.

## ABI Stability

ABI v3 remains the current NativeDynamic product ABI for the private loader path, and `zircon_runtime_interface::plugin_api` now exposes the plan-level public `ZrHostApiV3` domain table. The public interface table uses `ZR_PLUGIN_ENTRY_SYMBOL_V3` and separates ECS, asset, event, bridge, and diagnostics callbacks into `ZrHostEcsApiV1`, `ZrHostAssetApiV1`, `ZrHostEventApiV1`, `ZrHostBridgeApiV1`, and `ZrHostDiagnosticsApiV1`. The ECS domain carries `ZrSystemRegistrationV1` and `ZrComponentDescV1`, the bridge domain carries pre-resolved interface/method slots plus a byte payload/output buffer pair, and event drain and snapshot paths share `ZrByteBufferRef`. Private ABI v3 entry reports now also expose `bridge_methods: *const NativePluginBridgeMethodTableV3`, where each row provides an `interface_id`, `method_name`, callback pointer, and `user_data`; the manifest remains the source of method slot, parameter, and capability metadata.

The existing native loader still probes its private descriptor ABI v3 first, then falls back to ABI v2 and ABI v1. The C structs, symbol names, callback signatures, byte-slice contracts, owned-buffer contract, status codes, and private host callback table shape remain stable. The hardening work derives additional Rust-owned reports after raw metadata has been copied into safe Rust values.

The loader still probes ABI v3 first, then falls back to ABI v2 and ABI v1. ABI v2 entries still receive the v2 host function table and can produce clean compatibility validation reports with no v3 schema strings. ABI v1 entries have no behavior table, so the derived behavior report is invalid for behavior inspection while descriptor/package diagnostics remain available.

Host callback capture is entry-scoped. ABI v3 entries receive `NativePluginHostFunctionTableV3` with `host_log` and `host_diagnostic`; the loader stores those records in host-owned capture state during entry invocation and flattens them into existing entry diagnostics after the plugin returns. No runtime/editor object, `wgpu` object, Rust trait object, or borrowed world/editor state crosses the C ABI.

## Public Host API Adapter

`NativeHostApiV3RegistrationScope` is the registration-side bridge for the public v3 host table. A scope interns the plugin module owner in `RuntimeExtensionRegistry`, exposes a `ZrRuntimePluginHandle`, and publishes a `ZrHostApiV3` table whose callbacks are valid only while that registration scope is alive.

The ECS `register_system` callback reads `ZrSystemRegistrationV1`, resolves the stage through `SystemStage::ORDER`, interns declared system sets, maps before/after system ids into `SystemRef::System(...)`, and registers a boxed native system through the normal extension registry path. The optional native invoke callback is retained as a no-world payload callback, but the boxed schedule node uses `NativeDynamicAccess` so `SystemParamAccess::add_conservative_world_access()` marks it as a conservative world writer. The adapter deliberately does not pass a Rust `World`, `SystemParam`, or trait object through the C ABI, and it does not pretend native callbacks are conflict-free before a typed native access declaration ABI exists.

The ECS `register_component` callback reads `ZrComponentDescV1` and creates a `ComponentTypeDescriptor` owned by the plugin module. The component catalog plugin id is derived by stripping the `.runtime` suffix from the interned runtime module name, not by splitting on the first dot, so package ids such as `net.rpc` keep their full owner id. The schema and storage-kind fields remain ABI data for later typed reflection/runtime storage work; this slice records the type id, display name, and plugin owner in the existing component catalog. Spawn, asset, event, and registration-scope bridge callbacks currently return `UnsupportedVersion` rather than inventing partial behavior without the corresponding runtime endpoints.

`NativeHostBridgeCallScope` is the bridge-call-side adapter for the same public table. It owns a host handle, keeps a cloned `FrozenBridgeTable`, installs a dispatch map keyed by `(InterfaceSlot, method_slot)`, and exposes `method_count()` so reload reports can state how many bridge callbacks were rebuilt. The ABI callback rejects invalid non-empty null payloads with `InvalidArgument`, returns `UnsupportedVersion` for registration handles, `NotFound` for absent interface slots or missing method slots, and `BridgeNotEnabled` for installed bridge rows whose snapshot status is disabled. Enabled calls increment bridge diagnostics before dispatching the registered `NativeBridgeMethodFn` with a `NativeBridgeCall`; disabled calls increment the not-enabled diagnostic counter. The bridge performance baseline now guards this split directly: `NativeHostBridgeCallScope::from_method_descriptors(...)` may resolve interface ids before installing the dispatch table, but `native_host_bridge_call_v1(...)` must dispatch by incoming dense interface slot plus method slot without calling `resolve_slot(...)`.

Package manifests now provide the native bridge reflection source through `provides_interfaces.methods`. `native_bridge_method_descriptors_from_manifest(...)` pairs those manifest method rows with `NativeBridgeMethodBinding` values supplied by native code, rejects duplicate/missing/undeclared bindings, and returns `NativeBridgeMethodDescriptor` rows for `NativeHostBridgeCallScope::from_method_descriptors(...)`. Native live-host lifecycle integration now applies bridge provider events around load, unload, and hot reload. Loaded runtime native packages can build a `NativeHostBridgeCallScope` directly from their loaded package manifest and caller-supplied native method bindings through `runtime_bridge_call_scope_from_loaded_manifest(...)`, install validated bindings once and later call `runtime_bridge_call_scope_from_installed_bindings(...)`, or rely on ABI v3 `bridge_methods` discovery during runtime load/hot-reload. Discovery validates DLL-exposed callback rows against the currently loaded package manifest before replacing the installed binding registry; runtime unload clears that registry so stale DLL callback pointers are not retained.

## Behavior Validation

Every `NativePluginEntryReport` now carries a `NativePluginBehaviorValidationReport`. It is computed from copied metadata and callback availability only; it never invokes command, save, restore, or unload callbacks. The report records ABI version, module kind, plugin id, stateless flag, state schema version, command/event schema ids, manifest presence, callback availability, diagnostics, and health.

Health states are exactly:

- `Clean`: the report has no diagnostics.
- `Degraded`: the plugin can remain loaded with reduced capability. Current degraded cases include missing `unload`, missing `invoke_command` when no command manifest exists, and stateless behavior declaring a nonzero state schema version.
- `Invalid`: required metadata is inconsistent or unsupported. Current invalid cases include unsupported ABI v3 command/event schema ids, declaring a schema without non-empty matching manifest text, missing behavior metadata, and stateful behavior missing `save_state` or `restore_state`.

The supported ABI v3 schema ids are exactly `zircon.native.command-manifest/3` and `zircon.native.event-manifest/3`. Manifest validation is deliberately shallow in this slice: if a matching schema id is present, the matching manifest text must exist and contain at least one non-empty line. Typed command/event manifest parsing belongs to later SDK/examples work.

Callback rules are metadata-derived. Stateful behavior must provide both `save_state` and `restore_state`; stateless behavior may omit both. Missing `unload` remains no-op-compatible, and live-host unload still allows the native handle to drop when only the unload callback is missing. Missing `invoke_command` is reported before command execution attempts, while a command name containing an interior NUL returns a structured `NativePluginBehaviorCallReport` error with diagnostic `native plugin command name contained an interior NUL` before invoking the plugin callback.

## Report Flow

`LoadedNativePlugin` exposes `runtime_behavior_validation_report()`, `editor_behavior_validation_report()`, runtime/editor behavior health, and copied behavior metadata accessors. `NativePluginRuntimeBehaviorDescriptor` includes the runtime validation report so diagnostics UI and future Plugin Manager surfaces can inspect the metadata without touching callback pointers.

`NativePluginLoadReport::entry_diagnostics()` now includes entry diagnostics, v3 host callback diagnostics, and behavior-validation diagnostics using the existing `native plugin {plugin_id}: {message}` prefix. `diagnostics_for_runtime_plugin(...)` and `diagnostics_for_editor_plugin(...)` filter validation diagnostics by matching module kind, so runtime registration reports do not inherit editor-only behavior diagnostics and editor registration/status paths do not inherit runtime-only behavior diagnostics.

Runtime plugin registration projection still comes from package manifests. Validation reports add diagnostics to the registration report; they do not replace manifest ownership, create runtime modules, or register callable operations by themselves.

`NativePluginLiveHostLoadReport` now carries `bridge_lifecycle_reports`, and `NativePluginLiveHostOutcome` carries an optional `bridge_lifecycle_report`. The `*_with_bridge_lifecycle(...)` runtime load/unload/hot-reload helpers keep those reports structured instead of flattening them into diagnostics only. Load applies provider activation, hot reload applies provider reload, and unload first applies provider deactivation and returns the stable blocked diagnostic without unloading the native handle when a strong bridge dependent is present. `NativePluginLiveHostBridgeReloadReport` is the descriptor-reload companion report: it preserves the lifecycle report, the rebuilt `NativeHostBridgeCallScope`, and diagnostics such as `native.live_host.bridge_scope_reloaded`. Runtime load and hot reload also emit `native.live_host.bridge_bindings_discovered` or `native.live_host.bridge_bindings_discovery_failed` diagnostics when ABI v3 bridge method tables are present.

## Runtime State

`NativePluginRuntimePluginState` stores `plugin_id`, `state_schema_version`, and the opaque state bytes returned by `save_state`. `NativePluginLiveHost::save_runtime_plugin_states(...)` copies the current loaded plugin's runtime state schema version into each snapshot entry.

`NativePluginLiveHost::restore_runtime_plugin_states(...)` compares the snapshot schema with the currently loaded plugin schema before calling `restore_state`. If they differ, the plugin id is added to `skipped_plugin_ids`, no plugin callback is invoked, and the deterministic diagnostic is emitted:

```text
runtime plugin {plugin_id} restore-state skipped because snapshot state schema {snapshot_schema:?} does not match loaded state schema {loaded_schema:?}
```

Missing/unloaded plugins remain skipped restore diagnostics instead of host failures. Play-mode enter/exit continues to compose snapshot, command-dispatch, and restore reports so restore diagnostics are preserved through `combined_diagnostics()`.

Hot reload now also uses the runtime state path. `NativePluginHotReloadState` saves an owned `PluginStateSnapshot` for stateful runtime plugins before unloading the old behavior. After the replacement native library has loaded, the live host compares snapshot schema with the replacement schema and calls `restore_state` before inserting the replacement handle. If restore fails, the replacement behavior is unloaded, the previously loaded handle is reinserted when available, and the saved snapshot is restored into that old handle as part of rollback diagnostics. The plan-facing `hot_reload_failure_rolls_back_to_snapshot` alias exercises that failure shape directly by rejecting a mismatched replacement and restoring the saved payload into the previous runtime handle.

Stateless runtime behavior and editor behavior do not require a snapshot. Schema mismatches fail before invoking the replacement restore callback, so a plugin cannot receive state bytes authored for a different state schema version.

## Fixture Contract

`zircon_plugins/native_dynamic_fixture/native` is the real `cdylib` fixture for the loader. The fixture now exports ABI v3 descriptors and runtime/editor entry symbols by default while keeping an `abi_v2_only` feature to prove fallback. The runtime v3 behavior is stateful, declares state schema version `3`, uses the supported command/event schema ids, provides non-empty command/event manifest text, mirrors the ABI v3 bridge method table structs with null tables for current fixture entries, and implements invoke/save/restore/unload. The editor v3 behavior is stateless, leaves schema pointers null for empty manifests, supplies a denied stateless command callback, omits save/restore, and reports editor entry diagnostics through the v3 host ABI string. The v2 editor diagnostic remains available only for the `abi_v2_only` fallback path.

Focused contract coverage proves the clean ABI v3 fixture reports `NativePluginBehaviorHealth::Clean` for runtime and editor behavior, preserves host log and host diagnostic callback output, preserves v2 fallback behavior, validates plugin-owned byte-buffer free diagnostics, keeps runtime registration diagnostics scoped to runtime entries, and rejects accidental v2 editor diagnostics on the v3 entry path.

Runtime loader tests build that live fixture through an isolated temporary manifest that points `[lib].path` at `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`, runs Cargo offline into the test target directory, and therefore does not mutate `zircon_plugins/Cargo.lock`. The separate `zircon_plugins` workspace `--locked` check remains useful as a lockfile-integrity signal, but it is intentionally not required for runtime focused tests to compile the live fixture source.

## Acceptance Evidence

Scoped evidence recorded during the M1-M6 implementation stages:

- `cargo check -p zircon_runtime --lib --locked --jobs 1` passed after the module split and after the restore/schema changes, with only the pre-existing `entity_ids_matching_query_archetypes` dead-code warning.
- `cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1` passed after M4 with 13 tests passed.
- `cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1` passed after M4 with 37 tests passed.
- `cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1` passed after M5 with 3 tests passed.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1` passed after M5.
- `cargo fmt --all --check` was attempted during M6 and failed on unrelated `zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs` formatting from another active session.
- `cargo fmt -p zircon_runtime --check` passed during M6 as the scoped runtime substitute.
- `cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1` passed during M6 with 13 tests passed and the pre-existing `entity_ids_matching_query_archetypes` warning.
- `cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1` passed during M6 with 37 tests passed and the pre-existing warning.
- `cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1` passed during M6 with 3 tests passed and the pre-existing warning.
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1` passed during M6.
- `cargo test -p zircon_runtime_interface --lib plugin_api_contracts --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture` passed with 4 tests after adding public `ZrHostApiV3`.
- `cargo check -p zircon_runtime --lib --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short` passed with existing warnings after hot reload snapshot wiring.
- `cargo test -p zircon_runtime --lib native_live_host --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture` passed with 13 tests.
- `cargo test -p zircon_runtime --lib native_hot_reload --locked --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short -- --nocapture` passed with 2 tests.
- `cargo test -p zircon_runtime --lib host_api_adapter --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture` was attempted after adding the public host adapter but timed out after 5 minutes while other runtime Cargo jobs were compiling. No adapter-test pass is claimed for this slice.
- `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/mod.rs` passed after adding `NativeHostBridgeCallScope`, `NativeBridgeCall`, and method-slot bridge dispatch tests.
- Focused runtime Cargo execution for the bridge-call adapter is not claimed yet because unrelated `zircon_runtime` Cargo/rustc lanes were active in the shared checkout during this follow-up.
- M5-T2 conservative native-system access is implemented in `host_api_adapter.rs`: `NativeDynamicAccess` marks ABI native systems with conservative world access, and `native_system_enters_schedule_as_conservative_node` asserts the built node reports a `World` conflict. `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` passed after this update. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never` passed with existing warning noise, and the later focused `cargo test -p zircon_runtime --lib native_system_enters_schedule_as_conservative_node --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-m5-check-coremin-0613b --message-format short --color never -- --test-threads=1 --nocapture` passed 1 focused test. Earlier stale target-dir, timeout, process `-1`, and render-owned compile-error attempts are retained as historical validation noise, not assertion failures.
- M5-T3 snapshot rollback alias `hot_reload_failure_rolls_back_to_snapshot` is written in `native_plugin_live_host/tests.rs`, and `failed_registration_revoked_via_ownership` exposes the owner-tracked failed-registration rollback path. `rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs zircon_runtime/src/tests/plugin_extensions/extension_registry_metadata.rs` passed. The Cargo invocation for `hot_reload_failure_rolls_back_to_snapshot` exited `-1` with warning-only output, but the warmed lib-test binary then passed both `hot_reload_failure_rolls_back_to_snapshot` and `failed_registration_revoked_via_ownership` directly.
- M5-T4 native dynamic fixture coverage now asserts the default real fixture descriptor is ABI v3, runtime/editor entry names use v3 symbols, runtime/editor behavior reports `NativePluginBehaviorHealth::Clean`, editor diagnostics come from the v3 host ABI table, accidental v2 editor diagnostics are absent on the v3 path, and the `abi_v2_only` build remains the explicit v2 fallback guard. `rustfmt --edition 2021 --check zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs` passed after the helper isolation update. The runtime focused `native_loader_calls_real_fixture_descriptor_and_entries` Cargo command passed 1 focused test, and the warmed lib-test binary passed `native_loader_falls_back_to_v2_when_v3_descriptor_is_absent`. The fixture-only `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-fixture-0613 --message-format short --color never` still stops before compile because `zircon_plugins/Cargo.lock` needs update under `--locked`; the lockfile remains intentionally untouched.
- `rustfmt --edition 2021 --check zircon_runtime/src/plugin/package_manifest/plugin_interface_manifest.rs zircon_runtime/src/plugin/package_manifest/mod.rs zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs zircon_runtime/src/plugin/runtime_plugin/package_validation/interfaces/exports.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/mod.rs zircon_runtime/src/tests/plugin_extensions/package_manifest_declarations.rs zircon_runtime/src/tests/plugin_extensions/runtime_plugin_package_manifest.rs` passed after adding package-manifest bridge method metadata and native manifest descriptor generation. Direct conflict-marker scans passed for the same native/package-manifest slice. Focused Cargo execution was deferred because unrelated runtime Cargo/rustc lanes were active.
- `cargo test -p zircon_runtime --lib plugin_extensions::runtime_plugin_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-0612 --message-format short --color never -- --nocapture` was attempted after adding the descriptor-owned system-anchor contract but timed out after 10 minutes under concurrent runtime Cargo load. The timed-out target-dir process was cleaned up; no runtime-descriptor test pass is claimed for this slice.
- `rustfmt --check zircon_plugins/native_dynamic_fixture/native/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` passed after the fixture v3 editor diagnostic and dotted-plugin-id adapter updates.
- `git diff --check -- zircon_plugins/native_dynamic_fixture/plugin.toml zircon_plugins/native_dynamic_fixture/native/src/lib.rs zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs` passed after the same updates, with only line-ending warnings.
- Fresh 2026-06-13 native live-host bridge lifecycle and binding evidence: `NativePluginLiveHostBridgeLifecycleReport`, runtime `*_with_bridge_lifecycle(...)` helpers, load-report lifecycle application, unload deactivation, strong-dependent unload blocking, `runtime_bridge_call_scope_from_loaded_manifest(...)`, and the installed bridge binding registry are implemented in `native_plugin_live_host/bridge_lifecycle.rs`, `native_plugin_live_host.rs`, and `native_plugin_live_host/bridge_methods.rs`. Tests were added for load-report activation, unload-time deactivation, strong-dependent unload blocking, loaded-manifest bridge call scope construction, installed-binding reuse, reloaded-manifest descriptor rebuild, manifest-required binding install rejection, and missing native method binding rejection. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-bridge-0613 --message-format short --color never` passed earlier with existing warning noise. The installed-binding registry static validation passed `rustfmt --edition 2021 --check` for `host_api_adapter.rs`, `native_plugin_live_host.rs`, `bridge_methods.rs`, and `tests.rs`; direct conflict/trailing-whitespace scans passed; `git diff --check -- <native binding registry paths>` reported only LF/CRLF warnings. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never` passed with existing warning noise. Focused Cargo test remains unconfirmed: `cargo test -p zircon_runtime --lib native_live_host_rejects_installed_bridge_bindings_without_loaded_manifest --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` first timed out after 10 minutes during lib-test compilation, then a warmed rerun was blocked before test execution by unrelated `graphics/visibility/context/from_extract_with_history/construct.rs:535`, `:537`, and `:552` missing `STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES`.
- Fresh 2026-06-13 native live-host bridge reload evidence: `reload_runtime_bridge_provider_and_scope_from_installed_bindings(...)` now applies `RuntimePluginBridgeLifecycleEvent::reload_provider(...)`, rebuilds descriptors from the current loaded manifest plus installed bindings, returns `NativePluginLiveHostBridgeReloadReport`, and exposes the rebuilt dispatch count through `NativeHostBridgeCallScope::method_count()`. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-reload-coremin-0613 --message-format short --color never` passed with existing warning noise. The focused tests `native_live_host_reloads_bridge_lifecycle_and_installed_binding_scope`, `bridge_lifecycle_reload_replaces_provider_from_reloaded_registry`, `bridge_table_reloads_owner_exports_with_report`, and the updated `bridge_lifecycle_state_owns_frozen_table_for_provider_events` passed serially under the same `core-min` target directory. An earlier parallel test attempt against one target directory timed out from target-dir contention and was rerun serially.
- Fresh 2026-06-13 ABI bridge binding discovery evidence: private ABI v3 entry reports now carry `bridge_methods`, `bridge_method_bindings_from_abi_v3(...)` converts DLL tables into `NativeBridgeMethodBinding` rows, and the live host auto-installs validated bindings during runtime load/hot-reload while clearing them on runtime unload. `cargo test -p zircon_runtime --lib bridge_method_bindings_parse_abi_v3_callback_table --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture`, `cargo test -p zircon_runtime --lib native_live_host_auto_installs_discovered_bridge_bindings_from_load_report --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture`, and `cargo test -p zircon_runtime --lib native_host_bridge_call_scope_dispatches_registered_method --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never -- --test-threads=1 --nocapture` each passed one focused test. `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-architecture-native-abi-bindings-coremin-0613 --message-format short --color never` also passed with existing warning noise. Scoped `rustfmt --edition 2021 --check`, conflict/trailing-whitespace scans, `git diff --check -- <touched paths>`, and target-dir process checks passed; `git diff --check` only emitted LF/CRLF notices.
- Fresh 2026-06-14 native bridge performance baseline evidence: `bridge_performance_baseline_native_bridge_calls_use_pre_resolved_slots` now scans `host_api_adapter.rs` and requires descriptor construction to resolve interface ids before creating the `NativeHostBridgeCallScope`, while the ABI callback uses `InterfaceSlot::from_raw(interface_slot)`, `interface_snapshot(slot)`, and the `(interface_slot, method_slot)` dispatch map without re-entering `resolve_slot(...)`. The source-structure check passed independently; Cargo lib-test validation reached `zircon_runtime` compilation but stopped before this target test because active render code currently has E0061 call-arity errors outside the native loader.

This slice does not claim full workspace validation because the checkout had unrelated concurrent-session changes and full workspace formatting was blocked by an editor file outside the NativeDynamic scope.

## Non-Goals

This loader hardening slice does not add ABI v4, typed command/event manifest parsing, Plugin Manager UI, editor panes, `zircon_app` bootstrap changes, app provider composition changes, render/UI/material work, additional VM/ZrVM behavior beyond the existing bridge path, or Rust trait-object sharing across dynamic boundaries.

Future SDK/examples and Plugin Manager work should consume the existing validation reports and diagnostics rather than reimplementing ABI validation outside `zircon_runtime::plugin::native_plugin_loader`.
