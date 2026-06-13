---
related_code:
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
implementation_files:
  - docs/engine-architecture/native-plugin-boundary.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with native plugin loader isolated
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - native_plugin_public_surface M4 gate status, explicit count fields, symbol decision group, migration debt, and unclassified symbol checks
  - plugin_surface_lifecycle_boundary Runtime 06 mirror, app NativePlugin call-site count, V1/V2 fixture scope, export_build_plan V1/V2 zero-usage, pending validation anchors
doc_type: module-detail
---

# Native Plugin Boundary

## Purpose

The main plugin path is VM/plugin lifecycle, stable host handles, slot hot reload, and neutral package/capability contracts. Native dynamic loading is not the runtime public main path. It should be isolated behind tooling, export, tests, or a narrow handwritten facade instead of being broadly re-exported from `zircon_runtime::plugin`.

## Current Audit

The structural audit now includes `native_plugin_public_surface`. Its implementation is folder-backed in `runtime_structure_audits/native_plugin_public_surface.py`, so M4 native loader isolation evidence has a dedicated owner while the main audit script remains an orchestration boundary.

Current evidence:

- `zircon_runtime/src/plugin/mod.rs` publicly re-exports native loader and ABI symbols;
- `root_reexport_count = 68`;
- `public_reexport_location_count = 1`;
- exported names include native ABI versions, descriptor symbols, status constants, live-host types, runtime behavior calls, state snapshots, load reports, `NativePluginLoader`, and native bridge-method binding/report symbols;
- export source templates still call native loading through generated behavior.

This is M4 migration debt. It is also coupled to the generated-code boundary because generated source templates should not carry loader behavior.

## M4 Gate Output

The structural audit now reports `native_plugin_public_surface.m4_gate_status`. Current status is:

`migration-debt-present`

Current symbol classification:

- `native-abi-contract-public-debt = 31`
- `native-loader-discovery-public-debt = 6`
- `native-live-host-runtime-public-debt = 14`
- `native-behavior-report-public-debt = 3`
- `native-bridge-method-public-debt = 14`

Current gate evidence:

- `root_reexport_count = 68`
- `symbol_decision_count = 68`
- `symbol_decision_group_count = 5`
- `native_plugin_public_surface_migration_debt_count = 5`
- `unclassified_root_reexport_symbol_count = 0`
- `unclassified_root_reexport_symbols = []`
- `public_reexport_location_count = 1`

The classification means every current root re-export symbol has an explicit migration target. It does not mean the boundary is converged.

The Runtime 06 plan-status guard `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation` keeps this M4 debt visible until the `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane passes. The guard intentionally binds the current `m4_gate_status`, `migration-debt-present`, and `root_reexport_count = 68` evidence to Runtime 06, the runtime index, Runtime 05 closeout, and the M0 review so native plugin public-surface debt cannot be closed by documentation drift alone.

`plugin_surface_lifecycle_boundary` now mirrors the wider Runtime 06 state through the Python structural audit. Current evidence: Runtime 06 source 9/9, mirror docs 5/5, frontmatter `in_progress`, `last_refined = 2026-06-14`, native root re-export 68/68, M4 gate `migration-debt-present`, debt groups 5/5, unclassified native symbols 0/0, public native re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 6/6, `zircon_plugins` V1/V2 usage files 1/1, and export_build_plan V1/V2 usage 0/0. This confirms the boundary remains known migration debt; it does not promote Runtime 06 or the native loader to completed.

## M4 Decision Rules

`native-abi-contract-public-debt` covers ABI structs, version constants, descriptor symbols, status constants, callback status values, byte slices, owned buffers, host-function tables, and schema-version records. These may remain available only through an explicit native ABI contract namespace used by build/tooling paths, not by flattening them from `zircon_runtime::plugin`.

`native-loader-discovery-public-debt` covers `NativePluginLoader`, loaded-plugin records, manifest rows, candidates, and load reports. These belong behind the native loader/discovery owner or a narrow export/tooling facade.

`native-live-host-runtime-public-debt` covers live-host commands, runtime behavior descriptors, runtime command reports, play-mode reports, runtime plugin state, and runtime state snapshots. These belong behind an isolated native live-host bridge, not the main runtime plugin namespace.

`native-behavior-report-public-debt` covers behavior call and validation reports. These may be reachable through an explicit native diagnostics owner, but they should not be broad root plugin API.

`native-bridge-method-public-debt` covers native bridge-method descriptors, bindings, call table entries, registration scopes, and live-host bridge reports. These belong behind a native bridge-method owner used by native plugin bridge lifecycle tests or tooling, not the main runtime plugin namespace.

Any future `unclassified-native-plugin-symbol` entry is a review blocker. Classify it with a target owner or remove it from the public root re-export before accepting the boundary.

## Target Shape

The target shape is:

- `zircon_runtime::plugin` remains the stable plugin contract surface for manifests, runtime plugin descriptors, feature registration, extension registry, runtime profiles, and scene hooks.
- VM/plugin lifecycle remains the primary runtime plugin path.
- Native loader internals move behind an isolated namespace or tool/export-only facade.
- Public ABI declarations are exported only where needed by native plugin build/tooling contracts, not as root-level runtime plugin API.
- Generated export hosts call a stable handwritten export/native facade, if one is still needed, instead of directly loading native manifests.

## Hard-Cutover Rule

Do not preserve a compatibility re-export from `zircon_runtime::plugin` after the native loader moves. Call sites must update to the isolated owner path or to the new facade.

## Verification

Use the structural audit before calling this boundary converged:

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

The `native_plugin_public_surface.root_reexport_count` should move toward zero as M4 isolates the native loader. If a deliberately narrow native export facade remains public, the audit should allow only that facade and continue rejecting broad ABI/loader re-exports.

Before editing `zircon_runtime/src/plugin/mod.rs` or `native_plugin_loader`, inspect:

- `native_plugin_public_surface.root_reexport_count`
- `native_plugin_public_surface.symbol_decision_count`
- `native_plugin_public_surface.symbol_decision_group_count`
- `native_plugin_public_surface.symbol_decision_groups`
- `native_plugin_public_surface.native_plugin_public_surface_migration_debt_count`
- `native_plugin_public_surface.unclassified_root_reexport_symbols`
- `native_plugin_public_surface.unclassified_root_reexport_symbol_count`
- `native_plugin_public_surface.public_reexport_location_count`
- `native_plugin_public_surface.m4_gate_status`

The gate is not clear while `m4_gate_status` is `migration-debt-present`.
