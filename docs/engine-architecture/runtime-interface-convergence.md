---
related_code:
  - Cargo.toml
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
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
  - docs/zircon_runtime/scene/inspection.md
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
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
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
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-root-surface-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/source_assertions.rs
  - docs/zircon_runtime/builtin/runtime_modules.md
  - docs/zircon_plugins/first_party_runtime_catalog.md
  - docs/zircon_runtime/scene/inspection.md
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
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
plan_sources:
  - user: 2026-04-18 implement the runtime interface family and structural audit skill plan
  - user: 2026-04-19 continue absorbing modules into runtime
  - user: 2026-04-20 hard-cut runtime absorption without shim paths
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
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
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/runtime_api.md
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - docs/zircon_runtime/scene/ecs/query_state.md
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime
doc_type: module-detail
---

# Runtime Interface Convergence

## Purpose

This document records the current runtime interface family after the absorption cutover. It is not a compatibility map for removed package names. The authoritative shape is:

`zircon_app -> zircon_runtime::core::{runtime, manager, framework, math, resource} -> zircon_runtime modules -> zircon_editor`

The goal is to keep the roadmap vocabulary usable without reintroducing old package boundaries. Each interface name maps to one current Rust owner, and future work must deepen those owners instead of adding parallel abstractions.

The reference-engine evidence that constrains these owners is recorded in `docs/engine-architecture/runtime-reference-engine-evidence.md`. Bevy owns the app/plugin/schedule/ECS precedent, Fyrox owns the Rust runtime/editor split precedent, and Unreal owns the large-scale Runtime/Editor/Programs/module boundary precedent.

## Interface Family

Current mapping:

- `IEntry -> zircon_app::EngineEntry`
- `IModule -> zircon_runtime::engine_module::EngineModule`
- `IService -> zircon_runtime::engine_module::EngineService`
- `IDriver -> zircon_runtime::engine_module::EngineDriver`
- `IManager -> zircon_runtime::engine_module::EngineManager`
- `IPlugin -> zircon_runtime::engine_module::EnginePlugin` for module metadata, with VM/plugin runtime behavior converging through `zircon_runtime::script`
- `IObject -> zircon_runtime::scene::RuntimeObject`
- `ISystem -> zircon_runtime::scene::RuntimeSystem`
- `IEntity -> zircon_runtime::scene::semantics::EntityIdentity`
- `IComponent -> zircon_runtime::scene::semantics::ComponentData`

`EngineService` is a metadata-level runtime contract. Concrete managers remain contract traits or handle surfaces such as asset, input, render, UI, and VM plugin managers. Do not replace that with a monolithic service base class.

## Current Owners

### `zircon_app`

`zircon_app` owns process entry, profile selection, and host startup. It no longer owns optional runtime plugin implementation fan-out directly. The app keeps the config-to-manifest projection helper, then delegates linked first-party provider selection to `zircon_first_party_runtime_catalog`.

The app layer may:

- choose target/profile inputs;
- choose editor/runtime/headless host mode;
- append the editor host module for editor mode;
- project render-profile selections into the project plugin manifest;
- call runtime-owned assembly APIs.

The app layer must not:

- encode plugin implementation selection logic;
- own runtime module ordering details beyond editor host attachment;
- expose individual optional first-party plugin crates as a public build surface.

### `zircon_runtime::core::runtime`

This is the lifecycle, dependency, registration, activation, event, schedule, and shutdown authority. The current `CoreRuntime` shape stays intact; the review plan optimizes boundaries and hot paths around it instead of replacing it with a new kernel.

### `zircon_runtime::core::manager`

This namespace owns stable service names, resolver access, handles, activation records, and manager-facing access entries. Managers and driver-like services should be dependency-described and resolver-driven, not directly wired through app/editor construction.

### `zircon_runtime::core::framework`

This namespace owns neutral DTOs and shared contracts. It must not become the place where concrete business behavior hides. Feature behavior belongs in runtime modules or `zircon_plugins/*/runtime`, while framework data remains reusable by runtime, editor, and plugin consumers.

### `zircon_runtime::engine_module`

This namespace owns the Rust interface family for modules, services, managers, drivers, plugins, and descriptor-backed module ownership. Root files should stay structural; behavior belongs in folder-backed owners.

### `zircon_runtime` modules

The runtime crate owns absorbed built-in module surfaces:

- foundation
- platform
- input
- asset
- scene
- graphics
- script
- UI

The runtime crate root should remain a narrow module entry surface. Current audit output still reports 17 public modules, 3 public `pub use` locations, 75 crate-visible graphics re-export symbols, and direct backend-module exposure from `zircon_runtime/src/lib.rs`; that is M1/M6 migration debt.

### `zircon_editor`

`zircon_editor` owns authoring logic and editor host state. It may consume runtime inspection/reflection snapshots, manager handles, and command surfaces. It must not become the owner of runtime world state, module descriptors, or runtime serialization authority.

### `zircon_plugins/*/runtime`

First-party plugins own optional concrete runtime behavior. Runtime framework contracts stay neutral; plugin crates provide concrete managers, backends, hooks, package metadata, and feature registration.

### `zircon_first_party_runtime_catalog`

The catalog is a plugin-workspace package that maps selected `RuntimePluginId` values to compiled first-party provider registration reports. It is intentionally outside `zircon_runtime`, because the runtime crate must not depend on concrete plugin implementations. `zircon_app` depends on this single catalog feature boundary instead of depending on each first-party runtime plugin crate directly.

## Module Owner Convergence

The current audit reports real `EngineModule` owners for:

- `DescriptorBackedEngineModule`
- `EditorModule`
- `AssetModule`
- `DiagnosticsCoreModule`
- `FrameCountModule`
- `FoundationModule`
- `GraphicsModule`
- `InputModule`
- `LogDiagnosticsModule`
- `LogModule`
- `PlatformModule`
- `SceneModule`
- `ScriptModule`
- `TasksModule`
- `TimeModule`
- `UiModule`

The same audit reports no production `stub_module_descriptor` usage and no plugin runtime gaps. The remaining issue is not missing module identity; it is mixed ownership, app/plugin fan-out, wide root surfaces, stale documentation, and large production files.

## Runtime Assembly Status

The first M2 production slice has split `zircon_runtime/src/builtin/runtime_modules.rs` into a folder-backed assembly package. The facade now delegates to:

- `ids.rs` for target and runtime plugin identity;
- `manifest.rs` for default/profile manifest construction;
- `availability.rs` for structured availability diagnostics;
- `load_report.rs` for load-report and required-missing state;
- `extensions.rs` for extension-registry aggregation;
- `plugin_modules.rs` for built-in versus externalized plugin-domain mapping;
- `core_modules.rs` for built-in module vector construction;
- `assembly.rs` for profile/target orchestration.

This removes the mixed-file assembly hotspot without changing the public facade consumed by `zircon_app`.

The second M2 slice moved app-level optional plugin fan-out behind `zircon_first_party_runtime_catalog`. `zircon_app/Cargo.toml` now depends on one catalog package and forwards app features into catalog features. The direct `RuntimePluginId -> zircon_plugin_*_runtime::plugin_registration()` match arms are no longer owned by app entry code.

## Scene And Editor Boundary

Runtime scene ownership is limited to:

- runtime ECS world;
- hierarchy and entity identity;
- reflection/property access;
- serialization;
- render extract data;
- neutral inspection snapshots.

Editor authoring ownership stays in `zircon_editor`:

- selection;
- inspector DTOs;
- viewport state;
- overlays;
- gizmo state;
- edit commands.

The previous runtime-side editor projection path has been hard-cut to `zircon_runtime::scene::inspection`. Runtime now exposes `WorldInspection`, `WorldInspectionHierarchyRow`, `WorldInspectionField`, and `World::inspect_world(focused)`. `zircon_editor::scene` consumes that neutral snapshot and remains responsible for editor selection, editor viewport state, overlays, gizmos, and edit-mode DTOs.

The runtime scene/project serializer must not contain editor authoring state: selection, editor viewport tools, overlays, gizmo state, preview camera overrides, or preview lighting. Runtime camera render data remains valid serialization data. For example, `SceneViewportRectAsset` and `RenderViewportRect` describe camera render output, not editor viewport authoring state.

The structural audit now includes a `scene_project_serialization_boundary` section over the runtime world, dynamic scene, project I/O, and scene-asset files. `zircon_runtime/src/scene/tests/component_structure.rs` mirrors that source guard, while `zircon_runtime/src/scene/tests/world_basics.rs` checks the project roundtrip JSON for forbidden authoring-state keys.

## Structural Audit

The repository-local structural audit is the current evidence source:

```powershell
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
```

The audit currently reports:

- module descriptor distribution through the folder-backed `module_inventory` owner;
- production stub descriptor usage;
- `EngineModule` owner coverage;
- plugin runtime gaps through the folder-backed `plugin_runtime_gaps` audit owner;
- app static dependency fan-out;
- runtime root-surface risk, including M1 gate status, module decision groups, public-use decisions, and migration debt through `runtime_root_surface`;
- runtime scene editor-named public surface risk;
- runtime scene/project serialization authoring-state risk;
- dynamic API test-tree boundary;
- ECS query-state owner boundary;
- runtime interface ABI owner split through `runtime_api_boundary`;
- generated-code boundary risk, including M1 gate status, behavior decision groups, migration debt, and unclassified behavior labels through `generated_code_boundary`;
- native plugin public-surface risk, including M4 gate status, symbol decision groups, migration debt, and unclassified root re-export symbol checks through `native_plugin_public_surface`;
- stale architecture-document references;
- hard-cutover migration-smell risk, including production `legacy`/`compat`/`shim`/`bridge` counts, allowed business bridge counts, classification groups, migration debt, and unclassified reference checks through `hard_cutover_migration_smells`;
- non-network `server` naming risk, including M1 gate status, false-positive filtering, classification counts, migration debt, and unclassified reference checks through `non_network_server_naming`;
- large production-file hotspots;
- large-file ownership risk, including M1 gate status, threshold, owner decision groups, migration debt, and unclassified hotspot checks through `large_file_ownership_gate`;
- module classification through the folder-backed `module_inventory` owner.

Do not call a module family converged unless the audit and the relevant source files both support that claim.

## Current Diagnosis

Converged or mostly converged:

- the fixed root package shape;
- runtime module descriptor ownership, owner coverage, and classification are folder-backed by `runtime_structure_audits/module_inventory.py`; current evidence covers 3 module crates, zero stub descriptor usage, and 3 support crates outside module classification;
- production removal of stub module descriptors;
- runtime absorption of asset, scene, graphics, UI, script, foundation, platform, and input module registration;
- plugin runtime gap detection is folder-backed by `runtime_structure_audits/plugin_runtime_gaps.py`, and current evidence remains zero gaps;
- app-level first-party provider fan-out is now delegated to `zircon_first_party_runtime_catalog` instead of direct `zircon_plugin_*_runtime` dependencies in `zircon_app`; the `entry_static_dependencies` audit owner currently reports 4 app path dependencies, 0 optional runtime plugin path dependencies, 0 optional runtime plugin feature mentions, and 1 built-in entry/runtime module crate;
- stale standalone crate references in active engine architecture docs remain at zero through the folder-backed `legacy_standalone_references` audit owner;
- runtime scene editor-named public surface has been hard-cut to neutral `scene::inspection`, with current audit counts at zero through the folder-backed `runtime_scene_editor_surface` audit owner;
- runtime scene/project serialization sources currently report zero editor authoring-state locations in the `scene_project_serialization_boundary` audit. That audit owner is now folder-backed under `runtime_structure_audits/scene_project_serialization_boundary.py`, with current evidence still at 7 audited files and zero forbidden locations;
- `zircon_runtime::dynamic_api::session` has been split by private owner class so the FFI session file is below the large-file warning threshold while keeping the exported C ABI table unchanged;
- dynamic API tests are folder-backed by behavior owner instead of a single `tests.rs`, preserving 37 focused `dynamic_api` regressions after the split. The audit implementation for `dynamic_api_test_boundary` is also folder-backed under `runtime_structure_audits/`, and the current evidence reports the old file absent, all 9 owner modules declared, and no oversized owner module.
- large production-file hotspots and ownership classes are now rendered through the folder-backed `large_file_ownership` audit owner. The same owner now reports `large_file_ownership_gate` with M1 gate status `migration-debt-present`, threshold 1000, 33 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots. Current owner classes remain `editor-retained-host`, `editor-ui`, `runtime-framework-render`, `runtime-other`, and `support-hub`.
- M1/M2/M3/M4/M5 structural audit owners for module inventory, app static dependency fan-out, plugin runtime gaps, stale standalone-crate doc references, runtime root surface, runtime scene editor-named surface, generated-code behavior, hard-cutover migration-smell classification, non-network `server` naming M1 gate classification, scene/project serialization, native plugin M4 public-surface classification, and large-file ownership are folder-backed under `runtime_structure_audits/`; the main audit script remains an orchestration boundary.
- `zircon_runtime_interface::runtime_api` is folder-backed by ABI owner; `runtime_api_boundary` now reports 6/6 owner modules present, `runtime_api.rs` at 12/20 non-empty facade lines, no missing declarations or re-exports, no direct ABI declarations in the facade, and no owner file above 700 lines.
- `zircon_runtime::scene::ecs::query::QueryState` is folder-backed by query owner; `ecs_query_state_boundary` now owns both audit data and Markdown rendering, reports the old `query_state.rs` absent, all 6 owner modules present, root state/cache ownership at 123/180 non-empty lines, no oversized owner module, no boundary risk, and audit `runtime-other` large-file count reduced from 11 to 10.

Still needs refactor:

- app provider catalog cutover must remain guarded: current `entry_static_dependencies` evidence is 4 app path dependencies, 0 direct optional runtime plugin path dependencies, and 0 optional runtime plugin feature mentions;
- `zircon_runtime/src/lib.rs` root public surface: current audit output reports 17 public modules, 3 public `pub use` locations, 75 crate-visible graphics re-export symbols, direct `rhi_wgpu` backend exposure, and M1 gate status `migration-debt-present`. The detailed decision table is in `docs/engine-architecture/runtime-root-surface-m1.md`;
- export source templates that still generate runtime behavior: current audit output reports 13 architecture-sensitive generated behavior locations, M1 gate status `migration-debt-present`, 5 behavior decisions, 5 migration-debt entries, and zero unclassified behavior labels. The current groups are `handwritten-owner-required`, `native-loader-isolation`, `entry-glue-review`, and `data-adapter-review`;
- `zircon_runtime::plugin` native loader public surface: current audit output reports 54 root-level native loader/ABI re-export symbols, M4 gate status `migration-debt-present`, 4 migration-debt groups, and zero unclassified root re-export symbols. The current groups are `native-abi-contract-public-debt`, `native-loader-discovery-public-debt`, `native-live-host-runtime-public-debt`, and `native-behavior-report-public-debt`; the detailed decision table is in `docs/engine-architecture/native-plugin-boundary.md`;
- native/plugin loader and export-plan public/private boundaries;
- production hard-cutover migration-smell vocabulary: current audit output reports 142 `legacy` references, 0 `compat` references, 0 `shim` references, 231 allowed business `bridge` references, 0 migration-context bridge references, M1 gate status `migration-debt-present`, 5 migration-debt groups, and zero unclassified locations. The detailed decision table is in `docs/engine-architecture/hard-cutover-migration-smells-m1.md`;
- non-network `server` naming in editor workbench and render-framework paths: current audit output reports 58 suspect references, M1 gate status `migration-debt-present`, 72 ignored `observer` substring false positives, 93 allowed real server-context lines, 2 migration-debt groups, and zero unclassified locations. The detailed decision table is in `docs/engine-architecture/non-network-server-naming-m1.md`;
- large retained-host, editor UI, runtime UI/RHI/asset, runtime framework render, and hub support files: current audit output reports 33 hotspots above 1000 lines, M1 gate status `migration-debt-present`, 5 migration-debt owner groups, and zero unclassified hotspots. The detailed decision table is in `docs/engine-architecture/large-file-ownership-m1.md`.

## Next Convergence Targets

1. Refresh structural audit output for the app provider catalog cutover and keep the new source guard green.
2. Use `generated_code_boundary.behavior_decision_groups`, `generated_boundary_migration_debt_count`, and `unclassified_behavior_label_count` before any export-template edit; extend generated export provider output toward the same data/adapter boundary instead of duplicating linked-provider selection rules.
3. Keep the new runtime scene `inspection` surface neutral and move any richer edit-mode projection or authoring DTOs into `zircon_editor`.
4. Use `native_plugin_public_surface.symbol_decision_groups`, `native_plugin_public_surface_migration_debt_count`, and `unclassified_root_reexport_symbol_count` before any plugin root or native loader public-surface edit; move native loader and generated export behavior behind isolated plugin/tooling namespaces instead of runtime root paths.
5. Use `hard_cutover_migration_smells.classification_counts`, `hard_cutover_migration_smells.hard_cutover_migration_debt_count`, and `hard_cutover_migration_smells.unclassified_location_count` before any hard-cutover rename; keep business `bridge` terminology only when it names a real owner, not a migration forwarding layer.
6. Use `root_surface_audit.module_decision_groups`, `public_use_decisions`, and `root_surface_migration_debt_count` before any `zircon_runtime/src/lib.rs` cut; remove root-level graphics re-export fan-out after the WGPU/RHI session stabilizes.
7. Use `large_file_ownership_gate.decision_groups`, `classification_counts`, `large_file_migration_debt_count`, and `unclassified_hotspot_count` before any large-file split; split by owner class rather than by arbitrary line-count chunks.
8. Continue splitting large production and test files by owner class rather than by arbitrary line-count chunks.
9. Keep ECS query/cache hot-path work inside the new `query_state/{cached_direct,read_only,mutable,helpers,system_param}.rs` owners instead of recreating a mixed state file.

The interface family is usable, but the runtime architecture is not finished. Completion requires reducing the M1/M2 blockers in current audit output, not just documenting them.
