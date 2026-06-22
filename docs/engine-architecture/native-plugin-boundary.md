---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
implementation_files:
  - docs/engine-architecture/native-plugin-boundary.md
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with native plugin loader isolated
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-hot-update-0615 --message-format short --color never
  - cargo test -p zircon_runtime --lib native_runtime_hot_update --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-hot-update-0615 --message-format short --color never -- --test-threads=1 --nocapture
  - rustfmt --edition 2021 --check zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-hot-update-lifecycle-0615b --message-format short --color never
  - cargo test -p zircon_runtime --lib native_runtime_hot_update_report_applies_bridge_lifecycle_to_loaded_outcomes --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-hot-update-lifecycle-0615b --message-format short --color never -- --exact --test-threads=1 --nocapture
  - rustfmt --edition 2024 zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs zircon_runtime/src/plugin/native_plugin_loader/mod.rs zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-f1-ffi-guard-0622 --message-format short --color never
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-f1-ffi-guard-0622 --message-format short --color never ffi_panic_guard -- --test-threads=1
  - cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-f1-ffi-guard-0622 --message-format short --color never native_host_bridge_call_catches_plugin_method_panic -- --test-threads=1
  - native_runtime_hot_update_report_applies_bridge_lifecycle_to_loaded_outcomes (coverage added; focused lib-test lane timed out in compilation)
  - native_plugin_public_surface M4 gate status, explicit count fields, symbol decision group, migration debt, and unclassified symbol checks
  - plugin_surface_lifecycle_boundary Runtime 06 mirror, app NativePlugin call-site count, V3-only native ABI hard-cutover, unknown ABI rejection, hot reload failure injection, export_build_plan V1/V2 zero-usage, pending validation anchors
  - runtime_06_native_loader_tests_use_isolated_plugin_native_namespace
doc_type: module-detail
---

# Native Plugin Boundary

## Purpose

The main plugin path is VM/plugin lifecycle, stable host handles, slot hot reload, and neutral package/capability contracts. Native dynamic loading is not the runtime public main path. It should be isolated behind tooling, export, tests, or a narrow handwritten facade instead of being broadly re-exported from `zircon_runtime::plugin`.

## FFI Panic Boundary

All native host callbacks exposed through `extern "C"` must catch Rust panics before returning across the ABI. `ffi_panic_guard.rs` is the shared owner for that rule.

`host_api_adapter.rs` routes the 9 `ZrHostApiV3` callbacks through `catch_native_host_api_panic(...)`; panic is converted to `ZrStatusCode::Panic` with a stable diagnostic byte slice. `host_callbacks.rs` routes the 4 native plugin host-function-table callbacks through `catch_native_plugin_host_callback_panic(...)`; panic is converted to `ZIRCON_NATIVE_PLUGIN_STATUS_PANIC`.

The bridge-call regression `native_host_bridge_call_catches_plugin_method_panic` covers the real method-dispatch path: a panicking native bridge method records the enabled-call diagnostic and returns `ZrStatusCode::Panic` instead of unwinding through FFI.

## Current Audit

The structural audit now includes `native_plugin_public_surface`. Its scan, classification, and M4 gate implementation is folder-backed in `runtime_structure_audits/native_plugin_public_surface.py`; Markdown rendering is split into `runtime_structure_audits/native_plugin_public_surface_markdown.py`, so M4 native loader isolation evidence and output formatting have separate owners while the main audit script remains an orchestration boundary.

Current evidence:

- `zircon_runtime/src/plugin/mod.rs` exposes only the `pub mod native;` namespace seat for native loader and ABI symbols;
- `zircon_runtime/src/plugin/native.rs` owns the explicit native public namespace;
- `root_reexport_count = 0`;
- `native_namespace_reexport_count = 60`;
- `root_public_reexport_location_count = 0`;
- `public_reexport_location_count = 1`;
- `native_plugin_public_surface.py = 400`;
- `native_plugin_public_surface_markdown.py = 63`;
- rendered native public-surface Markdown output = 12 lines;
- native loader test files 3/3, native test namespace import files 2/2, and native test root import leaks 0/0;
- exported names under `plugin::native` include the V3 native ABI version, descriptor symbol, status constants, live-host types, runtime behavior calls, state snapshots, load reports, `NativePluginLoader`, and native bridge-method binding/report symbols;
- export source templates still call native loading through generated behavior.

The old root-surface native re-export migration is already cut over. The remaining NativeDynamic work is coupled to generated-code behavior and the pending Cargo/native validation lane.

## Runtime Hot Update Entry

`NativePluginLiveHost::hot_reload_runtime_plugins_from_export_root(...)` is the current NativeDynamic runtime hot-update application boundary. It discovers `plugins/native_plugins.toml` under an export root, filters the manifest package set to runtime-capable packages, and then routes each runtime package through the same live-host hot-reload state machine used by single-plugin reloads. The returned `NativePluginRuntimeHotUpdateReport` records manifest plugin ids, runtime plugin ids, loaded plugin ids, skipped non-runtime ids, per-plugin outcomes, and sorted diagnostics.

`NativePluginLiveHost::hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle(...)` is the lifecycle-attached variant for the same batch boundary. It first builds the normal `NativePluginRuntimeHotUpdateReport`, then calls `NativePluginRuntimeHotUpdateReport::apply_runtime_bridge_lifecycle(...)` to attach `Reload` provider lifecycle reports to successful runtime `HotReload` outcomes via the existing `RuntimePluginBridgeLifecycleState`. Skipped non-runtime packages and outcomes that already carry a bridge lifecycle report are left unchanged.

This does not claim a complete real cdylib success matrix. It establishes the manifest-driven runtime application/report surface that Hub/editor/export can call after package or delta promotion; platform-native signing profiles, notarization, real Cargo fixture success cases, and Hub/editor end-to-end invocation remain separate NativeDynamic slices.

## M4 Gate Output

The structural audit now reports `native_plugin_public_surface.m4_gate_status`. Current status is:

`classified-and-clear`

Current symbol classification:

- `native-abi-contract-public-debt = 31`
- `native-loader-discovery-public-debt = 7`
- `native-live-host-runtime-public-debt = 15`
- `native-behavior-report-public-debt = 3`
- `native-bridge-method-public-debt = 14`

Current gate evidence:

- `root_reexport_count = 0`
- `native_namespace_reexport_count = 60`
- `symbol_decision_count = 60`
- `symbol_decision_group_count = 5`
- `native_plugin_public_surface_migration_debt_count = 0`
- `unclassified_root_reexport_symbol_count = 0`
- `unclassified_native_namespace_symbol_count = 0`
- `root_public_reexport_location_count = 0`
- `public_reexport_location_count = 1`
- `native loader test files 3/3`
- `native test namespace import files 2/2`
- `native test root import leaks 0/0`

The classification now applies to the explicit `zircon_runtime::plugin::native` namespace. It confirms the old `zircon_runtime::plugin` root no longer carries native loader/ABI re-exports, that native loader tests import through the isolated namespace, that M3.1 has removed native plugin V1/V2 loader compatibility, and that M3.2 covers hot reload failure injection; it does not close Runtime 06 as a whole because the Cargo/native validation lane remains pending.

The Runtime 06 plan-status guard `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation` keeps the remaining `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane visible until runtime real-backend, plugin/native plugin, app, and plugin workspace checks have real evidence. The guard intentionally binds the current `m4_gate_status`, `classified-and-clear`, `root_reexport_count = 0`, and `native_namespace_reexport_count = 60` evidence to Runtime 06, the runtime index, Runtime 05 closeout, and the M0 review so native plugin public-surface evidence cannot drift.

`plugin_surface_lifecycle_boundary` now mirrors the wider Runtime 06 state through the Python structural audit, while `plugin_surface_lifecycle_markdown.py` owns the Markdown renderer. Current evidence: Runtime 06 source 14/14, mirror docs 5/5, `expected_source_file_count = 14`, `expected_doc_file_count = 5`, frontmatter `in_progress`, `last_refined = 2026-06-21`, `plugin_surface_lifecycle_boundary.py = 450`, `plugin_surface_lifecycle_markdown.py = 144`, native root re-export 0/0, native namespace re-export 60/60, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 0/0, `zircon_plugins` V1/V2 usage files 0/0, export_build_plan V1/V2 usage 0/0, native loader test files 3/3, native test namespace import files 2/2, native test root import leaks 0/0, fallback lifecycle failure tests 4/4, unknown ABI rejection, hot reload failure injection, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, and `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed` keep this document aligned with Runtime 06, the runtime index, runtime-interface convergence, and the M0 review.

## M4 Decision Rules

`native-abi-contract-public-debt` covers ABI structs, version constants, descriptor symbols, status constants, callback status values, byte slices, owned buffers, host-function tables, and schema-version records. These may remain available only through an explicit native ABI contract namespace used by build/tooling paths, not by flattening them from `zircon_runtime::plugin`.

`native-loader-discovery-public-debt` covers `NativePluginLoader`, loaded-plugin records, manifest rows, ABI v3 load-manifest contract rows, candidates, and load reports. These belong behind the native loader/discovery owner or a narrow export/tooling facade.

`native-live-host-runtime-public-debt` covers live-host commands, runtime behavior descriptors, runtime command reports, runtime hot-update reports, play-mode reports, runtime plugin state, and runtime state snapshots. These belong behind an isolated native live-host bridge, not the main runtime plugin namespace.

`native-behavior-report-public-debt` covers behavior call and validation reports. These may be reachable through an explicit native diagnostics owner, but they should not be broad root plugin API.

`native-bridge-method-public-debt` covers native bridge-method descriptors, bindings, call table entries, registration scopes, and live-host bridge reports. These belong behind a native bridge-method owner used by native plugin bridge lifecycle tests or tooling, not the main runtime plugin namespace.

Any future `unclassified-native-plugin-symbol` entry is a review blocker. Classify it with a target owner or remove it from the public root re-export before accepting the boundary.

## Target Shape

The current target shape is now the live shape:

- `zircon_runtime::plugin` remains the stable plugin contract surface for manifests, runtime plugin descriptors, feature registration, extension registry, runtime profiles, and scene hooks.
- VM/plugin lifecycle remains the primary runtime plugin path.
- Native loader internals are exposed only through the isolated `zircon_runtime::plugin::native` namespace or future tool/export-only facades.
- Public ABI declarations are exported only where needed by native plugin build/tooling contracts, not as root-level runtime plugin API.
- Generated export hosts call a stable handwritten export/native facade, if one is still needed, instead of directly loading native manifests.

## Hard-Cutover Rule

Do not preserve a compatibility re-export from `zircon_runtime::plugin` after the native loader moves. Call sites must update to the isolated owner path or to the new facade.

## Verification

Use the structural audit before calling this boundary converged:

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

The `native_plugin_public_surface.root_reexport_count` must remain zero after the M2.1 hard-cutover. If a deliberately narrow native export facade remains public, the audit should allow only that facade under an explicit owner namespace and continue rejecting broad root ABI/loader re-exports.

Before editing `zircon_runtime/src/plugin/mod.rs` or `native_plugin_loader`, inspect:

- `native_plugin_public_surface.root_reexport_count`
- `native_plugin_public_surface.native_namespace_reexport_count`
- `native_plugin_public_surface.symbol_decision_count`
- `native_plugin_public_surface.symbol_decision_group_count`
- `native_plugin_public_surface.symbol_decision_groups`
- `native_plugin_public_surface.native_plugin_public_surface_migration_debt_count`
- `native_plugin_public_surface.unclassified_root_reexport_symbols`
- `native_plugin_public_surface.unclassified_root_reexport_symbol_count`
- `native_plugin_public_surface.unclassified_native_namespace_symbol_count`
- `native_plugin_public_surface.root_public_reexport_location_count`
- `native_plugin_public_surface.public_reexport_location_count`
- `native_plugin_public_surface.m4_gate_status`

The gate is not clear if `m4_gate_status` stops being `classified-and-clear`, if `root_reexport_count` becomes nonzero, or if any native namespace symbol is unclassified.
