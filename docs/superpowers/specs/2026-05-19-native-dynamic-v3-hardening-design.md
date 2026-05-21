---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
implementation_files:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/tests/native_plugin_loader_contract.rs
  - zircon_plugins/native_dynamic_fixture/native/src/lib.rs
  - docs/engine-architecture/runtime-editor-pluginized-export.md
plan_sources:
  - user: 2026-05-19 approved NativeDynamic v3 hardening approach, architecture, data flow, error handling, tests, and docs
  - .codex/plans/ZirconEngine 周边设施与插件能力完善计划.md
  - .codex/plans/ZrVM 语言插件与反射注册计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
  - cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
  - cargo fmt --all --check
doc_type: approved-design
---

# NativeDynamic V3 Hardening Design

## Goal

Harden the existing NativeDynamic ABI v3 behavior surface without changing the ABI structs or expanding the callback table. The slice turns the current command/state/event/schema/log/diagnostic metadata into validated host-owned reports that can safely feed runtime diagnostics and a later Plugin Manager UX pass.

## Approved Direction

Use the current ABI v3 as the product boundary for this slice. ABI v3 already includes requested and negotiated capabilities, host ABI and capability queries, host log and diagnostic callbacks, command and event manifests, command/event manifest schema strings, state schema version, byte-command invocation, state save/restore, and unload callbacks. The next step is not a new ABI. The next step is host-side hardening: validate the metadata, preserve structured diagnostics, make snapshot/restore schema compatibility explicit, and keep all failures diagnostic-first.

`zircon_runtime::plugin::native_plugin_loader` remains the owner. `zircon_app` keeps the long-lived `NativePluginRuntimeBootstrap` owner and pass-through calls only. `zircon_editor` and Plugin Manager UI are follow-up consumers and are not edited in this slice.

## Current Baseline

`NativePluginLoader` probes ABI v3 first, then falls back to v2 and v1. ABI v3 entries receive `NativePluginHostFunctionTableV3`, and the host captures entry-time log and diagnostic callbacks as entry diagnostics. `LoadedNativePlugin` keeps the dynamic library handle alive while invoking behavior callbacks. `NativePluginLiveHost` owns runtime/editor loaded handles, hot reload, unload, runtime behavior descriptors, runtime command dispatch, state snapshots, restore reports, and play-mode enter/exit helpers.

The baseline proves real dynamic loading through `zircon_plugins/native_dynamic_fixture/native`. The fixture exports v1/v2/v3 descriptors and runtime/editor entry symbols. The runtime contract test already validates v3 schema fields, command and event manifests, host log/diagnostic capture, byte command invocation, plugin-owned buffer release diagnostics, state save/restore, denied commands, panic-to-status conversion, unload, v2 fallback, and runtime registration projection.

The main gap is that the host records these fields but does not yet classify behavior health. A malformed command manifest schema, inconsistent stateless/state callback set, or restore state schema mismatch is not currently surfaced as a first-class validation report before command/state execution.

## Reference Evidence

Bevy establishes the Rust-native lifecycle precedent: `dev/bevy/crates/bevy_app/src/plugin.rs` defines plugin lifecycle as build, ready, finish, and cleanup, while `dev/bevy/crates/bevy_app/src/plugin_group.rs` makes plugin enablement and ordering explicit. Zircon should keep NativeDynamic behavior lifecycle explicit in host-owned reports instead of relying on ad hoc command invocation failures.

Fyrox establishes editor/runtime lifecycle hooks as an authoring precedent. `dev/Fyrox/editor/src/plugin.rs` exposes editor plugin hooks for start, exit, sync-to-model, mode change, scene change, UI messages, preview-mode exit, update, post-update, and control messages. Zircon's NativeDynamic command and play-mode helpers should therefore remain lifecycle-aware and report their state transitions clearly.

Godot establishes the dynamic extension and reload precedent. `dev/godot/core/extension/gdextension.h` separates library open/close, initialization levels, deinitialization, reloadability, and instance state. `dev/godot/core/extension/gdextension.cpp` stores instance state during `prepare_reload()` and restores it during `finish_reload()`. Zircon should keep reload rollback and state migration explicit, but because NativeDynamic crosses a C ABI, it should do this through byte state blobs and schema versions rather than object references.

The current Zircon docs reinforce this boundary. `docs/engine-architecture/runtime-editor-pluginized-export.md` documents NativeDynamic as a manifest, diagnostics, export-package, load-manifest, and byte behavior backend. `.codex/plans/ZirconEngine 周边设施与插件能力完善计划.md` makes ABI v3 the formal NativeDynamic product ABI and keeps VM as a parallel backend. `.codex/plans/ZrVM 语言插件与反射注册计划.md` requires stable handles, capabilities, and reflection descriptors instead of raw Rust object sharing. This design follows those constraints.

## Architecture

### Owner Boundary

`zircon_runtime::plugin::native_plugin_loader` owns validation, report declarations, ABI parsing, behavior invocation, live-host aggregation, and tests. No runtime/editor objects cross the ABI. The native plugin only sees C strings, byte slices, owned byte buffers, status codes, and host callback functions.

`zircon_app` remains a lifetime owner for runtime bootstraps. `NativePluginRuntimeBootstrap` should continue to expose pass-through descriptor, command, dispatch, state, and play-mode APIs from the live host. It should not become a validator or duplicate plugin diagnostics logic.

`zircon_editor` remains out of scope. Plugin Manager can later consume behavior validation reports, command/event manifest summaries, and diagnostics, but this hardening slice does not add UI actions or panes.

### Module Shape

`native_plugin_abi.rs` is already at the 1000-line warning threshold. The implementation must split before adding substantial logic. The approved target shape is folder-backed and responsibility-oriented:

- ABI declarations and raw C-compatible structs stay in a narrow ABI declaration module.
- Descriptor and entry probing stay in a loader/probe module.
- Behavior invocation and owned-buffer release stay in a behavior-calls module.
- Host callback capture for logs and diagnostics moves into a host-callback module.
- Parsing and validation of string lists, schema strings, command/event manifests, and behavior consistency move into validation-focused modules.

`mod.rs` must remain structural. Existing public re-exports should stay curated, and no compatibility alias path should be added.

## Behavior Validation

Add a host-owned validation report derived from copied metadata. The report must be computed without invoking plugin callbacks, so an invalid plugin can still be inspected safely.

The report should cover:

- ABI version and module kind.
- Plugin id.
- Stateless flag.
- State schema version.
- Command manifest schema and event manifest schema.
- Command manifest and event manifest presence.
- Callback availability for invoke, save, restore, and unload.
- Validation diagnostics.
- A health state with exactly three values: `Clean`, `Degraded`, and `Invalid`.

The first implementation can keep manifest parsing intentionally shallow. Schema string validation should prove that the schema belongs to the supported Zircon native schema family, such as `zircon.native.command-manifest/3` and `zircon.native.event-manifest/3`, and should reject empty or unknown schema names for ABI v3 behavior. Command/event manifest content can initially be validated as present text with non-empty lines. A deeper typed command/event manifest parser is a later SDK/examples milestone, not part of this slice.

Health-state rules:

- `Clean`: no validation diagnostics.
- `Degraded`: behavior can remain loaded but has reduced capability, such as a missing `unload` callback or missing command callback on a module that only contributes state metadata.
- `Invalid`: required metadata is inconsistent or unsupported, such as an unsupported ABI v3 schema string, a stateful behavior missing either state callback, or a declared command/event manifest schema with no matching manifest text.

State callback consistency rules:

- Stateful behavior must provide both `save_state` and `restore_state`.
- Stateless behavior may omit `save_state` and `restore_state`.
- Missing `unload` remains a diagnostic-only no-op path because the live host already treats missing unload as a handle-release-compatible warning.
- Missing `invoke_command` means command dispatch is unavailable and should be reported before command invocation attempts.

## Data Flow

The loader continues to probe v3 first and falls back to v2/v1. ABI fallback behavior is unchanged.

When a v3 entry returns an entry report, the host copies all descriptor, report, behavior, manifest, schema, negotiated capability, and callback metadata into owned Rust structures. Then the validation layer derives a behavior validation report from the copied metadata. This report is added to runtime behavior descriptors and to load/hot-reload diagnostics. The report must not require a live callback invocation.

Runtime command calls still go through `LoadedNativePlugin::invoke_runtime_command(...)`. The call path should fail with `NativePluginBehaviorCallReport` when the behavior is missing, the command callback is missing, or the command name contains an interior NUL. Existing plugin-owned byte buffer copying and free-callback diagnostics stay intact.

Runtime state snapshots should carry enough metadata to validate restore compatibility before invoking the plugin. At minimum, each captured state should include plugin id, state bytes, and state schema version. Restore should report missing plugin ids, missing behavior, and schema mismatches deterministically. A schema mismatch should become a restore diagnostic and skipped call unless a later migration explicitly introduces state migration adapters.

Play-mode enter and exit remain composed from the same primitives: save all state, dispatch `play-mode.enter`, dispatch `play-mode.exit`, and restore captured state. Reports should preserve validation and restore diagnostics rather than flattening them into a single string too early.

## Error Handling

Native loading remains non-fatal. Missing libraries, missing symbols, invalid ABI metadata, unsupported schema strings, invalid manifests, null entry pointers, and entry diagnostics must produce diagnostics instead of panics.

Bad behavior metadata degrades the plugin. It should not crash the runtime, invalidate the whole live host, or silently disappear. Diagnostics should identify plugin id, module kind, schema field, and callback category where possible.

Hot reload rollback semantics remain unchanged. A failed replacement discovery keeps the previous handle available. A failed unload restores the old handle. A replacement that loads but has degraded behavior can be kept loaded with diagnostics unless it lacks required metadata for the selected module kind.

Host log and diagnostic callbacks remain entry-safe. The implementation must preserve the existing flattened entry diagnostics for current report consumers. Structured callback records may be added only if they stay inside the native-loader report boundary and do not require editor UI, app bootstrap, or core diagnostics-store work in this slice.

## Tests

Focused runtime tests should cover:

- A valid v3 fixture reports clean behavior validation metadata.
- Malformed command manifest schema is diagnostic-only and marks behavior `Invalid`.
- Malformed event manifest schema is diagnostic-only and marks behavior `Invalid`.
- Stateful behavior missing `save_state` or `restore_state` is unhealthy before state execution.
- Stateless editor behavior may omit state callbacks without being unhealthy.
- Runtime command names with interior NUL return a structured behavior call error.
- Runtime snapshot restore detects state schema mismatch before plugin restore callback execution.
- Unloaded plugin restore remains a restore diagnostic, not a host failure.
- Existing v2 fallback continues to load when the v3 descriptor is absent.
- Host log and diagnostic callback output remains visible after refactor.

Fixture feature flags may be added to `zircon_plugin_native_dynamic_fixture_native` only for malformed v3 cases that cannot be constructed with unit fixtures. Prefer local unit construction for validation-only rules and reserve real cdylib builds for ABI and callback behavior.

Expected validation commands for the implementation milestone are:

```powershell
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
cargo fmt --all --check
```

If active workspace contention makes full formatting or broad Cargo validation unsafe, the implementation report must state the scoped substitute and why it was used.

## Documentation

Update `docs/engine-architecture/runtime-editor-pluginized-export.md` with the accepted v3 hardening behavior, validation reports, schema mismatch restore handling, and exact test evidence.

If the native plugin loader module is reorganized, add or update source-mirrored docs under `docs/zircon_runtime/plugin/native_plugin_loader/`. Each affected document must include machine-readable related-code, implementation-files, plan-sources, and tests metadata.

## Non-Goals

This slice does not add a new ABI version, new host callback signatures, typed command/event manifest parsing, editor Plugin Manager UI, app profile/provider bootstrap changes, render provider wiring, material/zshader changes, VM/ZrVM integration, or real physics/animation backend behavior.

This slice also does not make NativeDynamic share Rust trait objects or host-owned runtime/editor objects. NativeDynamic continues to use stable C ABI values, byte payloads, capabilities, diagnostics, manifests, schema versions, and owned handles.

## Acceptance Criteria

- ABI v3 public C structs and callback signatures remain stable.
- Behavior metadata has a host-owned validation report available without invoking plugin callbacks.
- Invalid schema/callback combinations are diagnostic-first and visible through loader/live-host reports.
- Runtime state snapshots record schema version and restore reports schema mismatches before invoking incompatible restore callbacks.
- `native_plugin_abi.rs` no longer grows as an oversized mixed-responsibility file.
- Real fixture and focused live-host tests pass with recorded commands.
- Docs record implementation files, plan sources, tests, validation evidence, and remaining gaps.
