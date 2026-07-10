---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/ffi_panic_guard.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
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
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/bridge_method_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_strings.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/reports.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
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
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs::engine_compat_reports_empty_comparator_with_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/compatibility.rs::engine_compat_reports_invalid_version_component_with_typed_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs::review_f5_native_plugin_distribution_compat_uses_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs::native_host_api_adapter_reports_unknown_stage_with_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs::native_host_api_adapter_utf8_error_preserves_source
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs::review_f5_native_host_api_adapter_uses_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs::native_hot_reload_snapshot_save_reports_typed_status_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs::native_live_host_rollback_plan_reports_when_previous_plugin_was_restored
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs::hot_reload_state_restore_failure_rolls_back_and_reports
  - rustc --edition=2021 --test zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs::native_registration_replay_reports_typed_schema_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/registration_replay.rs::native_registration_replay_reports_typed_duplicate_system_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs::review_f5_native_live_host_hot_reload_uses_typed_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs::review_f5_native_live_host_registration_replay_uses_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs::native_live_host_bridge_methods_report_typed_missing_manifest_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/bridge_bindings.rs::native_live_host_bridge_methods_report_typed_missing_method_slot_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs::review_f5_native_live_host_bridge_methods_use_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs::native_live_host_runtime_behavior_reports_typed_unloaded_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs::review_f5_native_live_host_runtime_behavior_uses_typed_error
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs::native_live_host_loading_lock_reports_typed_error
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs::review_f5_native_live_host_loading_uses_typed_error
doc_type: module-detail
---

# Native Plugin Boundary

## Purpose

The main plugin path is VM/plugin lifecycle, stable host handles, slot hot reload, and neutral package/capability contracts. Native dynamic loading is not the runtime public main path. It should be isolated behind tooling, export, tests, or a narrow handwritten facade instead of being broadly re-exported from `zircon_runtime::plugin`.

## FFI Panic Boundary

All native host callbacks exposed through `extern "C"` must catch Rust panics before returning across the ABI. `ffi_panic_guard.rs` is the shared owner for that rule.

`host_api_adapter.rs` routes the 9 `ZrHostApiV3` callbacks through `catch_native_host_api_panic(...)`; panic is converted to `ZrStatusCode::Panic` with a stable diagnostic byte slice. `host_callbacks.rs` routes the 4 native plugin host-function-table callbacks through `catch_native_plugin_host_callback_panic(...)`; panic is converted to `ZIRCON_NATIVE_PLUGIN_STATUS_PANIC`.

The bridge-call regression `native_host_bridge_call_catches_plugin_method_panic` covers the real method-dispatch path: a panicking native bridge method records the enabled-call diagnostic and returns `ZrStatusCode::Panic` instead of unwinding through FFI.

## Hot-reload rollback disposition

`NativePluginHotReloadState` tracks the previous native package with an explicit disposition: not loaded, held for rollback, unloaded, or restored. When replacement state restore fails, the host unloads the replacement, restores the previous snapshot, reinserts the previous package, and then marks the disposition as restored before formatting the lifecycle error. The diagnostic therefore describes the actual live-host state instead of confusing an earlier unload with the final rollback result.

## Current Audit

The structural audit now includes `native_plugin_public_surface`. Its scan, classification, and M4 gate implementation is folder-backed in `runtime_structure_audits/native_plugin_public_surface.py`; Markdown rendering is split into `runtime_structure_audits/native_plugin_public_surface_markdown.py`, so M4 native loader isolation evidence and output formatting have separate owners while the main audit script remains an orchestration boundary.

Current evidence:

- `zircon_runtime/src/plugin/mod.rs` exposes only the `pub mod native;` namespace seat for native loader and ABI symbols;
- `zircon_runtime/src/plugin/native.rs` owns the explicit native public namespace;
- `root_reexport_count = 0`;
- `native_namespace_reexport_count = 64`;
- `root_public_reexport_location_count = 0`;
- `public_reexport_location_count = 1`;
- `native_plugin_public_surface.py = 400`;
- `native_plugin_public_surface_markdown.py = 63`;
- rendered native public-surface Markdown output = 12 lines;
- native loader test files 4/4, native test namespace import files 3/3, and native test root import leaks 0/0;
- exported names under `plugin::native` include the V3 native ABI version, descriptor symbol, status constants, live-host types, runtime behavior calls, state snapshots, load reports, `NativePluginLoader`, and native bridge-method binding/report symbols;
- export source templates still call native loading through generated behavior.

The old root-surface native re-export migration is already cut over. The remaining NativeDynamic work is coupled to generated-code behavior and the pending Cargo/native validation lane.

2026-07-01 Runtime 06 native hot-update/replay public-surface audit sync keeps the explicit `zircon_runtime::plugin::native` namespace as the only public native seat. `NativePluginRuntimeDeltaHotUpdateReport`, `NativePluginRuntimeDeltaHotUpdateRequest`, `NativePluginRuntimeRegistrationReplayReport`, and `NativePluginRuntimeRegistrationSystemReplay` are classified in the existing native live-host runtime group. Current evidence: `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, native root re-export 0/0, native namespace re-export 64/64, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, `last_refined = 2026-07-01`, `mirror_docs_guard_present = true`, `risks = []`, and standalone plugin_surface_lifecycle 3/3. This is a static mirror/audit sync only; Runtime 06 still waits on the declared `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane.

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
- `native_namespace_reexport_count = 64`
- `symbol_decision_count = 60`
- `symbol_decision_group_count = 5`
- `native_plugin_public_surface_migration_debt_count = 0`
- `unclassified_root_reexport_symbol_count = 0`
- `unclassified_native_namespace_symbol_count = 0`
- `root_public_reexport_location_count = 0`
- `public_reexport_location_count = 1`
- `native loader test files 4/4`
- `native test namespace import files 3/3`
- `native test root import leaks 0/0`

The classification now applies to the explicit `zircon_runtime::plugin::native` namespace. It confirms the old `zircon_runtime::plugin` root no longer carries native loader/ABI re-exports, that native loader tests import through the isolated namespace, that M3.1 has removed native plugin V1/V2 loader compatibility, and that M3.2 covers hot reload failure injection; it does not close Runtime 06 as a whole because the Cargo/native validation lane remains pending.

The Runtime 06 plan-status guard `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation` keeps the remaining `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane visible until runtime real-backend, plugin/native plugin, app, and plugin workspace checks have real evidence. The guard intentionally binds the current `m4_gate_status`, `classified-and-clear`, `root_reexport_count = 0`, and `native_namespace_reexport_count = 64` evidence to Runtime 06, the runtime index, Runtime 05 closeout, and the M0 review so native plugin public-surface evidence cannot drift.

`plugin_surface_lifecycle_boundary` now mirrors the wider Runtime 06 state through the Python structural audit, while `plugin_surface_lifecycle_markdown.py` owns the Markdown renderer. Current evidence: Runtime 06 source 14/14, mirror docs 5/5, `expected_source_file_count = 14`, `expected_doc_file_count = 5`, frontmatter `in_progress`, `last_refined = 2026-07-01`, `plugin_surface_lifecycle_boundary.py = 450`, `plugin_surface_lifecycle_markdown.py = 144`, native root re-export 0/0, native namespace re-export 64/64, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 0/0, `zircon_plugins` V1/V2 usage files 0/0, export_build_plan V1/V2 usage 0/0, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, fallback lifecycle failure tests 4/4, unknown ABI rejection, hot reload failure injection, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, and `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed` keep this document aligned with Runtime 06, the runtime index, runtime-interface convergence, and the M0 review.

## Distribution Compatibility Diagnostics

`plugin/native_plugin_loader/compatibility.rs` owns the NativeDynamic package distribution compatibility preflight used by `load_discovered.rs`. The loader still reports human-readable skip diagnostics through `native_distribution_compatibility_diagnostic(...)`, but the internal `engine_compat` range parser is now typed: `NativeDistributionCompatibilityError` covers empty comparators, empty versions, invalid major/minor/patch shape, and non-numeric version components such as `NativeDistributionCompatibilityError::NonNumericVersionComponent`.

Runtime 15 records this as `Runtime 15 F5 native plugin distribution compatibility typed errors` / `runtime_15_native_plugin_distribution_compat_typed_errors_static_passed_cargo_deferred`. The guard `review_f5_native_plugin_distribution_compat_uses_typed_error` locks the typed parser, the unchanged diagnostic boundary, and status/doc anchors. This change does not alter dist form filtering, ABI v3 matching, `engine_compat` range semantics, or the NativeDynamic public namespace.

Runtime 15 also records the registration-manifest parser follow-up as `Runtime 15 F5 native plugin registration manifest typed errors` / `runtime_15_native_plugin_registration_manifest_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/registration_manifest.rs` now reports TOML, schema, stage, and bridge-field validation through `NativePluginRegistrationManifestError`; `native_plugin_live_host/registration_replay.rs` still formats those errors at the live-host report boundary, so registration manifest replay keeps string diagnostics at the live-host boundary. The guard `review_f5_native_plugin_registration_manifest_uses_typed_error` locks the typed parser/validator, the unchanged replay diagnostic boundary, and the Runtime 15/status mirrors.

Runtime 15 records the behavior ABI parser follow-up as `Runtime 15 F5 native plugin behavior ABI typed errors` / `runtime_15_native_plugin_behavior_abi_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/behavior_calls.rs` now reports unsupported behavior ABI versions through `NativePluginBehaviorError::UnsupportedAbiVersion`; `plugin/native_plugin_loader/native_plugin_abi.rs` wraps that typed source in `NativePluginEntryAbiError::InvalidBehavior`, so behavior ABI parse is wrapped by the entry ABI typed error before the loader report boundary. The guard `review_f5_native_plugin_behavior_abi_uses_typed_error` locks the typed behavior parser, the entry ABI wrapper, and the Runtime 15/status mirrors.

Runtime 15 records the bridge-method table parser follow-up as `Runtime 15 F5 native bridge method ABI typed errors` / `runtime_15_native_bridge_method_abi_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/bridge_method_abi.rs` now reports table ABI mismatch, null method pointer with nonzero count, required C-string field failures, and missing callbacks through `NativeBridgeMethodAbiError::UnsupportedTableAbiVersion` and sibling variants; `plugin/native_plugin_loader/native_plugin_abi.rs` wraps that typed source in `NativePluginEntryAbiError::InvalidBridgeMethods`, so bridge method ABI parse is wrapped by the entry ABI typed error before the loader report boundary. The guard `review_f5_native_bridge_method_abi_uses_typed_error` locks the typed bridge-method parser, the entry ABI wrapper, and the Runtime 15/status mirrors.

Runtime 15 records the manifest-collection follow-up as `Runtime 15 F5 native plugin manifest collection typed errors` / `runtime_15_native_plugin_manifest_collection_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/collect_manifests.rs` now reports recursive directory enumeration and entry inspection failures through `NativePluginManifestCollectionError::EnumerateRoot` and `InspectEntry`, preserving the `std::io::Error` source. `plugin/native_plugin_loader/discover.rs` still formats that error into `NativePluginLoadReport.diagnostics`, so manifest collection keeps string diagnostics at the load-report boundary. The guard `review_f5_native_plugin_manifest_collection_uses_typed_error` locks the typed collection owner, the unchanged report boundary, and the Runtime 15/status mirrors.

Runtime 15 records the manifest-candidate follow-up as `Runtime 15 F5 native plugin manifest candidate typed errors` / `runtime_15_native_plugin_manifest_candidate_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/candidate_from_manifest.rs` now reports manifest file read failures, TOML parse failures, and missing runtime/editor module declarations through `NativePluginManifestCandidateError::ReadManifest`, `ParseManifest`, and `MissingRuntimeOrEditorModule`. `push_candidate_from_manifest_path(...)` still formats that error into `NativePluginLoadReport.diagnostics`, so manifest candidate keeps string diagnostics at the load-report boundary. The guard `review_f5_native_plugin_manifest_candidate_uses_typed_error` locks the typed candidate owner, the unchanged report boundary, and the Runtime 15/status mirrors.

Runtime 15 records the native string helper follow-up as `Runtime 15 F5 native plugin string helper typed errors` / `runtime_15_native_plugin_string_helper_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_strings.rs` now reports required C-string and embedded package manifest TOML failures through `NativeStringError::MissingRequiredField` and `NativeStringError::InvalidPackageManifest`, preserving the TOML source for invalid embedded manifests. `plugin/native_plugin_loader/native_plugin_abi.rs` still formats descriptor helper errors at the descriptor loader-report boundary and wraps entry helper errors in `NativePluginEntryAbiError::InvalidPackageManifest`; `bridge_method_abi.rs` wraps required-field failures in its own typed ABI error, so native string helpers keep string diagnostics at descriptor and entry loader-report boundaries. The guard `review_f5_native_plugin_string_helpers_use_typed_error` locks the typed helper owner and unchanged diagnostic boundaries.

Runtime 15 records the descriptor ABI parser follow-up as `Runtime 15 F5 native plugin descriptor ABI typed errors` / `runtime_15_native_plugin_descriptor_abi_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_abi.rs` now reports null descriptor symbols, unsupported descriptor ABI versions, required-field failures, and embedded package manifest failures through `NativePluginDescriptorAbiError::NullDescriptorSymbol`, `UnsupportedAbiVersion`, `InvalidRequiredField`, and `InvalidPackageManifest`. `probe_native_plugin_descriptor(...)` still formats that typed error at the loader report boundary, so descriptor ABI parse keeps string diagnostics at the loader report boundary. The guard `review_f5_native_plugin_descriptor_abi_uses_typed_error` locks the typed descriptor parser and unchanged loader report boundary.

Runtime 15 records the entry ABI parser follow-up as `Runtime 15 F5 native plugin entry ABI typed errors` / `runtime_15_native_plugin_entry_abi_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_abi.rs` now reports descriptor ABI gate failures, missing entry symbols, granted capability CString failures, null entry reports, unsupported entry-report ABI versions, behavior parse failures, embedded package manifest failures, and bridge method table failures through `NativePluginEntryAbiError::UnsupportedDescriptorAbiVersion`, `MissingEntrySymbol`, `InvalidGrantedCapabilities`, `NullEntryReport`, `UnsupportedEntryAbiVersion`, `InvalidBehavior`, `InvalidPackageManifest`, and `InvalidBridgeMethods`. `call_native_plugin_entry(...)` still formats that typed error at the loader report boundary, so entry ABI parse keeps string diagnostics at the loader report boundary. The guard `review_f5_native_plugin_entry_abi_uses_typed_error` locks the typed entry parser and unchanged loader report boundary.

Runtime 15 records the host API adapter follow-up as `Runtime 15 F5 native host API adapter typed errors` / `runtime_15_native_host_api_adapter_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/host_api_adapter.rs` now reports native host registration failures through `NativeHostApiAdapterError::InvalidPluginModuleOwner`, `InvalidUtf8`, `UnknownSystemStage`, `InvalidSystemSet`, `RegisterSystem`, `UnknownPluginModuleOwner`, and `RegisterComponent`, preserving `RuntimeExtensionRegistryError` and `Utf8Error` sources where they exist. `NativeHostApiV3RegistrationScope::new(...)` still formats the typed error at the public construction boundary and the C ABI callbacks still return `ZrStatusCode::Error` for failed register-system/register-component calls, so host API adapter keeps string diagnostics at public construction and C ABI status boundaries. The guard `review_f5_native_host_api_adapter_uses_typed_error` locks the typed adapter owner and unchanged host ABI status boundary.

Runtime 15 records the native live-host hot reload follow-up as `Runtime 15 F5 native live-host hot reload typed errors` / `runtime_15_native_live_host_hot_reload_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs` now reports runtime snapshot save and restore failures through `NativePluginHotReloadError::SaveRuntimeState`, `MissingRuntimeStatePayload`, `StateSchemaMismatch`, and `RestoreRuntimeState`, preserving plugin ids, callback status codes, schema versions, and restore diagnostics. `plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs` still formats that typed error only at the public hot-reload and rollback diagnostic boundary, so native live-host hot reload keeps string diagnostics at public lifecycle and rollback boundaries. The guard `review_f5_native_live_host_hot_reload_uses_typed_error` locks the typed hot-reload owner and unchanged lifecycle diagnostics boundary.

Runtime 15 records the native live-host registration replay follow-up as `Runtime 15 F5 native live-host registration replay typed errors` / `runtime_15_native_live_host_registration_replay_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs` now reports replay failures through `NativePluginRegistrationReplayError`, including `UnsupportedManifestSchema`, `InvalidRegistrationManifest`, `InvalidRegistrationSystem`, `UnknownBridgeInterface`, `RegistryInternPluginModule`, `RegistryInternSystemSet`, and `RegisterNativeSystem`. Those variants preserve plugin ids, system ids, bridge method context, and `NativePluginRegistrationManifestError` or `RuntimeExtensionRegistryError` sources. The public replay API still formats those errors into `NativePluginRuntimeRegistrationReplayReport.diagnostics`, so native live-host registration replay keeps string diagnostics at public replay report boundaries. The guard `review_f5_native_live_host_registration_replay_uses_typed_error` lives with the hot-reload guard in `typed_error_convergence/native_plugin_loader/live_host.rs`.

Runtime 15 records the native live-host bridge methods follow-up as `Runtime 15 F5 native live-host bridge methods typed errors` / `runtime_15_native_live_host_bridge_methods_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs` now reports bridge binding install, discovery, scope creation, provider reload, and manifest slot lookup failures through `NativePluginBridgeMethodError`, including `RuntimePluginNotLoaded`, `MissingDiscoveredBridgeMethodTable`, `MissingPackageManifest`, `MissingInstalledBridgeMethodBindings`, `InvalidBridgeMethodManifest`, `BridgeCallScope`, `BridgeLifecycleRejected`, and `MissingDeclaredBridgeMethod`. Those variants preserve plugin ids, bridge interface ids, method names, and `NativeBridgeMethodManifestError` or `RuntimeExtensionRegistryError` sources. The discovery diagnostic helper accepts a `Display` source so loading can pass `NativePluginBridgeMethodError` directly while lifecycle string diagnostics keep the same public formatting path. The public live-host methods still format typed errors at their API/report boundary, so native live-host bridge methods keep string diagnostics at public live-host boundaries. The guard `review_f5_native_live_host_bridge_methods_use_typed_error` lives with the native live-host typed-error guards in `typed_error_convergence/native_plugin_loader/live_host.rs`.

Runtime 15 records the native live-host runtime behavior follow-up as `Runtime 15 F5 native live-host runtime behavior typed errors` / `runtime_15_native_live_host_runtime_behavior_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/runtime_behavior.rs` now reports runtime descriptor lookup, command dispatch, state snapshot/restore, and play-mode enter/exit host failures through `NativePluginRuntimeBehaviorError`, including `LiveHostLock` and `RuntimePluginNotLoaded`. Those variants preserve the live-host lock diagnostic and plugin ids. The public live-host methods still format typed errors at their API/report boundary, and restore of missing plugins still records diagnostics in the restore report, so native live-host runtime behavior keeps string diagnostics at public live-host boundaries. The guard `review_f5_native_live_host_runtime_behavior_uses_typed_error` lives with the native live-host typed-error guards in `typed_error_convergence/native_plugin_loader/live_host.rs`.

Runtime 15 records the native live-host loading follow-up as `Runtime 15 F5 native live-host loading typed errors` / `runtime_15_native_live_host_loading_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/loading.rs` now reports load-report lock poison, unload-before-reload failure, and runtime bridge method binding installation failure through `NativePluginLiveHostLoadingError`, including `NativePluginLiveHostLoadingError::LiveHostLockPoisoned`, `UnloadBeforeReload`, and `RuntimeBridgeMethodBindings`. Public load and loaded-id APIs still format typed errors at the live-host API boundary, so native live-host loading keeps string diagnostics at public live-host boundaries. The bridge-method, registration-replay, and runtime-behavior typed wrappers preserve the loading error as their `source`; `review_f5_native_live_host_loading_uses_typed_error` locks that boundary with the other native live-host typed-error guards.

Runtime 15 records the native live-host behavior diagnostics follow-up as `Runtime 15 F5 native live-host behavior diagnostics typed errors` / `runtime_15_native_live_host_behavior_diagnostics_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs` now reports non-OK native behavior callback reports through `NativePluginBehaviorDiagnosticError`, including `NativePluginBehaviorDiagnosticError::FailedStatus`. Loading and lifecycle typed errors preserve that diagnostic helper as their `source`, while public live-host APIs and rollback diagnostics still format typed errors at existing boundaries, so native live-host behavior diagnostics keep string diagnostics at public live-host boundaries. Behavior report schema, callback status codes, lifecycle report shape, and the NativeDynamic public namespace are unchanged; `review_f5_native_live_host_behavior_diagnostics_use_typed_error` locks that helper in a separate native-loader diagnostics child owner.

Runtime 15 records the native live-host lifecycle follow-up as `Runtime 15 F5 native live-host lifecycle typed errors` / `runtime_15_native_live_host_lifecycle_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs` now reports unload and hot-reload lifecycle internals through `NativePluginLiveHostLifecycleError`, including `NativePluginLiveHostLifecycleError::HotReloadDidNotLoad`, `HotReloadSnapshot`, `HotReloadRestore`, `RuntimeBridgeMethodBindings`, and `UnsupportedLiveHostModuleKind`. Public unload/hot-reload APIs and export-root hot-update reports still format typed errors at their diagnostic boundary, so native live-host lifecycle keeps string diagnostics at public live-host and hot-update report boundaries. Hot reload rollback behavior, bridge binding semantics, lifecycle report schema, and the NativeDynamic public namespace are unchanged; `review_f5_native_live_host_lifecycle_uses_typed_error` locks the boundary with the other live-host guards.

Runtime 15 records the native live-host bridge lifecycle follow-up as `Runtime 15 F5 native live-host bridge lifecycle typed errors` / `runtime_15_native_live_host_bridge_lifecycle_typed_errors_static_passed_cargo_deferred`. `plugin/native_plugin_loader/native_plugin_live_host/bridge_lifecycle.rs` now reports load/hot-reload/unload wrapper failures through `NativePluginBridgeLifecycleError`, including `NativePluginBridgeLifecycleError::BridgeLifecycleRejected` and `UnloadRollback`. Public bridge-lifecycle live-host APIs still format typed errors at their diagnostic boundary, so native live-host bridge lifecycle keeps string diagnostics at public live-host boundaries. Bridge lifecycle report shape, provider transition behavior, and the NativeDynamic public namespace are unchanged; `review_f5_native_live_host_bridge_lifecycle_uses_typed_error` locks the boundary in a separate live-host child owner to keep review guard files below budget.

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
