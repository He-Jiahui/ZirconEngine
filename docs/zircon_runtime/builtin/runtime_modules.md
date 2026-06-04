---
related_code:
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/extensions.rs
  - zircon_runtime/src/builtin/runtime_modules/ids.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/support.rs
implementation_files:
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/extensions.rs
  - zircon_runtime/src/builtin/runtime_modules/ids.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/availability.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration.rs
  - rustfmt --edition 2021 zircon_runtime/src/builtin/runtime_modules.rs zircon_runtime/src/builtin/runtime_modules/*.rs zircon_runtime/src/builtin/runtime_modules/tests/*.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-modules-split-0604 --message-format short --color never
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
doc_type: module-detail
---

# Runtime Module Assembly

## Purpose

`zircon_runtime::builtin::runtime_modules` owns runtime module assembly for target modes and runtime profiles. It converts a target/profile plus project plugin manifest and registration reports into a `RuntimeModuleLoadReport` containing built-in engine modules, warnings, fatal diagnostics, and structured runtime plugin availability.

This boundary is runtime-owned. `zircon_app` may call the public assembly functions, but optional plugin implementation fan-out now sits behind the plugin-workspace `zircon_first_party_runtime_catalog` rather than process entry code.

## Module Layout

- `runtime_modules.rs` is the facade. It declares child modules and re-exports the stable public API.
- `ids.rs` owns `RuntimeTargetMode` and `RuntimePluginId`, including key/label/parse behavior.
- `load_report.rs` owns `RuntimeModuleLoadReport` and `RuntimeRequiredPluginMissing`.
- `core_modules.rs` owns built-in core module vector construction for target modes and minimal profiles.
- `manifest.rs` owns default target manifests, profile manifests, and manifest baseline overlay behavior.
- `availability.rs` owns structured runtime plugin availability reports for profiles, targets, manifests, and registration reports.
- `extensions.rs` owns aggregation of plugin extension registries into runtime-owned asset importer registries.
- `plugin_modules.rs` owns the current built-in versus externalized plugin-domain mapping.
- `assembly.rs` owns the orchestration functions that combine profile, manifest, registration reports, feature registration reports, extension registries, and module construction.
- `tests/` mirrors the behavior split: manifest baseline behavior, availability reporting, registration/bootstrap behavior, and shared fixtures.

## Architecture Notes

The split follows the M2 runtime module assembly decision in the runtime architecture review plan. It keeps Bevy-style profile/plugin composition in one runtime-owned facade, follows Fyrox-style Rust subsystem modules for runtime implementation details, and preserves Unreal-style separation between runtime assembly, plugin implementation domains, and editor/process hosts.

The current slice is intentionally a structural cutover rather than a behavior rewrite. It preserves the existing public function names and report types while removing the previous monolithic file shape.

The follow-up M2 provider slice moved linked first-party registration into `zircon_first_party_runtime_catalog`. The runtime assembly facade still consumes registration reports and stays independent of concrete plugin crates; the app wrapper only projects config and render-profile selections before delegating provider lookup to the catalog.

## Invariants

- Root `runtime_modules.rs` must stay structural: child module declarations, curated re-exports, and test module wiring only.
- Assembly code may orchestrate target/profile/plugin registration flow, but plugin identity parsing belongs in `ids.rs`, availability reports belong in `availability.rs`, manifest defaults belong in `manifest.rs`, and concrete built-in module vector construction belongs in `core_modules.rs`.
- The only built-in plugin module loaded from this boundary remains the optional UI module behind `plugin-ui`; other runtime plugin implementations remain externalized to `zircon_plugins/*`.
- Generated export code must consume this facade or runtime/plugin catalog APIs; it must not duplicate profile assembly, required-missing diagnostics, plugin-domain mapping, or linked-provider crate fan-out.

## Validation

The current implementation slice ran focused `zircon_runtime` checking after formatting. Workspace-wide validation remains a milestone testing-stage task because other active sessions are running concurrent Cargo lanes.
