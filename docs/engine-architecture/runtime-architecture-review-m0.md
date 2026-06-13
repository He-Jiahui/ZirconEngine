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
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/middle.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - docs/zircon_runtime/ui/architecture.md
  - docs/zircon_runtime/input/input_state.md
  - docs/zircon_runtime/core/root_surface.md
  - docs/zircon_runtime/core/job_system.md
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
  - docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
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
  - zircon_runtime/src/dynamic_api/exports.rs
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
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
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/middle.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - docs/zircon_runtime/ui/architecture.md
  - docs/zircon_runtime/core/root_surface.md
  - docs/zircon_runtime/core/job_system.md
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
  - plugin_surface_lifecycle_boundary Runtime 06 source/doc/status/Cargo-pending mirror, app NativePlugin call-site count, native V1/V2 fixture scope, and public-surface debt checks
  - non_network_server_references M1 gate status, explicit count fields, classification count, migration debt, and unclassified reference checks
  - runtime_naming_boundary editor/legacy gate status, classification counts, migration debt, and unclassified reference checks
  - hard_cutover_migration_smells gate status, explicit count fields, classification count, migration debt, allowed bridge count, and unclassified reference checks
  - large_file_ownership_gate M1 gate status, explicit count fields, classification count, migration debt, and unclassified hotspot checks
  - Select-String reference declaration evidence over Bevy, Fyrox, and Unreal source files listed in docs/engine-architecture/runtime-reference-engine-evidence.md
  - git diff --check -- docs/engine-architecture/runtime-reference-engine-evidence.md docs/engine-architecture/runtime-architecture-review-m0.md docs/engine-architecture/runtime-interface-convergence.md .codex/sessions/20260604-1232-runtime-architecture-review.md
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/middle.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
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
- `zircon_runtime/src/lib.rs` exposes 20 public namespace modules, 3 public `pub use` locations, 74 total lines, crate-private graphics alias debt, and direct `rhi_wgpu` backend exposure. The `runtime_root_surface` audit owner is folder-backed in `runtime_structure_audits/runtime_root_surface.py`, and `runtime_absorption::root_surface` now locks the executable root shape: no flattened subsystem `pub use` surfaces, exactly the current curated public re-export sites, crate-private M3.2 type alias debt such as `SceneRenderer` and `WgpuRenderFramework`, and the graphics alias block remains private until Runtime 02 M3 removes it in a render-owner window.
- `zircon_runtime::scene` had 4 editor-named production paths and 9 public editor-named locations before the M3 cutover. The M3 inspection slice now replaces that production surface with neutral `zircon_runtime::scene::inspection`, and the folder-backed `runtime_scene_editor_surface` audit owner currently reports zero editor-named runtime scene locations.
- Runtime scene/project serialization now has a separate authoring-state boundary audit over 7 source files. The audit owner is folder-backed in `runtime_structure_audits/scene_project_serialization_boundary.py`. The current audit reports zero forbidden locations; camera render viewport rectangles are allowed runtime data, while selection, editor viewport tools, overlays, gizmos, and preview overrides are forbidden serialization state.
- Runtime `editor` / `legacy` naming now has a runtime-only classification gate in `runtime_structure_audits/runtime_naming_boundary.py` plus `runtime_absorption::naming_boundary`. Current evidence is `gate_status = classified`, `editor_unclassified = 0`, and `legacy_unclassified = 0`; the standalone `rustc --test` guard for `runtime_editor_and_legacy_naming_is_classified_by_owner` passes. Full Cargo integration is still blocked by unrelated active UI/render compile errors, so this is not a workspace pass claim. The gate records `editor` as legal only in explicit editor-host profile/mode vocabulary, UI/component catalog metadata, asset/framework/scene reflection descriptors, diagnostics, or tests; remaining `legacy` references are debt buckets owned by runtime UI input/render, graphics, DDS policy, UI template/layout, input, asset, dynamic API, and scene schema owners.
- `zircon_runtime::plugin::export_build_plan` still has 13 architecture-sensitive generated behavior locations across export source templates; this is the M1/M2 generated-code boundary target. The `generated_code_boundary` audit owner now reports M1 gate status `migration-debt-present`, classifies behavior into handwritten-owner-required, native-loader-isolation, entry-glue-review, and data-adapter-review groups, and reports zero unclassified generated behavior labels.
- `zircon_runtime::plugin` still publicly re-exports 68 native loader/ABI/bridge-method symbols from the crate plugin namespace; this is the M4 native plugin isolation target. The `native_plugin_public_surface` audit owner now reports M4 gate status `migration-debt-present`, classifies symbols into native ABI contract, loader/discovery, live-host runtime, behavior-report, and bridge-method debt groups, and reports zero unclassified root re-export symbols. Runtime 06 now has the wider `plugin_surface_lifecycle_boundary` mirror tying that public-surface debt to source/doc/status anchors, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 6, fixture-only plugin V1/V2 usage, export_build_plan V1/V2 usage 0, and the pending `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation lane.
- Engine architecture docs now have a folder-backed stale standalone-crate reference audit in `runtime_structure_audits/legacy_standalone_references.py`; the first M1 documentation cleanup brought that audit count to zero and current evidence remains zero.
- Production Rust now has a folder-backed hard-cutover migration-smell audit in `runtime_structure_audits/hard_cutover_migration_smells.py`. Current evidence scans 5839 production Rust files, reports 212 `legacy` references, 0 `compat` references, 0 `shim` references, 300 allowed business `bridge` references, 0 migration-context bridge references, M1 gate status `migration-debt-present`, 7 migration-debt groups, and zero unclassified locations. The current legacy debt is classified into runtime UI input, hybrid GI render, runtime graphics, Hub archived-text message policy, texture importer DDS container, Net plugin HTTP backend dependency policy, and editor UI fixture owner groups. The previous runtime-interface diagnostics, UI layout, UI template, and runtime asset groups were resolved by making archived UI pipeline stage names an explicit stored-report policy, by renaming the WrapBox Flow slot note to the current runtime contract, by moving runtime schema conversion names to `source_template_fixture`, by cutting editor asset/session source-schema naming to `UiAssetSourceSchema::LayoutDocument`, by naming the editor host-template cache path as the tree-template compile/document cache, by renaming the editor view-projection non-v2 rejection guard from legacy asset-path wording to `NonV2AssetPath`, by narrowing animation asset binary fallback to explicit v1 payload conversion, and by naming runtime asset importer `.ui.toml` / `.v2.ui.toml` guards as source-template fixture policy.
- Production code currently has 59 suspect non-network `server` naming references after ignoring `observer` substring false positives and allowing real network/session/target-server/dev-server/UNC fixture/external API contexts. The `non_network_server_naming` audit owner is folder-backed in `runtime_structure_audits/non_network_server_naming.py` and reports M1 gate status `migration-debt-present`, with zero unclassified locations and classified debt in graphics render-framework and editor workbench authority-label groups. The Hub UNC path fixture is explicitly allowed, the stale editor scene comment group is resolved by naming the runtime scene inspection boundary directly, and the editor asset/resource owner group is resolved by naming retained-host app dependencies and resource-access fixtures as managers.
- Plugin runtime gap detection is now owned by `runtime_structure_audits/plugin_runtime_gaps.py`; current evidence remains zero gaps after preserving the `plugin_runtime_gaps` JSON field and `Plugin Runtime Gaps` Markdown section.
- Large production files are now grouped by the folder-backed `large_file_ownership` audit owner. Current gate evidence scans the 1000-line hotspot threshold, reports M1 gate status `migration-debt-present`, 41 hotspots, 5 migration-debt owner groups, and zero unclassified hotspots. Current owner classes are `runtime-framework-render`, `runtime-other`, `editor-retained-host`, `editor-ui`, and `support-hub`; the detailed decision table is in `docs/engine-architecture/large-file-ownership-m1.md`.

Existing runtime absorption guards already protect parts of the root shape:

- `runtime_absorption/mod.rs` is the absorption guard harness: any guard module mounted here must be listed in this review's machine-readable frontmatter and explained in the body, so plan-status self-checks can detect stale audit coverage before a runtime plan is marked complete.
- `runtime_absorption/asset_surface.rs` protects the absorbed Runtime 04 asset namespace shape and facade query vocabulary: `zircon_runtime::asset` owns module registration, artifact/assets/importer/pipeline/project/watch remain namespaced, the old `zircon_asset` wildcard re-export surface stays retired, the runtime asset root does not keep the legacy editor asset surface, and `runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free` keeps loading/status queries on `ProjectAssetManager`, typed `Assets<TAsset>`, and the `AssetManager` service trait instead of reintroducing asset server naming. The same guard now requires the Runtime 04 reference gap table to keep finalized handle/loader/processor decisions instead of unresolved comparison placeholders.
- `runtime_absorption/asset_worker_policy.rs` protects the Runtime 04 / Runtime 11 worker-pool decision: `AssetWorkerPoolOptions`, bounded backpressure, in-flight request coalescing, diagnostics, `TaskPoolIo` thread-budget derivation, and `asset.worker.budgeted_threads` must stay documented across asset worker source, tests, docs, plans, and runtime index.
- `runtime_absorption/root_entries.rs` requires `zircon_runtime` crate root to expose the plugin namespace rather than flattened plugin symbols.
- `runtime_absorption/root_surface.rs` requires `zircon_runtime/src/lib.rs` to stay a namespace root with 20 public module declarations, three curated public `pub use` sites, no flattened subsystem root re-exports, and only crate-private graphics alias debt until Runtime 02 M3. Its M3.2 type alias debt pre-guard status is `pre_m3_type_alias_guard_static_passed_pending_render_owner`.
- `runtime_absorption/builtin_modules.rs` verifies core runtime module order and missing required plugin reporting.
- `runtime_absorption/compatibility_shells.rs` rejects nested compatibility crates under `zircon_runtime/crates`.
- `runtime_absorption/dynamic_api_session.rs` protects Runtime 10 M1.2 and M1.3: `minimal` and `headless` dynamic-session profiles must keep `RuntimeRenderBridge` optional, skip render bridge bootstrap, return an empty encoded frame for capture, and treat surface bind/unbind/present as no-op operations while lifecycle Cargo validation remains pending; the same guard file also keeps `exports.rs` as the only C ABI edge for `ZrRuntimeApiV1`, requires `_ffi` wrappers plus `ZrStatusCode::Panic` translation, and keeps private session owner functions on Rust ABI so `catch_unwind` remains effective.
- `runtime_absorption/generated_code_guard.rs` protects Runtime 02 M4 generated-code boundaries: source markers must stay uniform, marked generated files remain leaf data only, export-template behavior remains classified, and generated export entry/templates delegate behavior to the app export-bootstrap/provider owners instead of reviving direct runtime lifecycle calls.
- `runtime_absorption/input_stack.rs` protects Runtime 12 M0-M2: frame input contract anchors, `InputAction`/`InputBinding`/`InputActionMap`/`InputActionState` exports, UI-filtered `InputActionEvaluator::evaluate_with_consumed_buttons(...)`, and the app gilrs -> runtime ABI -> `InputEvent::Gamepad*` bridge must stay documented while Cargo validation is pending.
- `runtime_absorption/naming_boundary.rs` protects the Runtime 05 naming classification gate: runtime `editor` and `legacy` references must remain assigned to explicit owner buckets, and new unclassified naming cannot enter production runtime source while closeout Cargo validation is pending.
- `runtime_absorption/plan_status.rs` protects the runtime plan set itself: each `docs/plans/zircon_runtime/runtime/*.md` subplan must use a known frontmatter status, keep the status/evidence table anchors, keep non-placeholder status rows and concrete evidence for started work through `runtime_subplan_status_records_keep_non_empty_evidence`, mirror that status in `runtime/index.md`, keep the index subplan map exact through `runtime_index_subplan_map_covers_existing_plan_files_without_stale_rows`, keep problem rows pointed at existing subplans through `runtime_index_problem_rows_reference_existing_subplans`, keep execution dependencies pointed at existing subplans through `runtime_index_execution_dependencies_reference_existing_subplans`, avoid `completed` while pending-validation markers remain, keep in-progress index rows tied to a remaining gate through `runtime_index_in_progress_rows_record_remaining_gate`, keep the known-backlog gap table owner/trigger cells through `runtime_known_backlog_gaps_keep_owner_and_trigger_columns`, keep Runtime 05 `in_progress` until the full `scene::` Cargo gate closes, keep `last_refined` at or after the latest recorded date in the subplan, and ensure this review documents every module mounted in `runtime_absorption/mod.rs` through `runtime_architecture_review_documents_all_absorption_guards`. The split `runtime_absorption/plan_status/recent_static_guards.rs` keeps recent Runtime 01-14 static and pending-gate anchors recorded across the subplans, mirror docs, review, and runtime index.
- `runtime_absorption/plan_status/cargo_gates.rs` owns the per-runtime pending Cargo gate harness split out of `plan_status.rs` after the Runtime 03 follow-up so the parent file stays below the large-file threshold. `runtime_absorption/plan_status/cargo_gates/early.rs` keeps Runtime 01 dependency decisions on `tech_stack/text_shaper/plugin physics` Cargo pending through `runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation`, Runtime 02 core/root/generated/export gates pending through `runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation`, Runtime 03 schedule/frame-loop rows and P3 on `ecs_schedule/time/session/schedule_parallel` Cargo pending through `runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation`, Runtime 04 on broader asset/worker Cargo pending through `runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation`, Runtime 06 on `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` through `runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation`, Runtime 07 on extract/ecs_query/performance profiling/FPS gates through `runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation`, and Runtime 08 on entity/observer/command/messages/change_tick/ecs filters through `runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation`. `runtime_absorption/plan_status/cargo_gates/middle.rs` keeps Runtime 09 on `ui/input/naming_boundary/layout/template` owner/Cargo pending through `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`. `runtime_absorption/plan_status/cargo_gates/late.rs` keeps Runtime 10 M1.3 on `dynamic_api` through `runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation`, Runtime 10 UI contract M2 on Runtime 09/editor UI owner handoff through `runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff`, Runtime 11 on `tasks/ecs_schedule/worker_pool/rayon` through `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`, Runtime 12 on input/action_map/gamepad/app through `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation`, Runtime 13 on script filters through `runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass`, and Runtime 14 on module-family Cargo/rustc through `runtime_14_module_family_cargo_gate_stays_visible_until_filters_pass`.
- `runtime_absorption/performance_hotspots.rs` protects Runtime 07 M1.3 while authoritative vampire FPS and trace values are still blocked: hotspot candidates must reference counted tests or the 10fps evidence session, unmeasured suspicions cannot seed M2 optimization work, and render submission evidence must stay routed to the render plan rather than Runtime 07 production code.
- `runtime_absorption/rayon_boundary.rs` protects Runtime 11 M2.1/M2.2: production direct Rayon is allowed only in `core/runtime/tasks/{pool,parallel_for}.rs` and the tracked render-owned `parallel_frustum.rs` exception. Current M2.1 pre-guard status is `pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner`; `parallel_frustum.rs` remains `render-owner-pending-runtime-11-m2-1-cutover`, and actual graphics cutover not executed.
- `runtime_absorption/job_system.rs` protects the Runtime 11 JobSystem structural mirror: `expected_module_count = 9`, `direct_rayon_paths = 3`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []` must stay synchronized across `job_system_boundary`, JobSystem module docs, Runtime 11, the runtime index, this review, and runtime-interface convergence.
- `runtime_absorption/resource_foundation.rs` protects the absorbed resource foundation: runtime-visible resource DTOs stay public through `core::resource`, while editor inspector residue such as `ResourceInspectorAdapterKey`, `ResourceTypeDescriptor`, and `inspector_adapter` stays out of the runtime resource surface.
- `runtime_absorption/script_absorption.rs` protects script subsystem absorption: `zircon_runtime/src/script/mod.rs` owns the script namespace, the old `zircon_script` wildcard re-export stays retired, the workspace manifest does not relist `zircon_script`, and the standalone crate path remains deleted.
- `runtime_absorption/schedule_frame_loop.rs` protects the Runtime 03 schedule/frame-loop structural mirror: source files 18/18, guard/test files 8/8, `SystemStage` count and variants 9/9, fixed-loop stages 3/3, dynamic-session `.tick_time(...)` calls 1/1, Runtime 03 guard anchors 14/14, no `WorldDriver` second `advance_time_by(...)` references, no dynamic-session raw-delta level tick references, and `risks = []` must stay synchronized across Runtime 03, the runtime index, this review, and runtime-interface convergence.
- `runtime_absorption/script_host_ledger.rs` protects Runtime 13 host binding status: the documented host ledger must match builtin/gameplay/bridge host registration counts, capability representative functions, bridge-module shape, and script ECS/gameplay access boundaries while script Cargo validation remains pending; `runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass` keeps the subplan/index/review state on the same pending script filters gate.
- `runtime_absorption/ui_architecture.rs` protects Runtime 09 M0: `docs/zircon_runtime/ui/architecture.md`, Runtime 09, and the runtime index must retain the 17-entry `ui/` map, current 20-entry `surface/` map, legacy/taffy baseline values, and `v2-replacement-mainline` verdict while UI production code remains owner-gated by the editor UI session.

## Runtime 01 Tech Stack Guard

The 2026-06-13 Runtime 01 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation`. This is a status guard over dependency-governance decisions that are statically landed but still awaiting the `tech_stack`, `extensions`, `text_shaper`, and plugin physics validation gates.

The guard keeps Runtime 01 `in_progress`, requires all M1/M2/M3 status rows to retain Cargo-pending language, keeps the declared `tech_stack/text_shaper/plugin physics` commands visible, and mirrors the same pending state through P10, the Runtime 01 subplan row, `runtime-tech-stack.md`, `text.md`, `physics-plugin-options.md`, the editor-only backlog, and this review. The protected anchors include the prerelease version pins, ZrVM path dependency gate, text stack matrix, complex-text `UiTextShaper` route, fontdue/editor-only backlog, Jolt-unavailable physics ruling, ZIP archive decision, and rfd/arboard exclusion.

The same 2026-06-13 follow-up added `runtime_structure_audits/tech_stack_boundary.py` and wired it into the Python structural audit. The current static mirror reports manifest files 5/5, corrected non-dependencies 6, tech-stack Rust/static guard anchors 11/11, editor-only dependency candidates 3, Jolt visible-unavailable feature slots 2, removed/editor-only manifest dependencies 0, Rapier/Avian manifest dependencies 0, and `risks = []`. This is still static structure evidence; `tech_stack/extensions/text_shaper/plugin physics` Cargo gates remain pending.

## Runtime 03 Schedule Frame-Loop Guard

The 2026-06-13 Runtime 03 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation`. This is a status guard over the schedule and frame-loop slices that are code/documentation landed but still awaiting the `ecs_schedule/time/session/schedule_parallel` Cargo validation lane.

The guard keeps Runtime 03 `in_progress`, requires M1/M2/M3 status rows to retain Cargo-pending language, keeps the declared `ecs_schedule`, `session`, `zircon_app`, `fixed_update`, `time`, and `schedule_parallel` commands visible, and mirrors the same pending state through P3, the Runtime 03 subplan row, `frame_schedule.md`, `schedule_parallel_executor.md`, and this review. The protected anchors include `schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`, `session_ui_extract_remains_documented_dynamic_session_side_path`, `world_driver_consumes_runtime_time_advance_without_advancing_clocks_again`, `level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`, `level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`, `level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance`, `fixed_step_plan_reports_overstep_fraction_in_unit_range`, `ScheduleParallelExecutionReport`, `schedule_parallel_execution_report_records_diagnostic_counts`, `representative_schedule_produces_multi_system_parallel_batches`, and `parallel_and_serial_execution_reach_identical_world_state`.

The same 2026-06-13 follow-up added `runtime_structure_audits/schedule_frame_loop_boundary.py` and wired it into the Python structural audit. The current static mirror reports source files 18/18, guard/test files 8/8, `SystemStage` count and variants 9/9, fixed-loop stages 3/3, dynamic-session `.tick_time(...)` calls 1/1, Runtime 03 guard anchors 14/14, no `WorldDriver` second `advance_time_by(...)` references, no dynamic-session raw-delta level tick references, and `risks = []`. `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` keeps Runtime 03, the runtime index, this review, and runtime-interface convergence aligned with those structure-audit counts. This is still static evidence only; Cargo validation stays on the existing `ecs_schedule/time/session/schedule_parallel` gate.

## Runtime 04 Asset Pipeline Guard

The 2026-06-13 Runtime 04 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation`. This is a status guard over the asset pipeline slices that are code/static landed and partly focused-Cargo validated, but still awaiting the broader `asset:: / worker_pool` filters.

The guard keeps Runtime 04 `in_progress`, requires the M1/M2 rows to retain Cargo-pending language, keeps the declared `load_state`, `resource`, `asset::`, `worker_pool`, and `watch` validation commands visible, and mirrors the same pending state through P7, the Runtime 04 subplan row, asset facade/worker docs, and this review. The already recorded focused evidence remains narrower: `artifact_store_roundtrips_scene_assets_with` passed 4/4 and `watcher` passed 7/7, while broader asset validation is still pending an owner-safe Cargo lane.

The same 2026-06-13 follow-up added `runtime_structure_audits/asset_pipeline_boundary.py` and wired it into the Python structural audit. The current static mirror reports source files 19/19, guard/test files 10/10, worker diagnostics 5/5, artifact-store scene roundtrip guards 4/4, Runtime 04 guard anchors 21/21, retired worker-count constructor references 0, old watch-loop `WATCH_DEBOUNCE` references 0, and `risks = []`. It now includes the facade query-surface guard that rejects stale asset server vocabulary and unresolved reference-gap placeholders. This is still static structure evidence; broader `asset::` / `worker_pool` Cargo filters remain pending.

## Runtime 06 Plugin Surface Lifecycle Gate

The 2026-06-13 Runtime 06 status follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation`. This guard records that the ZrVM empty-argument binding fix has local evidence, while runtime real-backend/fallback, plugin/native plugin, app, and `zircon_plugins` workspace validation are still pending.

The guard keeps Runtime 06 `in_progress`, requires M1 status rows to retain runtime Cargo-pending language, requires M2/M3 native lifecycle rows to remain `待开始`, keeps the declared `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation commands visible, and mirrors the same pending state through P4, the Runtime 06 subplan row, `native-plugin-boundary.md`, `runtime-interface-convergence.md`, Runtime 05 closeout, and this review. It also binds the native plugin public-surface evidence to `native_plugin_public_surface.m4_gate_status=migration-debt-present` and `root_reexport_count = 68`.

The 2026-06-14 Runtime 06 structural follow-up added `plugin_surface_lifecycle_boundary` to the Python audit. Current mirror evidence reports source 9/9, docs 5/5, frontmatter `in_progress`, `last_refined = 2026-06-14`, native root re-export 68/68, M4 gate `migration-debt-present`, debt groups 5/5, unclassified native symbols 0/0, public native re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 6/6, `zircon_plugins` V1/V2 usage files 1/1, export_build_plan V1/V2 usage 0/0, and full source/doc/validation anchors. This is static structure evidence only; Runtime 06 still waits on the declared script VM, native plugin, app, and plugins Cargo/native validation lane plus M2/M3 hard-cutover work.

## Runtime 09 UI Architecture Guard

The 2026-06-13 Runtime 09 follow-up added `runtime_absorption::ui_architecture` plus `docs/zircon_runtime/ui/architecture.md`. This is an M0 guard, not a UI production cutover: it locks the module boundary map, the baseline source-scan counts, and the v2 runtime/interface contract shape without editing `zircon_runtime::ui` production files.

The guard has three anchors: `runtime_09_ui_architecture_doc_records_current_boundaries`, `runtime_09_ui_architecture_baselines_match_current_source_scan`, and `runtime_09_v2_verdict_matches_runtime_and_interface_modules`. It intentionally records the current debt rather than resolving it: full-tree UI legacy hits are 167, production legacy hits are 102 across 12 files, production taffy hits are 161 across 7 files, and M1-M3 remain gated on an editor UI owner window plus later Cargo validation.

`ui_architecture_boundary` now mirrors the same Runtime 09 M0 shape through the Python structural audit. Current evidence reports source/doc files 11/11, `ui/` entries 17/17, `surface/` entries 20/20, UI legacy full-tree hits 167/167, UI legacy production hits 102/102, UI legacy production files 12/12, UI taffy production hits 161/161, UI taffy production files 7/7, runtime `ui::v2` anchors 10/10, interface `ui::v2` anchors 9/9, Runtime 09 guard anchors 4/4, pending UI owner/Cargo gate anchors 7/7, doc anchors 10/10, and `risks = []`. This remains static structure evidence; no `zircon_runtime::ui` production code is migrated by this audit.

## Runtime 09 UI Cargo Gate

The 2026-06-13 Runtime 09 status follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`. This guard keeps Runtime 09 `in_progress`, ties the M0 static-only evidence to `docs/zircon_runtime/ui/architecture.md`, and prevents the UI plan from closing before the `ui/input/naming_boundary/layout/template` filters and editor UI owner window provide real validation evidence.

## Runtime 10 Dynamic API Guards

The 2026-06-13 Runtime 10 follow-up added `runtime_absorption::dynamic_api_session::runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces`. This is a lifecycle boundary guard over the existing dynamic-session code, not a new ABI or render feature.

The guard keeps `RuntimeDynamicSession.render_bridge` optional, limits `uses_render_bridge()` to rendered `runtime`/`editor`/`dev` profiles, requires `minimal` and `headless` profiles to skip `RuntimeRenderBridge` bootstrap, and preserves no-render fallbacks: capture returns an empty encoded frame and surface bind/unbind/present return `Ok(())` without touching WGPU.

The same Runtime 10 guard module now includes `runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge`. It locks `dynamic_api::exports` as the final C ABI owner for `zircon_runtime_get_api_v1` and all 11 advertised `ZrRuntimeApiV1` session entries. Function-table entries must point at `_ffi` wrappers, the wrappers must translate unexpected unwinds to `ZrStatusCode::Panic`, table-acquisition unwinds must return a null pointer, and private `dynamic_api::session` owner functions must stay Rust-ABI `unsafe fn`.

`dynamic_runtime_api_boundary` now mirrors the Runtime 10 dynamic runtime API boundary through the Python structural audit. Current evidence reports `expected_source_file_count = 14`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `ui_pending_gate_anchors = 8/8`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 7/7`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps the module doc, Runtime 10, the runtime index, this review, runtime-interface convergence, and the cdylib loader doc aligned with those counts. It does not claim the pending `dynamic_api`, full app loader, or UI contract owner/Cargo gates.

## Runtime 10 UI Contract M2 Gate

The 2026-06-13 Runtime 10 status follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff`. This guard keeps UI 镜像契约 M2 pending while Runtime 09/editor UI owner work is still active and before the interface/ui/editor Cargo lane is available.

It requires Runtime 10 M2 rows to keep the owner-handoff language, keeps the declared `cargo test -p zircon_runtime_interface --locked`, `cargo test -p zircon_runtime --lib ui --locked`, and `cargo check -p zircon_editor --lib --locked` validation lane visible, and mirrors the same pending state through P13, the Runtime 10 subplan row, Runtime 05 closeout, and the runtime-interface convergence document. No `zircon_runtime::ui` or `zircon_runtime_interface::ui` production types are migrated by this gate.

## Runtime 08 ECS Data-Kernel Guard

The 2026-06-13 Runtime 08 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_08_ecs_kernel_cargo_pending_gate_stays_explicit_until_ecs_validation`. This is a status guard over the ECS data-kernel slices that are already code-complete but still awaiting Cargo validation.

The guard keeps Runtime 08 `in_progress`, requires the M1/M2/M3 rows to retain `code_complete_pending_cargo`, keeps the declared entity/observer/command/messages/change_tick/ecs filters visible, and mirrors the same pending state through the runtime index and this review. The protected anchors include `despawned_entity_handle_is_rejected_by_world_access`, `lifecycle_observer_fires_immediately_during_component_mutation`, `command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`, `events_require_explicit_update_and_keep_next_queue_hidden`, and `change_tick_comparison_survives_wraparound`.

The same follow-up now has `runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts`, and `runtime_structure_audits/ecs_kernel_data_boundary.py` is wired into the Python structural audit. The current static mirror reports `expected_source_file_count = 20`, `expected_test_file_count = 7`, `storage_anchors = 9/9`, `entity_lifecycle_anchors = 10/10`, `observer_anchors = 8/8`, `deferred_command_anchors = 11/11`, `event_message_anchors = 11/11`, `change_tick_anchors = 6/6`, `runtime_08_guard_anchors = 17/17`, `doc_anchors = 7/7`, `pending_cargo_gate_anchors = 6/6`, `mirror_docs_guard_present = true`, and `risks = []`. This is still static structure evidence; entity/observer/command/messages/change_tick/ecs Cargo filters remain pending.

## Runtime 12 Input Stack Guard

The 2026-06-13 Runtime 12 follow-up added `runtime_absorption::input_stack`. This is a boundary/documentation guard over the already-landed input stack slices: frame input transitions and clear timing, UI-first action mapping, and gamepad ABI bridging.

The guard now has four input-stack anchors: `runtime_12_input_stack_contracts_stay_documented_and_exported`, `runtime_12_action_mapping_keeps_ui_filtered_evaluation_path`, `runtime_12_gamepad_bridge_keeps_runtime_abi_path`, and `runtime_12_input_stack_mirror_docs_match_structure_audit_counts`. It deliberately keeps the state at static evidence plus pending Cargo: the runtime input/action/gamepad code and docs are source-checked, but `input`, `action_map`, `gamepad`, and app package Cargo filters still need an owner-safe build lane. `runtime_absorption::plan_status::cargo_gates::runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation` adds the narrower input/action_map/gamepad/app gate so Runtime 12 cannot be promoted before the declared filters have real evidence.

`input_stack_boundary` now mirrors the same Runtime 12 shape through the Python structural audit. Current evidence reports `expected_runtime_module_count = 10`, `expected_framework_module_count = 17`, `expected_test_module_count = 5`, `public_surface_anchors = 10/10`, `runtime_12_guard_anchors = 5/5`, `missing_doc_anchors = []`, `missing_test_anchors = []`, `missing_cargo_gate_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. It checks the runtime input owner tree, framework input contract tree, action evaluator UI-filtered path, app gilrs → runtime ABI → `InputEvent::Gamepad*` bridge, Rust guard anchors, and pending Cargo gate anchors without running Cargo.

## Runtime 13 Script Binding Guard

The 2026-06-13 Runtime 13 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass`. This is a status guard over script binding slices that are statically landed but still awaiting the `script` Cargo filter.

The guard keeps Runtime 13 `in_progress`, requires M1/M2 rows to retain `code_static_pending_cargo`, and mirrors the same pending state through P16, the Runtime 13 subplan row, the host function ledger, and this review. The protected anchors include `host_function_registry_matches_documented_ledger`, `host_capability_representatives_are_declared_on_registered_modules`, `host_function_without_required_capability_is_rejected_with_explicit_error`, `script_held_entity_handle_reports_invalid_after_despawn`, and `script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi`; the remaining validation is the Runtime 13 script filters gate.

`script_binding_boundary` now mirrors the same Runtime 13 shape through the Python structural audit. Current evidence reports `expected_source_file_count = 10`, `expected_test_file_count = 2`, fixed host ledger `6/50/2`, callback counts `builtin=11`, `gameplay=37`, `macro=2`, `host_capability_count = 11`, `native_ecs_abi_references = []`, `oversized_test_files = []`, and `risks = []`. It checks the host ledger, capability representatives, bridge dynamic-module anchors, `zr.zircon.gameplay` facade, native ECS ABI exclusion, Rust guard anchors, and pending script Cargo gate without running Cargo.

`runtime_absorption::plan_status::recent_static_guards::runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs` now ties the Runtime 01-14 static and pending-gate records together: detailed guard names must remain in their subplans and mirror docs, the review must keep the gate families visible, and the runtime index must keep the matching status summaries. Runtime 01 coverage includes dependency-governance, text-stack, physics-option, archive, editor-only dependency backlog anchors, and the `tech_stack_boundary` structural mirror. Runtime 02 coverage includes the core/root/generated gate, `pre_m3_type_alias_guard_static_passed_pending_render_owner`, and generated-code boundary status anchors. Runtime 03 coverage includes `RuntimeTimeAdvance`, fixed-step overstep, and schedule-parallel diagnostics. Runtime 04 coverage includes asset facade/resource state, worker-pool, watcher, artifact cache anchors, and the `asset_pipeline_boundary` structural mirror. Runtime 05 coverage keeps the `runtime_05_closeout_status_waits_for_full_scene_cargo_gate` / `pending_full_scene_cargo` closeout gate visible. Runtime 06 coverage includes the native plugin public-surface gate, `native_plugin_public_surface`, `plugin_surface_lifecycle_boundary`, and `root_reexport_count = 68`. Runtime 07 coverage includes `runtime_07_hotspot_inventory_requires_counted_evidence_before_m2` plus the `runtime_frame_schedule_stage.<SystemStage>` / `SceneScheduleRunner` stage-level span anchors from M0.3. Runtime 08 coverage includes ECS lifecycle, observer, command, event/message, change-tick anchors, and the `ecs_kernel_data_boundary` structural mirror. Runtime 09 coverage includes the `ui_architecture_boundary` structural mirror alongside the `ui/input/naming_boundary/layout/template` owner/Cargo gate. Runtime 10 coverage includes `runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces`, `runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge`, and the M2 UI contract owner gate `runtime_10_ui_contract_m2_gate_stays_pending_until_runtime_09_owner_handoff`; the UI owner gate must also remain in the runtime-interface convergence doc. Runtime 11 coverage includes `runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`, `tasks/ecs_schedule/worker_pool/rayon`, and the render-owned `parallel_frustum.rs` exception. Runtime 13 coverage includes the host ledger, capability, script-held entity invalidation, and gameplay facade/native ECS ABI split anchors. Runtime 14 coverage includes the module-family root-seat guard plus the navigation, animation, diagnostic-log, and engine-module mirror-doc guards. `runtime_10_m1_3_cargo_pending_gate_stays_explicit_until_dynamic_api_validation` adds the narrower gate that forbids promoting the FFI panic-boundary row before the declared `cargo test -p zircon_runtime --lib dynamic_api --locked -- --nocapture` validation has real evidence. `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation` does the same for the Runtime 12 input/action_map/gamepad/app validation lane.

Runtime 10 coverage also includes the `dynamic_runtime_api_boundary` structural mirror so the recent static guard list names the ABI/session/loader mirror alongside the Rust guard names and pending UI owner gate.

Runtime 09 coverage also includes the `ui_architecture_boundary` structural mirror so the recent static guard list names the UI architecture mirror alongside the Rust guard names and pending owner/Cargo gate.

## Runtime 11 Rayon Boundary Guard

The Runtime 11 guard currently treats `zircon_runtime/src/graphics/visibility/culling/parallel_frustum.rs` as the only render-owned direct-Rayon exception while render pipeline work is active. The pre-guard prevents new production Rayon call sites from appearing outside `core/runtime/tasks/pool.rs`, `core/runtime/tasks/parallel_for.rs`, and `parallel_frustum.rs`. This records status `pre_m2_1_rayon_render_exception_guard_static_passed_pending_render_owner`; the real `parallel_frustum.rs` migration to `parallel_for` or the compute pool still waits for a render-owner window and actual graphics cutover not executed.

`job_system_boundary` now mirrors the same Runtime 11 structure through the Python structural audit: `expected_module_count = 9`, `direct_rayon_paths = 3`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 4`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. It checks the task owner folder, `JobHandle` / `JobScheduler::schedule_after` / `parallel_for` / `JobSchedulerReport` anchors, the ECS batch dependency path, the direct-Rayon whitelist, and `runtime_11_job_system_mirror_docs_match_structure_audit_counts` without running Cargo.

## Runtime 11 JobSystem Cargo Gate

The 2026-06-13 Runtime 11 follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_11_job_system_cargo_gate_stays_visible_until_job_system_filters_pass`. This is a status guard over the JobSystem slices that are statically landed but still awaiting `tasks/ecs_schedule/worker_pool/rayon` validation and the render-owned `parallel_frustum` cutover window.

The guard keeps Runtime 11 `in_progress`, requires M1/M2/M3 rows to retain Cargo or render-owner pending language, keeps the declared `tasks`, `job`, `rayon`, `ecs_schedule`, and `worker_pool` commands visible, and mirrors the same pending state through P14, the Runtime 11 subplan row, `job_system.md`, Runtime 05 closeout, and this review.

## Runtime 14 Module Family Boundary

`module_family_boundary` now mirrors the Runtime 14 root-seat closeout through the Python structural audit. Current evidence reports `expected_family_count = 4`, `animation = 27`, `navigation = 3`, `diagnostic_log = 7`, `engine_module = 8`, `root_seat_guard_present = true`, `missing_doc_anchors = []`, `file_count_mismatches = []`, and `risks = []`. `runtime_14_module_family_mirror_docs_match_structure_audit_counts` keeps this review, Runtime 14, the runtime index, and runtime-interface convergence aligned with those structure-audit counts.

The audit checks that `animation`, `navigation`, `diagnostic_log`, and `engine_module` remain crate-root module-family seats, that their mirror docs retain the Runtime 14 judgements, and that the existing Rust guard anchors still exist. It does not promote Runtime 14 beyond static evidence; the Cargo filters remain pending.

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
- Done on 2026-06-04: `runtime_root_surface` moved into `runtime_structure_audits/runtime_root_surface.py`, reducing the main audit script from 690 lines to 643 lines while preserving root-surface evidence. Refreshed on 2026-06-14, the current evidence is 20 public modules, 3 public `pub use` locations, 80 crate-visible graphics re-exports, direct `rhi_wgpu` backend exposure, M1 gate status `migration-debt-present`, and 2 current root-surface risks.
- Done on 2026-06-04: `non_network_server_naming` moved into `runtime_structure_audits/non_network_server_naming.py`, reducing the main audit script from 643 lines to 600 lines while preserving the non-network `server` naming evidence at 179 suspect references and 20 sample locations.
- Done on 2026-06-04: `non_network_server_naming` now reports an M1 gate status of `migration-debt-present`, filters 72 `observer` substring false positives, allows 93 real server-context lines, and classifies the remaining 87 suspect references into graphics render-framework debt, editor asset/resource owner debt, and editor scene comment debt with zero unclassified locations.
- Done on 2026-06-04: `entry_static_dependencies` moved into `runtime_structure_audits/entry_static_dependencies.py`, reducing the main audit script from 600 lines to 528 lines while preserving app fan-out evidence at 4 app path dependencies, 0 optional runtime plugin path dependencies, 0 optional runtime plugin feature mentions, 1 built-in entry/runtime module crate, and no entry dependency risk.
- Done on 2026-06-04: `legacy_standalone_references` moved into `runtime_structure_audits/legacy_standalone_references.py`, reducing the main audit script from 528 lines to 476 lines while preserving stale standalone-crate architecture-doc evidence at zero counts and zero sample locations.
- Done on 2026-06-04: `runtime_scene_editor_surface` moved into `runtime_structure_audits/runtime_scene_editor_surface.py`, keeping the main audit script at 476 lines after stale unused helper cleanup while preserving M3 scene/editor boundary evidence at zero editor-named production paths, zero public editor-named locations, and zero risks.
- Done on 2026-06-04: `large_file_ownership` moved into `runtime_structure_audits/large_file_ownership.py`, reducing the main audit script from 476 lines to 420 lines while preserving large-file evidence at 10 reported top hotspots. Current 2026-06-14 owner-class counts are `editor-retained-host=11`, `editor-ui=8`, `runtime-framework-render=2`, `runtime-other=17`, and `support-hub=3`.
- Done on 2026-06-04, refreshed on 2026-06-14: `large_file_ownership` reports an M1 gate status of `migration-debt-present`, 41 hotspots above the 1000-line threshold, 5 migration-debt owner groups, decision groups for all current hotspots, and zero unclassified hotspots.
- Done on 2026-06-04: `plugin_runtime_gaps` moved into `runtime_structure_audits/plugin_runtime_gaps.py`, reducing the main audit script from 420 lines to 391 lines while preserving zero plugin runtime gaps.
- Done on 2026-06-04: `module_inventory` moved module descriptor distribution, stub descriptor usage, owner coverage, module classification, support-crate listing, workspace production-file inventory, and hotspot source data into `runtime_structure_audits/module_inventory.py`, reducing the main audit script from 391 lines to 231 lines while preserving 3 classified module crates, zero stub descriptor usage, 3 support crates, and 10 large-file hotspots.
- Done on 2026-06-04: `ecs_query_state_boundary` now owns its Markdown renderer as well as audit data, reducing the main audit script from 231 lines to 210 lines while preserving the old-file-absent, folder-backed owner module, root-line-budget, and zero oversized owner module evidence. The current 2026-06-13 follow-up accepts `query_state/stats.rs` as the Runtime 07 telemetry sidecar, so the audit now reports 8/8 owner modules and no boundary risk.
- Done on 2026-06-04: `scene_project_serialization_boundary` moved into `runtime_structure_audits/scene_project_serialization_boundary.py`, reducing the main audit script from 864 lines to 690 lines while preserving the 7-file/0-forbidden-location evidence.
- Done on 2026-06-04: the first runtime-other large-file production split reduced `zircon_runtime/src/dynamic_api/session.rs` from 1207 lines to 947 lines by extracting ABI status construction, host-request conversion, input-event conversion, and preview fallback helpers under `zircon_runtime/src/dynamic_api/session/`.
- Done on 2026-06-04: the matching dynamic API test split removed the 893-line `zircon_runtime/src/dynamic_api/tests.rs` and replaced it with folder-backed test owners. The current 2026-06-13 follow-up keeps the tree at 11 owner modules by splitting lifecycle overflow into `session_entry_points.rs`, `session_lifecycle.rs`, and `session_profiles.rs`.
- Done on 2026-06-04: the dynamic API test boundary now has both a Rust structure test and structural audit output that reject a recreated `tests.rs`, missing owner modules, missing declarations, non-navigational `mod.rs` content, and oversized owner test files.
- Done on 2026-06-04: `dynamic_api_test_boundary` itself moved out of the near-threshold audit script into `runtime_structure_audits/dynamic_api_test_boundary.py`, reducing the main audit script from 1095 lines to 992 lines while preserving the JSON and Markdown evidence.
- Done on 2026-06-04: `generated_code_boundary` and `native_plugin_public_surface` also moved into folder-backed audit owner modules, reducing the main audit script from 992 lines to 864 lines while preserving generated behavior count 13 and the then-current native root re-export count 54. The current Runtime 06 mirror has since moved this native count to 68 after native bridge-method exports entered the root surface.
- Done on 2026-06-04: `native_plugin_public_surface` now reports an M4 gate status of `migration-debt-present`. Current classification covers all 68 root re-export symbols across native ABI contract, loader/discovery, live-host runtime, behavior-report, and bridge-method debt groups, and reports zero unclassified root re-export symbols.
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

`exports.rs` remains the exported C ABI owner, while `session.rs` remains the private Rust-ABI session registry and `RuntimeDynamicSession` lifecycle/orchestration owner. This avoids creating a second public runtime API surface while reducing a runtime-other large-file hotspot below the audit threshold and keeping panic containment at the dynamic-library edge.

The matching test tree is now folder-backed:

- `tests/api_table.rs` for exported function-table and ABI version checks;
- `tests/profile_control.rs` for profiling JSON request/response behavior;
- `tests/viewport.rs` for viewport, surface, frame, present, and unbind validation;
- `tests/session_entry_points.rs` for cross-entry invalid, destroyed, and missing session handle rejection;
- `tests/session_lifecycle.rs` for create, destroy, and tick lifecycle paths;
- `tests/session_profiles.rs` for profile/source-shape guards including headless/minimal render-bridge behavior;
- `tests/host_requests.rs` for IME and gamepad host-request encoding/freeing;
- `tests/accessibility.rs` for accessibility tree/action fallback behavior;
- `tests/input_events.rs` for mouse-wheel, window-scale, and IME invalid-input rejection;
- `tests/structure.rs` for folder-backed test-tree regression checks;
- `tests/support.rs` for shared ABI fixtures and buffer free helpers.

New dynamic API assertions should land in the matching owner module, not in a recreated `tests.rs`. The audit reports this as `dynamic_api_test_boundary`; the current accepted state is `legacy_tests_file_exists = false`, 11 owner modules present, `session_lifecycle.rs = 136` lines after the split, and zero oversized owner modules over 250 lines. The audit implementation is now a dedicated owner module too, so future boundary checks should continue moving into `runtime_structure_audits/` rather than growing the main audit script.

The Runtime 10 dynamic runtime API contract is now separately mirrored as `dynamic_runtime_api_boundary`. That audit owner ties the ABI function-table inventory, session FFI wrapper table, headless/minimal lifecycle rules, app loader failure guards, UI pending gate, and Cargo-pending gate into a single static report: `expected_source_file_count = 14`, `function_table_structs = 10/10` including `ZrHostBridgeApiV1`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `ui_pending_gate_anchors = 8/8`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 7/7`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps those fields mirrored across the Runtime 10 docs.

## ECS QueryState Split

The second runtime-other large-file slice kept `zircon_runtime::scene::ecs::QueryState` as the public query state type and split only source ownership:

- `query_state/mod.rs` owns `QueryState`, access descriptors, cache rebuilds, cache counters, and cache metadata accessors;
- `query_state/cached_direct.rs` owns cached storage-location direct access for `CachedQueryData`;
- `query_state/read_only.rs` owns non-mutating query iteration, get/many/contains, cached read iteration, and combinations;
- `query_state/mutable.rs` owns mutable query access, duplicate-entity rejection, mutable many/combination iteration, and the narrow post-validation unsafe fetch;
- `query_state/helpers.rs` owns shared fixed-size collection and cached entity filtering helpers;
- `query_state/stats.rs` owns the Runtime 07 cache telemetry snapshot API;
- `query_state/system_param.rs` owns the `SystemParam` bridge into runtime systems.

This follows Bevy's query module precedent: keep state/cache, data/filter, iterator, telemetry, and system-param roles navigable instead of stacking every access family into one hot-path state file. `scene::tests::ecs_query_structure` and the structural audit's `ecs_query_state_boundary` both guard against recreating `query_state.rs`, missing owner files, behavior impl families in the root file, and owner files above the current budget. The current accepted audit state is old file absent, 8/8 owner modules present, root below the 180 non-empty-line budget, and no oversized owner modules. The audit owner now also renders its own Markdown section so `audit_runtime_structure.py` stays a short orchestration script.

## Runtime Plan Status Guard

The 2026-06-13 runtime plan follow-up added `runtime_absorption::plan_status` after Runtime 01-14 planning expanded the status surface beyond code-only root checks. The guard treats plan metadata as an executable contract: subplan frontmatter, the runtime index row, index-to-file coverage, problem-to-subplan coverage, dependency-to-subplan coverage, validation blockers, status/evidence tables, started-work evidence rows, Runtime 05 closeout state, and `last_refined` dates must stay synchronized.

Static verification for this guard passed `rustfmt --edition 2021 --check` over `plan_status.rs`, `asset_worker_policy.rs`, and `runtime_absorption/mod.rs`; conflict-marker and trailing-whitespace scans over the touched runtime guard/docs/session files; exact PowerShell mirror scans for index/frontmatter status, pending completed blockers, status/evidence anchors, index-to-file coverage, problem-to-subplan coverage, dependency-to-subplan coverage, and `last_refined` coverage; and scoped `git diff --check` with only LF-to-CRLF warnings. No Cargo or standalone rustc test pass is claimed for this slice because other runtime/plugin compile lanes were active.

`runtime_plan_status_boundary` now mirrors the same plan-status and Runtime 05 closeout governance through the Python structural audit. Current evidence reports plan-status support files 6/6, runtime subplans 14/14, index subplan rows 14/14, problem rows 17/17, known backlog rows 7/7, status counts `in_progress=14`, core guard anchors 13/13, pending Cargo gate anchors 15/15, doc anchors 8/8, Runtime 05 closeout status `in_progress`, and `risks = []`. Runtime 05 remains gated by `pending_full_scene_cargo`; no completed claim is made before `cargo test -p zircon_runtime --lib scene:: --locked`.

## Runtime Root Surface Guard

The 2026-06-13 Runtime 02 follow-up added `runtime_absorption::root_surface` plus `docs/zircon_runtime/core/root_surface.md`. This is a pre-M3 guard rather than the alias cutover itself: it records the current public root as 20 namespace modules and three curated `pub use` sites, rejects new flattened subsystem root re-exports, and keeps the graphics alias block explicitly crate-private while render-owned files are unstable.

The M3.2 type alias debt slice is also guarded now. `SceneRenderer`, `GraphicsError`, `WgpuRenderFramework`, `ViewportFrame`, `HybridGiRuntimeProvider`, `VirtualGeometryRuntimeProvider`, `SolariRuntimeProvider`, and `RendererFeatureReferenceListKind` remain crate-private debt only; the guard rejects promoting them to a public root `graphics` export. Current status is `pre_m3_type_alias_guard_static_passed_pending_render_owner`.

The real M3 work remains the hard removal of the `pub(crate) use graphics::...` alias block and its call-site migrations. That step still requires a render owner window because the call sites live under graphics/render paths. The guard prevents root-surface regression while that window is unavailable.

## Runtime 02 Core/Root/Generated Gate

The 2026-06-13 Runtime 02 status follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation`. This guard records that Runtime 02 has static evidence for core spine migration, root surface shape, and generated-code boundaries, while the full core/root/generated/export_build_plan/app/editor/plugin validation lane remains pending.

The guard keeps Runtime 02 `in_progress`, requires the M2 test-stage row, M3 root/type alias rows, and M4 generated row to keep Cargo or render-owner pending language, keeps the declared runtime/app/editor/plugin/generated/export validation commands visible, and mirrors the same pending state through P2, P8, the Runtime 02 subplan row, `root_surface.md`, `generated-code-boundary.md`, Runtime 05 closeout, and this review.

`core_spine_root_generated_boundary` now mirrors the Runtime 02 core/root/generated state through the Python structural audit. Current evidence reports core root entries 6/6, core public modules 5/5, retired core root entries 0, runtime root public modules 20/20, public `pub use` sites 3/3, crate-visible graphics alias debt 80/80, root-surface M1 gate `migration-debt-present`, generated export templates 9/9, generated behavior locations 6/6, generated allowed adapters 6/6, generated migration debt 0/0, generated-code M1 gate `classified-and-clear`, root_entries guard tests 12, root_surface guard tests 6/6, generated-code guard tests 7/7, and `risks = []`. `root_surface_interface_convergence_mirror_uses_current_audit_counts` keeps `runtime-interface-convergence.md` aligned with the same 20/80 root-surface evidence. This is static structure evidence only; the full `core/root/generated/export_build_plan/app/editor/plugin` Cargo lane and the M3 render-owner alias cutover remain pending.

## Runtime 07 Hotspot Inventory Guard

The 2026-06-13 Runtime 07 follow-up added `runtime_absorption::performance_hotspots` plus `docs/zircon_runtime/performance/hotspot_inventory.md`. This is a measurement-boundary guard, not an optimization claim: M1.3 now records a scaffolded hotspot inventory and rejects M2 entries that do not have named counter evidence. Current counted candidates are extract full rebuild samples, QueryState cache telemetry, change-detection scan telemetry, and asset-worker diagnostics. The old RenderDoc evidence for 230 draws, 231 pre-draw `vkCmdCopyBuffer` calls, and 31 render passes remains a render-plan diversion rather than a Runtime 07 implementation target.

Runtime 07 M0.3 also pins the stage-level span that makes the broad frame update phase measurable inside ECS scheduling. `SceneScheduleRunner::run_stage(...)` owns `runtime_frame_schedule_stage.<SystemStage>` through `profile_dynamic_scope!("runtime", "frame", ...)`, which preserves existing sorted-step iteration, deferred flush, hook execution, and final consistency sweep behavior while giving profiling traces a stage-level span boundary.

## Runtime 07 Performance Hotpath Guard

The 2026-06-13 Runtime 07 closeout follow-up added `runtime_absorption::plan_status::cargo_gates::runtime_07_performance_hotpath_cargo_gate_stays_visible_until_performance_validation`. This is a status guard over performance slices that have static counter/span/inventory evidence but still need extract/ecs_query/performance profiling/FPS gates before Runtime 07 can be promoted.

The guard keeps Runtime 07 `in_progress`, requires M0.3/M1.1/M1.2/M1.3 rows to retain Cargo/profiling/FPS pending language, keeps the declared `vampire_project_session_reports_runtime_fps_and_render_work`, `cargo check`, `extract`, and `ecs_query` commands visible, and mirrors the same pending state through P5, the Runtime 07 subplan row, `hotspot_inventory.md`, dynamic-session frame diagnostics, ECS stage-span docs, and this review.

The same follow-up added `runtime_structure_audits/performance_hotpath_boundary.py` and wired it into the Python structural audit. The 2026-06-14 refinement makes the Runtime 07 mirror consume `large_file_ownership_gate` as an owner-budgeted optimization gate before any M2 performance work. The current static mirror reports source files 10/10, guard/test files 5/5, frame span anchors 9/9, QueryState telemetry anchors 13/13, change-detection telemetry anchors 9/9, extract telemetry anchors 10/10, asset-worker candidate telemetry anchors 5/5, hotspot guard anchors 16/16, Runtime 07 counter assertion anchors 12/12, doc anchors 16/16, pending Cargo/profiling/FPS gate anchors 5/5, stale top3 placeholder false, large-file owner gate `migration-debt-present`, threshold 1000, hotspots 41, debt groups 5, owner classes 5, unclassified hotspots 0, and `risks = []`. `runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps that summary synchronized with `large-file-ownership-m1.md`, Runtime 07, the runtime index, `hotspot_inventory.md`, and the interface-convergence mirror while rejecting stale 33-hotspot/removed Hub `app/` anchors. This is still static structure evidence; extract/ecs_query/profiling/FPS validation remains pending.
