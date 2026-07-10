---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/tests/text_shaper.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
  - tools/tests/test_runtime_tech_stack_boundary.py
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime_interface/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/archive.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/copy.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize/paths.rs
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
plan_sources:
  - user: 2026-06-12 implement runtime architecture from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/Fyrox/fyrox-impl/Cargo.toml
tests:
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs::runtime_01_tech_stack_mirror_docs_match_structure_audit_counts
  - tools/tests/test_runtime_tech_stack_boundary.py
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs::native_dynamic_zip_archive_materialization_writes_generated_files_and_runtime_payloads
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs::native_dynamic_zip_archive_preview_reports_archive_without_writes
  - cargo test -p zircon_runtime --lib tech_stack --locked -- --nocapture
doc_type: module-detail
---

# Runtime Tech Stack

## Purpose

This document is the runtime-side dependency authority for `zirconEngine`. It separates dependencies that are part of the runtime product surface from editor-only candidates, plugin-owned stacks, and future backlog decisions. Manifest changes that alter these decisions must update this document and the matching source guard in `zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`.

## Executable Guard Anchors

Runtime 01 is code/static complete but remains Cargo-pending until `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation` can see real validation for the `tech_stack`, `extensions`, `text_shaper`, and plugin physics gates. The dependency authority is currently protected by `runtime_tech_stack_doc_exists_and_is_linked_from_architecture_index`, `runtime_manifest_keeps_pinned_prerelease_versions_until_upgrade_gate`, `zr_vm_path_dependency_gate_is_documented_with_version_pairing`, `interface_and_editor_dependency_boundaries_stay_documented_and_guarded`, `removed_or_editor_only_dependencies_do_not_silently_enter_runtime_stack`, `runtime_text_doc_records_three_layer_stack_and_cross_reference`, `complex_text_backends_can_only_enter_through_ui_text_shaper`, `fontdue_editor_retained_host_dependency_has_migration_owner`, `physics_backend_option_decision_keeps_jolt_feature_gated_and_plugin_owned`, `export_archive_policy_allows_zip_only_for_archive_materializer`, and `editor_only_dependency_candidates_have_editor_backlog_owner`.

Current validation refinement (2026-07-10): the exact locked `tech_stack` filter now passes 14/14 on the current source tree. This closes that focused dependency-policy gate only; the aggregate Runtime 01 status remains `in_progress` while the separate `text_shaper`, plugin-physics, and `export_build_plan` package gates are still pending. Concrete command output is owned by the numbered Runtime 01 output archive rather than this authority document.

The same-day manifest-audit refresh treats `glyphon` and `fontsdf` as optional graphics/text dependencies and recognizes only the hard-cut backend feature names `backend-zr-vm` and `backend-jolt`. The retired feature spellings have no compatibility aliases. The direct Runtime 01 audit reports no missing version anchors, no dependency-boundary violations, two visible Jolt slots, and `risks = []`.

`tech_stack_source_inventory.py` mirrors the Runtime 01 manifest/dependency/version/count inventory, `tech_stack_anchor_inventory.py` mirrors the doc, guard, behavior, decision, and pending Cargo-gate anchors, `tech_stack_boundary.py` owns audit reads/dependency scanning/risk aggregation, and `tech_stack_markdown.py` owns the Markdown renderer. The mirror guard now resolves its real folder-backed owner at `runtime_absorption/tech_stack/mirror_docs.rs`, rather than the route-only `tech_stack.rs` parent. Current evidence reports `expected_manifest_count = 5`, `expected_non_dependency_count = 5`, `zip_dependency_count = 1`, `expected_zip_dependency_count = 1`, `zip_dependency_violations = []`, `tech_stack_guard_count = 12`, `behavior_test_anchor_count = 6`, `missing_behavior_test_anchors = []`, `editor_only_candidate_count = 3`, `jolt_feature_slot_count = 2`, `declared_removed_dependencies = []`, `rapier_or_avian_dependencies = []`, `mirror_docs_guard_present = true`, and `risks = []`. The six behavior anchors cover two text-stack checks plus Jolt feature-off unavailability/no-fallback and feature-on ready/native-step behavior. `runtime_01_tech_stack_mirror_docs_match_structure_audit_counts` keeps this tech-stack authority doc, Runtime 01, the runtime index, the M0 review, and runtime-interface convergence aligned with those structure-audit counts. Direct `tech_stack_boundary_audit` verifies one runtime profile hook, one dependency-backed Physics feature, the optional plugin-owned `joltc-sys`, concrete Jolt backend owners, and no runtime solver dependency. This is structure evidence only; aggregate `extensions` and remaining product gates stay pending.

## Dependency Matrix

| Dependency / area | Current version or state | Owner crate | Feature gate | Upgrade or replacement gate |
|---|---:|---|---|---|
| `winit` | `0.31.0-beta.2` | `zircon_runtime` optional platform window path; `zircon_editor` retained host path | runtime: `platform-winit`; editor: direct dependency | Upgrade only in a dedicated milestone after `0.31` final is available and `ApplicationHandler` API impact is reviewed. |
| `wgpu` / `naga` | `29.0.1` / `29.0.1` | `zircon_runtime::graphics` | default runtime client/editor-host profile through render features | Renderer plan owns upgrades; `zircon_runtime_interface` must stay free of both dependencies. |
| `taffy` | `0.10` | `zircon_runtime::ui::layout` | runtime UI | Replace only behind the runtime layout bridge after editor UI plan sign-off. |
| `glam` | `0.32.1` with `serde` | workspace + interface/runtime/editor consumers | none | Precision and ABI seam decisions stay under runtime foundation docs. |
| `glyphon` | `0.11.0` | runtime render/text submission | runtime UI/render | Current layout metrics are served by `UiSharedTextShaper` through the `SharedTextService` active backend; glyphon is the native render/backend intent and font submission path. |
| `fontsdf` | `0.5.3` | runtime text/raster policy | runtime UI/render | Stays local to runtime text/raster policy; SDF render mode now shares layout metrics through `SharedTextService` while later atlas/raster milestones decide final draw/cache policy. |
| `image` | `0.25.10` | asset import and texture/image processing | none | Shared importer policy owns format expansion. |
| `gltf` / `tobj` | `1.4.1` / `4.0.3` | runtime asset import and mesh ingest | none | Model-importer plugin work may move behavior outward, but runtime still owns current built-in importer paths. |
| `notify` | `9.0.0-rc.3` | runtime/editor asset watch paths | none | Upgrade only in a dedicated milestone after `9.0` final is available and watcher event compatibility is checked. |
| `rayon` | `1.11.0` | runtime scheduling/asset parallelism | none | Replace only with an execution-policy milestone that covers ECS scheduling and asset worker behavior together. |
| `crossbeam-channel` / `crossbeam-utils` | `0.5.15` / `0.8.21` | runtime channels and worker support | none | Any replacement must preserve current runtime channel facade semantics. |
| `serde`, `serde_json`, `toml`, `ron`, `bincode` | workspace or crate-local pinned versions | manifests, project data, artifact cache, debug/config IO | none | Serialization format changes need explicit migration plans. |
| `libloading` | `0.9.0` | runtime cdylib loading and native dynamic plugin support | none | Dynamic ABI changes are governed by runtime interface convergence and plugin ABI plans. |
| `zstd` | `0.13.3` | runtime/export compression support | none | Remains available for compression support, but it is not the current archive container. |
| `zip` | `9.0.0-pre2` with `default-features = false`, `deflate-flate2` only | `zircon_runtime::plugin::export_build_plan::materialize::archive` export archive materializer | none | Only the runtime ZIP archive materializer may declare this dependency. Any feature expansion, additional archive format, or non-runtime owner must update this document and `tech_stack_boundary`. |
| `accesskit` | `0.22.0` optional | runtime accessibility | `accessibility-accesskit` | Upgrade with accessibility DTO compatibility checks. |
| gamepad input | app/runtime input stack | app/runtime input | `input-gamepad`, `gamepad-gilrs` | Browser gamepad remains a separate target path. |
| `zr_vm_rust_binding` / `zr_vm_rust_binding_sys` | external path dependency at `../../zr_vm/...` | runtime script backend | `backend-zr-vm` | Current decision is to keep the external checkout. Any move to submodule/vendor/published crate must pair with the empty-argument marshalling fix in the binding version. |

## Corrected Non-Dependencies

The runtime plan previously mentioned several libraries that are not present in the current workspace manifests. These are not runtime dependencies:

| Name | Current decision | Owner or follow-up |
|---|---|---|
| `cosmic-text` | Not introduced. The current text layout backend is `UiSharedTextShaper`; glyphon is the native render/backend intent and uses the shared text service for layout metrics. | Future complex text demand may introduce cosmic-text through `UiTextShaper`, not by bypassing that trait. |
| `kira` | Not introduced. Sound runtime uses the existing plugin-owned stack, currently based on `cpal` and custom mixer/DSP/HRTF/occlusion paths. | Sound plugin plan owns audio backend decisions. |
| `tar` | Not introduced. ZIP is the current desktop/editor archive container; `tar` remains a possible CI/server artifact format only after a separate artifact policy lands. | Do not add to manifests without a server/CI artifact policy and guard update. |
| `fontdue` | Not introduced in runtime. It remains a temporary `zircon_editor` retained-host text fallback. | Tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); remove or replace under the editor UI text plan once retained-host text rendering consumes runtime UI text/glyphon/SDF. |
| `rfd` | Not introduced in runtime. | Editor-only file-dialog candidate tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); do not add to runtime. |
| `arboard` | Not introduced in runtime. | Editor-only clipboard candidate tracked in [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); do not add to runtime. |

## Prerelease Version Governance

`winit 0.31.0-beta.2` and `notify 9.0.0-rc.3` remain intentionally pinned. They are allowed because they are already integrated and because replacing them without a targeted migration would touch platform/application lifecycle code and watcher behavior across runtime, app, and editor.

Upgrade gates:

1. `winit`: wait for `0.31` final, then verify `ApplicationHandler` and platform feature behavior in a dedicated milestone before changing the workspace dependency.
2. `notify`: wait for `9.0` final, then rerun asset watcher and UI hot-reload watch invalidation coverage before changing the workspace dependency.
3. Any silent manifest bump without this document and `tech_stack_dependency_guard.rs` changing together is invalid.

## External ZrVM Path Dependency

The current decision is option A from the runtime 01 plan: keep `../../zr_vm` as an external checkout and gate it behind `backend-zr-vm`. This keeps the default runtime build independent from a local ZrVM checkout while preserving the real backend for explicit validation.

The path dependency is not only a clone-layout issue. The runtime real-backend contract depends on a paired binding version that represents empty export argument lists as a valid non-null pointer with length `0`. Moving the dependency to a submodule, vendored crate, or published crate must include that binding fix as a version gate.

Required local layout for the real backend:

```text
E:/Git/ZirconEngine
E:/Git/zr_vm
```

## Export Archive Decision

The current `ExportPackagingStrategy` enum is not an archive-container enum. It describes how project/plugin code is materialized: `SourceTemplate`, `LibraryEmbed`, and `NativeDynamic`. The directory-first materialization API remains available for staged export directories, and the archive API is an explicit export-build-plan materialization step rather than a fourth packaging strategy.

Runtime 01 M3.2 selects ZIP as the future desktop/editor archive container. The reasons are cross-platform user tooling, Windows Explorer/macOS Finder/Linux desktop compatibility, existing editor-export expectations around a single distributable file, and a lower support burden than a custom container. `tar + zstd` remains a possible CI/server artifact format later, but it is not the primary runtime export container. A custom container is rejected for V1 because it would require custom inspection, extraction, and failure-recovery tooling before the runtime package format itself is stable.

ZIP archive materialization is implemented by `ExportBuildPlan::materialize_zip_archive(plugin_root, archive_path)` and `ExportBuildPlan::preview_zip_archive(plugin_root, archive_path)`. The archive writer sorts generated files by path, reuses `validated_materialized_relative_path(...)` to reject traversal and non-portable generated paths, writes native dynamic package payloads through the same copy eligibility rules as directory materialization, and records the produced archive through `ExportMaterializeReport.archive_file`. ZIP entries use a stable default timestamp and `0o644` permissions; NativeDynamic package source crates stay excluded from the archive exactly as they are excluded from directory materialization.

The admitted dependency is `zip 9.0.0-pre2` in `zircon_runtime/Cargo.toml`, pinned as `zip = { version = "9.0.0-pre2", default-features = false, features = ["deflate-flate2"] }` so the export archive materializer does not silently pull in optional crypto or alternate compression stacks. `tar` remains absent from workspace manifests.

## Text Stack Boundary

Runtime text currently has three separate responsibilities:

| Layer | Current owner | Current state |
|---|---|---|
| Layout and measurement | `zircon_runtime::ui::text::UiTextShaper` | Active backend is `SharedTextService`; `shared_text_shaper_matches_public_layout_entrypoint` and `text_shaper_stack_uses_shared_text_service_for_font_backends` lock that behavior. |
| Font/raster policy | `zircon_runtime::ui::text` | Font registry and raster policy exist; SDF/native layout backends are not connected yet. |
| GPU/native submission | runtime graphics/UI render paths with `glyphon` | Render-side dependency exists; Native/SDF render modes currently consume shared layout metrics until a future text milestone swaps the `UiTextShaper` implementation. |

`cosmic-text`, Parley, Swash, or HarfBuzz may only enter through a replacement implementation of `UiTextShaper`. They must not duplicate public text layout entry points or bypass the existing `UiResolvedTextLayout` contract.

## Interface And Editor Dependency Boundary

`zircon_runtime_interface` is a DTO/ABI crate. Its manifest must remain free of `wgpu` and `winit`. `zircon_editor` is allowed to keep a direct `winit` dependency for the retained host and `softbuffer` self-drawn shell, but it must remain free of `wgpu` unless the editor UI plan explicitly changes renderer ownership.

The editor-only candidates `fontdue`, `winit`, `softbuffer`, future `rfd`, and future `arboard` are not runtime dependency claims. `fontdue`, `rfd`, and `arboard` are tracked in the [Runtime Editor-Only Dependency Backlog](../editor-and-tooling/runtime-editor-only-dependency-backlog.md); all of these are editor-host concerns and should be moved or removed only under the editor plan.
