# NativeDynamic V3 Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the existing NativeDynamic ABI v3 behavior surface by deriving host-owned validation reports, preserving diagnostics, and preventing incompatible state restores before plugin callbacks run.

**Architecture:** `zircon_runtime::plugin::native_plugin_loader` remains the owner for ABI declarations, entry probing, behavior calls, validation, load reports, and live-host orchestration. ABI v3 C structs and callback signatures stay stable; new Rust-only validation DTOs are derived after ABI metadata is copied into owned Rust values. `zircon_app`, `zircon_editor`, Plugin Manager UI, and ZrVM integration remain consumers or future slices, not owners in this plan.

**Tech Stack:** Rust, Cargo, `libloading`, native `cdylib` fixture under `zircon_plugins/native_dynamic_fixture/native`, Zircon runtime plugin manifests, repository milestone-first validation policy.

---

## Context And Boundaries

This plan implements the approved design in `docs/superpowers/specs/2026-05-19-native-dynamic-v3-hardening-design.md` from the existing `main` checkout. It follows the repository policy that forbids worktrees and feature branches for this repository.

The owner role is the supporting runtime module `zircon_runtime::plugin::native_plugin_loader`. The loader already probes ABI v3 first, falls back to ABI v2 and ABI v1, captures host log and diagnostic callbacks, keeps `LoadedNativePlugin` library handles alive, and exposes live-host runtime behavior descriptors, command dispatch, state snapshots, restore reports, and play-mode helpers.

The accepted abstraction is a Rust-side behavior validation report computed from copied metadata. It is not a new ABI version, not a new callback table, and not a typed command/event manifest parser. The validation report must be available before invoking command, save, restore, or unload callbacks.

## Current Structure Baseline

`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs` is exactly 1000 lines and currently mixes C-compatible ABI declarations, symbol probing, entry invocation, descriptor conversion, behavior calls, owned-buffer release, host callback capture, package manifest parsing, and native string-list parsing. The first implementation milestone must split this file before adding meaningful validation logic.

`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs` currently defines runtime behavior descriptors and state snapshot DTOs inline. `NativePluginRuntimePluginState` stores only `plugin_id` and `state`, so restore cannot detect schema mismatches before invoking `restore_state`.

`zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs` currently flattens descriptor and entry diagnostics but does not expose validation diagnostics derived from behavior metadata.

`zircon_runtime/tests/native_plugin_loader_contract.rs` already builds the real native fixture and validates ABI v3 callback behavior, v2 fallback, host callback flattening, commands, owned-buffer release, save/restore, unload, and runtime registration projection.

## Files And Responsibilities

Create `zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs` to own `repr(C)` ABI declarations, callback type aliases, status constants, ABI version constants, descriptor symbol constants, and empty owned-buffer constructors. It must not own probing, parsing, validation, callback capture, or behavior invocation policy.

Create `zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs` to own C-string reads, required C-string error formatting, native string-list parsing, native symbol-name termination, and package-manifest TOML parsing helpers used by ABI conversion and host callback code.

Create `zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs` to own ABI v2/v3 host function table construction helpers, host ABI version callbacks, host capability queries, v3 host log/diagnostic capture, capture registration, capture draining, and flattening captured records into entry diagnostics.

Create `zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs` to own the internal `NativePluginBehavior` struct, `NativePluginBehaviorCallReport`, command/state/unload callback invocation, status-to-report conversion, missing-callback reports, and owned byte-buffer copying/free diagnostics.

Create `zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs` to own `NativePluginBehaviorHealth`, `NativePluginBehaviorValidationReport`, constants for supported schema ids, and pure validation functions that inspect copied metadata without invoking plugin callbacks.

Keep `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs` as the ABI orchestration façade for `probe_native_plugin_descriptor`, `call_native_plugin_entry`, `NativePluginDescriptor`, and `NativePluginEntryReport` conversion from ABI reports. After the split, it should contain wiring and descriptor/report conversion only.

Modify `zircon_runtime/src/plugin/native_plugin_loader/mod.rs` to declare the new child modules and re-export only the public ABI declarations, behavior call report, behavior validation report, health enum, descriptor, entry report, and loader/live-host types that are already intentionally public.

Modify `zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs` to expose runtime/editor behavior validation reports, runtime/editor behavior health, callback availability, and restore schema compatibility helpers through copied metadata, without leaking callback pointers or raw ABI structs outside the loader.

Modify `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs` to include behavior validation diagnostics in `entry_diagnostics()`, `diagnostics_for_runtime_plugin(...)`, and `diagnostics_for_editor_plugin(...)`, while preserving existing flattened host callback diagnostics.

Modify `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs` and `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs` so runtime behavior descriptors include validation reports and runtime snapshots store `state_schema_version` per plugin state. Restore must skip incompatible schema versions with diagnostics before invoking plugin restore callbacks.

Modify `zircon_runtime/tests/native_plugin_loader_contract.rs` to assert the valid fixture reports `Clean`, malformed fixture modes report `Invalid`, host callback diagnostics remain visible, v2 fallback remains unchanged, interior-NUL command names return structured behavior errors, and schema mismatch restore is skipped before the callback path.

Modify `zircon_plugins/native_dynamic_fixture/native/Cargo.toml` and `zircon_plugins/native_dynamic_fixture/native/src/lib.rs` only if real dynamic-library malformed cases are needed. Add feature flags for `malformed_command_schema`, `malformed_event_schema`, `stateful_missing_save`, or `stateful_missing_restore` only when unit construction cannot prove a rule without a real `cdylib`.

Update `docs/engine-architecture/runtime-editor-pluginized-export.md` with the implemented v3 validation report behavior, state schema restore gate, exact validation evidence, and remaining gaps.

Create or update `docs/zircon_runtime/plugin/native_plugin_loader/index.md` as the source-mirrored module document for the reorganized native loader. It must include machine-readable frontmatter with `related_code`, `implementation_files`, `plan_sources`, `tests`, and `doc_type`.

## Public Rust Types To Add

Add these Rust-only DTOs in `behavior_validation.rs` and re-export them through `native_plugin_loader::mod.rs` and `zircon_runtime::plugin` only if the existing public plugin surface already re-exports comparable native loader DTOs.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginBehaviorHealth {
    Clean,
    Degraded,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginBehaviorValidationReport {
    pub abi_version: u32,
    pub module_kind: PluginModuleKind,
    pub plugin_id: String,
    pub is_stateless: Option<bool>,
    pub state_schema_version: Option<u32>,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub has_command_manifest: bool,
    pub has_event_manifest: bool,
    pub has_invoke_command: bool,
    pub has_save_state: bool,
    pub has_restore_state: bool,
    pub has_unload: bool,
    pub diagnostics: Vec<String>,
    pub health: NativePluginBehaviorHealth,
}
```

The report uses `Option` for metadata that does not exist on ABI v1/v2 or when a behavior pointer is absent. A missing behavior is `Invalid` for a loaded ABI v3 entry that is expected to provide behavior metadata for runtime/editor inspection. ABI v2 behavior can produce a report with no schema ids and compatibility health semantics, but only ABI v3 unsupported schemas should produce v3 schema diagnostics.

## Validation Rules To Implement

The health states are exactly `Clean`, `Degraded`, and `Invalid`.

`Clean` means the validation report has no diagnostics.

`Degraded` means the plugin can remain loaded with reduced behavior capability. Degraded cases include missing `unload` and missing `invoke_command` when the plugin has no command manifest text to dispatch.

`Invalid` means required metadata is inconsistent or unsupported. Invalid cases include unsupported ABI v3 command manifest schema, unsupported ABI v3 event manifest schema, declared command schema with empty/missing command manifest text, declared event schema with empty/missing event manifest text, stateful behavior missing `save_state`, and stateful behavior missing `restore_state`.

Supported ABI v3 schema ids are exactly `zircon.native.command-manifest/3` and `zircon.native.event-manifest/3` for this slice.

Command/event manifest validation stays intentionally shallow. If the matching schema id is present, the matching manifest text must exist and contain at least one non-empty line. The plan does not add typed command/event manifest parsing or SDK-level schema validation.

Stateless behavior may omit `save_state` and `restore_state` without diagnostics. If stateless behavior declares a nonzero state schema version, report `Degraded` because the metadata is confusing but not unsafe.

Stateful behavior must provide both `save_state` and `restore_state`. If either callback is missing, report `Invalid` before state execution.

Missing `unload` remains diagnostic-only/no-op-compatible. `NativePluginLiveHost` must still allow handles to drop when unload is missing.

Runtime command names with interior NUL must return `NativePluginBehaviorCallReport { status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR, diagnostics: vec!["native plugin command name contained an interior NUL"], payload: None }` without invoking the plugin callback.

Runtime state snapshots must store `state_schema_version: Option<u32>` beside each `plugin_id` and state byte vector. Restore must compare snapshot and current plugin schema before invoking `restore_state`. If they differ, add the plugin id to `skipped_plugin_ids`, add a deterministic diagnostic, and skip callback invocation.

Use this schema mismatch diagnostic text so tests and future Plugin Manager UI can match it deterministically:

```text
runtime plugin {plugin_id} restore-state skipped because snapshot state schema {snapshot_schema:?} does not match loaded state schema {loaded_schema:?}
```

## Milestone 1: Module Split Without Behavior Change

Goal: Split `native_plugin_abi.rs` into responsibility-focused modules while preserving public ABI structs, symbols, callback signatures, behavior calls, diagnostics, and v2/v3 fixture behavior.

In-scope behaviors: ABI v1/v2/v3 declarations, descriptor probing, entry invocation, host callback capture, behavior call reports, owned-buffer release diagnostics, package manifest parsing, capability negotiation, and existing load/live-host behavior.

Dependencies: Current `native_plugin_abi.rs`, `loaded_native_plugin.rs`, `load_discovered.rs`, `native_plugin_load_report.rs`, and fixture contract tests.

Implementation slices:

- [ ] Create `abi_declarations.rs` and move the ABI constants, `repr(C)` structs, callback type aliases, and `NativePluginOwnedByteBufferV2::empty()` into it.
- [ ] Create `native_strings.rs` and move `read_required_c_string`, `read_optional_c_string`, `package_manifest_from_toml`, `parse_native_string_list`, and `native_symbol_name` into it with `pub(super)` visibility as needed.
- [ ] Create `behavior_calls.rs` and move internal `NativePluginBehavior`, public `NativePluginBehaviorCallReport`, callback invocation methods, status conversion, `error_report`, `missing_callback_report`, `status_diagnostics`, and `take_owned_bytes` into it.
- [ ] Create `host_callbacks.rs` and move v2/v3 host ABI version callbacks, host capability queries, v3 log/diagnostic callbacks, capture registry, capture guard, capture record structs, `granted_capabilities_for_entry`, and `module_capabilities` into it.
- [ ] Reduce `native_plugin_abi.rs` to descriptor/report conversion plus `probe_native_plugin_descriptor` and `call_native_plugin_entry`, importing helpers from the new child modules.
- [ ] Update `mod.rs`, `load_discovered.rs`, `loaded_native_plugin.rs`, and tests to use the new direct module exports with no compatibility modules, bridge files, or legacy re-export paths.
- [ ] Keep public names stable for `NativePluginAbiV1`, `NativePluginAbiV2`, `NativePluginAbiV3`, `NativePluginBehaviorV2`, `NativePluginBehaviorV3`, `NativePluginBehaviorCallReport`, `NativePluginByteSliceV2`, `NativePluginByteSliceV3`, `NativePluginCallbackStatusV2`, `NativePluginCallbackStatusV3`, `NativePluginDescriptor`, `NativePluginEntryReport`, host function table types, owned byte buffer types, schema version type, ABI version constants, descriptor symbol constants, and status constants.
- [ ] Add concise comments only where the split preserves a non-obvious invariant, such as keeping `LoadedNativePlugin.library` alive while callback pointers are invoked.

Testing stage: `M1 Native ABI Split Verification`.

Expected commands:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
```

Debug/correction loop: If imports break, fix direct caller paths to the new modules instead of adding old-path shims. If any behavior call test changes diagnostics, move the exact formatting code with the old behavior rather than rewriting assertions in this milestone.

Exit evidence: `native_plugin_abi.rs` is below the warning threshold and owns only ABI orchestration/conversion; scoped runtime check and live-host tests pass or failures are recorded as unrelated active workspace issues with exact diagnostics.

Lightweight checks during implementation: `cargo check -p zircon_runtime --lib --locked --jobs 1` may be run once before the testing stage if the split causes ambiguous Rust import/type errors.

## Milestone 2: Behavior Validation DTOs And Pure Rules

Goal: Add host-owned validation reports derived from copied behavior metadata without invoking plugin callbacks.

In-scope behaviors: Health enum, validation report DTO, supported schema ids, v2/v3 behavior validation, callback availability flags, manifest presence checks, validation diagnostics, unit tests for clean/degraded/invalid classification.

Dependencies: Milestone 1 module boundaries.

Implementation slices:

- [ ] Create `behavior_validation.rs` with `NativePluginBehaviorHealth`, `NativePluginBehaviorValidationReport`, supported schema id constants, and pure validation functions.
- [ ] Add callback availability accessors to `NativePluginBehavior` in `behavior_calls.rs`: `has_invoke_command()`, `has_save_state()`, `has_restore_state()`, and `has_unload()`.
- [ ] Add a validation method that builds a report from `plugin_id`, `module_kind`, `abi_version`, and `Option<&NativePluginBehavior>`.
- [ ] Store `behavior_validation: Option<NativePluginBehaviorValidationReport>` on `NativePluginEntryReport` or expose an equivalent computed report method that does not invoke callbacks.
- [ ] Ensure ABI v3 entries with clean fixture metadata produce `NativePluginBehaviorHealth::Clean`.
- [ ] Ensure missing `unload` produces `NativePluginBehaviorHealth::Degraded` with a diagnostic and does not become invalid.
- [ ] Ensure missing `invoke_command` with no command manifest produces `NativePluginBehaviorHealth::Degraded` with a diagnostic and does not block state-only behavior.
- [ ] Ensure unsupported command schema, unsupported event schema, missing manifest for declared schema, and stateful missing save/restore produce `NativePluginBehaviorHealth::Invalid`.
- [ ] Ensure stateless editor behavior with no state callbacks remains `Clean` when schema/manifest metadata is otherwise valid.
- [ ] Add unit tests in the native loader test tree that construct `NativePluginBehavior` or validation input values directly, avoiding real `cdylib` fixture variants for pure validation rules.

Testing stage: `M2 Behavior Validation Rule Verification`.

Expected commands:

```powershell
cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
```

Debug/correction loop: If pure validation tests fail because `NativePluginBehavior` is private to `behavior_calls.rs`, add narrow `pub(super)` constructors or `#[cfg(test)]` test builders inside the native loader module. Do not make callback pointers or raw ABI behavior structs public outside the native loader just for tests.

Exit evidence: Pure validation tests prove `Clean`, `Degraded`, and `Invalid` classifications and all required schema/callback edge cases.

Lightweight checks during implementation: Prefer `cargo check -p zircon_runtime --lib --locked --jobs 1` only if test-builder visibility or module imports become unclear.

## Milestone 3: Load Reports And Runtime Descriptors Carry Validation

Goal: Surface validation reports through loaded plugins, load reports, and runtime behavior descriptors while preserving existing flattened diagnostics.

In-scope behaviors: `LoadedNativePlugin` validation accessors, load-report diagnostics, runtime behavior descriptor validation field, live-host descriptor listing, runtime registration diagnostics, host callback diagnostic preservation.

Dependencies: Milestone 2 validation DTOs.

Implementation slices:

- [ ] Add `runtime_behavior_validation_report()` and `editor_behavior_validation_report()` accessors to `LoadedNativePlugin` returning cloned or borrowed validation reports.
- [ ] Add `validation_report: Option<NativePluginBehaviorValidationReport>` to `NativePluginRuntimeBehaviorDescriptor`.
- [ ] Populate runtime behavior descriptors from `LoadedNativePlugin::runtime_behavior_validation_report()` in `native_plugin_live_host.rs`.
- [ ] Extend `NativePluginLoadReport::entry_diagnostics()` to include validation diagnostics formatted as `native plugin {plugin_id}: {message}` while keeping current host log and host diagnostic entries.
- [ ] Extend `diagnostics_for_runtime_plugin(...)` and `diagnostics_for_editor_plugin(...)` to include only matching module-kind validation diagnostics.
- [ ] Keep `runtime_plugin_registration_reports()` and `runtime_plugin_feature_registration_reports()` attaching validation diagnostics through existing `diagnostics_for_runtime_plugin(...)` paths.
- [ ] Add or update unit tests proving validation diagnostics appear in per-plugin diagnostics and runtime descriptor reports without duplicating or dropping host callback diagnostics.

Testing stage: `M3 Validation Report Surfacing Verification`.

Expected commands:

```powershell
cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
```

Debug/correction loop: If diagnostics become duplicated, centralize sort/dedup in the report path that aggregates diagnostics. If host callback diagnostics disappear, restore `callback_capture.into_entry_diagnostics()` extension before validation diagnostics are appended.

Exit evidence: Runtime descriptor reports include validation reports, load reports include validation diagnostics, and existing host callback diagnostic assertions remain valid.

Lightweight checks during implementation: Use a scoped `cargo check` only if adding the descriptor field requires public API updates across crates.

## Milestone 4: State Snapshot Schema Guard And Command Hardening

Goal: Prevent incompatible state restores before callback invocation and keep command-call error handling deterministic.

In-scope behaviors: Snapshot state schema version storage, restore schema mismatch skip, unloaded plugin restore diagnostics, interior-NUL command report, play-mode preservation of restore diagnostics, unit tests.

Dependencies: Milestones 2 and 3 report accessors.

Implementation slices:

- [ ] Add `pub state_schema_version: Option<u32>` to `NativePluginRuntimePluginState`.
- [ ] Populate `state_schema_version` from `LoadedNativePlugin::runtime_state_schema_version()` when saving runtime plugin states.
- [ ] Update `native_plugin_live_host/tests.rs` snapshot literals to include `state_schema_version`.
- [ ] Add a restore helper in `loaded_native_plugin.rs` or `native_plugin_live_host.rs` that compares snapshot schema and current loaded schema before `restore_runtime_state(...)` is called.
- [ ] In `restore_runtime_plugin_states(...)`, skip schema mismatches by pushing the plugin id to `skipped_plugin_ids` and adding the exact schema mismatch diagnostic from this plan.
- [ ] Preserve unloaded plugin restore behavior as a diagnostic and skipped plugin, not a host failure.
- [ ] Add a unit test that creates a snapshot with mismatched schema and proves restore produces no callback calls for that plugin. If a callback-counting test plugin is needed, construct a `LoadedNativePlugin` using `this_process_library()` and a test behavior with a restore callback that increments a static counter.
- [ ] Add or preserve a test for `invoke_runtime_plugin_command("bad\0name", b"")` returning the structured error report without invoking the plugin callback.
- [ ] Verify `enter_runtime_play_mode()` and `exit_runtime_play_mode(...)` continue to combine command and restore diagnostics without flattening away restore mismatch detail.

Testing stage: `M4 Runtime State Restore Verification`.

Expected commands:

```powershell
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
```

Debug/correction loop: If old snapshot construction appears outside native loader tests, update every reachable call site directly rather than adding a compatibility constructor that hides missing schema metadata.

Exit evidence: Restore mismatch is reported and skipped before plugin restore callback execution; unloaded restore remains diagnostic-only; command interior-NUL behavior is deterministic.

Lightweight checks during implementation: Use `cargo check -p zircon_runtime --lib --locked --jobs 1` if the snapshot DTO field changes public callers outside the native loader module.

## Milestone 5: Real Fixture And Contract Coverage

Goal: Prove the valid ABI v3 fixture remains clean and malformed real-dynamic cases produce expected invalid validation reports where unit construction is insufficient.

In-scope behaviors: Fixture `Cargo.toml` feature flags if required, fixture constants or static behavior variants, contract test assertions for clean fixture, malformed schema fixture, host callback preservation, v2 fallback preservation.

Dependencies: Milestones 2 through 4.

Implementation slices:

- [ ] Update `native_loader_exposes_v3_behavior_boundary_from_real_fixture` to assert the runtime validation report health is `Clean`, validation diagnostics are empty, state schema version is `Some(3)`, command schema is `Some("zircon.native.command-manifest/3")`, and event schema is `Some("zircon.native.event-manifest/3")`.
- [ ] Keep existing assertions that runtime registration diagnostics include `runtime v3 entry reached with host ABI table`, host log output, and host diagnostic output.
- [ ] Keep existing v2 fallback assertions in `native_loader_falls_back_to_v2_when_v3_descriptor_is_absent` unchanged except for any new validation report assertions that make sense for ABI v2 compatibility.
- [ ] If unit tests cannot cover malformed v3 schema through copied metadata, add fixture features `malformed_command_schema` and `malformed_event_schema` in `zircon_plugins/native_dynamic_fixture/native/Cargo.toml`.
- [ ] If fixture malformed schema features are added, gate `COMMAND_MANIFEST_SCHEMA` and `EVENT_MANIFEST_SCHEMA` constants in `lib.rs` so the descriptor and callback table stay ABI v3-identical while only schema strings change.
- [ ] Add contract tests that build the fixture with each malformed schema feature and assert load remains diagnostic-first, the plugin remains inspectable, and the matching validation health is `Invalid`.
- [ ] Do not add fixture features for stateful missing save/restore unless pure unit tests cannot construct the validation input; callback absence is metadata and should be covered without a `cdylib` variant.

Testing stage: `M5 Native Fixture Contract Verification`.

Expected commands:

```powershell
cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
```

Debug/correction loop: If the real fixture build fails because of feature wiring, fix `zircon_plugins/native_dynamic_fixture/native/Cargo.toml` and fixture `#[cfg(...)]` guards. If the root workspace compiles but plugin workspace lockfile contention blocks `--locked`, record the exact Cargo lock error and do not regenerate a shared dirty lockfile unless that becomes explicit scope.

Exit evidence: Real valid v3 fixture is clean, malformed schema fixture paths are invalid when implemented, v2 fallback still loads, and host callback diagnostics remain visible.

Lightweight checks during implementation: Avoid repeated fixture builds until this testing stage unless a fixture feature compile error blocks progress.

## Milestone 6: Documentation And Acceptance Evidence

Goal: Update module and architecture documentation with implemented behavior, validation evidence, and remaining non-goals before considering the slice complete.

In-scope behaviors: Source-mirrored native loader docs, architecture doc update, session note update, exact validation evidence, remaining risk statement.

Dependencies: Milestones 1 through 5.

Implementation slices:

- [ ] Create `docs/zircon_runtime/plugin/native_plugin_loader/index.md` if it does not already exist.
- [ ] Use this frontmatter shape in the native loader doc and keep paths current:

```markdown
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
```

- [ ] Document the module split, ABI stability rule, validation health model, schema-id rules, callback consistency rules, load-report diagnostic flow, snapshot schema guard, and remaining non-goals in `docs/zircon_runtime/plugin/native_plugin_loader/index.md`.
- [ ] Update `docs/engine-architecture/runtime-editor-pluginized-export.md` with the NativeDynamic v3 hardening behavior, validation report availability, restore schema mismatch behavior, and exact test evidence.
- [ ] Update `.codex/sessions/20260519-0107-native-dynamic-behavior-design.md` with implementation status, touched modules, tests run, failures, and next step.
- [ ] If the implementation finishes cleanly and no handoff is needed, retire the active session note by deleting it or moving it to `.codex/sessions/archive/` with `status: completed`.

Testing stage: `M6 Documentation And Acceptance Verification`.

Expected commands:

```powershell
cargo fmt --all --check
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
```

Debug/correction loop: If formatting touches unrelated dirty files, inspect the diff before claiming success and do not revert other sessions' work. If full `cargo fmt --all --check` or scoped Cargo commands are blocked by active workspace contention, record the exact command, failure, and scoped substitute used.

Exit evidence: Docs include the machine-readable headers, code paths, plan sources, tests, validation evidence, and remaining gaps. Validation output is recorded with exact commands and status.

Lightweight checks during implementation: No separate lightweight checks are needed for docs-only edits unless markdown frontmatter is malformed enough to break repository tooling.

## Final Acceptance Gate

Run these commands in the final testing stage before claiming the NativeDynamic v3 hardening slice is complete:

```powershell
cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
cargo test -p zircon_runtime --lib native_plugin_loader --locked --jobs 1
cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1
cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
cargo fmt --all --check
```

If the root workspace has unrelated dirty compile failures or target contention, the final report must say that workspace-wide validation was not claimed and must include the exact scoped evidence that did run.

## Out Of Scope

This plan does not add ABI v4, new C structs, new host callback signatures, typed command/event manifest parsing, Plugin Manager UI, editor panes, `zircon_app` bootstrap behavior changes, app provider composition changes, render/UI/material work, VM/ZrVM integration, or Rust trait-object sharing across dynamic boundaries.

This plan does not preserve old module paths as compatibility shims after the split. Callers must be moved to the new direct module exports in the same change.

## Self-Review Checklist

- [ ] Spec coverage: every acceptance criterion from `2026-05-19-native-dynamic-v3-hardening-design.md` maps to at least one milestone.
- [ ] Placeholder scan: the plan contains no deferred implementation placeholders.
- [ ] Type consistency: report, health, descriptor, snapshot, and restore field names match across milestones.
- [ ] Boundary check: all implementation stays inside `zircon_runtime::plugin::native_plugin_loader`, tests, fixture, and docs.
- [ ] Validation check: every milestone has a named testing stage with exact commands and correction loop.
