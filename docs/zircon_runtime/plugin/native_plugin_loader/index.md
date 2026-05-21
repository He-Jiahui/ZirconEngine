---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
plan_sources:
  - docs/superpowers/specs/2026-05-19-native-dynamic-v3-hardening-design.md
  - docs/superpowers/plans/2026-05-20-native-dynamic-v3-hardening.md
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
tests:
  - cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
  - cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
  - cargo fmt --all --check
doc_type: module-detail
---

# Native Plugin Loader

`zircon_runtime::plugin::native_plugin_loader` owns the NativeDynamic discovery, ABI probing, behavior-call, validation, load-report, and live-host runtime boundary. It keeps C ABI data and callback pointers inside the loader and exposes owned Rust reports to the rest of runtime/editor tooling.

## Module Shape

The loader is split by responsibility so the ABI boundary does not accumulate behavior policy in one file:

- `abi_declarations.rs` owns stable `repr(C)` declarations, callback aliases, ABI version constants, descriptor symbol names, status constants, schema-version structs, and empty owned-byte-buffer constructors.
- `native_plugin_abi.rs` owns descriptor probing, entry symbol invocation, and conversion from raw ABI reports into owned `NativePluginDescriptor` and `NativePluginEntryReport` values.
- `native_strings.rs` owns C string reads, newline-list parsing, native symbol termination, and package-manifest TOML conversion helpers.
- `host_callbacks.rs` owns ABI v2/v3 host function table callbacks, capability negotiation, v3 host log/diagnostic capture, and callback-diagnostic draining.
- `behavior_calls.rs` owns copied behavior callback metadata, byte-command invocation, save/restore/unload calls, status conversion, plugin-owned byte-buffer copying, and free-callback diagnostics.
- `behavior_validation.rs` owns host-derived behavior health classification and schema/callback consistency checks.
- `loaded_native_plugin.rs` keeps the dynamic library handle alive while behavior callbacks are invoked and exposes copied behavior metadata and validation reports.
- `native_plugin_load_report.rs` aggregates discovery, descriptor, entry, callback, and validation diagnostics into package, runtime-registration, and editor-registration report surfaces.
- `native_plugin_live_host.rs` owns live runtime/editor native handles, hot reload/unload, runtime behavior descriptors, command dispatch, state snapshots, restore reports, and play-mode helper composition.

`mod.rs` remains structural. It declares the child modules and re-exports only the curated public DTOs and loader/live-host types. The split intentionally did not add compatibility modules or old-path shims.

## ABI Stability

ABI v3 remains the current NativeDynamic product ABI for this slice. The C structs, symbol names, callback signatures, byte-slice contracts, owned-buffer contract, status codes, and host callback table shape are unchanged. The hardening work derives additional Rust-owned reports after raw metadata has been copied into safe Rust values.

The loader still probes ABI v3 first, then falls back to ABI v2 and ABI v1. ABI v2 entries still receive the v2 host function table and can produce clean compatibility validation reports with no v3 schema strings. ABI v1 entries have no behavior table, so the derived behavior report is invalid for behavior inspection while descriptor/package diagnostics remain available.

Host callback capture is entry-scoped. ABI v3 entries receive `NativePluginHostFunctionTableV3` with `host_log` and `host_diagnostic`; the loader stores those records in host-owned capture state during entry invocation and flattens them into existing entry diagnostics after the plugin returns. No runtime/editor object, `wgpu` object, Rust trait object, or borrowed world/editor state crosses the C ABI.

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

## Runtime State

`NativePluginRuntimePluginState` stores `plugin_id`, `state_schema_version`, and the opaque state bytes returned by `save_state`. `NativePluginLiveHost::save_runtime_plugin_states(...)` copies the current loaded plugin's runtime state schema version into each snapshot entry.

`NativePluginLiveHost::restore_runtime_plugin_states(...)` compares the snapshot schema with the currently loaded plugin schema before calling `restore_state`. If they differ, the plugin id is added to `skipped_plugin_ids`, no plugin callback is invoked, and the deterministic diagnostic is emitted:

```text
runtime plugin {plugin_id} restore-state skipped because snapshot state schema {snapshot_schema:?} does not match loaded state schema {loaded_schema:?}
```

Missing/unloaded plugins remain skipped restore diagnostics instead of host failures. Play-mode enter/exit continues to compose snapshot, command-dispatch, and restore reports so restore diagnostics are preserved through `combined_diagnostics()`.

## Fixture Contract

`zircon_plugins/native_dynamic_fixture/native` is the real `cdylib` fixture for the loader. The fixture now exports ABI v3 descriptors and runtime/editor entry symbols by default while keeping an `abi_v2_only` feature to prove fallback. The runtime v3 behavior is stateful, declares state schema version `3`, uses the supported command/event schema ids, provides non-empty command/event manifest text, and implements invoke/save/restore/unload. The editor v3 behavior is stateless, leaves schema pointers null for empty manifests, supplies a denied stateless command callback, and omits save/restore.

Focused contract coverage proves the clean ABI v3 fixture reports `NativePluginBehaviorHealth::Clean` for runtime and editor behavior, preserves host log and host diagnostic callback output, preserves v2 fallback behavior, validates plugin-owned byte-buffer free diagnostics, and keeps runtime registration diagnostics scoped to runtime entries.

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

This slice does not claim full workspace validation because the checkout had unrelated concurrent-session changes and full workspace formatting was blocked by an editor file outside the NativeDynamic scope.

## Non-Goals

This loader hardening slice does not add ABI v4, new C structs, new callback signatures, typed command/event manifest parsing, Plugin Manager UI, editor panes, `zircon_app` bootstrap changes, app provider composition changes, render/UI/material work, VM/ZrVM integration, or Rust trait-object sharing across dynamic boundaries.

Future SDK/examples and Plugin Manager work should consume the existing validation reports and diagnostics rather than reimplementing ABI validation outside `zircon_runtime::plugin::native_plugin_loader`.
