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
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
implementation_files:
  - docs/engine-architecture/generated-code-boundary.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with generated-code boundaries
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - generated_code_boundary M1 gate status, explicit count fields, behavior decision group, migration debt, and unclassified behavior checks
doc_type: module-detail
---

# Generated Code Boundary

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

- `template_file_count = 9` under `zircon_runtime/src/plugin/export_build_plan`;
- `behavior_location_count = 13` architecture-sensitive generated behavior locations;
- current flagged categories include entry bootstrap, native loader use, runtime/plugin registration calls, generated `main`, and generated runtime selection functions.

## M1 Gate Output

The structural audit now reports `generated_code_boundary.m1_gate_status`. Current status is:

`migration-debt-present`

Current behavior classification:

- `handwritten-owner-required`: `entry-bootstrap`, `plugin-registration`
- `native-loader-isolation`: `native-loader`
- `entry-glue-review`: `generated-main`
- `data-adapter-review`: `runtime-selection-function`

Current gate evidence:

- `template_file_count = 9`
- `behavior_location_count = 13`
- `behavior_decision_count = 5`
- `generated_boundary_migration_debt_count = 5`
- `unclassified_behavior_label_count = 0`
- `unclassified_behavior_labels = []`

The classification means every current generated behavior label has an explicit migration target. It does not mean the boundary is converged.

## M1 Decision Rules

`entry-bootstrap` and `plugin-registration` must move to handwritten owners. Generated output may carry provider IDs, manifest rows, feature rows, or table data, but it must not directly decide bootstrap order or call plugin registration functions.

`native-loader` must move behind the M4 native loader boundary. Generated output may reference native manifest data or call an isolated loader facade, but it must not instantiate or drive `NativePluginLoader` directly.

`generated-main` is allowed only as entry glue. A generated `main` can call one stable handwritten export entry; it must not assemble runtime config, plugin registrations, feature registrations, or native loader state itself.

`runtime-selection-function` is allowed only as pure data adapter output. Functions that build `ExportProfile` or `ProjectPluginManifest` can remain generated if they are side-effect free and do not mix in registration calls, lifecycle decisions, scheduling, or loader behavior.

Any new generated behavior label that appears in `unclassified_behavior_labels` is a review blocker. Classify it with a target owner or remove it from generated output.

These findings do not mean export is broken. They mean the current export source-template model still carries too much handwritten runtime behavior inside generated output. The M2/M4 target is to move that behavior into runtime-owned export/plugin owners and leave generated files as data or adapters.

## Target Shape

The target export shape is:

- `zircon_runtime::plugin` owns export profile interpretation, plugin selection, linked registration, feature reports, and native loader isolation.
- Generated plugin-selection files become data tables or serialized manifests.
- Generated platform host files become thin ABI adapters.
- Generated `main` scaffolds call one stable handwritten export entry, not direct runtime/plugin assembly internals.
- Native plugin loading is isolated from the runtime public main path and does not appear as generated application behavior.

## Verification

Use the structural audit before calling this boundary converged:

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

The `generated_code_boundary.behavior_location_count` should move toward zero as export behavior moves behind handwritten owners. If generated adapters still need to call a public export facade, the audit should be updated to allow only that facade and continue rejecting direct bootstrap, loader, registration, and scheduling logic.

Before editing export templates, inspect:

- `generated_code_boundary.template_file_count`
- `generated_code_boundary.behavior_location_count`
- `generated_code_boundary.behavior_decision_count`
- `generated_code_boundary.behavior_decision_groups`
- `generated_code_boundary.generated_boundary_migration_debt_count`
- `generated_code_boundary.unclassified_behavior_labels`
- `generated_code_boundary.unclassified_behavior_label_count`
- `generated_code_boundary.m1_gate_status`

The gate is not clear while `m1_gate_status` is `migration-debt-present`.
