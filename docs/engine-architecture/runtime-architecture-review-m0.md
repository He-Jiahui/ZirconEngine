---
related_code:
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
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
  - docs/zircon_runtime/builtin/runtime_modules.md
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/hierarchy.rs
  - zircon_runtime/src/scene/inspection/field.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - docs/zircon_runtime/dynamic_api/session.md
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/runtime_api.md
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/helpers.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - docs/zircon_runtime/scene/ecs/query_state.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-root-surface-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/__init__.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
implementation_files:
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-root-surface-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/__init__.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/tests/mod.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/profile_control.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/accessibility.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/requests.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/runtime_api.md
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/helpers.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/system_param.rs
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - docs/zircon_runtime/scene/ecs/query_state.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - root_surface_audit M1 gate status and module decision group checks
  - generated_code_boundary M1 gate status, explicit count fields, behavior decision group, migration debt, and unclassified behavior checks
  - native_plugin_public_surface M4 gate status, explicit count fields, symbol decision group, migration debt, and unclassified symbol checks
  - non_network_server_references M1 gate status, explicit count fields, classification count, migration debt, and unclassified reference checks
  - hard_cutover_migration_smells gate status, explicit count fields, classification count, migration debt, allowed bridge count, and unclassified reference checks
  - large_file_ownership_gate M1 gate status, explicit count fields, classification count, migration debt, and unclassified hotspot checks
  - Select-String reference declaration evidence over Bevy, Fyrox, and Unreal source files listed in docs/engine-architecture/runtime-reference-engine-evidence.md
  - git diff --check -- docs/engine-architecture/runtime-reference-engine-evidence.md docs/engine-architecture/runtime-architecture-review-m0.md docs/engine-architecture/runtime-interface-convergence.md .codex/sessions/20260604-1232-runtime-architecture-review.md
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/runtime_api.md
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - docs/zircon_runtime/scene/ecs/query_state.md
doc_type: milestone-detail
---

# Runtime Architecture Review M0 Baseline

## Scope

This is the M0 evidence and decision record for the runtime architecture review. It fixes the current review order before broad code movement starts. The optimization target is a runtime that is developer-friendly, compact at public boundaries, hard-cutover oriented, and performance-aware without preserving old compatibility behavior.

Reference-engine direction for this review:

- Unreal-style module/plugin ownership: integration units are declared modules and plugins, not scattered launch-time match arms.
- Bevy-style app composition: application entry should compose profile/plugin graphs and should not statically know every optional runtime plugin implementation.
- Fyrox-style editor/runtime split: editor views project runtime state through explicit DTOs; runtime scene and runtime module code should not expose editor authoring concepts as core owners.

The source-backed reference matrix is recorded in `docs/engine-architecture/runtime-reference-engine-evidence.md`. Use that matrix as the review gate before M1 root-surface cuts, M2 assembly changes, M3 scene/editor boundary work, M4 plugin lifecycle convergence, M5 performance work, and M6 graphics/RHI public-surface cleanup. The concrete M1 root-surface gate is recorded in `docs/engine-architecture/runtime-root-surface-m1.md`; the non-network `server` naming gate is recorded in `docs/engine-architecture/non-network-server-naming-m1.md`; the hard-cutover migration-smell gate is recorded in `docs/engine-architecture/hard-cutover-migration-smells-m1.md`.

## Current Evidence

Review timestamp: 2026-06-04 12:34 +08:00 on branch `main`.

The structural audit currently reports no `stub_module_descriptor_usage` and no `plugin_runtime_gaps`. The plugin gap check is now folder-backed in `runtime_structure_audits/plugin_runtime_gaps.py`, so the first review layer is not a missing-module problem; it is an ownership, duplication, public-surface, and large-file problem.

Current audit classification:

- `zircon_app`: structurally converged, but still has direct static dependency pressure from first-party runtime plugin crates.
- `zircon_runtime`: needs refactor because production files still combine registry, profile, diagnostics, and feature assembly responsibilities.
- `zircon_editor`: needs refactor because several retained-host files are large enough to hide duplicated behavior and slow future UI/runtime boundary work.

Measured hotspots from the M0 audit:

- `zircon_runtime/src/builtin/runtime_modules.rs`: 1500 lines; owns target modes, plugin ids, availability reports, profile manifests, feature reports, module construction, and diagnostics in one file.
- `zircon_app/Cargo.toml`: 22 `zircon_plugin_` references; this keeps the process entry crate aware of optional plugin implementations.
- `zircon_app/src/entry/first_party_runtime_plugins.rs`: direct match arms map runtime plugin IDs to concrete first-party plugin crates.
- `zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot.rs`: 1495 lines; should not keep expanding as a mixed debug, DTO, projection, and report sink.
- `zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs`: 2059 lines.
- `zircon_editor/src/ui/retained_host/app/host_lifecycle.rs`: 1934 lines.

M1 audit hardening now makes the following blockers visible in both JSON and Markdown audit output:

- Module descriptor distribution, stub descriptor usage, `EngineModule` owner coverage, module classification, support-crate listing, workspace `zircon_*` production-file inventory, and large-file hotspot source data are now owned by `runtime_structure_audits/module_inventory.py`. Current evidence covers 3 classified module crates, 0 stub descriptor crates, 3 support crates outside module classification, and 10 large-file hotspots feeding the large-file owner audit.
- `zircon_app/Cargo.toml` now has 0 optional runtime plugin path dependencies and 0 optional runtime plugin feature mentions after the catalog cutover. The app still has 4 path dependencies, and the `entry_static_dependencies` audit owner is folder-backed in `runtime_structure_audits/entry_static_dependencies.py`.
- `zircon_runtime/src/lib.rs` exposes 17 public modules, 3 public `pub use` locations, 75 crate-visible graphics re-export symbols, and direct `rhi_wgpu` backend exposure. The `runtime_root_surface` audit owner is now folder-backed in `runtime_structure_audits/runtime_root_surface.py` and reports an M1 gate status of `migration-debt-present`, with root modules classified as stable facades, namespace entries, runtime module entries, graphics/RHI deferred entries, or backend public debt.
- `zircon_runtime::scene` had 4 editor-named production paths and 9 public editor-named locations before the M3 cutover. The M3 inspection slice now replaces that production surface with neutral `zircon_runtime::scene::inspection`, and the folder-backed `runtime_scene_editor_surface` audit owner currently reports zero editor-named runtime scene locations.
- Runtime scene/project serialization now has a separate authoring-state boundary audit over 7 source files. The audit owner is folder-backed in `runtime_structure_audits/scene_project_serialization_boundary.py`. The current audit reports zero forbidden locations; camera render viewport rectangles are allowed runtime data, while selection, editor viewport tools, overlays, gizmos, and preview overrides are forbidden serialization state.
- `zircon_runtime::plugin::export_build_plan` still has 13 architecture-sensitive generated behavior locations across export source templates; this is the M1/M2 generated-code boundary target. The `generated_code_boundary` audit owner now reports M1 gate status `migration-debt-present`, classifies behavior into handwritten-owner-required, native-loader-isolation, entry-glue-review, and data-adapter-review groups, and reports zero unclassified generated behavior labels.
- `zircon_runtime::plugin` still publicly re-exports 54 native loader/ABI symbols from the crate plugin namespace; this is the M4 native plugin isolation target. The `native_plugin_public_surface` audit owner now reports M4 gate status `migration-debt-present`, classifies symbols into native ABI contract, loader/discovery, live-host runtime, and behavior-report debt groups, and reports zero unclassified root re-export symbols.
- Engine architecture docs now have a folder-backed stale standalone-crate reference audit in `runtime_structure_audits/legacy_standalone_references.py`; the first M1 documentation cleanup brought that audit count to zero and current evidence remains zero.
- Production Rust now has a folder-backed hard-cutover migration-smell audit in `runtime_structure_audits/hard_cutover_migration_smells.py`. Current evidence scans 5414 production Rust files, reports 142 `legacy` references, 0 `compat` references, 0 `shim` references, 231 allowed business `bridge` references, 0 migration-context bridge references, M1 gate status `migration-debt-present`, and zero unclassified locations. The current legacy debt is classified into runtime UI input, hybrid GI render, runtime graphics, texture importer DDS container, and editor UI fixture owner groups. The previous runtime-interface diagnostics, UI layout, UI template, and runtime asset groups were resolved by making archived UI pipeline stage names an explicit stored-report policy, by renaming the WrapBox Flow slot note to the current runtime contract, by moving runtime schema conversion names to `source_template_fixture`, by cutting editor asset/session source-schema naming to `UiAssetSourceSchema::LayoutDocument`, by naming the editor host-template cache path as the tree-template compile/document cache, by renaming the editor view-projection non-v2 rejection guard from legacy asset-path wording to `NonV2AssetPath`, by narrowing animation asset binary fallback to explicit v1 payload conversion, and by naming runtime asset importer `.ui.toml` / `.v2.ui.toml` guards as source-template fixture policy.
- Production code currently has 58 suspect non-network `server` naming references after ignoring `observer` substring false positives and allowing real network/session/target-server/dev-server/external API contexts. The `non_network_server_naming` audit owner is folder-backed in `runtime_structure_audits/non_network_server_naming.py` and reports M1 gate status `migration-debt-present`, with zero unclassified locations and classified debt in graphics render-framework and editor workbench authority-label groups. The stale editor scene comment group is resolved by naming the runtime scene inspection boundary directly, and the editor asset/resource owner group is resolved by naming retained-host app dependencies and resource-access fixtures as managers.
- Plugin runtime gap detection is now owned by `runtime_structure_audits/plugin_runtime_gaps.py`; current evidence remains zero gaps after preserving the `plugin_runtime_gaps` JSON field and `Plugin Runtime Gaps` Markdown section.
- Large production files are now grouped by the folder-backed `large_file_ownership` audit owner. Current gate evidence scans the 1000-line hotspot threshold, reports M1 gate status `migration-debt-present`, 33 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots. Current owner classes are `runtime-framework-render`, `runtime-other`, `editor-retained-host`, `editor-ui`, and `support-hub`; the detailed decision table is in `docs/engine-architecture/large-file-ownership-m1.md`.

Existing runtime absorption guards already protect parts of the root shape:

- `runtime_absorption/root_entries.rs` requires `zircon_runtime` crate root to expose the plugin namespace rather than flattened plugin symbols.
- `runtime_absorption/builtin_modules.rs` verifies core runtime module order and missing required plugin reporting.
- `runtime_absorption/compatibility_shells.rs` rejects nested compatibility crates under `zircon_runtime/crates`.

## Architecture Gaps

1. App-level optional plugin fan-out is still too high.

   `zircon_app` should choose profile, target, and manifest inputs, then hand off to runtime-owned module/plugin assembly. It should not compile against every optional first-party runtime plugin. This is the clearest developer-experience and build-performance gap because adding a plugin currently leaks into process entry dependencies.

2. `runtime_modules.rs` has too many owners in one production file.

   Runtime target modes, plugin identity, profile defaults, manifest expansion, availability diagnostics, linked-plugin registration, and module vector construction are separate responsibilities. Keeping them together makes generated code harder to review and makes duplicate behavior more likely.

3. Scene inspection had been named and placed like editor ownership inside runtime.

   The old runtime `scene/editor_projection` path was a useful read-only world view, but the naming made the runtime/editor boundary ambiguous. The M3 inspection slice hard-cut that path to `zircon_runtime/src/scene/inspection/*`, with editor-specific interaction remaining in `zircon_editor`.

4. Plugin public surface needs a hard public/private split.

   The crate root already avoids flattening plugin symbols publicly, but the next review must confirm that native loader, generated export plans, package manifests, catalog entries, and runtime extension registries are each owned by a narrow module surface. Compatibility aliases should be deleted rather than preserved.

5. Large files are now architecture risks, not style risks.

   Large production files in runtime/editor are likely hiding repeated DTO conversion, repeated validation, and mixed lifecycle logic. Future optimization should split by ownership first, then look at allocation and clone behavior.

6. Historical architecture docs carried old standalone crate owners.

   M1 has rewritten the active engine-architecture index, architecture-first guide, and runtime-interface convergence document to the current three-package structure and runtime-internal core spine. The structural audit now reports no stale standalone-crate references in those architecture docs.

## Coordination Constraints

Several active sessions are touching adjacent areas. Broad production edits should avoid these zones until their session notes quiet down:

- Plugin ecosystem: `zircon_runtime::plugin`, `zircon_plugins::*`, and framework contracts for AI, animation, navigation, net, physics, sound, VM language, and related catalog/package metadata.
- Host editor UI: `zircon_runtime::ui::surface::*`, retained-host painter/style selector code, and editor host contract files.
- WGPU render chain: RHI, render graph, graphics runtime, scene rendering extraction, and GPU resource plumbing.
- Asset/material/mesh flow: asset import/resource streamer, material and mesh document flow, and scene/graphics resource handoff.
- Hub and web prototype sessions: avoid broad formatting or ownership edits under `zircon_hub` and prototype UI artifacts.

M0 therefore records decisions and guardrails first. Production code changes should be narrow and either outside those zones or explicitly coordinated with the owning session note.

## Review And Optimization Order

1. M1 - Public boundary and audit guardrails.

   Add hard checks for stale standalone-crate references, root-surface flattening, app static plugin dependency fan-out, hard-cutover migration-smell vocabulary, large production files, and compatibility shell regressions. This stage should make architecture drift visible before more generated code lands.

2. M2 - Runtime module and plugin assembly.

   Split `runtime_modules.rs` into folder-backed owners for target/profile identity, default manifests, selection expansion, availability diagnostics, linked registration, and final module vector construction. Then move app-side first-party plugin registration into a runtime-owned or generated registry path so `zircon_app` stops knowing optional plugin crates directly.

3. M3 - Scene/runtime/editor boundary.

   Done for the first M3 slice: replace `scene/editor_projection` with neutral runtime `scene/inspection`. Editor behavior consumes that snapshot from `zircon_editor`; runtime remains the scene authority. The follow-up serialization guard keeps scene/project saves free of editor selection, viewport-tool, overlay, gizmo, and preview override state.

4. M4 - Plugin lifecycle and generated export model.

   Review native loader, package discovery, export build plans, feature registration, generated files, hot reload, and VM plugin surfaces. Delete legacy compatibility paths and keep generated code behind stable registry contracts.

5. M5 - Runtime performance pass.

   After interfaces settle, reduce eager allocation and clone-heavy report construction, prefer stable typed IDs over string matching in hot paths, build registries only for selected target/profile combinations, and keep String-heavy diagnostics at IO or reporting edges.

6. M6 - Graphics/render runtime convergence.

   Split mixed debug/projection/report files, align render feature ownership with the runtime module/plugin contract, and coordinate with the WGPU render session before touching RHI or GPU resource paths.

7. M7 - Editor-facing runtime UX cleanup.

   Split retained-host lifecycle/painter files by workflow and remove repeated DTO conversion only after the runtime UI and editor host sessions finish their current slices.

## First Safe Implementation Slice

The first safe slice is M1-audit, not a broad production rewrite:

- Done: the runtime structural audit now reports app static plugin dependency count, stale standalone-crate references in architecture docs, root public surface flattening, runtime scene editor-named surface, non-network `server` naming M1 gate classification, and large-file ownership classes.
- Done: the runtime structural audit now reports generated-code boundary risk in export source templates and classifies each behavior label into an M1 gate decision group.
- Done: the runtime structural audit now reports native plugin root re-export breadth and classifies each re-export symbol into an M4 gate decision group.
- Done: the runtime structural audit now reports production Rust hard-cutover migration-smell debt, separates allowed business `bridge` terminology from migration bridge blockers, and classifies every current `legacy` reference into an owner group with zero unclassified locations.
- Done: the runtime structural audit now reports large-file ownership as an M1 gate with threshold, owner decision groups, migration debt, and unclassified hotspot checks.
- Done: the active engine-architecture entry docs now use the current `zircon_app` / `zircon_runtime` / `zircon_editor` package structure and `zircon_runtime::core::{runtime, manager, framework, math, resource}` spine instead of historical package paths.
- The M1 documentation and audit-hardening slice stayed outside active runtime/editor production modules.
- Done on 2026-06-04: the first M2 production split moved `zircon_runtime/src/builtin/runtime_modules.rs` into a folder-backed assembly package while preserving the public runtime-owned facade.
- Done on 2026-06-04: the second M2 slice moved linked first-party provider fan-out from `zircon_app` into `zircon_first_party_runtime_catalog`, leaving app entry responsible for profile/render-profile projection only.
- Done on 2026-06-04: the M3 scene boundary audit now tracks scene/project serialization separately and rejects editor authoring state in both source structure and project roundtrip JSON.
- Done on 2026-06-04: `runtime_root_surface` moved into `runtime_structure_audits/runtime_root_surface.py`, reducing the main audit script from 690 lines to 643 lines while preserving root-surface evidence at 17 public modules, 3 public `pub use` locations, 75 crate-visible graphics re-exports, and 2 current root-surface risks.
- Done on 2026-06-04: `non_network_server_naming` moved into `runtime_structure_audits/non_network_server_naming.py`, reducing the main audit script from 643 lines to 600 lines while preserving the non-network `server` naming evidence at 179 suspect references and 20 sample locations.
- Done on 2026-06-04: `non_network_server_naming` now reports an M1 gate status of `migration-debt-present`, filters 72 `observer` substring false positives, allows 93 real server-context lines, and classifies the remaining 87 suspect references into graphics render-framework debt, editor asset/resource owner debt, and editor scene comment debt with zero unclassified locations.
- Done on 2026-06-04: `entry_static_dependencies` moved into `runtime_structure_audits/entry_static_dependencies.py`, reducing the main audit script from 600 lines to 528 lines while preserving app fan-out evidence at 4 app path dependencies, 0 optional runtime plugin path dependencies, 0 optional runtime plugin feature mentions, 1 built-in entry/runtime module crate, and no entry dependency risk.
- Done on 2026-06-04: `legacy_standalone_references` moved into `runtime_structure_audits/legacy_standalone_references.py`, reducing the main audit script from 528 lines to 476 lines while preserving stale standalone-crate architecture-doc evidence at zero counts and zero sample locations.
- Done on 2026-06-04: `runtime_scene_editor_surface` moved into `runtime_structure_audits/runtime_scene_editor_surface.py`, keeping the main audit script at 476 lines after stale unused helper cleanup while preserving M3 scene/editor boundary evidence at zero editor-named production paths, zero public editor-named locations, and zero risks.
- Done on 2026-06-04: `large_file_ownership` moved into `runtime_structure_audits/large_file_ownership.py`, reducing the main audit script from 476 lines to 420 lines while preserving large-file evidence at 10 reported top hotspots and owner-class counts `editor-retained-host=11`, `editor-ui=8`, `runtime-framework-render=1`, `runtime-other=10`, and `support-hub=3`.
- Done on 2026-06-04: `large_file_ownership` now reports an M1 gate status of `migration-debt-present`, 33 hotspots above the 1000-line threshold, 5 migration-debt owner groups, decision groups for all current hotspots, and zero unclassified hotspots.
- Done on 2026-06-04: `plugin_runtime_gaps` moved into `runtime_structure_audits/plugin_runtime_gaps.py`, reducing the main audit script from 420 lines to 391 lines while preserving zero plugin runtime gaps.
- Done on 2026-06-04: `module_inventory` moved module descriptor distribution, stub descriptor usage, owner coverage, module classification, support-crate listing, workspace production-file inventory, and hotspot source data into `runtime_structure_audits/module_inventory.py`, reducing the main audit script from 391 lines to 231 lines while preserving 3 classified module crates, zero stub descriptor usage, 3 support crates, and 10 large-file hotspots.
- Done on 2026-06-04: `ecs_query_state_boundary` now owns its Markdown renderer as well as audit data, reducing the main audit script from 231 lines to 210 lines while preserving the same old-file-absent, 6/6 owner module, 123/180 root non-empty line, and zero oversized owner module evidence.
- Done on 2026-06-04: `scene_project_serialization_boundary` moved into `runtime_structure_audits/scene_project_serialization_boundary.py`, reducing the main audit script from 864 lines to 690 lines while preserving the 7-file/0-forbidden-location evidence.
- Done on 2026-06-04: the first runtime-other large-file production split reduced `zircon_runtime/src/dynamic_api/session.rs` from 1207 lines to 947 lines by extracting ABI status construction, host-request conversion, input-event conversion, and preview fallback helpers under `zircon_runtime/src/dynamic_api/session/`.
- Done on 2026-06-04: the matching dynamic API test split removed the 893-line `zircon_runtime/src/dynamic_api/tests.rs` and replaced it with `tests/{api_table,profile_control,viewport,session_lifecycle,host_requests,accessibility,input_events,support}.rs`.
- Done on 2026-06-04: the dynamic API test boundary now has both a Rust structure test and structural audit output that reject a recreated `tests.rs`, missing owner modules, missing declarations, non-navigational `mod.rs` content, and oversized owner test files.
- Done on 2026-06-04: `dynamic_api_test_boundary` itself moved out of the near-threshold audit script into `runtime_structure_audits/dynamic_api_test_boundary.py`, reducing the main audit script from 1095 lines to 992 lines while preserving the JSON and Markdown evidence.
- Done on 2026-06-04: `generated_code_boundary` and `native_plugin_public_surface` also moved into folder-backed audit owner modules, reducing the main audit script from 992 lines to 864 lines while preserving generated behavior count 13 and native root re-export count 54.
- Done on 2026-06-04: `native_plugin_public_surface` now reports an M4 gate status of `migration-debt-present`, classifies all 54 root re-export symbols into native ABI contract, loader/discovery, live-host runtime, and behavior-report debt groups, and reports zero unclassified root re-export symbols.
- Done on 2026-06-04: the support-crate ABI surface split reduced `zircon_runtime_interface/src/runtime_api.rs` from 1082 lines to a 12-non-empty-line facade backed by `runtime_api/{api_table,constants,events,host_requests,requests,viewport}.rs`, preserving the public `runtime_api::*` re-export shape and adding a boundary test against facade regression.
- Done on 2026-06-04: `runtime_api_boundary` is now part of the structural audit. It rejects missing or unexpected ABI owner modules, missing facade declarations or re-exports, direct ABI declarations in the facade, facade growth beyond 20 non-empty lines, and owner modules above 700 lines.
- Done on 2026-06-04: the second runtime-other large-file production split removed `zircon_runtime/src/scene/ecs/query/query_state.rs` and replaced it with folder-backed `query_state/{mod,cached_direct,helpers,mutable,read_only,system_param}.rs`, preserving the public `QueryState` export while reducing audit `runtime-other` hotspots from 11 to 10.
- Done on 2026-06-04: `ecs_query_state_boundary` is now part of the structural audit. It rejects a recreated `query_state.rs`, missing or unexpected owner modules, a missing `mod query_state;` declaration, root behavior impl drift, root growth beyond 180 non-empty lines, and owner modules above 450 lines.

## M2 Runtime Module Assembly Follow-Up

The first production M2 slice split the previous central assembly file into:

- `runtime_modules.rs` facade;
- `runtime_modules/ids.rs`;
- `runtime_modules/load_report.rs`;
- `runtime_modules/core_modules.rs`;
- `runtime_modules/manifest.rs`;
- `runtime_modules/availability.rs`;
- `runtime_modules/extensions.rs`;
- `runtime_modules/plugin_modules.rs`;
- `runtime_modules/assembly.rs`;
- `runtime_modules/tests/{manifest,availability,registration,support}.rs`.

This keeps the existing public API intact while separating target/profile identity, manifest construction, availability diagnostics, extension aggregation, plugin-domain mapping, core module construction, and orchestration. The root facade is now structural, and the largest new assembly owner is below the large-file warning threshold.

The second production M2 slice added `zircon_plugins/first_party_runtime_catalog` as the single linked-provider package consumed by `zircon_app`. App features now forward into catalog features:

- `first-party-runtime-plugins -> base-runtime-plugins`;
- `first-party-advanced-render-runtime-plugins -> advanced-render-runtime-plugins`;
- `first-party-navigation-runtime-plugin -> navigation-runtime-plugin`.

The direct app match from `RuntimePluginId` to `zircon_plugin_*_runtime::plugin_registration()` has moved to the catalog. `zircon_app/src/entry/tests/source_assertions.rs` guards against reintroducing individual first-party runtime plugin crate dependencies or direct provider calls in app entry code.

## Runtime Dynamic API Session Split

The first runtime-other large-file slice kept the exported `ZrRuntimeApiV1` function table unchanged and split only private session implementation helpers:

- `session/status.rs` for ABI `ZrStatus` constructors;
- `session/host_requests.rs` for IME and gamepad rumble host-request conversion;
- `session/input_events.rs` for ABI input/window/gamepad/IME constant conversion;
- `session/preview.rs` for fallback frame and accessibility preview payloads.

`session.rs` remains the FFI session registry and `RuntimeDynamicSession` lifecycle/orchestration owner. This avoids creating a second public runtime API surface while reducing a runtime-other large-file hotspot below the audit threshold.

The matching test tree is now folder-backed:

- `tests/api_table.rs` for exported function-table and ABI version checks;
- `tests/profile_control.rs` for profiling JSON request/response behavior;
- `tests/viewport.rs` for viewport, surface, frame, present, and unbind validation;
- `tests/session_lifecycle.rs` for create/tick/profile lifecycle paths;
- `tests/host_requests.rs` for IME and gamepad host-request encoding/freeing;
- `tests/accessibility.rs` for accessibility tree/action fallback behavior;
- `tests/input_events.rs` for mouse-wheel, window-scale, and IME invalid-input rejection;
- `tests/structure.rs` for folder-backed test-tree regression checks;
- `tests/support.rs` for shared ABI fixtures and buffer free helpers.

New dynamic API assertions should land in the matching owner module, not in a recreated `tests.rs`. The audit reports this as `dynamic_api_test_boundary`; the current accepted state is `legacy_tests_file_exists = false`, 9 owner modules present, and zero oversized owner modules over 250 lines. The audit implementation is now a dedicated owner module too, so future boundary checks should continue moving into `runtime_structure_audits/` rather than growing the main audit script.

## ECS QueryState Split

The second runtime-other large-file slice kept `zircon_runtime::scene::ecs::QueryState` as the public query state type and split only source ownership:

- `query_state/mod.rs` owns `QueryState`, access descriptors, cache rebuilds, cache counters, and cache metadata accessors;
- `query_state/cached_direct.rs` owns cached storage-location direct access for `CachedQueryData`;
- `query_state/read_only.rs` owns non-mutating query iteration, get/many/contains, cached read iteration, and combinations;
- `query_state/mutable.rs` owns mutable query access, duplicate-entity rejection, mutable many/combination iteration, and the narrow post-validation unsafe fetch;
- `query_state/helpers.rs` owns shared fixed-size collection and cached entity filtering helpers;
- `query_state/system_param.rs` owns the `SystemParam` bridge into runtime systems.

This follows Bevy's query module precedent: keep state/cache, data/filter, iterator, and system-param roles navigable instead of stacking every access family into one hot-path state file. `scene::tests::ecs_query_structure` and the structural audit's `ecs_query_state_boundary` both guard against recreating `query_state.rs`, missing owner files, behavior impl families in the root file, and owner files above the current budget. The current accepted audit state is old file absent, 6/6 owner modules present, root 123/180 non-empty lines, and no oversized owner modules. The audit owner now also renders its own Markdown section so `audit_runtime_structure.py` stays a short orchestration script.
