---
related_code:
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/generated_files.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/generated.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/native.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - docs/zircon_app/export-bootstrap.md
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
implementation_files:
  - docs/engine-architecture/generated-code-boundary.md
  - docs/zircon_app/export-bootstrap.md
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with generated-code boundaries
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs zircon_runtime/src/tests/runtime_absorption/mod.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - cargo check -p zircon_app --lib --no-default-features --features core-min --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-app-core-min --message-format short --color never
  - cargo test -p zircon_app --lib export_bootstrap --no-default-features --features core-min --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-app-core-min --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib export_build_plan --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-runtime --message-format short --color never -- --nocapture --test-threads=1
  - generated_code_boundary M1 gate status, explicit count fields, behavior decision group, migration debt, and unclassified behavior checks
doc_type: module-detail
---

# Generated Code Boundary

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](../plans/zircon_runtime/frameworks/development-conventions.md)；本文保留生成代码边界主题的细节论证与执行上下文，不再作为并列规则源。

## Purpose

Generated code is allowed only as a leaf artifact. It may carry data, tables, manifests, schema-shaped DTOs, and thin adapters to an existing owner. It must not own business rules, lifecycle order, runtime state mutation, plugin resolution, module registration, scheduling, or public architecture decisions.

This rule is part of the M1 public-surface and generated-boundary milestone. It exists because generated source templates can hide old behavior after the handwritten runtime has moved to a cleaner owner.

## Allowed Output

Generated files may contain:

- manifest data;
- package metadata;
- static lookup tables;
- schema DTOs;
- platform FFI adapters that immediately call a handwritten runtime/export owner;
- copied project data without behavioral interpretation.

The generated file should be replaceable without changing runtime semantics.

## Forbidden Output

Generated files must not contain:

- `EntryRunner` bootstrap logic;
- plugin registration or feature registration decisions;
- native plugin loading;
- module descriptor construction;
- service or manager resolution;
- ECS mutation, schedule construction, or frame-loop behavior;
- editor hierarchy, inspector, viewport, overlay, or gizmo logic;
- compatibility aliases for removed paths.

When generated output needs one of these behaviors, the behavior belongs in a handwritten runtime owner. The generated artifact should pass data into that owner.

## Current Audit

The structural audit now includes `generated_code_boundary`. Its implementation is folder-backed in `runtime_structure_audits/generated_code_boundary.py`, keeping the main audit script focused on orchestration while this owner module tracks export-template behavior drift.

Current evidence:

- `template_file_count = 10` under `zircon_runtime/src/plugin/export_build_plan`, including the `SourceTemplate` build-validation plan owner;
- `behavior_location_count = 6` architecture-sensitive generated behavior locations after the M4 export-bootstrap and provider-table migration slices;
- current flagged categories include thin export-bootstrap facade calls, generated `main`, and generated runtime selection functions. All 6 current locations are allowed generated adapters. Direct generated native loader use and direct generated `plugin_registration()` calls are no longer present. Provider-table rows are accepted as generated table adapters and are guarded separately against immediate execution.
- current broad `generated` term scan under `zircon_runtime/src/**/*.rs` hits 42 files. These are classified as domain wording, tests, export-build-plan source/template owners, or the runtime absorption guard itself; they are not all generated artifacts.
- current file-header generated marker scan has 0 production generated source files. The old "1 explicit marker" count was too broad because it can match test constants rather than a real generated file header.

## M1 Gate Output

The structural audit now reports `generated_code_boundary.m1_gate_status`. Current status is:

`classified-and-clear`

Current behavior classification:

- `handwritten-owner-required`: `entry-bootstrap`
- `entry-glue-review`: `generated-main`
- `data-adapter-review`: `runtime-selection-function`

Current gate evidence:

- `template_file_count = 10`
- `behavior_location_count = 6`
- `allowed_adapter_location_count = 6`
- `migration_debt_location_count = 0`
- `behavior_decision_count = 3`
- `generated_boundary_migration_debt_count = 0`
- `unclassified_behavior_label_count = 0`
- `unclassified_behavior_labels = []`

The classification means every current generated behavior label has an explicit owner decision and every current location is an allowed adapter shape. It does not mean generated files should grow new behavior; the guard still treats direct registration, native-loader calls, and direct bootstrap sequencing as regressions.

## M1 Decision Rules

`entry-bootstrap` must move to handwritten owners. The M4 export-bootstrap slice moved direct `EntryRunner` and native loader assembly into `zircon_app::entry::export_bootstrap`; generated entry files may call `zircon_app::bootstrap_export_runtime*` only as thin glue. The provider-table slice changed generated linked plugin output from direct `plugin_registration()` calls to `ExportRuntimePluginRegistrationProvider` / `ExportRuntimePluginFeatureRegistrationProvider` handoff rows. Generated output may carry provider IDs, manifest rows, feature rows, or provider-table rows, but it must not directly execute plugin registration from `main.rs`, platform `lib.rs`, or template-owned bootstrap sequencing.

`plugin-registration` is now a regression label, not current migration debt. It is reserved for direct generated `plugin_registration()` / `plugin_feature_registration()` calls or old `runtime_plugin_registrations()` report-builder entry points. Provider-table rows are treated as allowed generated data adapters because the app owner executes them.

`native-loader` must move behind the M4 native loader boundary. Generated output may reference native manifest data or call an isolated loader facade, but it must not instantiate or drive `NativePluginLoader` directly.

`generated-main` is allowed only as entry glue. A generated `main` can call one stable handwritten export entry; it must not assemble runtime config, plugin registrations, feature registrations, or native loader state itself.

`runtime-selection-function` is allowed only as pure data adapter output. Functions that build `ExportProfile` or `ProjectPluginManifest` can remain generated if they are side-effect free and do not mix in registration calls, lifecycle decisions, scheduling, or loader behavior.

Any new generated behavior label that appears in `unclassified_behavior_labels` is a review blocker. Classify it with a target owner or remove it from generated output.

These findings do not mean export is broken. The current export source-template model now stays within the allowed adapter shapes for this boundary; the remaining risk is regression if templates reintroduce direct runtime, loader, registration, or scheduling behavior.

## M4 Runtime Guard

`runtime_absorption::generated_code_guard` is now the in-crate structure guard for this boundary. It keeps six checks close to the runtime absorption tests:

- `generated_marker_format_is_uniform_when_source_files_are_marked` accepts only first-line source markers shaped as `// @generated <generator> - do not edit by hand`;
- `marked_generated_source_files_stay_leaf_data_only` rejects behavior tokens inside files that carry the marker;
- `export_template_generated_behavior_stays_classified_by_owner` requires every architecture-sensitive export-template behavior label to map to an explicit owner decision;
- `export_template_scan_scope_stays_folder_backed` keeps the export-template scan constrained to `src/plugin/export_build_plan/**`.
- `export_entry_templates_delegate_to_app_export_bootstrap_facade` rejects direct `EntryRunner`, `EntryConfig::new`, `NativePluginLoader`, `load_runtime_from_load_manifest`, and direct registration calls in generated entry templates while requiring the `zircon_app::bootstrap_export_runtime*` facade path.
- `export_plugin_selection_template_delegates_registration_execution_to_app_providers` rejects direct registration call forms in `plugin_selection_template.rs` and requires provider-table handoff.

The guard now distinguishes allowed adapters from migration debt. It prevents new unclassified generated behavior from entering and fails if direct bootstrap sequencing, native-loader calls, or direct plugin registration execution reappear in export templates.

## M4 Export Bootstrap Slice

The first M4 behavior migration slice moved generated startup sequencing into the `zircon_app` owner:

- `zircon_app/src/entry/export_bootstrap.rs` owns `ExportRuntimeBootstrapConfig`, `bootstrap_export_runtime`, `bootstrap_export_runtime_with_native_plugins_from_export_root`, and `discover_export_root`.
- `EntryRunner` now has a lower-level merge path that preserves linked registration reports and adds native dynamic reports loaded through `NativePluginLiveHost`.
- `main_template.rs` and platform host `src/lib.rs` templates call only `zircon_app::bootstrap_export_runtime*`.
- `plugin_selection_template.rs` now generates `export_runtime_bootstrap_config()` plus provider-table functions consumed by the app facade. It no longer generates immediate `plugin_registration()` or `plugin_feature_registration()` calls.

The source scan confirms generated entry templates no longer contain `EntryRunner::`, `EntryConfig::new`, `NativePluginLoader`, `load_runtime_from_load_manifest`, or direct registration function calls. The provider-table guard confirms `plugin_selection_template.rs` no longer contains `plugin_registration()` or `plugin_feature_registration()` call forms. The structural audit moved from 13 behavior locations / 5 behavior labels / 5 migration-debt buckets to 6 behavior locations / 3 behavior labels / 0 migration-debt buckets after accepting facade calls and provider rows as generated adapters.

## Target Shape

The target export shape is:

- `zircon_runtime::plugin` owns export profile interpretation, plugin selection, linked registration, feature reports, and native loader isolation.
- Generated plugin-selection files stay as side-effect-free data tables or serialized manifests.
- Generated platform host files become thin ABI adapters.
- Generated `main` scaffolds call one stable handwritten export entry, not direct runtime/plugin assembly internals.
- Native plugin loading is isolated behind the app export bootstrap facade and does not appear as generated application behavior.
- Export build-plan fatal diagnostics block mutating generated-file materialization before generated files or NativeDynamic packages are written; preview may still list planned generated paths because it is a no-write validation surface.

## Verification

Use the structural audit before calling this boundary converged:

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

The `generated_code_boundary.migration_debt_location_count` and `generated_boundary_migration_debt_count` must stay at zero. `behavior_location_count` may stay nonzero when the remaining locations are explicitly classified allowed adapters such as a public export facade or side-effect-free selection table.

Before editing export templates, inspect:

- `generated_code_boundary.template_file_count`
- `generated_code_boundary.behavior_location_count`
- `generated_code_boundary.allowed_adapter_location_count`
- `generated_code_boundary.migration_debt_location_count`
- `generated_code_boundary.behavior_decision_count`
- `generated_code_boundary.behavior_decision_groups`
- `generated_code_boundary.generated_boundary_migration_debt_count`
- `generated_code_boundary.unclassified_behavior_labels`
- `generated_code_boundary.unclassified_behavior_label_count`
- `generated_code_boundary.m1_gate_status`

The gate is clear only while `m1_gate_status` is `classified-and-clear` and `migration_debt_location_count = 0`.

Runtime 02 still keeps a separate plan-status validation gate, `runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation`. That guard records that the generated boundary is structurally clear but still needs the generated/export_build_plan/app validation lane to rerun cleanly alongside the broader core/root checks before Runtime 02 can close.

Runtime 02 also mirrors this generated boundary through `core_spine_root_generated_boundary` and `runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts`: core root entries 6/6, core public modules 5/5, retired core root entries 0, runtime root public modules 21/21, public `pub use` sites 2/2, crate-visible graphics alias debt 0/0, root-surface M1 gate `classified-and-clear`, generated export templates 10/10, generated behavior 6/6, generated allowed adapters 6/6, generated migration debt 0/0, generated-code M1 gate `classified-and-clear`, root_entries guard tests 13, root_surface guard tests 6/6, generated-code guard tests 7/7, `guard_test_anchor_count = 26`, `missing_guard_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`.
