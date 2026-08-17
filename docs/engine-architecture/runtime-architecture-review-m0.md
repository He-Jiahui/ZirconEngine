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
  - zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs
  - zircon_runtime/src/builtin/runtime_modules/ids.rs
  - zircon_runtime/src/builtin/runtime_modules/load_report.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/builtin/runtime_modules/plugin_modules.rs
  - docs/zircon_runtime/builtin/runtime_modules.md
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ui_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/service_registry_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/tech_stack.rs
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
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - docs/zircon_runtime/scene/world/project_io.md
  - zircon_runtime/src/scene/dynamic_scene/document/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/entity/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/value/mod.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
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
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
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
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/__init__.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
implementation_files:
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-root-surface-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - .codex/sessions/archive/20260604-1232-runtime-architecture-review.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/__init__.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_validation_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_kernel_data_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_surface_lifecycle_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/performance_hotpath_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/script_binding_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/tech_stack_markdown.py
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
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
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
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
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/compatibility_shells.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ui_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/ecs_kernel_data.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/job_system.rs
  - zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_binding.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - docs/zircon_runtime/ui/architecture.md
  - docs/zircon_runtime/core/root_surface.md
  - docs/zircon_runtime/core/job_system.md
plan_sources:
  - user: 2026-07-13 书面设计通过，批准 Runtime02 注册服务 CoreWeak 拆分设计并开始实施
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-13-service-corehandle-retention-cycle.md
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/service_registry_lifecycle.rs
  - zircon_runtime/src/tests/runtime_absorption/service_registry_ownership.rs::registry_owned_services_store_only_weak_runtime_back_references
  - zircon_editor/src/tests/host/manager/runtime_lifecycle.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - root_surface_audit M1 gate status and module decision group checks
  - generated_code_boundary M1 gate status, explicit count fields, behavior decision group, migration debt, and unclassified behavior checks
  - native_plugin_public_surface M4 gate status, explicit count fields, symbol decision group, migration debt, unclassified symbol checks, and native_plugin_public_surface_markdown renderer ownership
  - plugin_surface_lifecycle_boundary Runtime 06 source/doc/status/Cargo-pending mirror, app NativePlugin call-site count, V3-only native ABI hard-cutover, hot reload failure injection, unknown ABI rejection, public-surface debt checks, and plugin_surface_lifecycle_markdown renderer ownership
  - non_network_server_references M1 gate status, explicit count fields, classification count, migration debt, unclassified reference checks, and non_network_server_naming_markdown renderer ownership
  - runtime_naming_boundary editor/legacy gate status, classification counts, migration debt, and unclassified reference checks, with Markdown rendering owned by runtime_naming_markdown
  - hard_cutover_migration_smells gate status, explicit count fields, classification count, migration debt, allowed bridge count, unclassified reference checks, and hard_cutover_migration_smells_markdown renderer ownership
  - large_file_ownership_gate M1 gate status, explicit count fields, classification count, migration debt, and unclassified hotspot checks
  - Select-String reference declaration evidence over Bevy, Fyrox, and Unreal source files listed in docs/engine-architecture/runtime-reference-engine-evidence.md
  - git diff --check -- docs/engine-architecture/runtime-reference-engine-evidence.md docs/engine-architecture/runtime-architecture-review-m0.md docs/engine-architecture/runtime-interface-convergence.md .codex/sessions/archive/20260604-1232-runtime-architecture-review.md
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
- python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_api_test_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/entry_static_dependencies.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ecs_query_state_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/legacy_standalone_references.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/large_file_ownership.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_inventory_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/native_plugin_public_surface_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/plugin_runtime_gaps.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_api_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_naming_boundary.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_scene_editor_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
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
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/shared.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/headless_profiles.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/event_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/test_owner_split.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ffi_panic_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/ui_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/rayon_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/resource_foundation.rs
  - zircon_runtime/src/tests/runtime_absorption/script_absorption.rs
  - zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_08_owner_tree.rs
  - zircon_runtime/src/dynamic_api/tests/structure.rs
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/runtime_api.md
  - zircon_runtime/src/scene/tests/ecs_query_structure.rs
  - docs/zircon_runtime/scene/ecs/query_state.md
doc_type: milestone-detail
---

# Runtime Architecture Review M0 Baseline

Current-source UI architecture mirror 2026-08-14: `ui_architecture_boundary` reports `expected_source_file_count = 52`, `expected_ui_entry_count = 20`, `expected_surface_entry_count = 26`, `legacy_full_hits = 70`, `expected_legacy_full_hits = 70`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This current snapshot supersedes older dated counts without rewriting their history.

Runtime 08 current hard-cut sync (2026-08-10): `ecs_kernel_data_boundary` now owns `expected_source_file_count = 75`; the inventory removes `ArchetypeMove` and `TableComponentStorage`, adds the six-file archetype-table owner plus `typed_api/{component_row,projection_rebuild}.rs`, and keeps `expected_test_file_count = 10`. This supersedes earlier current-count paragraphs while preserving their dated historical evidence.

## Runtime Receipt Hard Cut (2026-08-03)

- Numbered plan documents and coordinator/Python tooling own plan lifecycle and validation receipts.
- Runtime Rust tests retain production architecture, owner, module-budget, and banned-API guards only.
- Receipt-only Rust trees, mirror tables, archive fallbacks, and their audit renderer are retired without compatibility exports.

2026-07-28 Runtime11 JobSystem mirror: `timer.rs` owns bounded process-level deadline dispatch, and `job_system_boundary` reports `expected_module_count = 10`; asset workers consume that owner instead of creating lifecycle-maintenance threads.


Runtime 13 current child-owner sync (2026-07-10): `script_binding_boundary` reports `expected_source_file_count = 19`, `expected_test_file_count = 3`, `expected_guard_file_count = 9`, `missing_guard_files = []`, `fixed_host_module_count = 6`, `fixed_host_function_count = 52`, `type_descriptor_count = 2`, `builtin_callback_count = 11`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 11`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. The nine guard owners include the two route parents plus ledger/capability/ECS-facade, gameplay-host/mirror, despawn behavior, and Runtime 13 Cargo children. `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, function ledger, M0 review, and interface-convergence mirror aligned; script package gates remain pending.

Runtime 12 current child-owner sync (2026-07-10): `input_stack_boundary` reports `expected_runtime_module_count = 12`, `expected_framework_module_count = 20`, `expected_test_module_count = 7`, `expected_guard_file_count = 6`, `missing_guard_files = []`, `public_surface_anchors = 26/26`, `runtime_12_guard_anchors = 5/5`, `missing_gamepad_abi_anchors = []`, `missing_cursor_host_request_anchors = []`, `missing_doc_anchors = []`, `missing_test_anchors = []`, `behavior_test_anchor_count = 15`, `missing_behavior_test_anchors = []`, `missing_cargo_gate_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. Current status anchors are `Frame Input Contract`, `input_frame_contract_static_passed_cargo_pending`, `arbitration_judgement_documented_static_passed`, `action_contract_static_passed_cargo_pending`, `action_evaluator_static_passed_cargo_pending`, `action_context_static_passed_cargo_pending`, `action_axis_value_static_passed_cargo_deferred`, `action_config_static_passed_cargo_deferred`, `action_manager_registration_static_passed_cargo_deferred`, `action_axis_consumption_static_passed_cargo_deferred`, `input_recording_replay_static_passed_cargo_deferred`, `cursor_host_request_static_passed_cargo_deferred`, `gamepad_bridge_static_passed_cargo_pending`, and `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation`. Pending command anchors remain `cargo test -p zircon_runtime --lib input --locked -- --nocapture`, `cargo test -p zircon_runtime --lib action_map --locked -- --nocapture`, `cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture`, and `cargo test -p zircon_app --locked`. `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, input module doc, M0 review, and interface-convergence mirror aligned; production input behavior is unchanged.

Runtime 11 current guard-owner sync (2026-07-10): `job_system_boundary` now reports `expected_guard_file_count = 2`, `missing_guard_files = []`, `mirror_docs_guard_present = true`, and `risks = []` by reading both the route parent `job_system.rs` and the real folder-backed `job_system/mirror_docs.rs` owner. `runtime_11_job_system_mirror_docs_match_structure_audit_counts` remains the aggregate mirror guard. JobSystem production behavior is unchanged; the named `tasks/ecs_schedule/worker_pool/rayon` filters retain historical passing evidence, while the broader full-lib final gate remains pending.

Runtime 08 current child-owner sync (2026-07-10): `ecs_kernel_data_boundary` reports `expected_source_file_count = 69`, `expected_test_file_count = 10`, `archetype_anchors = 15/15`, `storage_anchors = 9/9`, `component_storage_private_reexport_anchors = 9/9`, `component_identity_anchors = 18/18`, `entity_lifecycle_anchors = 10/10`, `observer_anchors = 8/8`, `deferred_command_anchors = 11/11`, `event_message_anchors = 12/12`, `resource_identity_anchors = 12/12`, `change_tick_anchors = 6/6`, `runtime_08_guard_anchors = 21/21`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `doc_anchors = 13/13`, `pending_cargo_gate_anchors = 6/6`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts` keeps the plan, runtime index, ECS module doc, M0 review, and interface-convergence mirror aligned. The 10 test owners explicitly include `ecs_kernel_data/inventory.rs` and `cargo_gates/early/runtime_08.rs`; this supersedes the historical 8-route-owner mirror and does not close the pending `entity/observer/command/messages/change_tick/ecs` Cargo gates.

Runtime 04 current owner sync (2026-07-10): `asset_pipeline_boundary` reports `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The 22-owner count includes the current folder-backed child guards and supersedes the earlier 11-owner historical mirror; broader asset Cargo gates remain pending.

## Scope

This is the M0 evidence and decision record for the runtime architecture review. It fixes the current review order before broad code movement starts. The optimization target is a runtime that is developer-friendly, compact at public boundaries, hard-cutover oriented, and performance-aware without preserving old compatibility behavior.

Reference-engine direction for this review:

- Unreal-style module/plugin ownership: integration units are declared modules and plugins, not scattered launch-time match arms.
- Bevy-style app composition: application entry should compose profile/plugin graphs and should not statically know every optional runtime plugin implementation.
- Fyrox-style editor/runtime split: editor views project runtime state through explicit DTOs; runtime scene and runtime module code should not expose editor authoring concepts as core owners.

The source-backed reference matrix is recorded in `docs/engine-architecture/runtime-reference-engine-evidence.md`. Use that matrix as the review gate before M1 root-surface cuts, M2 assembly changes, M3 scene/editor boundary work, M4 plugin lifecycle convergence, M5 performance work, and M6 graphics/RHI public-surface cleanup. The concrete M1 root-surface gate is recorded in `docs/engine-architecture/runtime-root-surface-m1.md`; the non-network `server` naming gate is recorded in `docs/engine-architecture/non-network-server-naming-m1.md`; the hard-cutover migration-smell gate is recorded in `docs/engine-architecture/hard-cutover-migration-smells-m1.md`.

## Runtime 01 Tech Stack Guard


The guard records Runtime 01 as `completed` after current-source `tech_stack`, `extensions`, `text_shaper`, `export_build_plan`, and Physics plugin Cargo gates closed. It keeps the declared commands and completion evidence visible through the Runtime 01 numbered outputs, subplan row, `runtime-tech-stack.md`, `text.md`, `physics-plugin-options.md`, the editor-only backlog, and this review. The protected anchors include the prerelease version pins, ZrVM path dependency gate, text stack matrix, complex-text `UiTextShaper` route, fontdue/editor-only backlog, plugin-owned feature-gated Jolt ruling, ZIP archive decision, and rfd/arboard exclusion.

The Runtime 01 dependency audit is split across `runtime_structure_audits/tech_stack_source_inventory.py`, `tech_stack_anchor_inventory.py`, `tech_stack_boundary.py`, and `tech_stack_markdown.py`. The current static mirror reports `expected_manifest_count = 5`, `expected_non_dependency_count = 4`, `kira_dependency_owners = [zircon_plugins/sound/runtime/Cargo.toml]`, `kira_owner_dependency_declaration_count = 1`, `kira_owner_dependency_versions = [0.12.2]`, `kira_owner_version_pinned = true`, `kira_owner_violations = []`, `manifest_scan_errors = []`, `zip_dependency_count = 1`, `expected_zip_dependency_count = 1`, `zip_dependency_violations = []`, `tech_stack_guard_count = 12`, `behavior_test_anchor_count = 6`, `missing_behavior_test_anchors = []`, `editor_only_candidate_count = 3`, `jolt_feature_slot_count = 2`, `declared_removed_dependencies = []`, `rapier_or_avian_dependencies = []`, `mirror_docs_guard_present = true`, and `risks = []`. The Kira contract permits only the Sound runtime manifest's exact 0.12.2 pin and rejects package aliases in every other current product manifest. The Jolt contract covers feature-off unavailability/no-fallback and feature-on ready/native stepping; `joltc-sys` remains optional and Physics-plugin-owned while `zircon_runtime` keeps only profile vocabulary. `runtime_01_tech_stack_mirror_docs_match_structure_audit_counts` keeps Runtime 01, this review, runtime-interface convergence, and `runtime-tech-stack.md` aligned. Current static evidence is closed; the historical five executable gates do not close the reopened Kira current-source managed Rust gate or fixed return.

`runtime_absorption/tech_stack.rs` is the Rust-side guard module for these Runtime 01 tech-stack static anchors and mirror-doc counts.

## Runtime 03 Schedule Frame-Loop Completion Guard


The guard records current Runtime results of `ecs_schedule` 77/77, `tests::time::` 4/4, `session` 165 passed / 0 failed / 10 ignored, and `schedule_parallel` 15/15. It also records the full `cargo test -p zircon_app --locked` result: main tests 135 passed / 0 failed / 1 ignored, runtime preview 0/0, PBR viewer 15/15, and doc tests 0/0. The declared `ecs_schedule`, `session`, `zircon_app`, `fixed_update`, `tests::time::`, and `schedule_parallel` commands remain visible, while P3 and the Runtime 03 subplan row now mirror completion. The protected anchors include `schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`, `session_ui_extract_remains_documented_dynamic_session_side_path`, `world_driver_consumes_runtime_time_advance_without_advancing_clocks_again`, `level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`, `level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`, `level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance`, `fixed_step_plan_reports_overstep_fraction_in_unit_range`, `ScheduleParallelExecutionReport`, `schedule_parallel_execution_report_records_diagnostic_counts`, `representative_schedule_produces_multi_system_parallel_batches`, and `parallel_and_serial_execution_reach_identical_world_state`.

The same 2026-06-13 follow-up added `runtime_structure_audits/schedule_frame_loop_boundary.py` and wired it into the Python structural audit. The 2026-06-21 split moves source/guard file inventory, stage/fixed-loop counts, and dynamic-session tick-count scans into `runtime_structure_audits/schedule_frame_loop_source_inventory.py`, moves SystemStage, RuntimeTimeAdvance, FixedStepPlan, UI extract, stage ordering, schedule runner, parallel executor, behavior-test, mirror-doc, and Cargo gate anchors into `runtime_structure_audits/schedule_frame_loop_anchor_inventory.py`, and moves Markdown rendering into `runtime_structure_audits/schedule_frame_loop_markdown.py`. `schedule_frame_loop_boundary.py` now owns only the audit reader, missing-anchor checker, and risk classifier at 368 lines; the Markdown owner is 146 lines. The current static mirror reports source files 19/19, guard/test files 11/11, `SystemStage` count and variants 9/9, fixed-loop stages 3/3, dynamic-session `.tick_time(...)` calls 1/1, Runtime 03 guard anchors 14/14, `behavior_test_anchor_count = 13`, `missing_behavior_test_anchors = []`, `doc_anchors = 10/10`, `mirror_docs_guard_present = true`, frame schedule module-doc anchors 3/3, no `WorldDriver` second `advance_time_by(...)` references, no dynamic-session raw-delta level tick references, and `risks = []`. `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` keeps Runtime 03, `docs/zircon_runtime/core/frame_schedule.md`, the runtime index, this review, and runtime-interface convergence aligned with those structure-audit counts; current static audit regression is 3/3 and the independent schedule/frame-loop guard is 2/2. Together with the current Runtime/App results above, the completion evidence is no longer static-only.

## Runtime 04 Asset Pipeline Guard


The guard keeps Runtime 04 `in_progress`, requires the M1/M2 rows to retain Cargo-pending language, keeps the declared `load_state`, `resource`, `asset::`, `worker_pool`, and `watch` validation commands visible, and mirrors the same pending state through P7, the Runtime 04 subplan row, asset facade/worker docs, and this review. The already recorded focused evidence remains narrower: `artifact_store_roundtrips_scene_assets_with` passed 4/4 and `watcher` passed 7/7, while broader asset validation is still pending an owner-safe Cargo lane.

The current `asset_pipeline_boundary` static mirror reports `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps those structural facts synchronized; this is static evidence only and does not replace the pending managed Cargo gates.


## Runtime 06 Plugin Surface Lifecycle Gate


The guard keeps Runtime 06 `in_progress`, requires M1 status rows to retain runtime Cargo-pending language, records M1.2 fallback failure tests as `code_static_passed_real_backend_pending`, records M2.1, M2.2, M3.1, and M3.2 as `code_static_passed_cargo_pending`, keeps the declared `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` validation commands visible, and mirrors the same pending state through P4, the Runtime 06 subplan row, `native-plugin-boundary.md`, `runtime-interface-convergence.md`, Runtime 05 closeout, and this review. It also binds the native plugin public-surface evidence to `native_plugin_public_surface.m4_gate_status=classified-and-clear`, `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, native loader test namespace isolation, fallback lifecycle failure tests 4/4, `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`, unknown ABI rejection, and hot reload failure injection.

The 2026-06-21 Runtime 06 structural follow-up keeps `plugin_surface_lifecycle_boundary` aligned with the current Python audit and moves the renderer into `plugin_surface_lifecycle_markdown.py`. Current mirror evidence reports source 14/14, docs 5/5, `expected_source_file_count = 14`, `expected_doc_file_count = 5`, frontmatter `in_progress`, `last_refined = 2026-07-01`, `plugin_surface_lifecycle_boundary.py = 450`, `plugin_surface_lifecycle_markdown.py = 144`, `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, native root re-export 0/0, native namespace re-export 64/64, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 0/0, `zircon_plugins` V1/V2 usage files 0/0, export_build_plan V1/V2 usage 0/0, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, fallback lifecycle failure tests 4/4, unknown ABI rejection, hot reload failure injection, `mirror_docs_guard_present = true`, `risks = []`, and full source/doc/validation anchors. `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, and `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed` keep Runtime 06, the runtime index, native-plugin-boundary, this review, and runtime-interface convergence aligned with those counts. This is static structure evidence only; Runtime 06 still waits on the declared script VM, native plugin, app, and plugins Cargo/native validation lane.

The 2026-06-21 native plugin public-surface follow-up splits `render_native_plugin_public_surface_markdown(...)` into `native_plugin_public_surface_markdown.py`; `native_plugin_public_surface.py` remains the 400-line scan/classification/M4 gate owner and the Markdown owner is 63 lines. Direct evidence remains `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, `symbol_decision_group_count = 5`, migration debt 0, unclassified root/namespace symbol counts 0/0, root/native public re-export locations 0/1, M4 gate `classified-and-clear`, risks 0, and rendered output 12 lines. This is a renderer-owner split only; it does not close Runtime 06 validation gates.

The 2026-07-01 Runtime 06 native hot-update/replay public-surface audit sync classifies `NativePluginRuntimeDeltaHotUpdateReport`, `NativePluginRuntimeDeltaHotUpdateRequest`, `NativePluginRuntimeRegistrationReplayReport`, and `NativePluginRuntimeRegistrationSystemReplay` in the existing native live-host runtime group. Current evidence: `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, native root re-export 0/0, native namespace re-export 64/64, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, `last_refined = 2026-07-01`, `mirror_docs_guard_present = true`, `risks = []`, and standalone plugin_surface_lifecycle 3/3. This keeps Runtime 06 in static evidence sync only; `script::vm/vampire_project_session/plugin/native_plugin/app/plugins` remains pending.

## Runtime 09 UI Architecture Guard

The 2026-06-13 Runtime 09 follow-up added `runtime_absorption::ui_architecture` plus `docs/zircon_runtime/ui/architecture.md`. This is an M0 guard, not a UI production cutover: it locks the module boundary map, the baseline source-scan counts, and the v2 runtime/interface contract shape without editing `zircon_runtime::ui` production files.

The guard has nineteen direct ui_architecture anchors: `runtime_09_ui_architecture_doc_records_current_boundaries`, `runtime_09_ui_architecture_baselines_match_current_source_scan`, `runtime_09_v2_verdict_matches_runtime_and_interface_modules`, `runtime_09_ui_input_events_route_through_single_dispatch_authority`, `runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt`, `runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt`, `runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt`, `runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt`, `runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt`, `runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt`, `runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt`, `runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt`, `runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt`, `runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt`, `runtime_09_taffy_layout_pass_order_uses_bridge_authority`, `runtime_09_virtualization_scroll_boundary_records_invalidation_authority`, `runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority`, `runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts`, and `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation`. It records the current Runtime 09 state after M1.1 route authority, the M1.2 navigation reply, pointer reply, pointer capture fallback, table row-label fallback, template component-name fallback, property visibility flag, responsive MUI visibility flag, accessibility open-state fallback rename, layout engine backend name cutover, and surface default interaction fallback rename, M2.1 Taffy bridge/pass-order authority, M2.2 virtualization/scroll boundary, and M3.1 template pipeline: normalized `UiInputEvent` dispatch writes `route_authority=runtime_09_m1_1_ui_input_route_authority`, direct pointer/navigation helpers are owner-verdicted through `runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers`, pointer local route results use `routed_result`, pointer capture ownership uses indexed `has_pointer_capture_for_owner`, table row-label fallback splitting uses `split_row_label_table_text`, template interaction fallback ownership uses `component_name_interaction_fallback`, property visibility transition and responsive MUI visibility DTO use `state_visible_flag`, accessibility open-state fallback properties use `fallback_properties`, layout reports use `UiLayoutEngineBackend::Zircon`, `UiLayoutEngineCapability::zircon()`, and `zircon_selected_count`, surface default open-state fallback uses `default_open_boolean_value(...)` with `fallback_properties`, `compute_taffy_child_frames(...)` owns Taffy tree build/compute, `UI_LAYOUT_PASS_ORDER` is consumed by full and incremental layout, `UiScrollVirtualizationPlan` / `plan_scrollable_virtual_window(...)` owns scroll offset, viewport/content extent, and visible-range invalidation, full-tree UI legacy hits are 54, production legacy hits are 0 across 0 files, production taffy hits are 175 across 10 files, and remaining UI behavior validation stays gated on an editor UI owner window plus later Cargo validation.

`ui_architecture_boundary` now mirrors the same Runtime 09 shape through the Python structural audit. Current evidence reports `expected_source_file_count = 52`, `expected_ui_entry_count = 18`, `expected_surface_entry_count = 20`, `legacy_full_hits = 54`, `expected_legacy_full_hits = 54`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts` keeps this review aligned with Runtime 09, the runtime index, UI architecture doc, runtime-interface convergence, and the Python audit. The M1.1 code change is `runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending` in `surface/input/{dispatch,route_authority}.rs`; the M1.2 code changes are `runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending`, `runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending`, `runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending`, `runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending`, `runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending` with `component_name_interaction_fallback`, `runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending` with `state_visible_flag`, `runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending` with `state_visible_flag`, and `runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending` with `fallback_properties`; `runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending` hard-cuts the layout engine contract to `UiLayoutEngineBackend::Zircon`, `UiLayoutEngineCapability::zircon()`, and `zircon_selected_count`; `runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending` records `default_open_boolean_value(...)` and `fallback_properties`; the M2.1 code change is `runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending` with `runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter`; the M2.2 code change is `runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending` with `virtualized_list_only_materializes_visible_window`, `scroll_offset_invalidates_virtualization_window`, and `non_virtualized_scroll_offset_keeps_full_window_dirty_domain`; the M3.1 code change is `runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending` with `UiTemplateRuntimePipeline`, `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`, `UiTemplateRuntimePipelineError`, and `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source`.

After Runtime 15 runtime UI dead-code support split, the current UI entry-map mirror records `expected_ui_entry_count = 18`: old production `ui/runtime_ui/` is gone, production `ui/public_runtime_frame.rs` is present, and runtime fixture/manager support is test-only under `ui/tests/runtime_ui_support`.

The 2026-07-01 Runtime 09 UI entry-map audit sync refreshes the current mirror to `expected_source_file_count = 52`, `expected_ui_entry_count = 19`, `expected_surface_entry_count = 21`, `legacy_full_hits = 54`, `expected_legacy_full_hits = 54`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The new current entries are `ui/platform_input/` and `ui/surface/property_mutation/`; `has_pointer_capture_or_unindexed_fallback_for_owner` remains the pointer-capture mirror doc anchor. This is a static audit/doc/status sync only.

The 2026-06-21 Runtime 09 renderer split moves `render_ui_architecture_boundary_markdown(...)` into `ui_architecture_markdown.py`; `ui_architecture_boundary.py` remains the 541-line audit/risk owner, the Markdown owner is 110 lines, and direct audit still reports source files 52/52, guard anchors 19/19, doc anchors 61/61, `mirror_docs_guard_present = true`, and `risks = []`.

## Runtime 09 UI Cargo Gate


## Runtime 10 Dynamic API Guards

The 2026-06-13 Runtime 10 follow-up added `runtime_absorption::dynamic_api_session::runtime_10_headless_profiles_keep_render_bridge_optional_and_noop_surfaces`. This is a lifecycle boundary guard over the existing dynamic-session code, not a new ABI or render feature.

The guard keeps `RuntimeDynamicSession.render_bridge` optional, limits `uses_render_bridge()` to rendered `runtime`/`editor`/`dev` profiles, requires `minimal` and `headless` profiles to skip `RuntimeRenderBridge` bootstrap, and preserves no-render fallbacks: capture returns an empty encoded frame and surface bind/unbind/present return `Ok(())` without touching WGPU.

The same Runtime 10 guard module now includes `runtime_10_ffi_panic_boundary_keeps_exports_as_only_c_abi_edge`. It locks `dynamic_api::exports` as the final C ABI owner for `zircon_runtime_get_api_v5` and all 22 advertised V5 session entries. Function-table entries must point at `_ffi` wrappers, the wrappers must translate unexpected unwinds to `ZrStatusCode::Panic`, table-acquisition unwinds must return a null pointer, and private `dynamic_api::session` owner functions must stay Rust-ABI `unsafe fn`.

`dynamic_runtime_api_boundary` now mirrors the Runtime 10 dynamic runtime API boundary through the Python structural audit, with Markdown rendering split into `dynamic_runtime_api_markdown.py`. Current evidence reports `dynamic_runtime_api_boundary.py` at 330 audit/risk lines, the Markdown owner at 65 lines, `expected_source_file_count = 35`, `function_table_structs = 10/10`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `runtime_diagnostics_anchors = 15/15`, `missing_runtime_diagnostics_anchors = []`, `scene_asset_reload_diagnostic_path_anchors = 21/21`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_contract_duplicate_public_types = 0`, `ui_v2_contract_sync_anchors = 9/9`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, and `risks = []`. The current Runtime 10 host-request payload inventory pins interface DTOs, runtime conversion, dynamic API host-request tests, and app-side host routing for IME, gamepad rumble, and cursor requests. Runtime diagnostics use `ProfileControlCommand::RuntimeDiagnosticsSnapshot` on the existing `profile_control` JSON ABI and add no new `ZrRuntimeApiV1` function pointer. `runtime_10_m2_1_ui_contract_duplicate_public_types_removed_static_passed_cargo_pending` records that runtime-local `UiBindingCodec` and `UiAssetSchemaVersionPolicy` duplicate definitions were removed while the interface owns both contract types. `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending` records that `v2-replacement-mainline`, interface `ui/v2` DTO ownership, runtime `ui/v2` consumption, and interface-owned `UiComponentApiVersion` validation remain synchronized. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps the module doc, Runtime 10, the runtime index, this review, runtime-interface convergence, and the cdylib loader doc aligned with those counts. It does not claim the pending `dynamic_api`, full app loader, or UI contract Cargo gates.

The diagnostics inventory split is recorded as `runtime_10_dynamic_api_diagnostics_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_diagnostics_inventory.py` owns the Runtime 10 diagnostic anchor tuples, and `dynamic_runtime_api_boundary` now reports `scene_asset_reload_diagnostic_path_anchors = 21/21` with `missing_scene_asset_reload_diagnostic_path_anchors = []` alongside `runtime_diagnostics_anchors = 15/15`.

The host-request inventory split is recorded as `runtime_10_host_request_payload_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_host_request_inventory.py` owns the 38 host-request payload anchor tuples, while `dynamic_runtime_api_boundary` remains the audit entry and still reports `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, and `risks = []`.

The UI contract inventory split is recorded as `runtime_10_ui_contract_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_ui_contract_inventory.py` owns the Runtime 10 UI pending-gate, single-source contract, and v2 sync anchor tuples, while `dynamic_runtime_api_boundary` still reports `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_v2_contract_sync_anchors = 9/9`, and `risks = []`.

The validation inventory split is recorded as `runtime_10_dynamic_api_validation_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_validation_inventory.py` owns the Runtime 10 behavior-test, pending Cargo gate, doc-anchor, and mirror-doc guard tuples, while `dynamic_runtime_api_boundary` still reports `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `missing_doc_anchors = []`, and `risks = []`.

The session lifecycle inventory split is recorded as `runtime_10_session_lifecycle_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_session_lifecycle_inventory.py` owns the Runtime 10 headless/minimal lifecycle anchor tuples, while `dynamic_runtime_api_boundary` still reports `headless_lifecycle_anchors = 12/12`, `missing_headless_lifecycle_anchors = []`, and `risks = []`.

The failure boundary inventory split is recorded as `runtime_10_failure_boundary_inventory_split_static_passed_cargo_deferred_tests_deferred`: `dynamic_runtime_api_failure_inventory.py` owns the Runtime 10 FFI panic and loader failure anchor tuples, while `dynamic_runtime_api_boundary` still reports `ffi_panic_anchors = 9/9`, `missing_ffi_panic_anchors = []`, `loader_failure_anchors = 10/10`, `missing_loader_failure_anchors = []`, and `risks = []`.

The ABI source inventory split is recorded as `runtime_10_dynamic_api_abi_inventory_split_static_passed_cargo_timeout_no_result_tests_deferred`: `dynamic_runtime_api_abi_inventory.py` owns the Runtime 10 source owner, function-table shape, and session operation tuples, while `dynamic_runtime_api_boundary` still reports source files 35/35, function tables 10/10, runtime session wrappers 11/11, no field-count mismatch, no direct table-entry bypass, `session_owner_extern_c_present = false`, and `risks = []`. The focused package check for `cargo test -p zircon_runtime --lib dynamic_api_session` timed out after 904s with no test result, so package-level gates are not promoted by this slice.

## Runtime 10 UI Contract M2 Gate


It requires Runtime 10 M2 rows to keep the declared `cargo test -p zircon_runtime_interface --locked`, `cargo test -p zircon_runtime --lib ui --locked`, and `cargo check -p zircon_editor --lib --locked` validation lane visible, mirrors the same pending state through P13, the Runtime 10 subplan row, Runtime 05 closeout, and the runtime-interface convergence document, records `ui_contract_duplicate_public_types = 0` after deleting the runtime-local duplicates for `UiBindingCodec` and `UiAssetSchemaVersionPolicy`, and records `ui_v2_contract_sync_anchors = 9/9` after synchronizing `UiComponentApiVersion` and `v2-replacement-mainline`.

## Runtime 08 ECS Data-Kernel Guard


The guard keeps Runtime 08 `in_progress`, requires the M1/M2/M3 rows to retain `code_complete_pending_cargo`, keeps the declared entity/observer/command/messages/change_tick/ecs filters visible, and mirrors the same pending state through the runtime index and this review. The protected anchors include `despawned_entity_handle_is_rejected_by_world_access`, `lifecycle_observer_fires_immediately_during_component_mutation`, `command_queue_on_despawned_entity_target_is_reported_not_silently_dropped`, `events_require_explicit_update_and_keep_next_queue_hidden`, and `change_tick_comparison_survives_wraparound`.

The same follow-up now has `runtime_absorption::ecs_kernel_data::runtime_08_ecs_kernel_data_mirror_docs_match_structure_audit_counts`, and `runtime_structure_audits/ecs_kernel_data_boundary.py` is wired into the Python structural audit. The current static mirror reports `expected_source_file_count = 69`, `expected_test_file_count = 8`, `archetype_anchors = 15/15`, `storage_anchors = 9/9`, `component_storage_private_reexport_anchors = 9/9`, `unexpected_component_storage_private_reexports = []`, `component_identity_anchors = 18/18`, `entity_lifecycle_anchors = 10/10`, `observer_anchors = 8/8`, `deferred_command_anchors = 11/11`, `event_message_anchors = 12/12`, `resource_identity_anchors = 12/12`, `change_tick_anchors = 6/6`, `runtime_08_guard_anchors = 21/21`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `doc_anchors = 13/13`, `pending_cargo_gate_anchors = 6/6`, `mirror_docs_guard_present = true`, and `risks = []`; the archetype/component/entity/event/message/resource/resource-store/component-storage/observer/commands facade/change-detection mirror now explicitly includes the folder-backed `archetype/{mod,id,index,move_result,record,signature}.rs`, `component/{mod,marker,id,registry}.rs`, `entity/{mod,despawned,error,internal,location,registry,slot,stable_location}.rs`, `events/{mod,cursor,id,metrics,queue,store,subscription}.rs`, `messages/{mod,cursor,id,queue,store}.rs`, `resource/{mod,marker,id,registry}.rs`, `resource_store/{mod,stored_resource,store}.rs`, `storage/component_storage/{mod,component_results,entry,location,sparse,store,table}.rs`, `observer/{mod,callback_registry,callbacks,entry,id,store}.rs`, `commands/commands/{mod,entity_commands,facade,param}.rs`, and `change_detection/{mod,change_tick,change_tick_window,component_ticks,stats,wrappers}.rs` owner sets, plus explicit root leaf owners `scene/ecs/{bundle,removal,storage_type}.rs` and `first_stage_updates_all_registered_event_channels` for First-stage `EventStore::update_all()` advancement. This is still static structure evidence; entity/observer/command/messages/change_tick/ecs Cargo filters remain pending.

The 2026-06-21 source/test inventory split keeps Runtime 08's counted file inventory out of the orchestration boundary: `ecs_kernel_data_source_inventory.py` owns `RUNTIME_08_SOURCE_FILES`, `RUNTIME_08_TEST_FILES`, `EXPECTED_SOURCE_FILE_COUNT = 69`, `EXPECTED_TEST_FILE_COUNT = 8`, and the mirror-doc guard name. `ecs_kernel_data_boundary.py` remains the domain-anchor and report renderer, direct audit still reports source files 69/69 and guard/test files 8/8 with `risks = []`, and this does not promote the pending entity/observer/command/messages/change_tick/ecs Cargo filters.



## Runtime 12 Input Stack Guard

The 2026-06-13 Runtime 12 follow-up added `runtime_absorption::input_stack`. This is a boundary/documentation guard over the already-landed input stack slices: frame input transitions and clear timing, UI-first action mapping, gamepad axis values, gamepad axis transition edges, InputConfig action-map data, action manager runtime registration, input recording/replay, cursor host requests, and gamepad ABI bridging.


`input_stack_boundary.py` now mirrors the same Runtime 12 shape through the Python structural audit as the 337-line audit reader, missing-anchor checker, and risk classifier; `input_stack_markdown.py` owns the 108-line Markdown renderer. Current evidence reports `expected_runtime_module_count = 12`, `expected_framework_module_count = 20`, `expected_test_module_count = 7`, `public_surface_anchors = 26/26`, `runtime_12_guard_anchors = 5/5`, `behavior_test_anchor_count = 15`, `missing_gamepad_abi_anchors = []`, `missing_cursor_host_request_anchors = []`, `missing_doc_anchors = []`, `missing_test_anchors = []`, `missing_behavior_test_anchors = []`, `missing_cargo_gate_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. It checks the runtime input owner tree, framework input contract tree, action evaluator UI-filtered, active-context, gamepad axis-value, consumed-axis, and axis-transition paths, InputConfig action-map data-source paths, `InputActionManager` / `DefaultInputActionManager` / `resolve_input_action_manager(...)` runtime registration, `InputRecording` / `InputReplayCursor` deterministic replay paths, `CursorHostRequest` frame-local host handoff, `ZrRuntimeHostRequestV1::Cursor` dynamic ABI, app host `apply_runtime_cursor_host_request(...)`, desktop `platform.cursor_options=supported:winit_window_options`, app gilrs -> runtime ABI -> `session/events.rs` -> `InputEvent::Gamepad*` bridge, M0/M1/M2/M4 behavior test anchors, Rust guard anchors, and pending Cargo gate anchors without running Cargo.


## Runtime 13 Script Binding Guard

Runtime13 scene-transition contract sync (2026-08-15): `script_binding_boundary` reports `expected_source_file_count = 24`, `expected_test_file_count = 3`, `expected_guard_file_count = 8`, `gameplay_callback_count = 40`, `host_capability_count = 13`, `missing_source_files = []`, `missing_guard_files = []`, and `risks = []`. `script.rs` remains the public facade for `argument_views`, `call_frame`, `descriptors`, `hot_path_metrics`, and `value_contracts`; capability-gated `request_scene_transition` emits only a latest-pending `ReplaceActive` request. It does not replace the active scene or publish completion: Runtime10 owns the missing project-session transaction, including pickup at a safe frame boundary, staged prepare/rollback, lifecycle handoff, and terminal result. The managed script Cargo gates remain pending.


The guard keeps Runtime 13 `in_progress`, requires M1/M2 rows to retain `code_static_pending_cargo`, and mirrors the same pending state through P16, the Runtime 13 subplan row, the host function ledger, and this review. The protected anchors include `host_function_registry_matches_documented_ledger`, `host_capability_representatives_are_declared_on_registered_modules`, `host_function_without_required_capability_is_rejected_with_explicit_error`, `script_held_entity_handle_reports_invalid_after_despawn`, and `script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi`; the remaining validation is the Runtime 13 script filters gate.

`script_binding_boundary` now mirrors the same Runtime 13 shape through the Python structural audit, while `script_binding_markdown.py` owns the Markdown renderer. Current evidence reports `script_binding_boundary.py = 352`, `script_binding_markdown.py = 106`, `expected_source_file_count = 19`, `expected_test_file_count = 3`, `fixed_host_module_count = 6`, `fixed_host_function_count = 52`, `type_descriptor_count = 2`, `builtin_callback_count = 11`, `gameplay_callback_count = 39`, `macro_host_function_count = 2`, `host_capability_count = 11`, `guard_anchor_count = 9`, `native_ecs_abi_references = []`, `oversized_test_files = []`, `mirror_docs_guard_present = true`, and `risks = []`. It checks the host ledger, capability representatives, bridge dynamic-module anchors, `zr.zircon.gameplay` facade, native ECS ABI exclusion, Rust guard anchors, and pending script Cargo gate, and scoped `script::vm` Cargo evidence. `runtime_13_script_binding_mirror_docs_match_structure_audit_counts` keeps Runtime 13, the runtime index, this review, runtime-interface convergence, and `function_ledger.md` aligned with those structure-audit counts; `script::vm` passed 48/48 on 2026-06-14, while broader script filters remain pending on non-gameplay-host scene/vampire/UI tests.

`runtime_absorption/script_binding.rs` is the Rust-side guard module for these Runtime 13 script-binding structural mirror anchors.


Runtime 10 coverage also includes the `dynamic_runtime_api_boundary` structural mirror so the recent static guard list names the ABI/session/loader mirror alongside the Rust guard names and pending UI owner gate.

Runtime 09 coverage also includes the `ui_architecture_boundary` structural mirror and `ui_architecture_markdown.py` renderer split so the recent static guard list names the UI architecture mirror alongside the Rust guard names and pending owner/Cargo gate.

## Runtime 11 Rayon Boundary Guard

The Runtime 11 guard now treats `core/runtime/tasks/pool.rs` and `core/runtime/tasks/parallel_for.rs` as the only production direct-Rayon owners. The previous render-owned `parallel_frustum.rs` exception has been cut over in status `runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending`: `WgpuRenderFramework` carries a runtime compute task pool, runtime graphics module construction supplies `core.task_pools().compute().clone()`, and frustum culling runs through `parallel_for(...)`.

`job_system_boundary` now mirrors the same Runtime 11 structure through the Python structural audit: `expected_module_count = 9`, `direct_rayon_paths = 2`, `schedule_parallel_executor_direct_rayon = []`, `diagnostic_anchor_count = 11`, `behavior_test_anchor_count = 27`, `missing_behavior_test_anchors = []`, `oversized_modules = []`, `mirror_docs_guard_present = true`, and `risks = []`. It checks all eleven task diagnostic keys, the task owner folder, `JobHandle` / `JobScheduler::schedule_after` / `JobScheduler::wait_all` / `parallel_for` / `JobSchedulerReport` anchors, the ECS batch dependency path, the M1/M3 behavior test anchors including lifecycle-conservation with dependency-waiting release/cancellation, overlapping-writer publication, combined-barrier panic propagation, continuation-unwind survivor delivery, and 1/2/4-worker pressure coverage, the detached-panic helper anchor, the direct-Rayon whitelist, and `runtime_11_job_system_mirror_docs_match_structure_audit_counts` without running Cargo. The 2026-06-21 inventory split records `job_system_inventory_split_static_passed_cargo_deferred_tests_deferred`: `job_system_source_inventory.py` owns the task owner module list, line budget, and direct-Rayon scan, `job_system_anchor_inventory.py` owns the declaration/API/schedule/test/doc anchors. The follow-up `job_system_markdown_split_static_passed_cargo_deferred_tests_deferred` slice keeps `job_system_boundary.py` at 193 lines as the audit reader, missing-anchor calculator, and risk aggregator, while `job_system_markdown.py` owns the 64-line Markdown renderer.

## Runtime 11 JobSystem Cargo Gate


The guard keeps Runtime 11 `in_progress`, requires M1/M2/M3 rows to retain pending Cargo language, keeps the declared `tasks`, `job`, `rayon`, `ecs_schedule`, and `worker_pool` commands visible, and mirrors the same pending state through P14, the Runtime 11 subplan row, `job_system.md`, Runtime 05 closeout, and this review.

## Runtime 14 Module Family Boundary

`module_family_boundary` now mirrors the Runtime 14 root-seat closeout through the Python structural audit, while `module_family_markdown.py` owns `render_module_family_boundary_markdown` under `module_family_markdown_split_static_passed_cargo_deferred_tests_deferred`. Current evidence reports `expected_family_count = 4`, `animation = 28`, `navigation = 9`, `diagnostic_log = 7`, `engine_module = 8`, `root_seat_guard_present = true`, `animation_status_json_guard_present = true`, `animation_status_json_anchor_count = 8`, `missing_animation_status_json_anchors = []`, `module_family_guard_anchor_count = 7`, `missing_module_family_guard_anchors = []`, `missing_doc_anchors = []`, `file_count_mismatches = []`, `cargo_gate_anchor_count = 5`, `missing_cargo_gate_anchors = []`, and `risks = []`. `module_family_boundary.py` is now the 305-line audit data/risk owner; `module_family_markdown.py` is the 61-line Markdown renderer owner. `navigation` is now a folder-backed fallback runtime owner split rather than a single behavior-heavy `runtime.rs`. `runtime_14_module_family_mirror_docs_match_structure_audit_counts` keeps this review, Runtime 14, the runtime index, and runtime-interface convergence aligned with those structure-audit counts.

The audit checks that `animation`, `navigation`, `diagnostic_log`, and `engine_module` remain crate-root module-family seats, that their mirror docs retain the Runtime 14 judgements, that the 7 explicit Rust guard anchors still exist, and that the pending gates `cargo test -p zircon_runtime --lib animation --locked`, `cargo test -p zircon_runtime --lib navigation --locked`, `cargo test -p zircon_runtime --lib diagnostic_log --locked`, `cargo test -p zircon_runtime --lib engine_module --locked`, and `cargo test -p zircon_runtime --lib --locked` remain visible. It does not promote Runtime 14 beyond static evidence; the Cargo filters remain pending.

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
- Done on 2026-06-04: `runtime_root_surface` moved into `runtime_structure_audits/runtime_root_surface.py`, reducing the main audit script from 690 lines to 643 lines while preserving root-surface evidence. Refreshed on 2026-06-17, the current evidence is 19 public modules, 2 public `pub use` locations, 0 crate-visible graphics re-exports, `rhi_wgpu` crate-private backend ownership, builtin helper types namespaced under `builtin`, M1 gate status `classified-and-clear`, and 0 current root-surface risks. Refined on 2026-06-21, `runtime_root_surface_markdown.py` owns `render_runtime_root_surface_markdown`, leaving `runtime_root_surface.py` as the 268-line audit/risk owner and the Markdown owner at 35 lines.
- Done on 2026-06-04: `non_network_server_naming` moved into `runtime_structure_audits/non_network_server_naming.py`, reducing the main audit script from 643 lines to 600 lines while preserving the non-network `server` naming evidence at 179 suspect references and 20 sample locations.
- Done on 2026-06-04: `non_network_server_naming` now reports an M1 gate status of `migration-debt-present`, filters 72 `observer` substring false positives, allows 93 real server-context lines, and classifies the remaining 87 suspect references into graphics render-framework debt, editor asset/resource owner debt, and editor scene comment debt with zero unclassified locations.
- Done on 2026-06-21: `non_network_server_naming_markdown` now owns the non-network server naming Markdown renderer, leaving `non_network_server_naming.py` as a 323-line token scan/allowed-context/classification/risk owner while the renderer sits in a 41-line module. The current direct audit reports count 77, classification groups 2, allowed contexts 94, observer false positives 95, migration debt groups 2, and zero unclassified locations.
- Done on 2026-06-04: `entry_static_dependencies` moved into `runtime_structure_audits/entry_static_dependencies.py`, reducing the main audit script from 600 lines to 528 lines while preserving app fan-out evidence at 4 app path dependencies, 0 optional runtime plugin path dependencies, 0 optional runtime plugin feature mentions, 1 built-in entry/runtime module crate, and no entry dependency risk.
- Done on 2026-06-21: `entry_static_dependencies_markdown.py` now owns `render_entry_static_dependencies_markdown(...)`, leaving `entry_static_dependencies.py` as the 73-line Cargo path dependency, optional runtime plugin fan-out, builtin entry/runtime module, and risk scanner. The current direct probe reports app path dependency count 4, optional runtime plugin path dependency count 0, optional runtime plugin feature mention count 0, builtin entry/runtime module count 1, risk count 0, Markdown owner 29 lines, and rendered output 5 lines.
- Done on 2026-06-04: `legacy_standalone_references` moved into `runtime_structure_audits/legacy_standalone_references.py`, reducing the main audit script from 528 lines to 476 lines while preserving stale standalone-crate architecture-doc evidence at zero counts and zero sample locations.
- Done on 2026-06-21: `legacy_standalone_references_markdown.py` now owns `render_legacy_standalone_references_markdown(...)`, leaving `legacy_standalone_references.py` as the 83-line architecture-doc stale standalone crate scanner/sample owner. The current direct probe reports 10 legacy standalone crate terms, reference count 0, referenced term count 0, sample location count 0, Markdown owner 14 lines, and rendered output 2 lines.
- Done on 2026-06-04, refreshed on 2026-06-21: `runtime_scene_editor_surface` moved into `runtime_structure_audits/runtime_scene_editor_surface.py`, keeping the main audit script at 476 lines after stale unused helper cleanup while preserving M3 scene/editor boundary evidence at zero editor-named production paths, zero public editor-named locations, and zero risks. The Markdown renderer now lives in `runtime_scene_editor_surface_markdown.py`, with the scan/risk owner at 92 lines and renderer at 25 lines.
- Done on 2026-06-04: `large_file_ownership` moved into `runtime_structure_audits/large_file_ownership.py`, reducing the main audit script from 476 lines to 420 lines while preserving large-file evidence at 10 reported top hotspots. Current 2026-06-20 owner-class counts are `editor-retained-host=3`, `editor-ui=8`, `runtime-framework-render=3`, `runtime-other=13`, and `support-hub=3`.
- Done on 2026-06-04, refreshed through 2026-07-01: `large_file_ownership` reports an M1 gate status of `classified-and-clear`, 0 hotspots above the 1000-line threshold, 0 migration-debt owner groups, and zero unclassified hotspots. Runtime 07 split animation and scene asset payload ownership out of former large files into `zircon_runtime/src/core/framework/animation/asset/` and `zircon_runtime/src/asset/assets/scene/`, split scene project I/O conversion owners into `zircon_runtime/src/scene/world/project_io/{camera,physics,post_process,references,script,transform}.rs`, split dynamic-session event routing into `zircon_runtime/src/dynamic_api/session/events.rs`, split artifact cache payload wire owners into `zircon_runtime/src/asset/artifact/cache_payload/{json_value,mesh,toml_value}.rs`, split render product diagnostics into `zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs`, split virtual geometry debug snapshot DTO owners into `zircon_runtime/src/core/framework/render/virtual_geometry_debug_snapshot/{bvh_visualization,cpu_reference,cull_input,execution,node_and_cluster_cull,snapshot,sources}.rs`, Runtime 13 split gameplay host domains into `zircon_runtime/src/script/vm/gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs`, and Runtime 14 split navigation fallback runtime owners into `zircon_runtime/src/navigation/runtime/{baked_mesh,world_scan,avoidance,state,math,tests}.rs` so those entry files stay below the owner budget; the current hotspot set is empty.
- Done on 2026-06-21, refreshed on 2026-07-01: `large_file_ownership_markdown.py` now owns large-file hotspot, ownership-class, and ownership-gate Markdown rendering. `large_file_ownership.py` remains the owner classification, hotspot summary, migration-debt, and risk gate owner; the direct probe reports 0 hotspots, 0 owner classes, 0 migration-debt groups, zero unclassified hotspots, gate status `classified-and-clear`, and no boundary risks.
- Done on 2026-06-04: `plugin_runtime_gaps` moved into `runtime_structure_audits/plugin_runtime_gaps.py`, reducing the main audit script from 420 lines to 391 lines while preserving zero plugin runtime gaps.
- Done on 2026-06-21: `plugin_runtime_gaps_markdown.py` now owns `render_plugin_runtime_gaps_markdown(...)`, leaving `plugin_runtime_gaps.py` as the 77-line runtime plugin registration/resolve/context gap scanner. The current direct probe reports 6 `zircon_*` crates, 7583 production Rust files, zero plugin gaps, Markdown owner 11 lines, and rendered output 2 lines.
- Done on 2026-06-04: `module_inventory` moved module descriptor distribution, stub descriptor usage, owner coverage, module classification, support-crate listing, workspace production-file inventory, and hotspot source data into `runtime_structure_audits/module_inventory.py`, reducing the main audit script from 391 lines to 231 lines while preserving 3 classified module crates, zero stub descriptor usage, 3 support crates, and 10 large-file hotspots.
- Done on 2026-06-21: `module_inventory_markdown.py` now owns module descriptor distribution, stub descriptor usage, `EngineModule` owner coverage, module classification, and support-crate Markdown rendering. `module_inventory.py` is now the 193-line inventory/classification/hotspot-source owner, the Markdown owner is 62 lines, and the direct probe reports 6 `zircon_*` crates, 3 module crates, 3 support crates, zero stub descriptor crates, zero plugin gaps, and 30 large-file hotspot source entries.
- Done on 2026-06-04: `ecs_query_state_boundary` moved QueryState structural audit data out of the main audit script while preserving the old-file-absent, folder-backed owner module, root-line-budget, and zero oversized owner module evidence. The current 2026-06-17 follow-up accepts `query_state/cache.rs` as the cache behavior owner and `query_state/stats.rs` as the Runtime 07 telemetry sidecar, so the audit now reports 9/9 owner modules and no boundary risk. The 2026-06-21 follow-up moves Markdown rendering into `ecs_query_state_markdown.py`, leaving `ecs_query_state_boundary.py` as the 141-line audit/risk owner.
- Done on 2026-06-04: `scene_project_serialization_boundary` moved into `runtime_structure_audits/scene_project_serialization_boundary.py`, reducing the main audit script from 864 lines to 690 lines while preserving the then-current 7-file/0-forbidden-location evidence. On 2026-06-15 the audit file list followed the scene asset and project I/O folder split; on 2026-06-21 it was realigned with the current 24-file folder-backed Rust source guard and `scene_project_serialization_markdown.py` took over the Markdown renderer, leaving the boundary as the 127-line scan/risk owner with forbidden_location_count=0.
- Done on 2026-06-04: the first runtime-other large-file production split reduced `zircon_runtime/src/dynamic_api/session.rs` from 1207 lines to 947 lines by extracting ABI status construction, host-request conversion, input-event conversion, and preview fallback helpers under `zircon_runtime/src/dynamic_api/session/`.
- Done on 2026-06-04: the matching dynamic API test split removed the 893-line `zircon_runtime/src/dynamic_api/tests.rs` and replaced it with folder-backed test owners. The current follow-ups keep the tree at 12 owner modules by splitting lifecycle overflow into `session_entry_points.rs`, `session_lifecycle.rs`, and `session_profiles.rs`, and by moving host-request payload encode/free checks into `host_request_payloads.rs`.
- Done on 2026-06-04: the dynamic API test boundary now has both a Rust structure test and structural audit output that reject a recreated `tests.rs`, missing owner modules, missing declarations, non-navigational `mod.rs` content, and oversized owner test files.
- Done on 2026-06-04: `dynamic_api_test_boundary` itself moved out of the near-threshold audit script into `runtime_structure_audits/dynamic_api_test_boundary.py`, reducing the main audit script from 1095 lines to 992 lines while preserving the JSON and Markdown evidence.
- Done on 2026-06-21: `dynamic_api_test_markdown` now owns the Dynamic API test boundary Markdown renderer, leaving `dynamic_api_test_boundary.py` as an 89-line audit/risk owner while the renderer sits in a 35-line module. The direct audit still reports owner modules 12/12, legacy `tests.rs` absent, no missing declarations, no oversized owner modules, and `risks = []`.
- Done on 2026-06-04: `generated_code_boundary` and `native_plugin_public_surface` also moved into folder-backed audit owner modules, reducing the main audit script from 992 lines to 864 lines while preserving generated behavior count 13 and the then-current native root re-export count 54. The current Runtime 06 mirror now has `root_reexport_count = 0` and V3-only native ABI after the M2.1/M3.1 hard-cutovers.
- Done on 2026-06-04 and hard-cutover updated on 2026-06-16: `native_plugin_public_surface` now reports an M4 gate status of `classified-and-clear` for the plugin root. Current classification covers all 60 `plugin::native` namespace symbols across native ABI contract, loader/discovery, live-host runtime, behavior-report, and bridge-method groups, with `root_reexport_count = 0`, `native_namespace_reexport_count = 64`, and zero unclassified root or namespace symbols.
- Done on 2026-06-21: `native_plugin_public_surface_markdown` now owns the native plugin public-surface Markdown renderer, leaving `native_plugin_public_surface.py` as a 400-line scan/classification/M4 gate owner while the renderer sits in a 63-line module. Direct evidence still reports root re-export 0, native namespace re-export 60, symbol decision groups 5, migration debt 0, zero unclassified root/namespace symbols, and rendered output 12 lines.
- Done on 2026-06-21: `hard_cutover_migration_smells_markdown` now owns the hard-cutover migration-smell Markdown renderer, leaving `hard_cutover_migration_smells.py` as a 408-line scanner/classifier/risk owner while the renderer sits in a 60-line module. The same audit now classifies Runtime 05 dynamic scene document legacy refs under `legacy-runtime-scene-document-debt`, with source files 8520, legacy refs 152, allowed business bridge refs 316, migration debt groups 6, and zero unclassified locations.
- Done on 2026-06-04: the support-crate ABI surface split reduced `zircon_runtime_interface/src/runtime_api.rs` from 1082 lines to a 12-non-empty-line facade backed by `runtime_api/{api_table,constants,events,host_requests,requests,viewport}.rs`, preserving the public `runtime_api::*` re-export shape and adding a boundary test against facade regression.
- Done on 2026-06-04: `runtime_api_boundary` is now part of the structural audit. It rejects missing or unexpected ABI owner modules, missing facade declarations or re-exports, direct ABI declarations in the facade, facade growth beyond 20 non-empty lines, and owner modules above 700 lines.
- Done on 2026-06-21: `runtime_api_markdown` now owns the Runtime API boundary Markdown renderer, leaving `runtime_api_boundary.py` as a 143-line audit/risk owner while the renderer sits in a 39-line module. The direct audit still reports owner modules 6/6, facade 12/20 non-empty lines, no missing declarations or re-exports, no direct ABI declarations in the facade, no oversized owner modules, and `risks = []`.
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

The first runtime-other large-file slice kept the then-current runtime function table unchanged and split only private session implementation helpers; the later runtime-table hard cut removed the retired table surface.

- `session/status.rs` for ABI `ZrStatus` constructors;
- `session/host_requests.rs` for IME, gamepad rumble, and cursor host-request conversion;
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
- `tests/host_request_payloads.rs` for IME, gamepad rumble, cursor, and free-token host-request payload encoding/freeing;
- `tests/host_requests.rs` for drain/session-flow host request behavior;
- `tests/accessibility.rs` for accessibility tree/action fallback behavior;
- `tests/input_events.rs` for mouse-wheel, window-scale, and IME invalid-input rejection;
- `tests/structure.rs` for folder-backed test-tree regression checks;
- `tests/support.rs` for shared ABI fixtures and buffer free helpers.

New dynamic API assertions should land in the matching owner module, not in a recreated `tests.rs`. The audit reports this as `dynamic_api_test_boundary`; the current accepted state is `legacy_tests_file_exists = false`, 12 owner modules present, `host_requests.rs = 98`, `host_request_payloads.rs = 156`, and zero oversized owner modules over 250 lines. The audit implementation is now a dedicated 89-line owner module and its Markdown renderer lives in `dynamic_api_test_markdown.py`, so future boundary checks should continue moving into `runtime_structure_audits/` rather than growing the main audit script.

The Runtime 10 dynamic runtime API contract is now separately mirrored as `dynamic_runtime_api_boundary`. That audit owner ties the ABI function-table inventory, session FFI wrapper table, headless/minimal lifecycle rules, app loader failure guards, runtime diagnostics profile-control snapshot, host-request payload chain, UI pending gate, UI contract single-source guard, UI v2 contract synchronization, and Cargo-pending gate into a single static report while `dynamic_runtime_api_markdown.py` owns the Markdown renderer. Current evidence is `dynamic_runtime_api_boundary.py = 330`, `dynamic_runtime_api_markdown.py = 65`, `expected_source_file_count = 35`, `function_table_structs = 10/10` including `ZrHostBridgeApiV1`, `field_count_mismatches = 0`, `missing_repr_c_tables = 0`, `runtime_session_ffi_wrappers = 11/11`, `direct_session_table_entry_bypasses = 0`, `session_owner_extern_c_present = false`, `headless_lifecycle_anchors = 12/12`, `ffi_panic_anchors = 9/9`, `loader_failure_anchors = 10/10`, `behavior_test_anchor_count = 16`, `missing_behavior_test_anchors = []`, `runtime_diagnostics_anchors = 15/15`, `missing_runtime_diagnostics_anchors = []`, `host_request_payload_anchors = 38/38`, `missing_host_request_payload_anchors = []`, `ui_pending_gate_anchors = 8/8`, `ui_contract_single_source_anchors = 7/7`, `ui_contract_duplicate_public_types = 0`, `ui_v2_contract_sync_anchors = 9/9`, `pending_cargo_gate_anchors = 5/5`, `doc_anchors = 13/13`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts` keeps those fields mirrored across the Runtime 10 docs.

## ECS QueryState Split

The second runtime-other large-file slice kept `zircon_runtime::scene::ecs::QueryState` as the public query state type and split only source ownership:

- `query_state/mod.rs` owns `QueryState`, construction, access descriptors, cache fields, and telemetry fields;
- `query_state/cache.rs` owns cache rebuilds, cache-slot lookup, cached entity/component-location accessors, and cache metadata accessors;
- `query_state/cached_direct.rs` owns cached storage-location direct access for `CachedQueryData`;
- `query_state/read_only.rs` owns non-mutating query iteration, get/many/contains, cached read iteration, and combinations;
- `query_state/mutable.rs` owns mutable query access, duplicate-entity rejection, mutable many/combination iteration, and the narrow post-validation unsafe fetch;
- `query_state/many_item_array.rs` owns shared many-query fixed-array collection and cached entity filtering helpers;
- `query_state/stats.rs` owns the Runtime 07 cache telemetry snapshot API and change-detection telemetry accumulation;
- `query_state/system_param.rs` owns the `SystemParam` bridge into runtime systems.

This follows Bevy's query module precedent: keep state, cache rebuilds, data/filter, iterator, telemetry, and system-param roles navigable instead of stacking every access family into one hot-path state file. `scene::tests::ecs_query_structure` and the structural audit's `ecs_query_state_boundary` both guard against recreating `query_state.rs`, missing owner files, behavior impl families in the root file, and owner files above the current budget. The current accepted audit state is old file absent, 9/9 owner modules present, root at 84/180 non-empty lines, and no oversized owner modules. `ecs_query_state_markdown.py` now renders the audit section so `ecs_query_state_boundary.py` stays an audit/risk owner and `audit_runtime_structure.py` stays a short orchestration script.

## Runtime Root Surface Guard

The 2026-06-13 Runtime 02 follow-up added `runtime_absorption::root_surface` plus `docs/zircon_runtime/core/root_surface.md`. The guard now records the current public root as 19 namespace modules and three curated `pub use` sites, rejects new flattened subsystem root re-exports, keeps graphics aliases removed from the crate root, and keeps `rhi_wgpu` crate-private behind the `rhi` owner.

The M3.2 type alias debt slice is now closed at the crate root. `SceneRenderer`, `GraphicsError`, `WgpuRenderFramework`, `ViewportFrame`, `HybridGiRuntimeProvider`, `VirtualGeometryRuntimeProvider`, `SolariRuntimeProvider`, and `RendererFeatureReferenceListKind` are no longer root aliases; callers must use `crate::graphics::...` owner paths. Current status is `graphics_alias_block_removed_static_passed_cargo_pending`.

The M3 root work has removed the `pub(crate) use graphics::...` alias block and the concrete `rhi_wgpu` root public module. The guard prevents root-surface regression while broader graphics/RHI validation remains pending.

## Runtime 02 Core/Root/Generated Gate


The guard keeps Runtime 02 `in_progress`, requires the M2 test-stage row, M3 root/type alias rows, and M4 generated row to keep Cargo or render-owner pending language, keeps the declared runtime/app/editor/plugin/generated/export validation commands visible, and mirrors the same pending state through P2, P8, the Runtime 02 subplan row, `root_surface.md`, `generated-code-boundary.md`, Runtime 05 closeout, and this review.

`core_spine_root_generated_boundary.py` now mirrors the Runtime 02 core/root/generated state through the Python structural audit, while `core_spine_root_generated_markdown.py` owns the Markdown renderer. Current evidence reports core root entries 6/6, core public modules 5/5, retired core root entries 0, runtime root public modules 19/19, public `pub use` sites 2/2, crate-visible graphics alias debt 0/0, root-surface M1 gate `classified-and-clear`, generated export templates 10/10, generated behavior 6/6, generated allowed adapters 6/6, generated migration debt 0/0, generated-code M1 gate `classified-and-clear`, root_entries guard tests 13, root_surface guard tests 6/6, generated-code guard tests 7/7, `guard_test_anchor_count = 26`, `missing_guard_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`; the boundary owner is 315 lines and the Markdown owner is 60 lines. `runtime_root_surface.py` itself remains the 268-line root public-surface audit/risk owner, while `runtime_root_surface_markdown.py` is the 35-line renderer owner for `render_runtime_root_surface_markdown`. `runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts` keeps Runtime 02, the runtime index, `root_surface.md`, generated-code-boundary, this review, and runtime-interface convergence aligned with those counts. `root_surface_interface_convergence_mirror_uses_current_audit_counts` keeps `runtime-interface-convergence.md` aligned with the same 19-module, 2-public-use, zero-root-debt root-surface evidence, `rhi_wgpu` crate-private backend owner status, and builtin facade cutover. This is static structure evidence only; the full `core/root/generated/export_build_plan/app/editor/plugin` Cargo lane remains pending.

## Runtime 07 Hotspot Inventory Guard

The 2026-06-13 Runtime 07 follow-up added `runtime_absorption::performance_hotspots` plus `docs/zircon_runtime/performance/hotspot_inventory.md`. This is a measurement-boundary guard, not an optimization claim: M1.3 now records a scaffolded hotspot inventory and rejects M2 entries that do not have named counter evidence. Current counted candidates are extract full rebuild samples, QueryState cache telemetry, change-detection scan telemetry, and asset-worker diagnostics including `AssetWorkerPoolFrameSampler`, `asset.worker.frame_completed`, and `asset.worker.frame_failed`. The old RenderDoc evidence for 230 draws, 231 pre-draw `vkCmdCopyBuffer` calls, and 31 render passes remains a render-plan diversion rather than a Runtime 07 implementation target.

Runtime 07 M0.3 also pins the stage-level span that makes the broad frame update phase measurable inside ECS scheduling. `SceneScheduleRunner::run_stage(...)` owns `runtime_frame_schedule_stage.<SystemStage>` through `profile_dynamic_scope!("runtime", "frame", ...)`, which preserves existing sorted-step iteration, deferred flush, hook execution, and final consistency sweep behavior while giving profiling traces a stage-level span boundary.

## Runtime 07 Performance Hotpath Guard


The 2026-07-12 completion replaces that pending guard with
`runtime_07_performance_hotpath_records_completed_authoritative_validation`.
Runtime 07 now has two exact Vampire FPS samples with `9.521868%` deviation,
accepted native/Perfetto trace execution, fixed extract/query/change counter
behavior, and production publication through
`EcsFramePerformanceDiagnostics::publish(...)`. The shared full-package build
was concurrently blocked by active Shader 06 and Physics 03 source, so that
external owner validation is recorded separately instead of keeping Runtime 07
design work falsely open.

The guard keeps Runtime 07 `in_progress`, requires M0.2/M0.3/M1.1/M1.2/M1.3 rows to retain Cargo/profiling/FPS pending language, keeps the declared `vampire_project_session_reports_runtime_fps_and_render_work`, `cargo check`, `extract`, and `ecs_query` commands visible, and mirrors the same pending state through P5, the Runtime 07 subplan row, `hotspot_inventory.md`, dynamic-session frame diagnostics, ECS stage-span docs, and this review. The 2026-06-17 M0.1 attempt now has the local `ZR_VM_RUST_BINDING_LIB_DIR` / DLL path identified and repaired three support-layer lib-test blockers (`UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION` public re-export imports plus a `RenderBloomSettings` default initializer), but the follow-up command timed out after 904s with no test result and no `vampire_runtime_perf` sample; residual cargo/rustc processes from that run were stopped. The M0.2 profiling build lane now has reproducible entry points through `tools/zircon_build.py --mode profiling --runtime-features target-client,profiling,profiling-tracy` and `tools/dev-fast-build.ps1 -CargoProfile profiling`, but the actual profiling build duration and bottleneck segment remain deferred until the shared Cargo/rustc lanes are clear.

The same follow-up added `runtime_structure_audits/performance_hotpath_boundary.py` and wired it into the Python structural audit. The 2026-06-21 renderer split moved `render_performance_hotpath_boundary_markdown(...)` into `performance_hotpath_markdown.py`; the follow-up inventory split now moves Runtime 07 source/test lists and expected counts into `performance_hotpath_source_inventory.py`, moves frame/query/change/extract/asset-worker/animation/profile-counter/test/doc/Cargo anchors into `performance_hotpath_anchor_inventory.py`, and leaves `performance_hotpath_boundary.py` as the 353-line audit reader, missing-anchor calculator, large-file gate consumer, and risk aggregator. Current line ownership is source inventory 70 lines, anchor inventory 244 lines, boundary 353 lines, and renderer 139 lines. The 2026-06-20 refinement makes the Runtime 07 mirror consume the QueryState cache owner split, QueryState and ChangeDetection frame auto-collection chains, the `NonNull<QueryState<D, F>>` cached iterator lifetime guard, the dynamic-session `RuntimeFrameExtractCache` M2 slice, the asset-worker frame sampler, the animation scene hook `AnimationSceneFrameDiagnostics` counters, the generic profiling counter hotspot export, the artifact cache payload owner split, the render product diagnostics owner split, the virtual geometry debug snapshot owner split, and the navigation runtime owner split in addition to the owner-budgeted large-file gate. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_07_large_file_owner_budget_gate_stays_in_sync_with_structure_audit` keeps that owner-budget summary synchronized with `large-file-ownership-m1.md`, Runtime 07, the runtime index, `hotspot_inventory.md`, and the interface-convergence mirror while rejecting stale 30/33/36/37/38/39/40/41/42-hotspot and removed Hub `app/` anchors. `runtime_07_project_io_folder_split_keeps_entry_and_converter_owners` keeps the new project_io folder split from regressing back into a single converter-heavy entry file, `runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` keeps JSON/Mesh/TOML cache wire owners under `cache_payload/`, `runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed` keeps `render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs` and the remaining product diagnostic families folder-backed, `runtime_07_profile_counter_hotspot_export_keeps_generic_counter_evidence_visible` keeps `CounterHotspotReport`, `counter_hotspots.json`, `analyze_counter_hotspots`, and `ProfileControlResponse.counter_hotspot_report` visible as profiling evidence output, `runtime_07_virtual_geometry_debug_snapshot_owner_split_keeps_contracts_folder_backed` keeps `virtual_geometry_debug_snapshot/{cull_input,node_and_cluster_cull,snapshot}.rs` and the remaining debug snapshot DTO owners folder-backed, and `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` keeps this review and the Runtime 07 mirror docs aligned with those structure-audit counts, including `query_state/cache.rs`, `extract.cache_hits`, `extract.cache_misses`, `asset.worker.frame_completed`, `asset.worker.frame_failed`, `animation.scene.scanned_entities`, `animation.scene.output_poses`, and `counter_hotspots.json`. This is still static structure evidence; extract/ecs_query/profiling/FPS validation remains pending.

Current owner-budget mirror refresh 2026-07-01: `performance_hotpath_boundary` now reports M1 gate status `classified-and-clear`, 0 hotspots, 0 migration-debt owner groups, and zero unclassified hotspots. Exact anchors are `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, and `risks = []`. This only updates the large-file gate mirror; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-01: `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` now lives in `performance_hotspots/owner_budget/mirror_docs.rs`, and Runtime 07 audit input includes `performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs`. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 14`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-05: `Runtime 15 M3 Runtime 07 submit-context guard child-owner split` / `runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/submit_context/{sources,source_extract_payloads,camera_loop_sharing,feedback_sidebands,status_docs,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 20`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-05: `Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split` / `runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/{sources,evidence_gate_docs,ecs_extract_counters,profiling_trace_render,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 25`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split` / `runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/{sources,parent_routes,child_routes,source_inventory,line_budgets,status_docs,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 32`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.




Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split` / `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/artifact_render_diagnostics_splits/{artifact_cache_payload,render_product_diagnostics,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 35`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project guard child-owner split` / `runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/scene_project_splits/{scene_asset,project_io,dynamic_session_event,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 39`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters child-owner split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/{query_change,extract_cache,asset_animation,frame_diagnostics,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 44`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/mirror_docs/{sources,performance_guard,source_inventory,audit_wiring,doc_mirrors,split_layout}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 50`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/split_layout/{sources,route,source_inventory,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 54`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/split_layout/{route,source_inventory,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 57`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 submit-context split-layout guard folder-backed split` / `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/submit_context/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The focused split guard is `runtime_15_runtime_07_submit_context_split_layout_guard_folder_backed_split`; historical `runtime_15_runtime_07_submit_context_guard_child_owner_split` remains as a wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split` / `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/scene_project_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 65`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The focused split guard is `runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split`; historical `runtime_15_runtime_07_scene_project_guard_child_owner_split` remains as a wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 artifact/render diagnostics split-layout guard folder-backed split` / `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/artifact_render_diagnostics_splits/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 69`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The focused split guard is `runtime_15_runtime_07_artifact_render_diagnostics_split_layout_guard_folder_backed_split`; historical `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split` remains as a wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 hotspot-inventory ECS/extract counters split-layout guard folder-backed split` / `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/hotspot_inventory/ecs_extract_counters/split_layout/{route,source_inventory,sources,status_docs}.rs` to Runtime 07 audit input. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 73`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The focused split guard is `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_split_layout_guard_folder_backed_split`; historical `runtime_15_runtime_07_hotspot_inventory_ecs_extract_counters_child_owner_split` remains as a wrapper. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current mirror-docs owner refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget mirror-docs sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/mirror_docs/sources/{assertions,load,views}.rs` to Runtime 07 audit input while `performance_hotspots/owner_budget/mirror_docs/sources.rs` remains the route/type owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 76`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The focused split guard is `runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget source refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget sources guard folder-backed split` / `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/sources/load.rs` to Runtime 07 audit input while `performance_hotspots/owner_budget/sources.rs` remains the route/type owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 77`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The focused split guard is `runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split`. This only updates static mirror/test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.
Current owner-budget child-routes refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget child-routes guard folder-backed split` / `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/child_routes/{submit_context,hotspot_inventory,scene_project,artifact_render_diagnostics,owner_budget}.rs` to Runtime 07 audit input while `performance_hotspots/owner_budget/child_routes.rs` remains the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 82`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The focused split guard is `runtime_15_runtime_07_owner_budget_child_routes_guard_folder_backed_split`. This only updates static test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget line-budgets refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget line-budgets guard folder-backed split` / `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/line_budgets/{root,artifact_render_diagnostics,hotspot_inventory,owner_budget,scene_project,submit_context}.rs` to Runtime 07 audit input while `performance_hotspots/owner_budget/line_budgets.rs` remains the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 88`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The focused split guard is `runtime_15_runtime_07_owner_budget_line_budgets_guard_folder_backed_split`. This only updates static test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

Current owner-budget split-layout route refresh 2026-07-06: `Runtime 15 M3 Runtime 07 owner-budget split-layout route guard folder-backed split` / `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_static_passed_cargo_deferred` adds `performance_hotspots/owner_budget/split_layout/route/{parent_route,split_route,support_routes}.rs` to Runtime 07 audit input while `performance_hotspots/owner_budget/split_layout/route.rs` remains the route owner. The current static mirror reports `expected_source_file_count = 46`, `expected_test_file_count = 91`, `frame_span_anchor_count = 9`, `query_counter_anchor_count = 32`, `change_counter_anchor_count = 13`, `extract_counter_anchor_count = 21`, `asset_worker_anchor_count = 13`, `animation_scene_anchor_count = 19`, `profile_counter_hotspot_anchor_count = 8`, `hotspot_guard_anchor_count = 32`, `test_anchor_count = 29`, `doc_anchor_count = 35`, `cargo_gate_anchor_count = 5`, `stale_hotspot_placeholder_present = false`, `large_file_m1_gate_status = classified-and-clear`, `large_file_hotspot_count = 0`, `large_file_migration_debt_count = 0`, `large_file_owner_class_count = 0`, `large_file_unclassified_hotspot_count = 0`, `missing_large_file_owner_classes = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, `risks = []`, and keeps `runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts` visible. The focused split guard is `runtime_15_runtime_07_owner_budget_split_layout_route_guard_folder_backed_split`. This only updates static test-owner inventory; extract/ecs_query/profiling/FPS Cargo gates remain pending.

## Runtime 10 V7-only ABI Sync

Current V7 owner inventory: `expected_source_file_count = 60`.

The dynamic runtime boundary exposes only the 25-field `zircon_runtime_get_api_v7` /
`ZrRuntimeApiV7` pair. V7 includes opaque allocation release, plugin-event subscription/delivery,
required operation lifecycle, highlight submission, and query/watch/unwatch/drain world sync; the app rejects missing or incomplete V7
libraries without downgrade. `dynamic_runtime_api_boundary` pins
`expected_source_file_count = 60`, `function_table_structs = 12/12`, the 25-field table,
`runtime_session_ffi_wrappers = 23/23`, direct-entry bypass absence, and the FFI panic boundary.
The parent `dynamic_api/session.rs` remains a façade over explicit child owners, including
`session/world_sync.rs`. The permanent guard is
`runtime_10_dynamic_runtime_api_mirror_docs_match_structure_audit_counts`.
2026-07-22 Runtime 06 native callback diagnostics public-surface sync supersedes the older 64-symbol current-tree snapshots while retaining them as dated history. The new diagnostics and load-report/layout symbols stay under the existing five owner groups and the sole `plugin::native` public seat. Current evidence is `root_reexport_count = 0`, `native_namespace_reexport_count = 68`, native root re-export 0/0, native namespace re-export 68/68, M4 gate `classified-and-clear`, debt groups 0/0, native namespace symbol groups 5/5, unclassified native root symbols 0/0, unclassified native namespace symbols 0/0, root public native re-export locations 0/0, public native namespace re-export locations 1/1, app NativePlugin current call-site files: 7, native loader V1/V2 implementation files 0/0, `zircon_plugins` V1/V2 usage files 0/0, export_build_plan V1/V2 usage 0/0, unknown ABI rejection, hot reload failure injection, native loader test files 4/4, native test namespace import files 3/3, native test root import leaks 0/0, fallback lifecycle failure tests 4/4, `runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed`, `runtime_06_native_loader_tests_use_isolated_plugin_native_namespace`, `mirror_docs_guard_present = true`, `risks = []`, and `runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts`.

Current Runtime 04 source-owner synchronization (2026-08-14): `asset_pipeline_boundary` reports `expected_source_file_count = 26`; `core/resource/manager/commit.rs` owns reload transaction-state mutation in the current tree. This replaces the previous public-facade-only inventory; broader Cargo gates remain pending.
