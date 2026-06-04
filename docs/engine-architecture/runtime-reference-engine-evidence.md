---
related_code:
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/bevy/crates/bevy_app/src/plugin_group.rs
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/schedule.rs
  - dev/bevy/crates/bevy_ecs/src/query/state.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/UnrealEd.Build.cs
  - dev/UnrealEngine/Engine/Plugins
implementation_files:
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - Test-Path over Bevy, Fyrox, Unreal reference files listed in related_code
  - Select-String declaration evidence for App, Plugin, schedules, QueryState, Engine, Editor, ModuleRules, and UnrealEd editor target guard
  - git diff --check -- docs/engine-architecture/runtime-reference-engine-evidence.md docs/engine-architecture/runtime-architecture-review-m0.md docs/engine-architecture/runtime-interface-convergence.md .codex/sessions/20260604-1232-runtime-architecture-review.md
  - conflict-marker scan over the same docs and session note
doc_type: architecture-evidence
---

# Runtime Reference Engine Evidence

## Purpose

This document pins the runtime architecture review to concrete source evidence in the local `dev/` reference trees. It is not a porting plan. The decision rule is to copy durable boundaries, ownership pressure, and validation gates, not implementation style wholesale.

The evidence supports the current staged plan:

- M1 public-surface and generated-code boundary cleanup;
- M2 runtime module and plugin catalog assembly;
- M3 runtime/editor scene boundary;
- M4 plugin lifecycle convergence;
- M5 performance pass over service resolution, query/cache, schedule, and manifest lookup;
- M6 graphics/RHI public-surface cleanup after the active render session settles.

## Reference Routing

Bevy leads app composition, plugin registration, schedule ownership, and ECS query-cache discipline. The relevant local evidence is `dev/bevy/crates/bevy_app/src/app.rs`, `plugin.rs`, `plugin_group.rs`, `main_schedule.rs`, and `dev/bevy/crates/bevy_ecs/src/{schedule,query}`.

Fyrox leads Rust-native engine/editor separation and plugin lifecycle placement. The relevant local evidence is `dev/Fyrox/fyrox-impl/src/engine/mod.rs`, `dev/Fyrox/fyrox-impl/src/plugin/mod.rs`, and `dev/Fyrox/editor/src/lib.rs`.

Unreal leads large-scale Runtime, Editor, Programs, plugin descriptor, and module-rule separation. The relevant local evidence is `dev/UnrealEngine/Engine/Source/{Runtime,Editor,Programs}` plus plugin descriptors under `dev/UnrealEngine/Engine/Plugins`.

## Evidence Matrix

| Decision pressure | Bevy evidence | Fyrox evidence | Unreal evidence | Zircon decision |
| --- | --- | --- | --- | --- |
| App host versus plugin implementation fan-out | `App` is declared at `bevy_app/src/app.rs:86`; `App::add_plugins` is at `app.rs:641`; plugin contract and groups live in `plugin.rs:57` and `plugin_group.rs:204/233`. | `Engine` owns `plugins: Vec<PluginContainer>` in `fyrox-impl/src/engine/mod.rs:295`; plugin behavior is under `fyrox-impl/src/plugin/mod.rs:574`. | Runtime modules and engine plugins are explicit integration units through per-module `*.Build.cs` files and `*.uplugin` descriptors. | `zircon_app` remains the host/profile selector. Concrete first-party plugin registration stays behind `zircon_first_party_runtime_catalog` and runtime-owned assembly APIs, not direct app match arms. |
| Runtime schedule and lifecycle authority | `MainSchedulePlugin` is declared in `main_schedule.rs:309`; `Schedules` and `Schedule` are declared in `bevy_ecs/src/schedule/schedule.rs:46/382`; system insertion lives in schedule owner files. | `Plugin` exposes lifecycle hooks including `register`, `init`, `on_loaded`, `update`, and `post_update` in `fyrox-impl/src/plugin/mod.rs:577-622`; engine update and post-update paths are in `engine/mod.rs:1595/1700`. | `Core.Build.cs` declares Runtime/Core dependencies, while editor-only module loading is conditional. | Keep the existing `CoreRuntime` descriptor/context/handle/state shape. Optimize registration, dependency resolution, schedule execution, and plugin entry points around that owner instead of replacing the kernel. |
| ECS query and cache ownership | `QueryState` is a dedicated owner in `bevy_ecs/src/query/state.rs:79`; schedules and query state are separated by crate and folder. | Fyrox is less ECS-centric, but its `Engine` keeps scene containers, resource managers, script processor, task pool, and plugin list as explicit runtime owners. | Unreal's scale pressure reinforces subsystem modules rather than large mixed root files. | Keep Zircon `QueryState` folder-backed under `zircon_runtime/src/scene/ecs/query/query_state/`. Hot-path work belongs in `cached_direct`, `read_only`, `mutable`, `helpers`, and `system_param`, not a recreated mixed `query_state.rs`. |
| Editor/runtime boundary | Bevy is the weaker reference here, but `App`/`SubApp` composition still supports separating host concerns from runtime schedules. | Fyrox `Editor` is a separate owner at `editor/src/lib.rs:615` and stores `engine: Engine` plus `plugins: EditorPluginsContainer` at `editor/src/lib.rs:646-647`. | `UnrealEd.Build.cs` is an Editor module and rejects non-editor targets through the `bCompileAgainstEditor` guard at `UnrealEd.Build.cs:10-12`. | Runtime scene exports neutral inspection/reflection snapshots only. Selection, inspector DTOs, viewport state, overlays, gizmos, and authoring commands stay in `zircon_editor`. |
| Runtime root public surface | Bevy keeps app and ECS responsibilities in different crates (`bevy_app`, `bevy_ecs`) and routes behavior through narrow owners. | Fyrox separates implementation, editor, resources, UI, and plugin concerns by crate/folder. | Unreal uses top-level `Runtime`, `Editor`, `Programs`, and `Plugins` divisions; `Core.Build.cs` and `UnrealEd.Build.cs` make dependency direction explicit. | `zircon_runtime/src/lib.rs` should become a narrow facade. Root-level graphics internals and backend re-exports remain M1/M6 migration debt until RHI/WGPU active work settles. |
| Generated code versus runtime behavior | Bevy code generation is not the runtime behavior owner for plugin registration or schedule execution. | Fyrox plugin lifecycle is hand-owned by engine/plugin modules rather than generated export templates. | Unreal keeps build/program tooling separate from Runtime and Editor modules. | Generated output may emit DTOs, tables, adapters, and entry glue only. Runtime rules, plugin selection, lifecycle decisions, and state mutation move back to hand-written owner modules. |
| Native plugin loader isolation | Bevy's plugin model is trait/lifecycle based and does not put native loader details at app root. | Fyrox dynamic plugin restoration is expressed as plugin lifecycle state such as `on_loaded`, not as a broad root public ABI surface. | Unreal plugins are descriptor and module driven; editor plugins and runtime modules are separate integration surfaces. | `zircon_runtime::plugin` should expose the stable VM/plugin lifecycle and host handles as the main path. Native loader and ABI internals should move behind isolated tooling/test namespaces. |
| Non-network `server` naming | Bevy uses app, schedule, world, system, query, and resource vocabulary for non-network runtime coordination. | Fyrox uses engine, plugin, scene, resource, editor, and viewer vocabulary for non-network ownership. | Unreal reserves program/module/editor/runtime vocabulary for tooling and ownership rather than arbitrary non-network servers. | Keep `server` for real network/service-host/dev-server/target semantics and external APIs that literally use that term. Treat `observer` as unrelated vocabulary. Rename the remaining non-network runtime/editor/resource owners during their bounded slices instead of preserving old names. |
| Hard-cutover migration smell vocabulary | Bevy exposes current app/plugin/schedule/query owners directly instead of preserving old owner paths as compatibility layers. | Fyrox keeps plugin and editor/runtime owners explicit; old migration wording is not a normal boundary. | Unreal's Runtime/Editor/Programs/module split relies on direct module ownership instead of generic shim or compatibility folders as long-term architecture. | `legacy`, `compat`, `shim`, and migration-context `bridge` wording are audit debt in production Rust. Business `bridge` terms remain valid only when they name a real UI, navigation, native, or resource owner. |
| Large-file ownership pressure | Bevy keeps app, plugin, schedule, ECS query, and renderer behavior in separate crates/modules instead of one host file. | Fyrox separates engine, editor, resource, UI, scene, plugin, and script owners even when the engine object orchestrates them. | Unreal scale is managed through Runtime/Editor/Programs modules, plugin descriptors, and per-module source ownership. | Large Zircon files are owner-pressure evidence. Split by runtime/editor/Hub owner and behavior family before claiming performance or developer-experience convergence. |

## Reference-Specific Decisions

Bevy should guide Zircon's app/plugin/schedule/ECS architecture, but Zircon should not become a Bevy clone. The useful part is the ownership split: app composition is one layer, plugin contracts are another, schedule state is another, and query cache ownership is isolated.

Fyrox should guide the Rust landing zone for engine/editor and plugin lifecycle decisions. The useful part is that editor state owns editor behavior while the engine still owns runtime resources, scenes, plugin execution, and update/post-update flow.

Unreal should guide scale boundaries. The useful part is not C++ macros or UObject design; it is the strong separation of Runtime, Editor, Programs, plugins, and explicit module dependency rules. Zircon should preserve that clarity through Rust crates, folder-backed modules, and hard-cut public surfaces.

## Follow-Up Gates

M1 public-surface cleanup must check each root export against this evidence. A root export is allowed only when it is a stable facade or a deliberate module entry point.

M1 non-network naming cleanup must keep `server` limited to real server-context semantics and external API vocabulary, not generic runtime/editor owners. `observer` substring matches are audit noise, not debt.

M1 hard-cutover migration cleanup must use `hard_cutover_migration_smells.classification_counts`, `hard_cutover_migration_smells.hard_cutover_migration_debt`, and `hard_cutover_migration_smells.unclassified_locations` before any rename. `compat` and `shim` are direct blockers; `legacy` must resolve to an owner cut or explicit version policy; `bridge` is allowed only for real business bridge owners.

M1 large-file cleanup must use `large_file_ownership_gate.decision_groups`, `large_file_ownership_gate.large_file_migration_debt`, and `large_file_ownership_gate.unclassified_hotspots` before any split. A split is valid only when it creates coherent owner modules or structural facades, not arbitrary line buckets.

M2 assembly work must keep app fan-out at zero direct optional first-party plugin dependencies. Provider selection belongs behind the catalog and runtime assembly boundary.

M3 scene work must keep runtime inspection neutral. Any richer projection, selection, viewport, overlay, gizmo, or authoring DTO belongs in `zircon_editor`.

M4 plugin lifecycle work must use `native_plugin_public_surface.symbol_decision_groups`, `native_plugin_public_surface_migration_debt`, and `unclassified_root_reexport_symbols` before editing native loader public surfaces. It must reduce native loader root exposure and add failure-path tests around descriptor resolution, VM slot lifecycle, stable host handles, and hot-reload state migration.

M5 performance work must start with diagnostics or counters for service resolution, registry name parsing, ECS query/cache, schedule execution, render extract cloning/allocation, and plugin manifest lookup. Optimization should remove repeated string lookup, repeated vector construction, long lock holds, and avoidable clones without changing ownership boundaries.

M6 graphics/RHI cleanup must wait for the active WGPU/RHI work to settle, then remove root-level graphics internals and keep backend-specific behavior behind explicit backend modules.
