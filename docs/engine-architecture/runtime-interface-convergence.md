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
  - zircon_runtime/src/dynamic_api/exports.rs
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
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
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
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
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
  - zircon_runtime/src/dynamic_api/exports.rs
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
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/session_profiles.rs
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
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
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
  - plugin_surface_lifecycle_boundary Runtime 06 source/doc/cargo-pending mirror, app NativePlugin call-site count, native V1/V2 fixture scope, and public-surface debt checks
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
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
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

The runtime crate root should remain a narrow module entry surface. Current audit output reports 20 public modules, 3 public `pub use` locations, 80 crate-visible graphics re-export symbols, direct `rhi_wgpu` backend exposure, and M1 gate status `migration-debt-present`; the `graphics`, `render_graph`, and `rhi` root seats remain graphics/RHI-deferred debt, while `rhi_wgpu` remains backend-public debt until the M3 render-owner alias cutover.

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

## Runtime 10 ABI Inventory

Runtime 10 treats the dynamic runtime API table and the native/plugin host tables as one ABI governance family, while leaving native plugin behavior ownership in the plugin lifecycle plan. The current inventory source is `zircon_runtime_interface/src/runtime_api/api_table.rs` plus `zircon_runtime_interface/src/plugin_api.rs`.

### Function Table Families

| Table | Version | Field count | Owner source | Producer | Consumer |
|---|---:|---:|---|---|---|
| `ZrHostApiV1` | 1 | 4 | `runtime_api/api_table.rs` | host app/editor loader | `zircon_runtime_get_api_v1` |
| `ZrRuntimeApiV1` | 1 | 13 | `runtime_api/api_table.rs` | `zircon_runtime::dynamic_api::exports` | host app/editor loader |
| `ZrHostApiV3` | 3 | 7 | `plugin_api.rs` | runtime native host adapter | v3 native runtime plugins |
| `ZrHostEcsApiV1` | 1 | 3 | `plugin_api.rs` | embedded in `ZrHostApiV3` | v3 native runtime plugins |
| `ZrHostAssetApiV1` | 1 | 1 | `plugin_api.rs` | embedded in `ZrHostApiV3` | v3 native runtime plugins |
| `ZrHostEventApiV1` | 1 | 2 | `plugin_api.rs` | embedded in `ZrHostApiV3` | v3 native runtime plugins |
| `ZrHostBridgeApiV1` | 1 | 1 | `plugin_api.rs` | embedded in `ZrHostApiV3` | v3 native runtime plugins |
| `ZrHostDiagnosticsApiV1` | 1 | 2 | `plugin_api.rs` | embedded in `ZrHostApiV3` | v3 native runtime plugins |
| `ZrPluginStateSnapshotApiV1` | 1 | 4 | `plugin_api.rs` | plugin entry report | runtime native live host |
| `ZrPluginApiV1` | 1 | 4 | `plugin_api.rs` | plugin entry report | runtime native live host |

The permanent guard is `zircon_runtime_interface/src/tests/abi_safety_contracts.rs`: `function_table_structs_are_all_repr_c` requires the 10 listed `Zr*Api*` function-table structs to stay in the inventory and to keep local `#[repr(C)]` attributes. `function_table_field_counts_match_runtime_10_inventory` locks the table field-count matrix above, and `runtime_api_session_operation_surface_matches_inventory` locks the `ZrRuntimeApiV1` session operation field order below. `repr_c_guard_fails_on_missing_local_attribute` is the representative negative self-check. The same source pair currently contains 14 `#[repr(C)]` records overall; the guard intentionally locks function-table shape while leaving non-table DTO layout tests to their owner modules.

### Cross-Boundary DTO Domains

| Domain | Owner files | Transport form | Governance |
|---|---|---|---|
| Handles, status, buffers, version | `handles.rs`, `status.rs`, `buffer.rs`, `version.rs` | raw `#[repr(C)]` values and borrowed/owned byte buffers | layout and boundary tests; no runtime/editor dependency imports |
| Dynamic runtime function table and requests | `runtime_api/{api_table,requests,events,viewport,constants}.rs` | `#[repr(C)]` values plus optional function pointers | `runtime_api_boundary`, `abi_safety_contracts`, and focused event/frame/viewport tests |
| Runtime host requests and profiling | `runtime_api/host_requests.rs`, `profiling.rs` | serde payloads carried through `ZrOwnedByteBuffer` or `ZrByteSlice` | serde roundtrip tests and profile-control dynamic API tests |
| Native/plugin host tables and plugin reports | `plugin_api.rs`, `plugin_events.rs`, `plugin_diagnostics.rs`, `manifest.rs` | `#[repr(C)]` tables/DTOs plus byte-slice payloads | `plugin_api_contracts` for layout and `abi_safety_contracts` for function-table inventory |
| Reflect and resource contracts | `reflect/**`, `resource/**` | serde DTOs, stable IDs, and handles | interface boundary tests; no OS/GPU/runtime object imports |
| UI contracts | `ui/**` | serde DTOs and interface-owned Rust contracts | UI contract tests; Runtime 10 M2 remains gated by Runtime 09/editor UI owner handoff before duplicate-definition drift governance can move production types |

`interface_public_signatures_stay_free_of_dynamic_object_exports` scans public signature lines in production interface sources for `Box<dyn`, `Rc<`, `Arc<dyn`, and literal `impl Trait`. `public_signature_guard_fails_on_dynamic_object_export` is the representative negative self-check. Constructor helpers that use concrete generic shorthand such as `impl Into<String>` are not raw ABI exports and remain governed by their owner tests; the guard targets object/dynamic-carrier leakage across the interface boundary.

### Runtime Session Operation Surface

`ZrRuntimeApiV1` exposes 10 handle-taking session operations plus `create_session`. Session validation lives in Rust-ABI `zircon_runtime::dynamic_api::session` owner functions and is intentionally reached only after ABI/version/viewport/payload preflight checks. `zircon_runtime::dynamic_api::exports` owns the final FFI panic boundary: `zircon_runtime_get_api_v1` returns null if table acquisition unexpectedly unwinds, and the function table points to `_ffi` wrappers that translate unexpected session-operation unwinds into `ZrStatusCode::Panic`. `runtime_api_table_entries_are_panic_wrapped_at_ffi_boundary` keeps this wrapper layer from being bypassed and rejects private session owners drifting back to `extern "C"`.

| Function-table field | Runtime function | Preflight before session lookup | Failure-path guard |
|---|---|---|---|
| `create_session` | `create_session` | output pointer, ABI version, profile, project root bytes | create-session lifecycle tests |
| `destroy_session` | `destroy_session` | handle validity | `session_destroy_reports_explicit_not_found_after_first_destroy`, invalid-handle test |
| `handle_event` | `handle_event` | event ABI version | invalid/destroyed handle guard |
| `capture_frame` | `capture_frame` | request ABI version, viewport | invalid/destroyed handle guard with non-null output |
| `capture_accessibility_tree` | `capture_accessibility_tree` | request ABI version, viewport, null-output fast path | invalid/destroyed handle guard with non-null output |
| `bind_viewport_surface` | `bind_viewport_surface` | request ABI, target ABI, viewport, native surface descriptor | invalid/destroyed handle guard with valid Win32 descriptor |
| `unbind_viewport_surface` | `unbind_viewport_surface` | viewport | invalid/destroyed handle guard |
| `present_viewport` | `present_viewport` | request ABI version, viewport | invalid/destroyed handle guard |
| `profile_control` | `profile_control` | null-output fast path, non-empty valid JSON request | invalid/destroyed handle guard with snapshot request |
| `tick_frame` | `tick_frame` | none beyond handle validation | existing unknown-handle test plus invalid/destroyed handle guard |
| `drain_host_requests` | `drain_host_requests` | null-output fast path | invalid/destroyed handle guard with non-null output |

### Version Strategy

The rule is deliberately conservative: any function-table field addition, removal, reorder, type change, or meaning change creates a new table version. Runtime does not rely on silent tail-field extension under the same version, even though `size_bytes` records layout size. `size_bytes` remains a diagnostic/validation field for explicit negotiation, not permission to mutate an existing version in place.

`ZIRCON_RUNTIME_ABI_VERSION_V1` governs the dynamic runtime C ABI DTO family. Plugin host tables carry explicit table versions (`ZrHostApiV3` with v1 domain sub-tables today) and should bump the narrowest affected table when a domain changes. Serde payload domains may evolve through their own schema/version fields when the outer ABI carrier is unchanged; changing the carrier itself still follows the table/DTO bump rule.

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
- Runtime 01 dependency manifest, prerelease/version policy, ZrVM external path, text stack, physics/archive/editor-only decisions, and pending validation gate boundary through `tech_stack_boundary`;
- Runtime 02 core spine, root surface, generated/export adapter, pending Cargo, and render-owner alias debt boundary through `core_spine_root_generated_boundary`;
- Runtime 03 schedule/frame-loop source, fixed-step tests, UI extract side path, explicit stage ordering, parallel executor diagnostics, and pending validation gate boundary through `schedule_frame_loop_boundary`;
- Runtime 04 asset facade/query/resource-state, worker-pool, watcher/error, artifact-cache, and pending validation gate boundary through `asset_pipeline_boundary`;
- Runtime 05 plan-status, subplan/index map, pending gate, `last_refined`, backlog, review coverage, and full scene closeout boundary through `runtime_plan_status_boundary`;
- Runtime 07 performance-hotpath frame spans, extract counters, QueryState/change-detection telemetry, asset-worker candidate diagnostics, hotspot inventory guard, owner-budgeted large-file optimization gate, and pending validation gate boundary through `performance_hotpath_boundary`;
- Runtime 08 ECS data-kernel storage, entity lifecycle, observer, deferred command, event/message, change-tick, documentation, and pending validation gate boundary through `ecs_kernel_data_boundary`;
- Runtime 09 UI architecture module map, source-scan baseline, v2 verdict, and pending owner/Cargo gate boundary through `ui_architecture_boundary`;
- Runtime 11 JobSystem task owner, dependency-scheduling, diagnostics, and direct-Rayon boundary through `job_system_boundary`;
- Runtime 12 input runtime owner, framework input contract, action-evaluator, gamepad ABI, and pending validation gate boundary through `input_stack_boundary`;
- Runtime 13 script binding host ledger, capability, gameplay facade, native ECS ABI, and pending validation gate boundary through `script_binding_boundary`;
- Runtime 14 module-family root seats and mirror-doc judgements through `module_family_boundary`;
- runtime interface ABI owner split through `runtime_api_boundary`;
- Runtime 10 dynamic runtime API ABI/session/loader boundary through `dynamic_runtime_api_boundary`;
- generated-code boundary risk, including M1 gate status, behavior decision groups, migration debt, and unclassified behavior labels through `generated_code_boundary`;
- native plugin public-surface risk, including M4 gate status, symbol decision groups, migration debt, and unclassified root re-export symbol checks through `native_plugin_public_surface`, plus Runtime 06 source/doc/status/Cargo-pending mirror coverage through `plugin_surface_lifecycle_boundary`;
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
- dynamic API tests are folder-backed by behavior owner instead of a single `tests.rs`, preserving focused `dynamic_api` regressions after the split. The audit implementation for `dynamic_api_test_boundary` is also folder-backed under `runtime_structure_audits/`, and the current evidence reports the old file absent, all 11 owner modules declared, `session_lifecycle.rs` split down to 136 lines, no oversized owner module, and no boundary risk.
- large production-file hotspots and ownership classes are now rendered through the folder-backed `large_file_ownership` audit owner. The same owner now reports `large_file_ownership_gate` with M1 gate status `migration-debt-present`, threshold 1000, 41 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots. Current owner classes remain `editor-retained-host=11`, `editor-ui=8`, `runtime-framework-render=2`, `runtime-other=17`, and `support-hub=3`.
- M1/M2/M3/M4/M5 structural audit owners for module inventory, app static dependency fan-out, plugin runtime gaps, stale standalone-crate doc references, runtime root surface, runtime scene editor-named surface, generated-code behavior, hard-cutover migration-smell classification, non-network `server` naming M1 gate classification, scene/project serialization, Runtime 01 tech-stack boundary, Runtime 02 core/root/generated boundary, Runtime 03 schedule/frame-loop boundary, Runtime 04 asset pipeline boundary, Runtime 05 plan-status boundary, Runtime 06 plugin surface/lifecycle boundary, Runtime 07 performance-hotpath boundary, Runtime 08 ECS data-kernel boundary, Runtime 09 UI architecture boundary, Runtime 10 dynamic runtime API boundary, Runtime 11 JobSystem boundary, Runtime 12 input-stack boundary, Runtime 13 script-binding boundary, Runtime 14 module-family boundary, native plugin M4 public-surface classification, and large-file ownership are folder-backed under `runtime_structure_audits/`; the main audit script remains an orchestration boundary. The non-network server gate now treats Hub UNC project-path test literals as allowed path fixtures instead of unclassified runtime owner debt. The hard-cutover migration-smell gate now classifies the current Runtime UI input split plus Hub archived-text message policy and Net HTTP backend hyper-client API debt instead of leaving those references unowned.
- `zircon_runtime_interface::runtime_api` is folder-backed by ABI owner; `runtime_api_boundary` now reports 6/6 owner modules present, `runtime_api.rs` at 12/20 non-empty facade lines, no missing declarations or re-exports, no direct ABI declarations in the facade, and no owner file above 700 lines.
- `zircon_runtime_interface::tests::abi_safety_contracts` guards the runtime/plugin function-table inventory: 10 current `Zr*Api*` structs must keep local `#[repr(C)]` attributes, their documented field counts must stay in sync with the Runtime 10 matrix, the `ZrRuntimeApiV1` session operation surface must keep its documented order, and public interface signature lines remain free of dynamic object carriers (`Box<dyn`, `Rc<`, `Arc<dyn`, and literal `impl Trait`).
- `zircon_runtime::scene::ecs::query::QueryState` is folder-backed by query owner; `ecs_query_state_boundary` now owns both audit data and Markdown rendering, reports the old `query_state.rs` absent, all 8 owner modules present including `stats.rs`, root state/cache ownership below the 180 non-empty-line budget, no oversized owner module, and no boundary risk.
- Runtime 01 dependency governance is mirrored by `tech_stack_boundary`; it reports audited manifest files 5/5, corrected non-dependencies 6, tech-stack Rust/static guard anchors 11/11, editor-only dependency candidates 3, Jolt visible-unavailable feature slots 2, removed/editor-only manifest dependencies 0, Rapier/Avian manifest dependencies 0, and `risks = []`. This does not promote Runtime 01 beyond static evidence; `tech_stack/extensions/text_shaper/plugin physics` Cargo gates remain pending.
- Runtime 02 core/root/generated governance is mirrored by `core_spine_root_generated_boundary`; it reports core root entries 6/6, core public modules 5/5, retired core root entries 0, runtime root public modules 20/20, public `pub use` sites 3/3, crate-visible graphics alias debt 80/80, root-surface M1 gate `migration-debt-present`, generated export templates 9/9, generated behavior locations 6/6, generated allowed adapters 6/6, generated migration debt 0/0, generated-code M1 gate `classified-and-clear`, root_entries guard tests 12, root_surface guard tests 6/6, generated-code guard tests 7/7, and `risks = []`. `root_surface_interface_convergence_mirror_uses_current_audit_counts` keeps this document from drifting back to the stale 17-module / 75-symbol root-surface mirror. This does not promote Runtime 02 beyond static evidence; `core/root/generated/export_build_plan/app/editor/plugin` Cargo gates and the M3 render-owner alias cutover remain pending.
- Runtime 03 `zircon_runtime::scene` schedule/frame-loop semantics are mirrored by `schedule_frame_loop_boundary`; it reports source files 18/18, guard/test files 8/8, `SystemStage` count and variants 9/9, fixed-loop stages 3/3, dynamic-session `.tick_time(...)` calls 1/1, Runtime 03 guard anchors 14/14, no `WorldDriver` second `advance_time_by(...)` references, no dynamic-session raw-delta level tick references, and `risks = []`. `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` keeps Runtime 03, the runtime index, the M0 review, and this convergence document aligned with those structure-audit counts. This does not promote Runtime 03 beyond static evidence; `ecs_schedule/time/session/schedule_parallel` Cargo filters remain pending.
- Runtime 04 `zircon_runtime::asset` pipeline semantics are mirrored by `asset_pipeline_boundary`; it reports audited source files 19/19, guard/test files 10/10, worker diagnostics 5/5, artifact-store scene roundtrip guards 4/4, Runtime 04 guard anchors 21/21, retired worker-count constructor references 0, old watch-loop `WATCH_DEBOUNCE` references 0, and `risks = []`. `runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free` keeps loading/status queries on `ProjectAssetManager`, typed `Assets<TAsset>`, and the `AssetManager` service trait instead of reintroducing asset server naming, while also requiring finalized handle/loader/processor decisions in the Runtime 04 reference gap table. This does not promote Runtime 04 beyond static evidence; broader `asset::` / `worker_pool` Cargo filters remain pending.
- Runtime 05 plan-status and closeout semantics are mirrored by `runtime_plan_status_boundary`; it reports plan-status support files 6/6, runtime subplans 14/14, runtime index subplan rows 14/14, problem rows 17/17, known backlog rows 7/7, status counts `in_progress=14`, core guard anchors 13/13, pending Cargo gate anchors 15/15, doc anchors 8/8, Runtime 05 closeout status `in_progress`, and `risks = []`. This does not close Runtime 05; `pending_full_scene_cargo` and `cargo test -p zircon_runtime --lib scene:: --locked` remain the closeout gate.
- Runtime 06 plugin surface/lifecycle semantics are mirrored by `plugin_surface_lifecycle_boundary`; it reports Runtime 06 source 9/9, mirror docs 5/5, frontmatter `in_progress`, `last_refined = 2026-06-14`, native root re-export 68/68, M4 gate `migration-debt-present`, native public-surface debt groups 5/5, unclassified native symbols 0/0, public native re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 6/6, `zircon_plugins` V1/V2 usage files 1/1, export_build_plan V1/V2 usage 0/0, and full source/doc/validation anchors. This does not close Runtime 06; the `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane and M2/M3 native hard-cutover remain pending.
- Runtime 07 performance hotpath measurement boundary is mirrored by `performance_hotpath_boundary`; it reports audited source files 10/10, guard/test files 5/5, frame span anchors 9/9, QueryState telemetry anchors 13/13, change-detection telemetry anchors 9/9, extract telemetry anchors 10/10, asset-worker candidate telemetry anchors 5/5, hotspot guard anchors 16/16, counter assertion anchors 12/12, doc anchors 16/16, pending Cargo/profiling/FPS gate anchors 5/5, stale top3 placeholder false, large-file owner gate `migration-debt-present`, threshold 1000, hotspots 41, debt groups 5, owner classes 5, unclassified hotspots 0, and `risks = []`. `runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps that owner-budget summary aligned with Runtime 07, the runtime index, the hotspot inventory, the M0 review, and `large-file-ownership-m1.md`. This does not promote Runtime 07 beyond static evidence; extract/ecs_query/profiling/FPS validation remains pending.
- Runtime 08 `zircon_runtime::scene::ecs` data-kernel semantics are mirrored by `ecs_kernel_data_boundary`; it reports `expected_source_file_count = 20`, `expected_test_file_count = 7`, `storage_anchors = 9/9`, `entity_lifecycle_anchors = 10/10`, `observer_anchors = 8/8`, `deferred_command_anchors = 11/11`, `event_message_anchors = 11/11`, `change_tick_anchors = 6/6`, `runtime_08_guard_anchors = 17/17`, `doc_anchors = 7/7`, `pending_cargo_gate_anchors = 6/6`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` keeps the ECS module doc, Runtime 08, the runtime index, the M0 review, and this convergence document aligned with those structure-audit counts. This does not promote Runtime 08 beyond static evidence; entity/observer/command/messages/change_tick/ecs Cargo filters remain pending.
- Runtime 09 UI architecture is mirrored by `ui_architecture_boundary`; it reports audited source/doc files 11/11, `ui/` entries 17/17, `surface/` entries 20/20, UI legacy full-tree hits 167/167, UI legacy production hits 102/102, UI legacy production files 12/12, UI taffy production hits 161/161, UI taffy production files 7/7, runtime `ui::v2` anchors 10/10, interface `ui::v2` anchors 9/9, Runtime 09 guard anchors 4/4, pending UI owner/Cargo gate anchors 7/7, doc anchors 10/10, and `risks = []`. This does not promote Runtime 09 beyond static evidence; `ui/input/naming_boundary/layout/template` owner/Cargo gates remain pending.
- Runtime 10 dynamic runtime API boundary is mirrored by `dynamic_runtime_api_boundary`; it reports `expected_source_file_count = 14`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `ui_pending_gate_anchors = 8/8`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 7/7`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps the dynamic API module doc, Runtime 10, the runtime index, the M0 review, this convergence document, and the cdylib loader doc aligned with those structure-audit counts. This does not promote Runtime 10 beyond static evidence; `dynamic_api`, full app loader, and UI contract owner/Cargo gates remain pending.
- Runtime 11 `zircon_runtime::core::runtime::tasks` is folder-backed by JobSystem owner; `job_system_boundary` reports `expected_module_count = 9`, `direct_rayon_paths = 3`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` keeps JobSystem module docs, Runtime 11, the runtime index, the M0 review, and this convergence document aligned with those structure-audit counts. This does not promote Runtime 11 beyond static evidence; `tasks/ecs_schedule/worker_pool/rayon` Cargo filters and the render-owned `parallel_frustum` cutover remain pending.
- Runtime 12 `zircon_runtime::input` is folder-backed by input-stack owner; `input_stack_boundary` reports `expected_runtime_module_count = 10`, `expected_framework_module_count = 17`, `expected_test_module_count = 5`, `public_surface_anchors = 10/10`, `runtime_12_guard_anchors = 5/5`, `missing_doc_anchors = []`, `missing_test_anchors = []`, `missing_cargo_gate_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` keeps the input module doc, Runtime 12, the runtime index, the M0 review, and this convergence document aligned with those structure-audit counts. This does not promote Runtime 12 beyond static evidence; input/action_map/gamepad/app Cargo filters remain pending.
- Runtime 13 `zircon_runtime::script` binding semantics are folder-backed by script-binding owner; `script_binding_boundary` reports audited source files 10/10, guard/test files 2/2, fixed host ledger 6 modules / 50 functions / 2 type descriptors, callback counts builtin=11/11, gameplay=37/37, macro=2/2, host capability anchors 11/11, Runtime 13 guard anchors 7/7, native ECS ABI references in script source = 0, no oversized guard/test files, and `risks = []`. This does not promote Runtime 13 beyond static evidence; script Cargo filters remain pending.
- Runtime 14 module-family root seats are folder-backed by the module-family owner; `module_family_boundary` reports `expected_family_count = 4`, `animation = 27`, `navigation = 3`, `diagnostic_log = 7`, `engine_module = 8`, `root_seat_guard_present = true`, no missing doc anchors, no file-count mismatch, and `risks = []`. `runtime_14_module_family_mirror_docs_match_structure_audit_counts` keeps Runtime 14, the runtime index, the M0 review, and this convergence document aligned with the same structure-audit counts.

## Runtime 10 UI Contract M2 Gate

Runtime 10 M2 is intentionally still a pending owner handoff, not a production UI/interface migration. `runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff` keeps the duplicate-definition cleanup and v2 synchronization rows pending until Runtime 09/editor UI owner work supplies the concrete `interface/ui` versus `runtime/ui` duplicate list and the interface/ui/editor Cargo lane is available.

This gate protects the current boundary in two directions: runtime-only UI behavior types must not be pushed into `zircon_runtime_interface`, and cross-boundary DTOs must not be silently duplicated in `zircon_runtime::ui`. The next implementation slice may only migrate a concrete pair after the owner handoff identifies it and after the consuming editor/runtime path can be tested.

Still needs refactor:

- app provider catalog cutover must remain guarded: current `entry_static_dependencies` evidence is 4 app path dependencies, 0 direct optional runtime plugin path dependencies, and 0 optional runtime plugin feature mentions;
- `zircon_runtime/src/lib.rs` root public surface: current audit output reports 20 public modules, 3 public `pub use` locations, 80 crate-visible graphics re-export symbols, direct `rhi_wgpu` backend exposure, and M1 gate status `migration-debt-present`. The detailed decision table is in `docs/engine-architecture/runtime-root-surface-m1.md`;
- export source templates that still generate runtime behavior: current audit output reports 13 architecture-sensitive generated behavior locations, M1 gate status `migration-debt-present`, 5 behavior decisions, 5 migration-debt entries, and zero unclassified behavior labels. The current groups are `handwritten-owner-required`, `native-loader-isolation`, `entry-glue-review`, and `data-adapter-review`;
- `zircon_runtime::plugin` native loader public surface: current audit output reports 68 root-level native loader/ABI/bridge-method re-export symbols, M4 gate status `migration-debt-present`, 5 migration-debt groups, and zero unclassified root re-export symbols. The current groups are `native-abi-contract-public-debt`, `native-loader-discovery-public-debt`, `native-live-host-runtime-public-debt`, `native-behavior-report-public-debt`, and `native-bridge-method-public-debt`; the detailed decision table is in `docs/engine-architecture/native-plugin-boundary.md`. Runtime 06 now keeps this status tied to `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation` and `plugin_surface_lifecycle_boundary` until the `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane passes; the wider Runtime 06 mirror also records app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 6, fixture-only plugin V1/V2 usage, and export_build_plan V1/V2 usage 0;
- native/plugin loader and export-plan public/private boundaries;
- production hard-cutover migration-smell vocabulary: current audit output reports 212 `legacy` references, 0 `compat` references, 0 `shim` references, 300 allowed business `bridge` references, 0 migration-context bridge references, M1 gate status `migration-debt-present`, 7 migration-debt groups, and zero unclassified locations. The current owner groups include runtime UI input, hybrid GI render, runtime graphics, Hub archived-text message policy, texture importer DDS, Net plugin HTTP backend dependency policy, and editor UI fixtures. The detailed decision table is in `docs/engine-architecture/hard-cutover-migration-smells-m1.md`;
- non-network `server` naming in editor workbench and render-framework paths: current audit output reports 59 suspect references, M1 gate status `migration-debt-present`, 87 ignored `observer` substring false positives, 94 allowed real server-context or fixture lines, 2 migration-debt groups, and zero unclassified locations. The detailed decision table is in `docs/engine-architecture/non-network-server-naming-m1.md`;
- large retained-host, editor UI, runtime UI/RHI/asset/render/script/scene, runtime framework render, and hub support files: current audit output reports 41 hotspots above 1000 lines, M1 gate status `migration-debt-present`, 5 migration-debt owner groups, and zero unclassified hotspots. The detailed decision table is in `docs/engine-architecture/large-file-ownership-m1.md`.

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
