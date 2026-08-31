---
related_code:
  - docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md
  - tests/acceptance/runtime-priority-plan-output-archive-ownership.md
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/asset/assets/texture/upload_support/dds.rs
  - zircon_runtime/src/asset/tests/assets/texture_upload_readiness.rs
  - zircon_runtime/src/asset/tests/assets/texture_upload_readiness/container_fixtures.rs
  - docs/zircon_runtime/asset/render-assets.md
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/entries/camera.rs
  - zircon_runtime/src/scene/world/property_access/entries/mesh.rs
  - zircon_runtime/src/scene/world/property_access/entries/lighting.rs
  - zircon_runtime/src/scene/world/property_access/entries/animation.rs
  - tools/tests/test_runtime_scene_property_entry_owner_structure.py
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access/error.rs
  - tools/tests/test_runtime_native_system_access_owner_structure.py
  - zircon_runtime/src/plugin/package_manifest/constructors.rs
  - zircon_runtime/src/plugin/package_manifest/constructors/module.rs
  - zircon_runtime/src/plugin/package_manifest/constructors/package.rs
  - tools/tests/test_runtime_plugin_manifest_constructor_owner_structure.py
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/keyboard.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/pointer.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/ime.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/window.rs
  - tools/tests/test_runtime_ui_winit_translation_owner_structure.py
  - zircon_runtime/src/ui/surface/surface/pointer_component_events.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs
  - tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py
  - tools/tests/test_runtime_ui_pointer_template_action_owner_structure.py
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs
  - tools/tests/test_runtime_ui_asset_surface_node_resource_owner_structure.py
  - docs/zircon_runtime/ui/platform_input.md
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/tests/asset_schema_migration.rs
  - zircon_runtime_interface/src/ui/template/asset/schema/report.rs
  - docs/zircon_runtime/ui/template/pipeline.md
  - docs/zircon_runtime_interface/ui/mod.md
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/operation/service/admission.rs
  - zircon_runtime/src/operation/service/json_budget.rs
  - zircon_runtime/src/operation/service/limits.rs
  - zircon_runtime/src/operation/service/prepare_completion.rs
  - zircon_runtime/src/operation/service/task_state.rs
  - zircon_runtime/src/operation/tests/phase_indexes.rs
  - zircon_runtime/src/operation/tests/source_guards.rs
  - tools/tests/test_runtime_operation_service_structure.py
  - zircon_runtime/src/dynamic_api/session/project.rs
  - zircon_runtime/src/dynamic_api/session/project/tests.rs
  - zircon_runtime/src/dynamic_api/session/project/runtime61_characterization.rs
  - tools/tests/test_runtime_dynamic_project_test_structure.py
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs
  - zircon_runtime/src/dynamic_api/session/events/gamepad.rs
  - tools/tests/test_runtime_dynamic_event_input_owner_structure.py
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration/tests.rs
  - tools/tests/test_runtime_plugin_system_registration_test_structure.py
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/worker.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs
  - tools/tests/test_runtime_shader_prewarm_test_structure.py
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
  - zircon_runtime/src/graphics/shader/template/module_registry/tests.rs
  - tools/tests/test_runtime_shader_module_registry_test_structure.py
  - zircon_runtime/src/core/runtime/tests/resolution/behavior.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/dependency_cycles.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/exact_dependency_resolution.rs
  - zircon_runtime/src/core/runtime/tests/resolution/behavior/factory_panics.rs
  - tools/tests/test_runtime_resolution_behavior_test_structure.py
  - zircon_runtime/src/ui/tests/widget_menu_behavior.rs
  - zircon_runtime/src/ui/tests/widget_menu_behavior/control_anchored_overlays.rs
  - tools/tests/test_runtime_widget_menu_behavior_test_structure.py
  - zircon_runtime/src/core/runtime/tests/activation/behavior/activation.rs
  - zircon_runtime/src/core/runtime/tests/activation/behavior/activation/contention.rs
  - tools/tests/test_runtime_activation_contention_test_structure.py
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/gpu_resolution.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs
  - zircon_runtime/src/dynamic_api/session/events/gamepad.rs
  - tools/tests/test_runtime_frame_profiler_gpu_resolution_owner_structure.py
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/animation/module.rs
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/animation_assets.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/runtime_helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - docs/zircon_runtime/animation/runtime.md
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - docs/zircon_runtime/input/input_state.md
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/input.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/input/mouse_wheel_line_delta.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_input_mouse_wheel.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_hub_raw_text.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/net/http1_client_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_net_http.rs
  - zircon_runtime/src/graphics/hybrid_gi_runtime_provider/mod.rs
  - zircon_runtime/src/graphics/virtual_geometry_runtime_provider/mod.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
  - docs/zircon_runtime/dynamic_api/session.md
  - zircon_runtime/src/core/framework/camera_controller/mod.rs
  - zircon_runtime/src/core/framework/camera_controller/controller_output.rs
  - docs/zircon_runtime/core/framework/camera_controller.md
  - zircon_runtime/src/scene/tests/ecs_systems.rs
  - zircon_runtime/src/scene/tests/ecs_systems/many_single_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_ecs_systems.rs
  - zircon_runtime/src/dynamic_api/session/tests/lock_poison.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/mod.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - docs/zircon_runtime/core/resource.md
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - docs/zircon_runtime/navigation/runtime.md
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/handle/states.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/asset/watch/mod.rs
  - zircon_runtime/src/asset/watch/shutdown_on_drop.rs
  - zircon_runtime/src/asset/watch/asset_change_construction.rs
  - zircon_runtime/src/scene/ecs/observer/mod.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/ecs/observer/callback_registry.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/many_item_array.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cached_direct.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mutable.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only.rs
  - zircon_runtime/src/scene/ecs/query/query_state/read_only_cached.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/component_results.rs
  - zircon_runtime/src/scene/tests/ecs_query.rs
  - zircon_runtime/src/scene/tests/ecs_query/cached_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/top_level.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/classifiers.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/lexical_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/support.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/camera_controller.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_framework/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/render_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/observer_callback_registry.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/query_state_many_item_array.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/component_storage_component_results.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/asset_watch.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/dynamic_api_vampire.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/scene_ecs_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_dynamic/texture_containers.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/plugin_static_manifest/contract_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_plugin_static_manifest.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/graphics/render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/banned_names/scene_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic_asset_watch.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic_dynamic_api_vampire.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_asset_dynamic_scene_ecs_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_framework_render_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_core_scene_ecs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_graphics.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/naming_boundary_banned_names.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/editor_workbench.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_construction.rs
  - zircon_runtime/src/graphics/backend/render_backend/mod.rs
  - zircon_runtime/src/graphics/backend/render_backend/offscreen_target_construct/construct.rs
  - docs/zircon_runtime/core/diagnostics.md
  - docs/zircon_runtime/core/runtime/lifecycle.md
  - docs/zircon_runtime/core/state.md
  - docs/zircon_runtime/core/tasks.md
  - docs/zircon_runtime/scene/ecs.md
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/production_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/folder_backing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/mounts.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/diagnostics_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
  - zircon_runtime/src/core/framework/tests/framework_surfaces.rs
  - zircon_runtime/src/core/framework/tests/render_product_surface.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/default_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/range_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/shared_core/layout_surface.rs
  - zircon_runtime/src/ui/tests/shared_core/box_flow.rs
  - zircon_runtime/src/ui/tests/shared_core/input_visibility.rs
  - zircon_runtime/src/ui/tests/shared_core/navigation.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f12_dead_code.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/audit_surfaces.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/runtime_roots.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/sdk_builder.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source/support.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/render_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/reads.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/inventory/current/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/delegation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/sources/status_mirrors.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/asset_schema.rs
  - docs/zircon_runtime/asset/assets/font.md
  - docs/zircon_runtime/asset/assets/ui.md
  - tools/plugin_structure_audits/capability.py
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs
  - zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/runtime-interface-convergence.md
implementation_files:
  - docs/zircon_runtime/structure/module-convention.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_convention_gate_markdown.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/helpers.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/module_doc_frontmatter.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/output_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/debt_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/audit_status.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate/split_layout.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler/gpu_resolution.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/asset/prelude.rs
  - zircon_runtime/src/scene/prelude.rs
  - zircon_runtime/src/ui/prelude.rs
  - zircon_runtime/src/graphics/prelude.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/tests/pipeline/worker_pool.rs
  - zircon_runtime/src/core/runtime/state/mod.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/state/module_entry.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - docs/zircon_runtime/core/resource.md
  - zircon_runtime/src/animation/manager/mod.rs
  - zircon_runtime/src/animation/manager/pose.rs
  - zircon_runtime/src/animation/manager/sampling.rs
  - zircon_runtime/src/animation/sequence/apply.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/core/framework/animation/error.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - docs/zircon_runtime/animation/runtime.md
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - docs/zircon_runtime/input/input_state.md
  - zircon_runtime/src/navigation/runtime.rs
  - zircon_runtime/src/navigation/runtime/tests.rs
  - docs/zircon_runtime/navigation/runtime.md
  - zircon_runtime/src/core/runtime/handle/registration/register_module.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/mod.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/types.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/multi.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/specialized.rs
  - zircon_runtime/src/core/runtime/handle/registration/service_lists/shutdown.rs
  - zircon_runtime/src/core/runtime/handle/core_handle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/mod.rs
  - zircon_runtime/src/core/runtime/tests/registration/structure/behavior_layout.rs
  - zircon_runtime/src/asset/watch/mod.rs
  - zircon_runtime/src/asset/watch/shutdown_on_drop.rs
  - zircon_runtime/src/scene/ecs/observer/mod.rs
  - zircon_runtime/src/scene/ecs/observer/store.rs
  - zircon_runtime/src/scene/ecs/observer/callback_registry.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/mod.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/store.rs
  - zircon_runtime/src/scene/ecs/storage/component_storage/component_results.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/top_level.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/classifiers.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/lexical_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/support.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/split_layout.rs
  - docs/zircon_runtime/core/runtime/lifecycle.md
  - docs/zircon_runtime/scene/ecs.md
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - docs/zircon_runtime/script/vm/host/function_ledger.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/facade_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/runtime_ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/production_scan.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/runtime_dead_code/status_anchor_cleanup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/sources.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/folder_backing.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/mounts.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/budgets.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/split_layout/status_mirrors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/diagnostics_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/framework/tests/framework_surfaces.rs
  - zircon_runtime/src/core/framework/tests/render_product_surface.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
  - zircon_runtime/src/ui/tests/v2_asset/default_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/range_controls.rs
  - zircon_runtime/src/ui/tests/v2_asset/demo_and_builder.rs
  - zircon_runtime/src/ui/tests/v2_asset/composite_components.rs
  - zircon_runtime/src/ui/tests/v2_asset/file_cache.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/shared_core/layout_surface.rs
  - zircon_runtime/src/ui/tests/shared_core/box_flow.rs
  - zircon_runtime/src/ui/tests/shared_core/input_visibility.rs
  - zircon_runtime/src/ui/tests/shared_core/navigation.rs
  - zircon_runtime/src/ui/tests/shared_core/scroll_mutation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/module_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/renderer_output_accessors.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/backend_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/gpu_resource_owners.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/graphics_dead_code/resource_streamer_cleanup.rs
tests:
  - python -B -m unittest tools.tests.test_runtime_frame_profiler_gpu_resolution_owner_structure
  - python -B -m unittest tools.tests.test_runtime_dynamic_event_input_owner_structure
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/child_ownership.rs::runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner: Runtime 15 M3 typed-error child-ownership source-tree reconciliation / runtime_15_typed_error_child_ownership_source_tree_reconciliation_static_passed_cargo_deferred; focused typed_error_child_owners passed 93/93 and code_review_findings passed 218/218 on 2026-07-07; package Cargo deferred
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/status/delegation.rs::runtime_15_typed_error_status_docs_are_folder_backed: Runtime 15 M3 typed-error status-doc source/status-map reconciliation / runtime_15_typed_error_status_doc_source_status_map_reconciliation_static_passed_cargo_deferred; focused typed_error_status_doc standalone guard passed 51/51 on 2026-07-07; package Cargo deferred
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_mixed_visibility_has_facade_note --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib prelude --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib structure_convention --no-default-features --features core-min --locked
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs::runtime_15_module_convention_gate_reports_non_render_debt_boundary
  - cargo test -p zircon_runtime --lib runtime_15_runtime_ui_dead_code_surface_is_test_support --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_owned_dead_code_suppression_cleanup --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_script_host_value_descriptors_do_not_suppress_dead_code --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_graphics_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_provider_boilerplate_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_facade_surface_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_dead_code_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_ui_host_surface_splits_production_frame_from_test_support --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_diagnostics_guard_is_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_core_framework_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_v2_asset_tests_are_folder_backed --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime --lib runtime_15_ui_shared_core_tests_are_folder_backed --no-default-features --features core-min --locked
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - cargo fmt --all --check
doc_type: structure-plan
status: in_progress
last_refined: 2026-08-03
---

# 15 · Runtime 代码结构与模块规范收束计划

## 2026-08-03 receipt-test hard cut

- Source implementation complete: receipt-only Rust trees and status mirrors are deleted; production structure guards remain mounted.
- Lifecycle ownership moved to numbered plan documents and coordinator/Python tooling; no Rust facade, archive fallback, alias, or shim remains.
- Local Python hard-cut and Runtime03 structure regressions pass 5/5; the guard now rejects every retired receipt/status route fragment, including the stale naming-boundary `status_evidence.rs` include; independent review is Critical/Important/Minor = `0/0/0`, while managed Runtime lib-test acceptance remains pending.
- open 待验证：[plan-status receipt test compile debt](15/failure-2026-08-02-plan-status-receipt-test-compile-debt.md)；源码硬切与二次审查已完成，受管 Runtime lib-test receipt 尚未返回。
- open 待验证：Runtime15 的 current-source manifest 已完整覆盖删除墓碑与存活编译输入；[Coordinator01 large-manifest stdin transport fixed return](../../mvp/00/fixed-2026-08-04-validation-ticket-large-manifest-cli-transport.md) 已解除 Windows command-line 长度限制，下一次 immutable snapshot 必须经该 transport 提交完整 manifest，不省略删除路径或恢复旧架构。精确计数与 SHA-256 只记录在不参与该 manifest 的 failure 记录中，避免状态文档自引用导致哈希漂移；Runtime lib-test 与上游审计仍待该刷新 snapshot 的 terminal evidence。
- Retained cross-plan returns: [depth-prepass source guard owner drift](../../zircon_editor/editor/02/fixed-2026-07-15-depth-prepass-source-guard-owner-drift.md) and [derived reflection hard-cut guard](../../zircon_plugins/08/fixed-2026-07-14-derived-reflection-hard-cut-guard.md).
- Retained Runtime15 returns: [manager service reactivation lifecycle](15/fixed-2026-07-14-manager-service-reactivation-lifecycle.md), [UI text manager cross-frame retention](15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md), and [UI text project asset manager consumer drift](15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md).

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-code-structure-module-conventions",
  "goal": "收敛 runtime façade、命名、测试组织、文件预算、prelude 与死代码边界，并以 folder-backed owner 和硬切守卫阻止旧架构回流。",
  "milestones": [
    {"id": "M1", "title": "façade 与可见性收束", "depends_on": []},
    {"id": "M2", "title": "命名与冗余前缀硬切", "depends_on": ["M1"]},
    {"id": "M3", "title": "测试组织单一规则", "depends_on": []},
    {"id": "M4", "title": "行数硬上限与 folder-backed owner 收口", "depends_on": ["M1", "M2", "M3"]},
    {"id": "M5", "title": "prelude 完整化与死代码清除", "depends_on": ["M1"]}
  ]
}
```

<!-- 机器 workflow 于 2026-07-17 late-adoption；M1-M3 的历史实现与验证仍由既有 child records 负责，本次只提交当前 M4.1 精确切片，不把 M4 或父计划整体标记完成。 -->

- [x] **M4.1 Screen-space UI text font-id report owner and canonical archive consumer hard cut.**

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](../frameworks/development-conventions.md)；本文保留 Runtime 结构收束主题的细节论证与执行上下文，不再作为并列规则源。


最新完成：`Runtime 15 M3 UI text pipeline test owner split` / `runtime_15_m3_ui_text_pipeline_test_owner_split_static_passed_cargo_deferred` 已删除旧 `zircon_runtime/src/ui/tests/text_pipeline` flat owner，并硬切为 folder-backed test tree。该记录的历史验证为：scoped rustfmt passed；focused `text_pipeline` cargo test 15/15 passed；direct `runtime_15_no_oversized_test_files` 1/1 passed；当时 `tests::runtime_absorption::structure_convention` 为 1226/1303 passed、77 failed remaining，剩余失败来自 status/root owner guard groups，不再包含 `text_pipeline` 或 oversized-test-file failure。2026-08-24 Text02 硬切随后物理删除无生产 consumer 的 `font_registry.rs` source/test owner；当前测试树为 `zircon_runtime/src/ui/tests/text_pipeline/{mod.rs,fixtures.rs,layout_request.rs,measure_cache.rs,render_extract_prewarm.rs,surface_cache.rs}`。该切片不保留旧 `text_pipeline.rs`、compat/shim/re-export；当前 Cargo gate 仍须走受管验证，不声明 package/workspace Cargo 全通过。

最新完成：`Runtime 15 M3/M4 shader variant miss report test owner split` / `runtime_15_shader_variant_miss_report_test_owner_split_source_complete_static_passed_managed_validation_deferred` 已把 `core/framework/render/shader/variant_miss_report.rs` 的 11 个内联测试硬切到 folder-backed `core/framework/render/shader/variant_miss_report/tests.rs`；生产 owner 从 1013 行收敛到 549 行，测试 owner 为 463 行，根文件测试属性 0、子 owner 测试属性 11、显式路径挂载 1。scoped rustfmt、所有权扫描与 diff check 已通过；全局静态镜像仍有 50 个既有生产文件达到或超过 800 行，受管 Cargo gate 仍待补，因此只关闭本 owner 的源码结构切片，不声明 Runtime 15 M3/M4 或全局预算完成。详情见 [`15/2026-08-27-shader-variant-miss-report-test-owner-split.md`](15/2026-08-27-shader-variant-miss-report-test-owner-split.md)。

最新完成：`Runtime 15 M3/M4 IBL source cubemap staging test owner split` / `runtime_15_ibl_source_cubemap_staging_test_owner_split_source_complete_static_passed_managed_validation_deferred` 已把 `asset/artifact/ibl_source_cubemap_staging.rs` 的 15 个内联测试硬切到 folder-backed `asset/artifact/ibl_source_cubemap_staging/tests.rs`；生产 owner 从 1414 行收敛到 776 行，测试 owner 为 638 行，根文件测试属性 0、子 owner 测试属性 15、显式路径挂载 1。scoped rustfmt、所有权扫描与 diff check 已通过；全局静态镜像当前仍有 50 个其它生产文件达到或超过 800 行，受管 Cargo gate 仍待补，因此只关闭本 owner 的源码结构切片，不声明 Runtime 15 M3/M4 或全局预算完成。详情见 [`15/2026-08-27-ibl-source-cubemap-staging-test-owner-split.md`](15/2026-08-27-ibl-source-cubemap-staging-test-owner-split.md)。

最新完成：`Runtime 15 M3/M4 frame hit-test owner split` / `runtime_15_frame_hit_test_owner_split_source_complete_static_passed_managed_validation_deferred` 已把 `ui/surface/frame_hit_test.rs` 中段的 6 个 projected-grid 测试硬切到 folder-backed `ui/surface/frame_hit_test/tests.rs`，并把测试源码守卫相对路径同步为 `../frame_hit_test.rs`；生产 owner 从 1066 行收敛到 757 行，测试 owner 为 308 行，测试模块之后的 2 个公开 debug hit-test 函数完整保留。scoped rustfmt、所有权/后续生产函数扫描与 diff check 已通过；本轮三个已拆 owner 均不再超预算，全局静态镜像仍有 49 个其它生产文件达到或超过 800 行，受管 Cargo gate 仍待补，因此不声明 Runtime 09/15 或全局预算完成。详情见 [`15/2026-08-27-frame-hit-test-owner-split.md`](15/2026-08-27-frame-hit-test-owner-split.md)。

最新完成：`Runtime 15 M3 current-child route plus IBL runtime writeback budget cleanup` / `runtime_15_m3_current_child_route_ibl_writeback_budget_cleanup_static_passed_cargo_deferred` 已把 ResourceStreamer diagnostics/material capture、asset mesh/asset pack、dynamic scene、editor/hub status row routes、module-layout 与 core scene ECS naming guards 对齐到 current child owners；`graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs` 从 948 行生产+测试混合文件硬切为 56 行 production route owner，runtime graph writeback 测试移入 `graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs`，seam/irradiance metrics 拆入 `graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests/metrics.rs`。验证：scoped rustfmt passed；focused structure guards 12/12 passed；production-file budget guard passed；`runtime_graph_writeback` 4/4 passed；当前源码全量 `tests::runtime_absorption::structure_convention` 经后续 UI text pipeline split 为 1226/1303 passed、77 failed remaining。该切片不保留旧 route mirror/compat/shim/re-export；Cargo gate deferred，不声明 package/workspace Cargo 全通过。




验证补记：standalone structure-convention harness 重新编译通过（327 个既有 unused/dead-code warning）；focused `typed_error_child_owners --test-threads=1` 通过 93/93；宽回归 `code_review_findings --test-threads=1` 通过 218/218；package/workspace Cargo 未声明通过。


验证补记：standalone structure-convention harness 重新编译通过（303 个既有 unused/dead-code warning）；focused `typed_error_status_doc --test-threads=1` 通过 51/51；package/workspace Cargo 未声明通过。
























最新完成：`Runtime 15 M3 core spine root/generated audit source sync` / `runtime_15_core_spine_root_generated_audit_source_sync_static_passed_cargo_deferred` 已把 Runtime 02 full Core Spine 审计的 root_surface/generated-code guard 来源同步到当前 folder-backed 子 owner：Python `core_spine_root_generated_boundary.py` 现在扫描 `root_surface/{public_surface,graphics_alias,docs}.rs`、`generated_code_guard/{markers,behavior,scope,delegation}.rs` 与 `core_spine_root_generated/mirror_docs.rs`；Rust 镜像 guard 的 `EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS` 与 `runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts` 同步改为读取 generated-code child owners。验证状态：scoped rustfmt passed，standalone `core_spine_root_generated` 2/2 passed，doc/status anchor scan passed，focused Python audit 返回 root_entries 13、root_surface 6、generated-code 7、guard_test_anchor_count 26、`missing_guard_test_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`。该切片只同步 Runtime 02/15 测试守卫审计来源，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 root entries module-families guard folder-backed split` / `runtime_15_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/root_entries/module_families.rs` 从 navigation、animation backlog、animation status JSON、root-seat 与 mirror-doc mixed guard owner 收束为 route owner；实际守卫拆入 `tests/runtime_absorption/root_entries/module_families/navigation.rs`、`animation_backlog.rs`、`animation_status_json.rs`、`root_seats.rs` 与 `mirror_docs.rs`，split 状态与跨文档锚点由 `tests/runtime_absorption/root_entries/module_families/split_layout.rs` 维护。新增 `runtime_15_root_entries_module_families_guard_is_folder_backed` 锁定子 owner 挂载、旧五个 Runtime 14/02 语义守卫不回流、line budgets、Runtime 02/14 Python audit 聚合路径、status row data、M3 status/date maps、Runtime 15/index/review/structure/module docs 与 session note；Frameworks mirror id: `frameworks_02_m3_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred`。验证状态：scoped rustfmt passed，standalone root_entries `module_families` 6/6，standalone structure-convention `root_entries` 1/1，scoped Python audit root_entries/module-family anchors passed；full Core Spine audit 仍有非本切片 root_surface/generated-code 风险未计通过。该切片只整理 runtime_absorption root entries 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 test file budget root-layout assertions guard folder-backed split` / `runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs` 从 root parent、asset、runtime/scene、render/status 与 UI 断言混合 owner 收束为 route owner；实际断言拆入 `structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs`、`structure_convention/test_file_budget/root_layout/folder_backed/assertions/asset_children.rs`、`structure_convention/test_file_budget/root_layout/folder_backed/assertions/runtime_scene_children.rs`、`structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_status_children.rs` 与 `structure_convention/test_file_budget/root_layout/folder_backed/assertions/ui_children.rs`。新增 `runtime_15_test_file_budget_root_layout_assertions_guard_is_folder_backed` 锁定二级 split、status row data、M3 status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/test-file-budget 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred due low D: space and active cargo/rustc lanes。














最新完成：`Runtime 15 M3 typed-error source inventory guard folder-backed split` / `runtime_15_typed_error_source_inventory_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs` 收束为 source-inventory route/helper owner，并把 source path 清单、source read 聚合、line budget、历史委派断言和状态镜像分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/reads.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/budgets.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/status_mirrors.rs`。旧 `runtime_15_typed_error_source_inventory_is_child_owner` 继续作为历史入口；新增 `runtime_15_typed_error_source_inventory_guard_is_folder_backed`、`runtime_15_typed_error_source_inventory_guard_folder_backed_status_is_current` 与 `runtime_15_typed_error_source_inventory_children_line_budgets_are_current` 锁定 folder-backed 拆分、状态镜像和预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory source helper child split` / `runtime_15_typed_error_source_inventory_source_helper_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs` 继续收束为挂载/转发 owner，并把 child-source 聚合、child inventory、路径/状态常量与本轮状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/source_helper_ownership.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/source_helper_status.rs`。新增 `runtime_15_typed_error_source_inventory_source_helpers_are_child_backed` 与 `runtime_15_typed_error_source_inventory_source_helper_status_is_current` 锁定 child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory child sources folder-backed split` / `runtime_15_typed_error_source_inventory_child_sources_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources.rs` 从 root source struct、source-helper child 聚合、delegation child 聚合和 source blob helper 混合 owner 收束为 route/test-entry owner。root source struct 与 direct child reads 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/root_sources.rs`，source-helper/child-inventory source 聚合拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/source_helper_sources.rs`，delegation/folder-backed source 聚合拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/delegation_sources.rs`，blob helper 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/source_blobs.rs`，结构/状态守卫拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/structure_guard.rs`。新增 `runtime_15_typed_error_source_inventory_child_sources_are_folder_backed` 锁定 child_sources folder-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory child sources structure guard child split` / `runtime_15_typed_error_source_inventory_child_sources_structure_guard_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/structure_guard.rs` 从 child_sources route ownership、跨文档状态镜像和预算混合 owner 收束为 route/test-entry owner。父路由与不回流检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/structure_guard/route_ownership.rs`，状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/structure_guard/status_mirrors.rs`，预算检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_sources/structure_guard/budgets.rs`。新增 `runtime_15_typed_error_source_inventory_child_sources_structure_guard_is_child_backed` 锁定 structure guard child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory child inventory folder-backed split` / `runtime_15_typed_error_source_inventory_child_inventory_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory.rs` 从 root/source-helper/delegation/folder-backed/ownership child group 清单混合 owner 收束为 route/child-list owner，并把各组清单拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/root_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/source_helper_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/delegation_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/folder_backed_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/folder_backed_ownership_children.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/status_current.rs`。新增 `runtime_15_typed_error_source_inventory_child_inventory_is_folder_backed` 锁定 child inventory folder-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory child inventory status-current child split` / `runtime_15_typed_error_source_inventory_child_inventory_status_current_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/status_current.rs` 从 route ownership、跨文档状态镜像和预算混合 owner 收束为 route/test-entry owner，并把实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/status_current/route_ownership.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/status_current/status_mirrors.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/child_inventory/status_current/budgets.rs`。新增 `runtime_15_typed_error_source_inventory_child_inventory_status_current_is_child_backed` 锁定 status-current child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory metadata status-current child split` / `runtime_15_typed_error_source_inventory_metadata_status_current_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current.rs` 从 route、状态镜像、source blob 聚合与预算检查混合 owner 收束为 route/test-entry owner。metadata parent ownership 检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current/route_ownership.rs`，状态文档镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current/status_mirrors.rs`，source blob 聚合拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current/source_blobs.rs`，预算检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current/budgets.rs`。新增 `runtime_15_typed_error_source_inventory_metadata_status_current_is_child_backed` 锁定 status-current child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory metadata child split` / `runtime_15_typed_error_source_inventory_metadata_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata.rs` 从路径常量、状态切片常量、guard 常量与 status-map 路径混合 owner 收束为 metadata route/export owner。root/source helper 路径拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/root_paths.rs`，child-inventory 路径拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/child_inventory_paths.rs`，delegation/folder-backed 路径拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/delegation_paths.rs`，状态切片拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_slices.rs`，review map 路径拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/review_guard_paths.rs`，当前状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/metadata/status_current.rs`。新增 `runtime_15_typed_error_source_inventory_metadata_is_child_backed` 锁定 metadata child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory delegation child split` / `runtime_15_typed_error_source_inventory_delegation_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation.rs` 收束为 28 行 route/helper owner，并把 typed-error structure parent delegation、source-inventory parent mounts、path/read ownership、folder-backed body 与状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation/parent_delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation/source_inventory_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation/source_ownership.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation/folder_backed.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory/delegation/status_current.rs`。旧 `runtime_15_typed_error_source_inventory_guard_is_folder_backed` 继续作为 wrapper；新增 `runtime_15_typed_error_source_inventory_delegation_is_child_backed` 与 `runtime_15_typed_error_source_inventory_delegation_status_is_current` 锁定 child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split` / `runtime_15_typed_error_status_doc_mirrors_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors.rs` 收束为 route/helper owner，并把 status slice anchors、source path anchors、guard anchors 与当前状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/status_slices.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/source_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/guard_anchors.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/doc_mirrors/status_current.rs`。旧 `assert_typed_error_status_doc_mirrors_are_synced` 继续作为 helper；新增 `runtime_15_typed_error_status_doc_mirrors_are_folder_backed` 与 `runtime_15_typed_error_status_doc_mirrors_folder_backed_status_is_current` 锁定 folder-backed 拆分和状态镜像。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error status-doc source helper child split` / `runtime_15_typed_error_status_doc_source_helper_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs` 收束为挂载/转发 owner，并把 child-source 聚合、路径/状态常量、status-row/doc source 聚合与本轮状态镜像拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/child_sources.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/sources.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/source_helper_ownership.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs/source_helper_status.rs`。新增 `runtime_15_typed_error_status_doc_source_helpers_are_child_backed` 与 `runtime_15_typed_error_status_doc_source_helper_status_is_current` 锁定 child-backed 拆分、状态镜像和父/子预算；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error child-ownership guard folder-backed split` / `runtime_15_typed_error_child_ownership_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership.rs` 收束为 typed-error child-ownership route/source owner，实际断言拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/budgets.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/review_guards.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/status_mirrors.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/structure_subtree.rs`。旧 `runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner` 继续保留为历史入口，新增 `runtime_15_typed_error_child_ownership_guard_is_folder_backed` 与 `runtime_15_typed_error_child_ownership_guard_folder_backed_status_is_current` 锁定 folder-backed 拆分和状态镜像；只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error child-ownership root inventory child split` / `runtime_15_typed_error_child_ownership_root_inventory_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership.rs` 从 route/source owner 进一步收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership/root_inventory.rs`。守卫 `runtime_15_typed_error_child_ownership_root_inventory_is_child_owned` 锁定 typed-error row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新验证：typed-error child-ownership root inventory scoped rustfmt/static scans passed；父/子行数为 38/75/60/71/68/90/38/19/57/98/88；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。Cargo 未启动新门禁，因为外部 zircon_runtime render-product cargo/rustc 车道仍在运行，因此本切片仍为 Cargo gate deferred。

最新完成：`Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split` / `runtime_15_plugin_importer_dx_status_docs_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs.rs` 收束为 status-doc route/helper owner，长文档锚点、status/date map 锚点和当前状态镜像分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs/doc_mirrors.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs/status_maps.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs/status_mirrors.rs`。旧 `runtime_15_plugin_importer_dx_status_docs_are_child_owner` 继续保留 status-doc child-owner 锚点，新增 `runtime_15_plugin_importer_dx_status_docs_are_folder_backed` 与 `runtime_15_plugin_importer_dx_status_docs_folder_backed_status_is_current` 锁定 folder-backed 拆分和状态镜像；只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings status-doc status anchors folder-backed split` / `runtime_15_code_review_findings_status_docs_status_anchors_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors.rs` 收束为 status-anchor route/constant-forwarding owner，长锚点数组拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors/child_anchors.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors/map_anchors.rs`。新增 `runtime_15_code_review_findings_status_docs_status_child_anchors_are_child_owned`、`runtime_15_code_review_findings_status_docs_status_map_anchors_are_child_owned` 与 `runtime_15_code_review_findings_status_docs_status_anchors_are_folder_backed`，只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split` / `runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs` 收束为 status-anchor guard route/helper owner，并把历史 child-owner 断言、folder-backed 拆分检查、line budget 与状态镜像分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/folder_backing.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/status_mirrors.rs`。旧 `runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner` 与 `runtime_15_code_review_findings_status_docs_status_anchors_are_folder_backed` 继续作为迁移后的子守卫；新增 `runtime_15_code_review_findings_status_docs_status_anchor_guard_is_folder_backed`、`runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_status_docs_status_anchor_guard_children_line_budgets_are_current` 锁定 folder-backed 拆分、状态镜像和父/子预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings structure guard children budget-status child split` / `runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs` 收束为预算/状态子守卫的 route owner，实际行数预算下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets/line_counts.rs`，状态镜像下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets/status_mirrors.rs`。新增 `runtime_15_code_review_findings_structure_guard_children_budget_status_is_child_owned`、`runtime_15_code_review_findings_structure_guard_children_line_budgets_are_child_owned` 与 `runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_status_is_current`，只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 F7 artifact cache JSON number typed errors` / `runtime_15_artifact_cache_json_number_typed_errors_static_passed_cargo_deferred` 已把 `asset/artifact/cache_payload/json_value.rs` 的 cached JSON number restore 从 `.expect(...)` 收敛到 `AssetImportError::CachedJsonNonFiniteNumber` / `AssetImportError::CachedJsonNumberParse` typed boundary。`ArtifactCacheJsonValue::into_json(...)`、`cache_table_to_json(...)`、Data/Scene/Prefab artifact cache restore 现在向上传播 `AssetImportError`，array/table/object restore 使用 `collect::<Result<Vec<_>, _>>()?`，不再把非法或不可解析 JSON number 变成 panic。

最新验证：artifact cache JSON number scoped rustfmt --check 通过；direct static scans 确认 `asset/artifact/cache_payload/json_value.rs` 的 `.unwrap()` / `.expect(` 命中数为 0，foundation row-data 为 21/23/18/11、合计 73，五份优先文档旧 72-row 锚点命中数为 0，F7 状态锚覆盖 8 处。Standalone rustc guards 与 Cargo package gate 因外部 cargo/rustc 编译车道 active deferred，本轮不声明 workspace Cargo 通过。

最新完成：`Runtime 15 M4 shader prewarm owner guard sync` / `runtime_15_shader_prewarm_owner_guard_sync_static_passed_cargo_deferred` 已把 shader-prewarm build-tool、plugin descriptor、runtime staged-cache 与 product staged-cache 结构守卫同步到当前 owner 边界：`tools/zircon_build.py` 只保留 orchestrator import/call/pass-through，`tools/zircon_build_plugin_assets.py` 承接 `distribution.assets` asset-root normalization，`tools/zircon_build_plugin_shader_descriptors.py` 承接 `geometry_sources` / `shading_models` descriptor parsing，`tools/zircon_build_plugin_packages.py` 保留 plugin package DTO 字段，运行时 staged-cache 断言读取 `mesh_pipeline_cache/ensure_pipeline/tests.rs`，产品 staged-cache 断言锁定 `include_content_hashes.contains(request_source_hash)` 语义而不是旧的一行赋值锚点。

最新完成：`Runtime 15 M4 deferred GBuffer template output guard sync` / `runtime_15_deferred_gbuffer_template_output_guard_sync_static_passed_cargo_deferred` 已把 Deferred GBuffer template 守卫拆为共享 surface type owner 与 entry template owner：`zr_surface_types.wgsl` 锁定 `ZrDeferredGBufferOutput` 及 albedo/normal/material location layout，`zr_template_deferred_gbuffer.wgsl` 只锁定 alpha clip 与 `encode_gbuffer(surface, zr_build_shading_context(input))` 委托。该同步不改变 WGSL 输出结构、GBuffer pipeline、mesh cache key 或 WGPU 行为。

最新验证：scoped rustfmt passed；standalone `structure_convention.rs` compiled cleanly；8 个 shader-prewarm plugin/registry/product staged-cache guards each passed 1/1；`runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned` passed 1/1；full standalone `structure_convention_current_refresh.exe --format terse` passed 622/622。Cargo/WGPU/RenderDoc/product gates 仍按 Runtime 15/Plan 08 owning lanes deferred，本轮不声明 workspace Cargo 通过。

最新完成：`Runtime 15 M4 shader prewarm manifest path helper owner split` / `runtime_15_shader_prewarm_manifest_path_helpers_owner_split_static_passed_cargo_deferred` 已把 `bin/zircon_shader_prewarm/manifest.rs` 中的 asset-root WGSL path discovery、primary `.zshader` lookup 与 content hash helper 下沉到 `bin/zircon_shader_prewarm/manifest/paths.rs`。父 manifest 继续负责 manifest assembly、source/material expansion、registry/resource overlay 和 tests mount；`paths.rs` 现在承接 `wgsl_files_for_document(...)`、`primary_zshader_path(...)`、`content_hash(...)` 与 `ShaderSourceOutsidePackageDir` path-boundary diagnostics。该切片只整理 shader prewarm CLI manifest owner 边界，不改变 manifest 输出语义、cache key、registry overlay、WGPU validation 或产品预热路径。


最新完成：`Runtime 15 M4 SDF atlas/render tests folder-backed guard sync` / `runtime_15_sdf_atlas_render_tests_folder_backed_guard_sync_static_passed_cargo_deferred` 已把 SDF atlas/render 的 Runtime 15 守卫同步到当前 folder-backed owners：`graphics/scene/scene_renderer/ui/sdf_atlas/tests/{mod.rs,plan.rs,allocation.rs,cache_report.rs,owner.rs}` 与 `graphics/scene/scene_renderer/ui/sdf_render/tests/{mod.rs,draw_plan.rs,shader_contract.rs,layout_placement.rs,prepare_report.rs}`。本轮同时关闭旧守卫暴露出的 SDF render 父 owner 预算问题，把 glyph-run resolution、SDF vertex assembly、glyph frame placement、UV clipping 与 NDC conversion 下沉到 `graphics/scene/scene_renderer/ui/sdf_render/vertices.rs`；`sdf_render.rs` 保留 renderer lifecycle、WGPU pipeline/atlas resource/upload、prepare report 和 test/module mounts。该切片只做结构收束，不改变 atlas allocation、render planning 行为、shader 语义、UI text routing 或产品提交路径。


最新完成：`Runtime 15 M3 typed-error moved-guard absence guard folder-backed split` / `runtime_15_typed_error_moved_guard_absence_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs` 收束为 moved-guard absence 的 route/helper owner，并把实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/preserved_guards.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/parent_backflow.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/path_anchors.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence/status_mirrors.rs`。新增 `runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned`、`runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned`、`runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned`、`runtime_15_typed_error_moved_guard_absence_children_line_budgets_are_current` 与 `runtime_15_typed_error_moved_guard_absence_guard_folder_backed_status_is_current`，并保留 `runtime_15_typed_error_structure_moved_guard_absence_is_child_owner` 作为 structure_assertions mount anchor。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。


最新完成：`Runtime 15 M3 code review findings structure guard typed-error folder-backed split` / `runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error.rs` 收束为 typed-error structure guard 的 route/helper owner，并把实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/top_level.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/status_mirrors.rs`。新增 `runtime_15_code_review_findings_structure_guard_typed_error_top_level_checks_are_child_owned`、`runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned`、`runtime_15_code_review_findings_structure_guard_typed_error_children_line_budgets_are_current` 与 `runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_status_is_current`，并继续通过 `runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner` 挂到 `structure_guard_children.rs`。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。




最新完成：`Runtime 15 M3 typed-error structure guard folder-backed split` / `runtime_15_typed_error_structure_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs` 收束为 route/path inventory owner，并把实际 guard 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_mirrors.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/budgets.rs`。该 top-level child tree 继续挂载 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/child_ownership.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/status_mirrors.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs`。historical `Runtime 15 M3 code review findings typed-error structure guard child-owner split` / `runtime_15_code_review_findings_typed_error_structure_guard_child_owner_split_static_passed_cargo_deferred` 和 `runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner` 迁入 child_ownership child；新增 `runtime_15_typed_error_structure_guard_is_folder_backed`、`runtime_15_typed_error_structure_guard_folder_backed_status_is_current` 与 `runtime_15_typed_error_structure_guard_budgets_are_focused`。状态镜像继续锁定 `tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs`、`tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs`、`tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs`、`tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs`、`tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs`、`review_f5_texture_loader_uses_typed_error`、`review_f5_mesh_loader_and_obj_decoder_use_typed_errors`、`review_f5_asset_authoring_uses_typed_error`、`review_f5_native_plugin_descriptor_abi_uses_typed_error`、`review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary` 与 `review_f7_asset_artifact_errors_use_asset_import_error_sources`。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。



最新完成：`Runtime 15 M3 plugin-importer DX structure guard root inventory child split` / `runtime_15_plugin_importer_dx_structure_guard_root_inventory_child_split_target_server_direct_binary_passed` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs` 从 route/path inventory owner 收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/root_inventory.rs`。守卫 `runtime_15_plugin_importer_dx_structure_guard_root_inventory_is_child_owned` 锁定 plugin-importer DX row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；target-server direct binary passed，broad runtime/root/plugin integration 仍 pending。

最新验证：plugin-importer DX structure guard root inventory scoped rustfmt/static scans passed；父/root 子行数为 47/51/28/67/29/85，existing top-level child 行数为 67/103/84/74/131/37/89 并继续使用既有 child-owner budget；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。同步修正 recent root-inventory delegation guards，使 folder-backed status anchors 从 `root_statuses` children 读取而非回流 parent route file。Cargo 未启动新门禁，因为外部 cargo/rustc 车道仍在运行，因此本切片仍为 Cargo gate deferred。


最新完成：`Runtime 15 M3 P0 robustness root inventory child split` / `runtime_15_p0_robustness_root_inventory_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners.rs` 从 route/path inventory owner 进一步收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_child_owners/root_inventory.rs`。守卫 `runtime_15_p0_robustness_root_inventory_is_child_owned` 锁定 P0 robustness row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新验证：P0 robustness root inventory scoped rustfmt/static scans passed；父/子行数为 29/57/111/72/26/45/26/45/89/81；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。Cargo 未启动新门禁，本切片按 Runtime 15 implementation-slice cadence 仍为 Cargo gate deferred，不声明 Cargo pass。


最新完成：`Runtime 15 M3 late API cleanup root inventory child split` / `runtime_15_late_api_cleanup_root_inventory_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners.rs` 从 route/path inventory owner 进一步收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/late_api_cleanup_child_owners/root_inventory.rs`。守卫 `runtime_15_late_api_cleanup_root_inventory_is_child_owned` 锁定 late API cleanup row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新验证：late API cleanup root inventory scoped rustfmt/static scans passed；父/子行数为 29/66/104/58/26/41/26/56/86/87；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。Cargo 未启动新门禁，因为外部 zircon_runtime cargo/rustc 车道仍在运行，因此本切片仍为 Cargo gate deferred。

最新完成：`Runtime 15 M3 P0 native fixture leaf-owner root inventory child split` / `runtime_15_p0_native_fixture_leaf_owner_root_inventory_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners.rs` 从 route/path inventory owner 收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_native_fixture_leaf_owners/root_inventory.rs`。守卫 `runtime_15_p0_native_fixture_leaf_owner_root_inventory_is_child_owned` 锁定 P0 native fixture row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新验证：P0 native fixture leaf-owner root inventory scoped rustfmt/static scans passed；父/子行数为 25/55/52/72/27/38/29/48/29/85；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。Cargo 未启动新门禁，因为外部 cargo/rustc 车道仍在运行，因此本切片仍为 Cargo gate deferred。


最新完成：`Runtime 15 M3 F8 child-owner root inventory child split` / `runtime_15_f8_child_owner_root_inventory_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs` 从 route/path inventory owner 进一步收束为 root route/helper owner。root paths、root statuses、child-row inventory、source readers 与 root inventory guard 分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/root_paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/root_statuses.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/root_child_rows.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/root_sources.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/root_inventory.rs`。守卫 `runtime_15_f8_child_owner_root_inventory_is_child_owned` 锁定 F8 row data、M3 review status/date maps、Runtime 15/index/review/structure/module docs 与 session note；该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 F8 route ownership guard child split` / `runtime_15_f8_route_ownership_guard_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership.rs` 从 142 行父路由、descriptor builder 路由、descriptor privacy 路由、leaf-owner assertions 与状态镜像混合 owner 收束为 route/helper owner。实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/parent_routes.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/descriptor_builder_routes.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/descriptor_privacy_routes.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/leaf_owners.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/child_ownership.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership/status_mirrors.rs`。旧 `runtime_15_f8_api_convergence_review_guards_are_child_owners` 保留为父 route wrapper；新增 `runtime_15_f8_route_ownership_guard_is_child_backed` 与 `runtime_15_f8_route_ownership_status_mirrors_are_current` 锁定 child-backed 拆分、status rows/maps/docs/session 镜像与 child budgets。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新验证：F8 child-owner root inventory scoped rustfmt/static scans passed；父/子行数为 25/57/147/68/26/44/26/46/93/81；parent-backflow、root inventory anchor parity、row-data/status-date map、文档锚点、冲突标记、尾随空白与 scoped `git diff --check` 复核通过，`git diff --check` 仅报告 LF/CRLF normalization warnings。Cargo 未启动新门禁，因为外部 zircon_runtime render-product cargo/rustc 车道仍在运行，因此本切片仍为 Cargo gate deferred。







































最新完成：`Runtime 15 M3 priority plan docs moved guard path mirror` / `runtime_15_priority_plan_docs_moved_guard_path_mirror_static_passed_cargo_deferred` 新增 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current`，要求两份优先计划文档、Runtime 15 子计划、runtime index、module-convention、session note 与 status row data 不再保留 `priority_plan_docs.rs` 旧父文件函数锚点，并镜像 full priority-plan-doc moved guard inventory：`priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current`、`priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current`、`priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current`、`priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries`、`priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete`、`priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked`、`priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed`、`priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory`、`priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed`、`priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current` 与 `priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory`。该切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档 moved guard path 镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。




最新完成：`Runtime 15 M3 priority plan docs guard-test child-owner split` / `runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs` 收束为 nested route owner，并把实际守卫拆入 full priority-plan-doc guard-test child inventory：`structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs` 与 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs`。新增 `runtime_15_priority_plan_docs_guard_test_children_are_folder_backed` 锁定二级 child owner 挂载、行数预算、status row data 与 status/date maps；该切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs guard child-owner split` / `runtime_15_priority_plan_docs_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/priority_plan_docs.rs` 收束为 route/shared-helper owner，并把具体守卫拆入 full priority-plan-doc child inventory：`structure_convention/test_file_budget/priority_plan_docs/code_paths.rs`、`structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs`、`structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs`、`structure_convention/test_file_budget/priority_plan_docs/header_sections.rs`、`structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs` 与 `structure_convention/test_file_budget/priority_plan_docs/test_paths.rs`。新增 `runtime_15_priority_plan_docs_guard_children_are_folder_backed` 锁定父/子 owner 挂载、行数预算、status row data 与 status/date maps；该切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs guard-test listing guard` / `runtime_15_priority_plan_docs_guard_test_listing_guard_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed`，要求两份优先计划文档的 `tests:` 头部显式列出 full priority-plan-doc guard inventory：`priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current`、`priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current`、`priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current`、`priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries`、`priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete`、`priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked`、`priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed`、`priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory`、`priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed`、`priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current` 与 `priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory`。该切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档测试证据列表守卫，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs plan-source cross-link guard` / `runtime_15_priority_plan_docs_plan_source_cross_link_guard_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked`，要求两份优先计划文档的 `plan_sources:` 都保留 `user:` 源头、`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md` Runtime 15 子计划来源，以及彼此的 companion priority plan source。`docs/plans/engine-code-structure-convention.md` 现在显式引用 `docs/plans/engine-code-review-findings-2026-06.md` 与本 Runtime 15 子计划；review findings 的首条来源改为 `user:`。本切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档来源链路守卫，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs required header sections guard` / `runtime_15_priority_plan_docs_required_header_sections_guard_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete`，要求 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md` 的 YAML frontmatter 按机器可读 lookup 顺序保留 `related_code:`、`implementation_files:`、`plan_sources:`、`tests:`、`doc_type:`、`status:`，并要求四个列表 section 非空。本切片补齐 review findings 缺失的 `implementation_files:`，同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档 required header 守卫，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs frontmatter status guard` / `runtime_15_priority_plan_docs_frontmatter_status_guard_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current`，锁定 `docs/plans/engine-code-structure-convention.md` 的 `doc_type: convention-authority`、`docs/plans/engine-code-review-findings-2026-06.md` 的 `doc_type: review-findings`，并要求两份优先计划文档在仍记录 Runtime 15 Cargo-deferred 实施切片时保持 `status: in_progress`。本切片同步 Runtime 15 子计划、runtime index、结构规范、review findings、module-convention docs、session note、status row data 与 status/date maps；只整理优先计划文档 frontmatter lifecycle 守卫，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。



最新完成：`Runtime 15 M3 runtime services lock-poison guard child-owner split` / `runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/lock_poison_policy/runtime_services.rs` 中的 plugin bridge、dynamic API/session/spawn/ECS parallel executor、navigation/resource manager 守卫拆入 `structure_convention/lock_poison_policy/runtime_services/plugin_bridge.rs`、`structure_convention/lock_poison_policy/runtime_services/dynamic_scene.rs` 与 `structure_convention/lock_poison_policy/runtime_services/navigation_resource.rs`。父 owner 只保留 child mount 与 `runtime_15_runtime_services_lock_poison_guard_child_owner_split` 布局守卫；`runtime_15_lock_poison_policy_guard_is_folder_backed` 同步升级为四层 child-owner 计数与路径预算守卫。该切片只整理 structure-convention 测试守卫、status row data、status/date maps 与文档镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 asset/render/input lock-poison guard child-owner split` / `runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/lock_poison_policy/asset_render_input.rs` 中的 asset project/worker、WGPU framework、animation/input、script VM registry 与 ZrVM runtime-lock 守卫拆入 `structure_convention/lock_poison_policy/asset_render_input/asset_pipeline.rs`、`structure_convention/lock_poison_policy/asset_render_input/render_animation.rs` 与 `structure_convention/lock_poison_policy/asset_render_input/input_script.rs`。父 owner 只保留 `mod asset_pipeline;`、`mod input_script;`、`mod render_animation;` 和 `runtime_15_asset_render_input_lock_poison_guard_child_owner_split` 布局守卫；`runtime_15_lock_poison_policy_guard_is_folder_backed` 同步升级为三层 child-owner 计数与路径预算守卫。该切片只整理 structure-convention 测试守卫、status row data、status/date maps 与文档镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。


最新完成：`Runtime 15 M1 graphics facade visibility review findings mirror` / `runtime_15_graphics_facade_visibility_review_findings_mirror_static_passed_cargo_deferred` 已补齐 `docs/plans/engine-code-review-findings-2026-06.md` 中缺失的 `Runtime 15 graphics facade visibility note` / `runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift` 镜像。`runtime_15_mixed_visibility_has_facade_note` 现在同时读取 review findings，新增 `runtime_15_graphics_facade_visibility_review_findings_mirror_is_recorded` 锁定 Runtime 15/index/review/structure/module docs、status rows/maps 与 session note 同步。该切片只补文档/guard 同步，不改 graphics/runtime 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings status-doc status anchors child-owner split` / `runtime_15_code_review_findings_status_docs_status_anchors_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs` 中的 code-review findings 状态切片/状态码/owner/guard 长锚点数组下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors.rs`。父 status-doc 守卫现在只保留 source 读取、status/date maps、session note 与 `status_anchors::{STATUS_DOC_CHILD_ANCHORS,STATUS_DOC_MAP_ANCHORS,STATUS_DOC_SESSION_ANCHORS}` 调度；新 `runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner` 锁定 status anchors 不回流、父/子 800 行预算、status rows/maps 与五份文档镜像。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 owner-budget mirror-docs child-owner split` / `runtime_15_runtime_07_owner_budget_mirror_docs_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/owner_budget.rs` 中的 Runtime 07 performance-hotpath mirror-doc audit 下沉到 `tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs`。父 owner-budget 守卫现在只保留 Runtime 15 performance hotspot folder split 自检和 `large_file_gate` / `mirror_docs` / `virtual_geometry_debug_snapshot` 三个子 owner 挂载；`performance_hotpath_source_inventory.py` 同步把 Runtime 07 `expected_test_file_count = 14`，新增 `performance_hotspots/owner_budget/{large_file_gate,mirror_docs,virtual_geometry_debug_snapshot}.rs` 三个 audit 输入。新增 `runtime_15_runtime_07_owner_budget_mirror_docs_is_child_owner` 锁定 mirror-docs guard 不回流、父/子 800 行预算、status rows/maps、Runtime 07 当前审计锚与五份文档镜像。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 submit-context guard child-owner split` / `runtime_15_runtime_07_submit_context_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/submit_context.rs` 从单个 462 行混合守卫收束为 route owner；source 读取下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/sources.rs`，共享 extract payload 断言下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/source_extract_payloads.rs`，camera-loop sharing 断言下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/camera_loop_sharing.rs`，feedback sideband 断言下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/feedback_sidebands.rs`，状态文档锚点下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/status_docs.rs`，结构自检下沉到 `tests/runtime_absorption/performance_hotspots/submit_context/split_layout.rs`。`performance_hotpath_source_inventory.py` 当前同步为 `expected_test_file_count = 20`；新增 `runtime_15_runtime_07_submit_context_guard_child_owner_split` 锁定 route/child owner、status rows/maps、Runtime 07/15 文档和 session note。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 hotspot-inventory guard child-owner split` / `runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs` 从单个 446 行混合守卫收束为 route owner；source 读取下沉到 `tests/runtime_absorption/performance_hotspots/hotspot_inventory/sources.rs`，evidence-gate/doc/counter-hotspot 断言下沉到 `tests/runtime_absorption/performance_hotspots/hotspot_inventory/evidence_gate_docs.rs`，ECS/extract/asset/animation counter 断言下沉到 `tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs`，profiling/trace/render diversion 断言下沉到 `tests/runtime_absorption/performance_hotspots/hotspot_inventory/profiling_trace_render.rs`，结构自检下沉到 `tests/runtime_absorption/performance_hotspots/hotspot_inventory/split_layout.rs`。`performance_hotpath_source_inventory.py` 当前同步为 `expected_test_file_count = 25`；新增 `runtime_15_runtime_07_hotspot_inventory_guard_child_owner_split` 锁定 route/child owner、status rows/maps、Runtime 07/15 文档和 session note。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 owner-budget guard folder-backed split` / `runtime_15_runtime_07_owner_budget_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/owner_budget.rs` 从 359 行 mixed self-check owner 收束为 route/test-entry owner；parent layout、child route、source inventory、line budget 与 status-doc checks 分别拆入 `tests/runtime_absorption/performance_hotspots/owner_budget/{parent_routes,child_routes,source_inventory,line_budgets,status_docs}.rs`，共享 source loader 和 split-layout 自检分别落在 `owner_budget/sources.rs` 与 `owner_budget/split_layout.rs`。精确子文件锚点包含 `tests/runtime_absorption/performance_hotspots/owner_budget/parent_routes.rs`。`performance_hotpath_source_inventory.py` 当前同步为 `expected_test_file_count = 32`；新增 `runtime_15_runtime_07_owner_budget_guard_folder_backed_split` 锁定二级 folder-backed owner、moved assertion block 不回流、status rows/maps、Runtime 07/15 文档和 session note。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 artifact/render diagnostics guard child-owner split` / `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits.rs` 从 artifact cache payload 与 render product diagnostics 混合守卫收束为 route owner；artifact cache payload 断言下沉到 `tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/artifact_cache_payload.rs`，render product diagnostics 断言下沉到 `tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/render_product_diagnostics.rs`，结构自检下沉到 `tests/runtime_absorption/performance_hotspots/artifact_render_diagnostics_splits/split_layout.rs`。`performance_hotpath_source_inventory.py` 当前同步为 `expected_test_file_count = 35`；新增 `runtime_15_runtime_07_artifact_render_diagnostics_guard_child_owner_split` 锁定 route/child owner、status rows/maps、Runtime 07/15 文档和 session note。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 scene/project guard child-owner split` / `runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/scene_project_splits.rs` 从 scene asset、project I/O 与 dynamic session event 混合守卫收束为 route owner；scene asset 断言下沉到 `tests/runtime_absorption/performance_hotspots/scene_project_splits/scene_asset.rs`，project I/O 断言下沉到 `tests/runtime_absorption/performance_hotspots/scene_project_splits/project_io.rs`，dynamic session event 断言下沉到 `tests/runtime_absorption/performance_hotspots/scene_project_splits/dynamic_session_event.rs`，结构自检下沉到 `tests/runtime_absorption/performance_hotspots/scene_project_splits/split_layout.rs`。`performance_hotpath_source_inventory.py` 当前同步为 `expected_test_file_count = 39`；新增 `runtime_15_runtime_07_scene_project_guard_child_owner_split` 锁定 route/child owner、status rows/maps、Runtime 07/15 文档和 session note。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 owner-budget large-file gate child-owner split` / `runtime_15_runtime_07_owner_budget_large_file_gate_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/owner_budget.rs` 中的 Runtime 07 large-file owner-budget audit gate 下沉到 `tests/runtime_absorption/performance_hotspots/owner_budget/large_file_gate.rs`。父 owner-budget 守卫现在只保留 performance-hotpath mirror docs、Runtime 15 performance hotspot folder split 自检，以及 `large_file_gate` / `virtual_geometry_debug_snapshot` 子 owner 挂载；mirror-docs 守卫显式读取两个子 owner，避免已拆分测试锚点从 Runtime 07 边界扫描里消失。新增 `runtime_15_runtime_07_owner_budget_large_file_gate_is_child_owner` 锁定 large-file gate 不回流、父/子 800 行预算、status rows/maps 与五份文档镜像。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 Runtime 07 owner-budget virtual-geometry guard child-owner split` / `runtime_15_runtime_07_owner_budget_virtual_geometry_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/performance_hotspots/owner_budget.rs` 中的 virtual-geometry debug snapshot folder-backed 合约检查下沉到 `tests/runtime_absorption/performance_hotspots/owner_budget/virtual_geometry_debug_snapshot.rs`。父 owner-budget 守卫现在只保留 Runtime 07 large-file owner budget、performance-hotpath mirror docs 和 Runtime 15 performance hotspot folder split 自检；新 `runtime_15_runtime_07_owner_budget_virtual_geometry_guard_is_child_owner` 锁定 virtual-geometry 守卫不回流、父/子 800 行预算、status rows/maps 与五份文档镜像。该切片只整理 performance-hotspots 测试守卫 owner，不改 runtime/render/editor/text 生产代码；Cargo gate deferred。


最新完成：`Runtime 15 M3 code review findings F12 direct assertions child-owner split` / `runtime_15_code_review_findings_f12_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs` 中的 F12 dead-code direct source check 下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs`。`direct_review_assertions.rs` 现在只保留 F12/F8/P0/render/root-parent helper delegation 与行数预算；新 `runtime_15_code_review_findings_f12_direct_assertions_are_child_owner` 锁定 F12 child mount、dead-code review guard anchor、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split` / `runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/review_guard.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/status_mirrors.rs`。旧 `runtime_15_code_review_findings_f12_direct_assertions_are_child_owner` 继续作为历史入口；新增 `runtime_15_code_review_findings_f12_direct_assertions_guard_is_folder_backed`、`runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_f12_direct_assertions_children_line_budgets_are_current` 锁定 F12 direct assertions folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings render direct assertions child-owner split` / `runtime_15_code_review_findings_render_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs` 中的 render structure F16 direct source check 下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs`。`direct_review_assertions.rs` 现在保留 F12 direct check、F8/P0/render/root-parent helper delegation 与行数预算；新 `runtime_15_code_review_findings_render_direct_assertions_are_child_owner` 锁定 render child mount、F16 review guard anchor、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings render direct assertions guard folder-backed split` / `runtime_15_code_review_findings_render_direct_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/review_guard.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render/status_mirrors.rs`；`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs` 同步读取 render nested child source blob。旧 `runtime_15_code_review_findings_render_direct_assertions_are_child_owner` 继续作为历史入口；新增 `runtime_15_code_review_findings_render_direct_assertions_guard_is_folder_backed`、`runtime_15_code_review_findings_render_direct_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_render_direct_assertions_children_line_budgets_are_current` 锁定 render direct assertions folder-backed 拆分、`assert_render_direct_sources_are_folder_backed` route/helper 入口、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings root-parent direct assertions child-owner split` / `runtime_15_code_review_findings_root_parent_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs` 中的 code-review findings root parent mount 与 moved-test absence 检查下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs`。`direct_review_assertions.rs` 现在保留 render/F12 direct checks、F8/P0/root-parent helper delegation 与行数预算；新 `runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner` 锁定 root parent child mount、moved-review guard absence anchors、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split` / `runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/parent_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/backflow.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent/status_mirrors.rs`。旧 `runtime_15_code_review_findings_root_parent_direct_assertions_are_child_owner` 继续作为历史入口；新增 `runtime_15_code_review_findings_root_parent_direct_assertions_guard_is_folder_backed`、`runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_root_parent_direct_assertions_children_line_budgets_are_current` 锁定 root-parent direct assertions folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings F8 direct assertions child-owner split` / `runtime_15_code_review_findings_f8_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs` 中的 F8 API convergence parent/leaf owner checks 下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs`。`direct_review_assertions.rs` 现在保留 code-review parent moved-test absence、render/F12 direct checks、F8/P0 helper delegation 与行数预算；新 `runtime_15_code_review_findings_f8_direct_assertions_are_child_owner` 锁定 F8 child mount、F8 review guard anchors、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split` / `runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/parent_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/review_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/status_mirrors.rs`；`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs` 同步改为读取 F8/P0 nested child source blobs，避免要求测试体回流到 route/helper parents。旧 `assert_f8_direct_sources_are_folder_backed` 继续作为历史 helper；新增 `runtime_15_code_review_findings_f8_direct_assertions_guard_is_folder_backed`、`runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_f8_direct_assertions_children_line_budgets_are_current` 锁定 F8 direct assertions folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings P0 direct assertions child-owner split` / `runtime_15_code_review_findings_p0_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs` 中的 P0 robustness parent/leaf owner checks 下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs`。后续 F8 direct assertions 已继续下沉到 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs`，因此 `direct_review_assertions.rs` 现在保留 code-review parent moved-test absence、render/F12 direct checks 与 F8/P0 helper delegation；`runtime_15_code_review_findings_p0_direct_assertions_are_child_owner` 锁定 P0 child mount、P0 review guard anchors、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error native plugin loader structure guard child-owner split` / `runtime_15_typed_error_native_plugin_loader_structure_guard_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs` 中的 native plugin loader、ABI surfaces、plugin descriptor、live-host、lifecycle/replay-runtime 与 manifest source 结构断言拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs`。`structure_assertions.rs` 现在保留非 native typed-error parent/child mount、moved-guard helper 与 native child delegation；新 `runtime_15_typed_error_native_plugin_loader_structure_is_child_owner` 锁定 native child mount、native parent route-only checks、父/子 800 行预算与 status rows/maps。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split` / `runtime_15_typed_error_native_plugin_loader_structure_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs` 收束为 route/source owner，并把实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader/budgets.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader/routes.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader/status_mirrors.rs`。旧 `runtime_15_typed_error_native_plugin_loader_structure_is_child_owner` 继续锁定 native child owner；新增 `runtime_15_typed_error_native_plugin_loader_structure_guard_is_folder_backed` 与 `runtime_15_typed_error_native_plugin_loader_structure_guard_folder_backed_status_is_current` 锁定 folder-backed 拆分和状态镜像。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error convergence mounts guard folder-backed split` / `runtime_15_typed_error_convergence_mounts_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts.rs` 收束为 route/source owner，并把顶层 convergence parent、asset parent、runtime/script/UI parent、预算与状态镜像分别拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts/top_level.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts/asset_parents.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts/runtime_parents.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts/status_mirrors.rs`。旧 `assert_typed_error_convergence_parents_are_folder_backed` 继续作为历史 helper；新增 `runtime_15_typed_error_convergence_mounts_guard_is_folder_backed` 与 `runtime_15_typed_error_convergence_mounts_guard_folder_backed_status_is_current` 锁定 folder-backed 拆分和状态镜像。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。


Runtime 15 精确锚点补记 2026-07-01：`Runtime 15 M3 render shader template assembly assertion contract child-owner split` / `runtime_15_render_shader_template_assembly_assertion_contract_child_owner_split_static_passed_cargo_deferred` 精确锚点包括 `structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/template_contracts.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_cache_contracts.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/mesh_pipeline_shadow_graph_contracts.rs`、`structure_convention/production_file_budget/render_shader_template_assembly/assembly_assertions/owner_budget.rs` 与 `runtime_15_render_shader_template_assembly_support_children_are_folder_backed`。

最新完成：`Runtime 15 M3 mesh pipeline shader source tests child-owner split` / `runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred` 已把 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs` 的 module-local source assembly 测试下沉到 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs`；生产 source owner 只保留 `#[cfg(test)]`、`#[path = "shader_source/tests.rs"]` 与 `mod tests;` 挂载，`tests.rs` 继续挂载 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs` 的 WGPU module validation child。`runtime_15_render_shader_template_assembly_support_children_are_folder_backed` 锁定生产/测试 child 挂载、代表性 moved tests 不回流、三侧 800 行预算与 Runtime 15/status/module/session docs；该切片只整理 shader source 测试 owner，不改 shader template、WGSL、pipeline cache 或 runtime render 行为；Cargo gate deferred。

Runtime 15 精确锚点补记 2026-07-01：`Runtime 15 M3 mesh pipeline shader source tests child-owner split` / `runtime_15_mesh_pipeline_shader_source_tests_child_owner_split_static_passed_cargo_deferred` 精确锚点包括 `graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs`、`graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs`、`graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests/runtime_shading_model_sources.rs` 与 `runtime_15_render_shader_template_assembly_support_children_are_folder_backed`。



最新完成：`Runtime 15 M3 structure guard plugin-importer child split` / `runtime_15_structure_guard_plugin_importer_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs` 从 138 行 plugin-importer DX top-level child tree、structure assertions subtree、source inventory/status-doc checks 与 status mirror wrapper 混合 owner 收束为 route/helper owner。实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/top_level_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/structure_assertions.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/source_inventory.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_docs.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/child_ownership.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer/status_mirrors.rs`。旧 `runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned` 与 `assert_plugin_importer_dx_children_are_mounted` 保留为 route wrappers；新增 `runtime_15_structure_guard_plugin_importer_is_child_backed` 与 `runtime_15_structure_guard_plugin_importer_status_mirrors_are_current` 锁定 child-backed 拆分、status rows/maps/docs/session 镜像与 child budgets。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。



最新完成：`Runtime 15 M3 code review findings folder-backed summary child-ownership guard folder-backed split` / `runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership.rs` 从 direct/source 子树结构断言、直接断言子 owner 检查、source inventory 检查、line budgets 与 status mirrors 混合 owner 收束为 route/helper owner，并新增 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/parent_absence.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/direct_assertions.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/source_inventory.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership/status_mirrors.rs` 分别承接 historical `runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned`、parent-absence、direct assertion child checks、source inventory child checks、line budgets 与 status mirrors。新增 `runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_is_folder_backed`、`runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_folder_backed_summary_child_ownership_children_line_budgets_are_current` 锁定该 child-ownership guard 不回流、父/子 800 行预算与 status rows/maps/docs/session 镜像。scoped validation 已覆盖 rustfmt 与 rustfmt --check、status/doc anchor scan、parent-boundary scan、child helper scan、冲突/尾随空白扫描、父/子行数 90/12/68/97/45/46/67 与 scoped diff-check；diff-check 仅报告 touched files 的 LF/CRLF warnings。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred because external cargo/rustc lanes were active。


最新完成：`Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split` / `runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership/parent_absence.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership/entry_points.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership/status_mirrors.rs`。旧 `runtime_15_code_review_findings_direct_assertions_children_are_child_owned` 继续作为历史入口；新增 `runtime_15_code_review_findings_direct_assertions_child_ownership_guard_is_folder_backed`、`runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_direct_assertions_child_ownership_children_line_budgets_are_current` 锁定 child-ownership folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split` / `runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/parent_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/review_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/status_mirrors.rs`。旧 `assert_p0_direct_sources_are_folder_backed` 继续作为历史 helper；新增 `runtime_15_code_review_findings_p0_direct_assertions_guard_is_folder_backed`、`runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_code_review_findings_p0_direct_assertions_children_line_budgets_are_current` 锁定 P0 direct assertions folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。




最新完成：`Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split` / `runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/sources.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/parent_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/review_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/review_mounts/status_mirrors.rs`。旧 `assert_plugin_importer_dx_review_mounts_are_folder_backed` 继续作为历史 helper；新增 `runtime_15_plugin_importer_dx_review_mounts_guard_is_folder_backed`、`runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_status_is_current` 与 `runtime_15_plugin_importer_dx_review_mounts_children_line_budgets_are_current` 锁定 DX review mount folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。2026-07-05 追加 core-min direct-binary 证据：`E:\cargo-targets\zircon-runtime15-plugin-importer-dx-root-inventory-coremin-0704\debug\deps\zircon_runtime-1ff53e05a9088131.exe runtime_15_plugin_importer_dx_review_mounts_guard_is_folder_backed --format terse --test-threads=1` 通过 1/1、6550 filtered。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo wrapper、target-server/default-feature runtime lib、root workspace、plugin workspace 与 Frameworks02 full integration 仍 pending。

最新完成：`Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split` / `runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/sources.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/review_children.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/status_mirrors.rs`。旧 `assert_plugin_importer_d13_sdk_child_owners_are_folder_backed` 继续作为历史 helper；新增 `runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed`、`runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_folder_backed_status_is_current` 与 `runtime_15_plugin_importer_d13_sdk_structure_assertions_children_line_budgets_are_current` 锁定 D13 SDK structure folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split` / `runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_child_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts.rs` 从 142 行 D13 delegation、review child mount checks、folder-backed assertions 与 status mirror wrapper 混合 owner 收束为 route/helper owner。实际检查拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/review_mounts.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/folder_backed.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/child_ownership.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk/parent_mounts/status_mirrors.rs`。旧 `runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner` 与 `runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_is_folder_backed` 仍作为父路由 wrapper；新增 `runtime_15_plugin_importer_d13_sdk_parent_mounts_guard_is_child_backed` 与 `runtime_15_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current` 锁定 child-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。




最新完成：`Runtime 15 M3 foundation row-data priority-doc frontmatter sync` / `runtime_15_foundation_row_data_priority_doc_frontmatter_sync_static_passed_cargo_deferred` 已新增 `runtime_15_foundation_row_data_priority_doc_frontmatter_records_stale_count_guard`，要求 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md` 的 frontmatter 同时列出 `row_count.rs`、`status_support/row_data_and_budget.rs`、M3 structure-support status/date maps，以及 row-count/stale-count 两个测试锚点。该切片只整理优先计划文档 frontmatter 和状态镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 module convention module-doc frontmatter uniqueness guard` / `runtime_15_module_convention_module_doc_frontmatter_uniqueness_static_passed_cargo_deferred` 已新增 `structure_convention/module_convention_gate.rs::runtime_15_module_convention_module_doc_frontmatter_has_unique_entries`，解析 `docs/zircon_runtime/structure/module-convention.md` 的 `related_code`、`implementation_files`、`plan_sources` 与 `tests` frontmatter section 并要求 frontmatter duplicate count 0。该切片清理 module-convention 模块文档头部 29 个 `related_code` 重复项与 7 个 `implementation_files` 重复项，同步 `module_convention_status.rs`、M3 structure-support status/date maps、runtime index、review findings、structure convention、module-convention 与 session note；不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 module convention gate guard folder-backed split` / `runtime_15_module_convention_gate_guard_folder_backed_static_passed_cargo_deferred` 已把 `structure_convention/module_convention_gate.rs` 从 616 行混合守卫 owner 收束为 6 行 route owner；实际检查拆入 `structure_convention/module_convention_gate/helpers.rs`、`structure_convention/module_convention_gate/module_doc_frontmatter.rs`、`structure_convention/module_convention_gate/output_contract.rs`、`structure_convention/module_convention_gate/debt_boundary.rs`、`structure_convention/module_convention_gate/audit_status.rs` 与 `structure_convention/module_convention_gate/split_layout.rs`。新增 `runtime_15_module_convention_gate_guard_is_folder_backed` 锁定父/子挂载、旧测试体不回流、status row data、status/date maps、Frameworks02、Runtime 15/index/review/structure/module/session anchors。该切片只整理 structure-convention module-convention 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo gate deferred。


最新完成：`Runtime 15 M3 priority plan docs listing prose full inventory sync` / `runtime_15_priority_plan_docs_listing_prose_full_inventory_sync_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory`，要求 Runtime 15、runtime index、两份优先文档、module-convention 与 session note 中的 listing guard 说明都使用 full priority-plan-doc guard inventory，并显式包含 `priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources` 与 `priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory`。该切片只整理优先计划文档 listing prose/status 镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs moved mirror full inventory sync` / `runtime_15_priority_plan_docs_moved_mirror_full_inventory_sync_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory`，要求 moved guard path mirror 状态说明使用 full priority-plan-doc moved guard inventory，并显式包含 `priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources`、`priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory` 与 `priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory`。该切片只整理优先计划文档 moved mirror/status 镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs guard-test child prose full inventory sync` / `runtime_15_priority_plan_docs_guard_test_child_prose_full_inventory_sync_static_passed_cargo_deferred` 已新增 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory`，要求 guard-test child-owner split 状态说明使用 full priority-plan-doc guard-test child inventory，并显式包含 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs` 与 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs`。该切片只整理优先计划文档 guard-test child/status 镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs child prose full inventory sync` / `runtime_15_priority_plan_docs_child_prose_full_inventory_sync_static_passed_cargo_deferred` 已扩展 `structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed`，要求 priority-plan-doc guard child-owner split 状态说明使用 full priority-plan-doc child inventory，并显式包含 `structure_convention/test_file_budget/priority_plan_docs/code_paths.rs`、`structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs`、`structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs`、`structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs`、`structure_convention/test_file_budget/priority_plan_docs/header_sections.rs`、`structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs` 与 `structure_convention/test_file_budget/priority_plan_docs/test_paths.rs`。该切片只整理优先计划文档 child/status 镜像，不改 runtime/plugin/render/editor 生产代码；Cargo gate deferred。
















最新完成：`Runtime 15 M3 typed-error structure moved-guard absence child-owner split` / `runtime_15_typed_error_structure_moved_guard_absence_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs` 中的 moved-guard absence 与 review guard preservation 长断言拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs`。`structure_assertions.rs` 现在保留 typed-error parent/child mount 与 `moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned` 委派；新 `runtime_15_typed_error_structure_moved_guard_absence_is_child_owner` 锁定 47 个 typed-error review guard preservation、parent backflow guard list、full child path anchors 与父/子预算。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error structure assertions guard child-owner split` / `runtime_15_typed_error_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs` 中的 typed-error parent/child mount 与结构断言委派拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs`。父守卫从 522 行降到 116 行，只保留 source-inventory/status-doc/structure-assertions 子模块挂载、委托、47 个 guard count 与父/子预算；`runtime_15_typed_error_structure_assertions_are_child_owner` 锁定 mount/委派不回流。后续 moved-guard split 又把 moved-guard absence 与 review guard preservation 长断言下沉到 `structure_assertions/moved_guard_absence.rs`。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。


最新完成：`Runtime 15 M3 plugin-importer DX source inventory guard child-owner split` / `runtime_15_plugin_importer_dx_source_inventory_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs` 中的 plugin-importer DX source path inventory、line budget helper 与 11 guard count helper 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs`。父守卫只保留 parent/child mount、review guard ownership assertions、docs/status anchor checks 与 source-inventory helper delegation；`runtime_15_plugin_importer_dx_source_inventory_is_child_owner` 锁定 `PLUGIN_IMPORTER_DX_SOURCE_PATHS` 不回流、helper delegation、11 个 plugin-importer DX review guards 和父/子预算。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split` / `runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs` 收束为 route/helper owner，实际职责拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory/paths.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory/reads.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory/budgets.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory/delegation.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory/status_mirrors.rs`。旧 `runtime_15_plugin_importer_dx_source_inventory_is_child_owner` 继续作为历史入口；新增 `runtime_15_plugin_importer_dx_source_inventory_guard_is_folder_backed`、`runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_status_is_current` 与 `runtime_15_plugin_importer_dx_source_inventory_children_line_budgets_are_current` 锁定 source inventory folder-backed 拆分、status rows/maps/docs/session 镜像与父/子 800 行预算。2026-07-05 追加 core-min direct-binary 证据：`E:\cargo-targets\zircon-runtime15-plugin-importer-dx-root-inventory-coremin-0704\debug\deps\zircon_runtime-1ff53e05a9088131.exe runtime_15_plugin_importer_dx_source_inventory_guard_is_folder_backed --format terse --test-threads=1` 通过 1/1、6550 filtered。该切片只整理 structure-convention/code-review 测试守卫 owner，不改 runtime/plugin/render/editor/text/ZUI 生产代码；Cargo wrapper、target-server/default-feature runtime lib、root workspace、plugin workspace 与 Frameworks02 full integration 仍 pending。

最新完成：`Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split` / `runtime_15_plugin_importer_dx_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs` 中的 plugin-importer DX parent/child mount、D1/D5/D6/D8/D9/D10/D11/D12/D13 review guard ownership 与 no-test backflow 长断言拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs`。父守卫只保留 structure-assertions/source-inventory/status-doc helper delegation 与父/子预算；`runtime_15_plugin_importer_dx_structure_assertions_are_child_owner` 锁定结构断言不回流、11 个 plugin-importer DX review guards 和父/子预算。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split` / `runtime_15_plugin_importer_d13_sdk_structure_assertions_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs` 中的 D13 importer SDK parent mount、runtime-crates/runtime-exports/runtime-manifests/manifest-parity 结构断言拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs`。父 structure-assertions child 只保留顶层 plugin-importer DX owner 检查与 `d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed` 委派；`runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner` 锁定 D13 SDK 结构断言不回流、3 个 D13 importer SDK review guards 和父/子预算。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 typed-error source inventory guard child-owner split` / `runtime_15_typed_error_source_inventory_guard_child_owner_split_static_passed_cargo_deferred` 已把 `structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs` 中的 typed-error source path inventory、source aggregation、line budget 与 47 guard count helpers 拆入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs`。父守卫从 593 行降到 522 行，只保留 parent/child mount、moved-guard absence、status-doc delegation 与 source-inventory helper delegation；新 source-inventory child 为 145 行，`runtime_15_typed_error_source_inventory_is_child_owner` 锁定清单不回流、helper delegation、47 个 typed-error review guards 和父/子预算。该切片只整理 structure-convention 测试守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。




最新完成：`Runtime 15 M3 priority plan docs code-path integrity guard` / `runtime_15_priority_plan_docs_code_path_integrity_guard_static_passed_cargo_deferred` 新增 `structure_convention/test_file_budget/priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current`，锁定 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md` 的 `related_code` / `implementation_files` 机器可读头部路径全部仍存在，并要求 Runtime 15/index/review findings/structure/module docs、session note、status row data 与 status/date maps 同步同一状态。该切片只整理优先计划文档路径完整性守卫，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 priority plan docs test-path integrity guard` / `runtime_15_priority_plan_docs_test_path_integrity_guard_static_passed_cargo_deferred` 新增 `structure_convention/test_file_budget/priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current`，锁定 `docs/plans/engine-code-structure-convention.md` 与 `docs/plans/engine-code-review-findings-2026-06.md` 的 `tests:` 机器可读头部路径型条目全部仍存在，并要求 Runtime 15/index/review findings/structure/module docs、session note、status row data 与 status/date maps 同步同一状态。该切片只整理优先计划文档测试路径完整性守卫，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings structure guard typed-error child-owner split` / `runtime_15_code_review_findings_structure_guard_typed_error_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs` 中的 typed-error parent、structure assertions、moved-guard 与 source inventory 结构检查迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error.rs`。主 structure guard child 现在只保留顶层 code-review child mounts、F8/P0/late-api/plugin-importer/status-doc child checks、folder-backed summary/typed-error 委派与行数预算；新 `runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner` 锁定 typed-error 结构检查不回流、typed-error child owners、父/子 800 行预算与 status rows/maps/docs 同步。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings structure guard folder-backed summary child-owner split` / `runtime_15_code_review_findings_structure_guard_folder_backed_summary_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs` 中的 folder-backed summary/direct/source 三个子 owner 结构检查迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary.rs`。主 structure guard child 现在只保留顶层 code-review child mounts、F8/P0/late-api/plugin-importer/typed-error/status-doc child checks 与行数预算；新 `runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner` 锁定 folder-backed summary 结构检查不回流、summary/direct/source 三个 owner、父/子 800 行预算与 status rows/maps/docs 同步。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split` / `runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary.rs` 继续降为 route/shared helper parent，并把结构委派、direct assertions、source inventory、line budgets 与 status mirrors 分别迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/delegation.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/direct_assertions.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/source_inventory.rs`、`tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/budgets.rs` 与 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/status_mirrors.rs`。`runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner`、`runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned`、`runtime_15_code_review_findings_structure_guard_folder_backed_summary_source_inventory_is_child_owned`、`runtime_15_code_review_findings_structure_guard_folder_backed_summary_children_line_budgets_are_current` 与 `runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_status_is_current` 锁定该 folder-backed guard 不回流、父/子 800 行预算与 status rows/maps/docs 同步。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings direct assertions child-owner split` / `runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs` 中的 P0/F8/render/F12 direct review source 结构断言迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs`。后续 F8/P0/root-parent direct assertions 已分别继续下沉到 `direct_review_assertions/f8.rs`、`direct_review_assertions/p0.rs` 与 `direct_review_assertions/root_parent.rs`，因此 direct assertions parent 现在保留 render/F12 direct checks、F8/P0/root-parent helper delegation 与行数预算；`runtime_15_code_review_findings_direct_assertions_are_child_owner` 锁定 direct assertions 不回流、render/F12 direct source checks、child delegation、父/子 800 行预算与 status rows/maps/docs 同步。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings source inventory child-owner split` / `runtime_15_code_review_findings_source_inventory_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs` 中的 code-review findings source reads、path inventory、direct review guard count 与 line-budget helper 迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory.rs`。folder-backed summary child 现在只保留聚合断言、P0/F8/late API/plugin-importer/typed-error helper 调用、78 个 review guards 总数检查和 status-doc 委托；新 `runtime_15_code_review_findings_source_inventory_is_child_owner` 锁定 source inventory 不回流、15 个 directly counted review guards、父/子 800 行预算与 status rows/maps/docs 同步。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

最新完成：`Runtime 15 M3 code review findings folder-backed summary child-owner split` / `runtime_15_code_review_findings_folder_backed_summary_child_owner_split_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs` 中的 code-review findings folder-backed 总览、line budget、78 个 review guards 聚合和 status-doc 委托迁入 `tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs`。父守卫现在只保留子模块挂载和原 `runtime_15_code_review_findings_tests_are_folder_backed` 入口委托；新子守卫 `runtime_15_code_review_findings_folder_backed_summary_is_child_owner` 锁定聚合断言不回流、`runtime_15_code_review_findings_structure_guard_children_are_mounted` 与 `runtime_15_code_review_findings_status_docs_are_child_owner` 同步覆盖新 owner。该切片只整理结构守卫 owner，不改 runtime/plugin/render/editor 生产代码，Cargo gate deferred。

























最新完成：`Runtime 15 M3 core-scene naming ECS owner split-layout folder-backed split` / `runtime_15_core_scene_naming_ecs_owner_split_layout_folder_backed_static_passed_cargo_deferred` 已把 `tests/runtime_absorption/naming_boundary/runtime_15_m2/core_scene/scene_ecs_owners.rs` 收束为 route owner，只挂载 `scene_ecs_owners/observer_callback_registry.rs`、`scene_ecs_owners/query_state_many_item_array.rs`、`scene_ecs_owners/component_storage_component_results.rs` 与 `scene_ecs_owners/split_layout.rs`。新增 `runtime_15_core_scene_naming_ecs_owner_split_layout_is_folder_backed` 锁定二级 split-layout route、moved guards 不回流、status rows/maps、Runtime 15/index/Frameworks/review/structure/module docs 与 session note；Frameworks 镜像 `frameworks_02_m3_core_scene_naming_ecs_owner_split_layout_folder_backed_static_passed_cargo_deferred`。该切片只整理 naming-boundary scene ECS 测试 owner，不改 scene ECS/runtime 生产代码；验证：scoped rustfmt/static/conflict scans passed，`CORE_SCENE_ECS_SPLIT_LAYOUT_STATIC_OK route=32 observer=119 query=129 component=117 split=157 docs=10`，standalone scene_ecs_owners harness 4/4，scoped `git diff --check` 仅 LF/CRLF warnings；Cargo gate deferred（D_FREE_GB=2.49，Cargo/rustc lanes active）。





> 状态：in_progress · 范围：`zircon_runtime` 全模块（含 `graphics/**`，render 子计划引用）
> 上游权威：[`docs/plans/engine-code-structure-convention.md`](../../engine-code-structure-convention.md)（本计划只**执行**该规范，不重定义规则）
> 原则：硬切不留兼容层；按 ownership 拆不按行数切；root 接线薄；规则收敛由审计 gate + guard 测试机器化验收。

## 1. 目标

把 `zircon_runtime` 的模块布局、命名、公共 API、测试组织收敛到 `engine-code-structure-convention`，让用户对 runtime 做 code review / inspect 时有**一致、可预测、接口友好**的结构，并以 `module_convention_gate` 防止回归。

## 2. 现状缺口（按代码实查，带路径证据）

| # | 缺口 | 规范条目 | 证据路径 |
|---|------|---------|---------|
| S1 | `mod.rs` / `module.rs` 混用，无统一判据 | R1.2 | `animation/module.rs` vs `input/mod.rs`（直塞注册）vs `graphics/mod.rs`（注册分散无 `module.rs`） |
| S2 | 巨型扁平 re-export façade（100+ 符号一坨） | R3.1 | `asset/mod.rs`、`scene/mod.rs`、`plugin/mod.rs` |
| S3 | `pub mod` / `pub(crate) mod` 无规则混排 | R3.4 | `graphics/mod.rs:1-93` |
| S4 | `runtime_*` 前缀泛滥到失去语义 | R2.2 | `graphics/{hybrid_gi,virtual_geometry}_runtime_provider/runtime_{feedback,state,stats,update}.rs` |
| S5 | `manager.rs` 与 `manager/` 并存、规则不明 | R1.2 | `animation/manager.rs` + `animation/manager/{graph,parameters,pose,sampling,state_machine}.rs` |
| S6 | 巨型 `tests.rs` + 三套测试组织并行 | R1.4 / R4.1-R4.3 | `core/framework/tests.rs`(1848)、`ui/tests/v2_asset.rs`(3806)、`ui/tests/shared_core.rs`(3145) |
| S7 | 复数 / 单数失序 | R2.1 | `scene/components/`（复数）vs `animation/manager/`（单数）规则缺失 |
| S8 | 单文件多职责协调 | R1.3 | `dynamic_api/session.rs`(773，协调 17 子模块) |
| S9 | **prelude 完全遗漏 asset/scene/ecs/ui/graphics**，gameplay/authoring 用户须深路径 import | R3.3 | `prelude.rs`（`crate::{asset,scene,ui,graphics}` 零命中） |
| S10 | **200 处 `#[allow(dead_code)]`（55 文件）掩盖僵尸**。`runtime_ui` 生产子模块已在 Runtime 15 runtime UI dead-code support split 中拆为生产 frame DTO + test-only support；Runtime 15 F12 runtime-owned dead-code suppression cleanup 已清理 asset worker test-only receiver guard 与 core `ModuleEntry` descriptor 生产字段的抑制；Runtime 15 F12 script host value descriptor dead-code cleanup 已移除 `Vec3` / `ColorRgba` 的 suppression 并用字段布局哨兵保持反射描述器 live；Runtime 15 F12 script reflection macro fixture dead-code cleanup 已移除 `reflection_docs.rs` 中 TestVec3/TestEnum/Point 宏 fixture 的 suppression 并由测试断言读取字段/变体；Runtime 15 F12 offscreen target texture owner cleanup 已把固定帧 WGPU 纹理 owner 变成生产绑定入口会读取的保活契约；Runtime 15 F12 render backend state owner cleanup 已把 `RenderBackend` 的 instance/adapter/config owner 转为 `caps()` 路径可检查的保活契约；Runtime 15 F12 gpu texture resource owner cleanup 已把 `GpuTextureResource` 的 id/texture/view/sampler 转为材质绑定入口可检查的资源驻留契约；Runtime 15 F12 gpu material uniform owner cleanup 已把 `GpuMaterialUniformResource` 的 buffer/payload byte len/buffer byte len 转为 uniform 绑定入口和诊断 accessor 可检查的资源驻留契约；Runtime 15 F12 gpu mesh order signature cleanup 已把 `GpuMeshResource::indirect_order_signature()` 接入 prepared mesh draw 排序 tie-breaker；Runtime 15 F12 gpu model identity cleanup 已把 `GpuModelResource::id()` 接入 `ResourceStreamer::model(...)` 缓存身份校验；Runtime 15 F12 post-process LUT texture owner cleanup 已把 `PostProcessLutTextureResource::view()` 接入 3D LUT 绑定查询并显式保活 texture/view owner；Runtime 15 F12 output target texture owner cleanup 已把 `OutputTargetTextureResource` descriptor/texture/view/sampler 与 `PreparedOutputTargetTexture` cache owner 接入 graph import、writeback 和材质采样路径；Runtime 15 F12 material runtime capture seed cleanup 已把 `MaterialCaptureSeed`、`MaterialRuntime::capture_seed()`、`ResourceStreamer::material_capture_seed(...)` 与 `sample_texture_rgba(...)` 收进 test-only 配置并移除 material runtime suppression；Runtime 15 F12 resource streamer diagnostics accessor cleanup 已把测试诊断 accessor 收进 `#[cfg(test)]` 并保留生产 readiness bridge live；Runtime 15 F12 resource streamer resolve texture id cleanup 已删除未使用的 `resolve_texture_id(...)` helper 并保留生产 `resolve_texture_reference(...)` / `resolve_texture_reference_with_support(...)` 入口；Runtime 15 F12 particle GPU readback output accessor cleanup 已删除已接入 runtime feedback 的 particle readback 输出 accessor stale suppression；Runtime 15 F12 production dead-code current-state wording cleanup 已同步 runtime production `allow(dead_code)` 零命中、`runtime_15_production_sources_do_not_allow_dead_code_suppression` 与 `runtime_15_f12_production_dead_code_current_state_is_zero_hit`，剩余风险只是不声明完整 Runtime 15 Cargo sweep 通过 | 规范 E6 | `ui/public_runtime_frame.rs`、`ui/tests/runtime_ui_support/`、`asset/pipeline/worker_pool.rs`、`core/runtime/state/module_entry.rs`、`core/runtime/diagnostics/devtools.rs`、`script/vm/host/builtin_host_modules.rs`、`script/vm/tests/reflection_docs.rs`、`graphics/backend/render_backend/offscreen_target.rs`、`graphics/backend/render_backend/render_backend.rs`、`graphics/scene/resources/gpu_texture/gpu_texture_resource.rs`、`graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs`、`graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs`、`graphics/scene/resources/gpu_model/gpu_model_resource.rs`、`graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs`、`graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs`、`graphics/scene/resources/output_target_texture/output_target_texture_resource.rs`、`graphics/scene/resources/prepared/prepared_output_target_texture.rs`、`graphics/scene/resources/runtime/material_runtime.rs`、`graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs`、`graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs`、`graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs`、`graphics/runtime/render_framework/submit_frame_extract/submit/collect_runtime_feedback.rs`、`graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs`、`graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs`、`graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs`、`scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs`、`runtime_15_runtime_ui_dead_code_surface_is_test_support`、`runtime_15_runtime_owned_dead_code_suppression_cleanup`、`runtime_15_script_host_value_descriptors_do_not_suppress_dead_code`、`runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code`、`runtime_15_offscreen_target_texture_owner_cleanup`、`runtime_15_render_backend_state_owner_cleanup`、`runtime_15_gpu_texture_resource_owner_cleanup`、`runtime_15_gpu_material_uniform_owner_cleanup`、`runtime_15_gpu_mesh_order_signature_cleanup`、`runtime_15_gpu_model_identity_cleanup`、`runtime_15_post_process_lut_texture_owner_cleanup`、`runtime_15_output_target_texture_owner_cleanup`、`runtime_15_material_runtime_capture_seed_cleanup`、`runtime_15_resource_streamer_diagnostics_accessor_cleanup`、`runtime_15_resource_streamer_resolve_texture_id_cleanup`、`runtime_15_particle_gpu_readback_output_accessor_cleanup`、`runtime_15_f12_production_dead_code_current_state_wording_static_passed_cargo_deferred`、`runtime_15_f12_production_dead_code_current_state_is_zero_hit`；runtime production `allow(dead_code)` 零命中已由全局 gate 锁定，完整 Runtime 15 Cargo sweep 仍 pending；2026-06-22 render F11 已删除 `graphics/material/shading_models/registry.rs` 的未接线 dead-code surface |
| S11 | **4 套 `*_runtime_provider` 近 99% 复制粘贴** + 7+ 个 `*Diagnostics` 命名/嵌套重复。F14 子切片已新增 `FrameDiagnostics` trait 并移除 `WorldEcsFramePerformanceDiagnostics` 纯包装；F13 registration 子切片已抽 `RuntimeProviderRegistration<P: ?Sized>` + `define_runtime_provider_registration!`；F13 update stats 子切片已抽 `RuntimeProviderUpdate<S>` + `define_runtime_provider_update!`；F13 feedback shared payload 子切片已抽 `RuntimeProviderFeedback<G, V>`；F13 prepare input 子切片已抽 `RuntimeProviderPrepareInput<'a, E>`；full F13 audit 已由 `runtime_15_no_duplicated_provider_boilerplate` 总守卫闭合 | 规范 E5 | `graphics/runtime_provider/{registration,update,feedback,prepare_input}.rs`、`graphics/{hybrid_gi,virtual_geometry,solari}_runtime_provider/provider_registration.rs`、`graphics/{hybrid_gi,virtual_geometry}_runtime_provider/{prepare_input,runtime_update,runtime_feedback}.rs`、`graphics/{particle,solari}_runtime_provider/{prepare_input,runtime_feedback}.rs`、`core/runtime/diagnostics/frame_diagnostics.rs`、`scene/world/performance_diagnostics.rs`、`scene/world/world.rs` |
| S12 | 渲染器 construction owner 使用 `*_new` 后缀，读如迁移残留 | R2.5 | 2026-06-22 render F19 已硬切为 `scene_renderer_core_construct` / `scene_renderer_construct`，新增 `review_f19_scene_renderer_construction_modules_use_construct_names`，状态 `render_scene_renderer_construct_modules_coremin_passed`；2026-06-25 Runtime 15 M2 已把 asset watcher 的 `asset_change_new.rs` 硬切为 `asset_change_construction.rs`，状态 `runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred`，把 graphics resource streamer 的 `resource_streamer_new.rs` 硬切为 `resource_streamer_construction.rs`，状态 `runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred`，并把 render backend 的 `offscreen_target_new/` 硬切为 `offscreen_target_construct/`，状态 `runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result`；2026-06-27 Runtime 15 M2 已把 graphics construction 裸 `new.rs` / `new/` owner 硬切为 `construct.rs` / `construct/`，状态 `runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred`；同日把 dynamic scene document 的 `legacy.rs` / `LegacyProjectDocument` 硬切为 `v1_project_document.rs` / `V1ProjectDocument`，状态 `runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred` |

> S9-S12 来自 [`engine-code-review-findings-2026-06.md`](../../engine-code-review-findings-2026-06.md)（F1/F9/F10/F12/F13/F14/F19）。注：native host callback panic guard 的 P0/F1 已由 Runtime 15 F1 native host callback panic guard 固定当前闭合状态；渲染热路径的 String 错误、每帧 clone、`render_compiled_scene` 533 行等 P0/P1 项归 Runtime 07 / render，不在本计划但与 M4 large-file 联动；F19 的目录命名债已作为 R2.5 样例闭合。

## 3. 目标结构（收敛后形态）

- 每个子系统 `mod.rs` = 分组注释的精选 façade + `pub mod prelude;`；crate 级 `prelude` 收窄为聚合各子系统 prelude。
- `module.rs` 仅出现在注册子系统目录（有 `module_descriptor()`）；`input/` 注册下沉 `input/module.rs`，`graphics/` 注册收口到 `graphics/module.rs`。
- `*_runtime_provider/` 内部模块去 `runtime_` 前缀（`state.rs`/`feedback.rs`/`stats.rs`/`update.rs`）。
- `animation/manager.rs` 删除，`animation/manager/mod.rs` 作薄 façade 拥有子模块。
- 测试统一：小测内联、行为测试 folder-backed 镜像源树、无 > 800 行 `tests.rs`。

## 4. 里程碑（任务级执行蓝本）

切片期 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked`；里程碑末进测试。命令省略共用尾部。

| 里程碑 | 任务 | 改动文件（代表） | 依赖 | 验收命令 / 测试函数 |
|---|---|---|---|---|
| **M1 façade / 可见性** | T1 大 façade 分组 + 建子系统 prelude | `asset/mod.rs`、`scene/mod.rs`、`plugin/mod.rs`、新 `asset/prelude.rs` 等 | — | `cargo check -p zircon_runtime --lib`；`runtime_15_facade_reexport_within_budget`、`runtime_15_no_glob_reexport_at_subsystem_facade` |
| | T2 收 `pub` / `pub(crate)` 混排 | `graphics/mod.rs` | T1 | `runtime_15_mixed_visibility_has_facade_note` |
| | T3 crate 级 `prelude` 收窄为聚合 | `lib.rs`、`prelude.rs` | T1 | `cargo test -p zircon_runtime --lib prelude` |
| **M2 命名 / 前缀** | T1 去冗余 `runtime_` 前缀 | `graphics/{hybrid_gi,virtual_geometry}_runtime_provider/*` | M1 | grep 旧符号零命中；`runtime_15_module_prefix_in_vocabulary` |
| | T2 复数 / 单数对齐 | 相关目录 rename | M1 | `runtime_15_collection_dirs_plural_owner_dirs_singular` |
| | T3 去 `_inner` / `_helper` / `util` | 命名债文件；`core/runtime/state/runtime_inner.rs` 已硬切为 `core/runtime/state/core_runtime_state.rs`，`scene/ecs/observer/utils.rs` 已硬切为 `scene/ecs/observer/callback_registry.rs`，`scene/ecs/query/query_state/helpers.rs` 已硬切为 `scene/ecs/query/query_state/many_item_array.rs`，`scene/ecs/storage/component_storage/utils.rs` 已硬切为 `scene/ecs/storage/component_storage/component_results.rs`，`asset/watch/drop_impl.rs` 已硬切为 `asset/watch/shutdown_on_drop.rs`，`scene/tests/ecs_query/cache_helpers.rs` 已硬切为 `scene/tests/ecs_query/cached_queries.rs`，`dynamic_api/session/tests/helpers.rs` 已硬切为 `dynamic_api/session/tests/vampire_runtime_support.rs`，`core/framework/camera_controller/common.rs` 已硬切为 `core/framework/camera_controller/controller_output.rs`，`scene/tests/ecs_systems/query_helpers.rs` 已硬切为 `scene/tests/ecs_systems/many_single_queries.rs`，`asset/tests/assets/texture_upload_readiness/common.rs` 已硬切为 `asset/tests/assets/texture_upload_readiness/container_fixtures.rs`，`plugin_extensions/static_manifest_contracts/{feature_bundles,package_coordinates,package_identity,package_kind}/helpers.rs` 已硬切为 `feature_bundle_rows.rs`、`package_coordinate_resolution.rs`、`package_id_tokens.rs` 与 `package_kind_fields.rs`，`ui/component/catalog/editor_showcase/helpers.rs` 已硬切为 `ui/component/catalog/editor_showcase/descriptor_builders.rs`；2026-06-27 已把 `graphics/runtime/render_framework/render_framework_impl/trait_impl.rs` 与 `graphics/runtime/render_framework/wgpu_render_framework_new/new.rs` 硬切为 `render_framework_trait_binding/wgpu_framework.rs` 与 `wgpu_render_framework_construction/construct.rs`，并把 `scene/dynamic_scene/document/legacy.rs` / `LegacyProjectDocument` 硬切为 `scene/dynamic_scene/document/v1_project_document.rs` / `V1ProjectDocument`，对应父模块只挂载新 owner，旧路径不保留兼容模块 | M1 | `runtime_15_no_banned_name_modules`；`runtime_15_core_runtime_state_module_uses_owner_name`；`runtime_15_scene_ecs_observer_callback_registry_uses_owner_name`；`runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name`；`runtime_15_scene_ecs_component_storage_component_results_uses_owner_name`；`runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name`；`runtime_15_scene_ecs_query_cached_queries_uses_owner_name`；`runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name`；`runtime_15_camera_controller_output_uses_owner_name`；`runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name`；`runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name`；`runtime_15_plugin_static_manifest_contract_owners_use_domain_names`；`runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name`；`runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred`；`runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name` |
| | T4 `manager.rs` + `manager/` 消解 | `animation/manager.rs` 已硬切删除，`animation/manager/mod.rs` 作薄 façade 并继续挂载 `graph` / `parameters` / `pose` / `sampling` / `state_machine` child owners | M1 | `runtime_15_module_rs_only_for_registration_subsystems`；`runtime_15_animation_manager_is_folder_backed` |
| **M3 测试组织** | T1 拆 > 800 行测试 | `core/framework/tests.rs`、`ui/tests/v2_asset.rs`、`ui/tests/shared_core.rs` → folder-backed；graphics dead-code guard 聚合已拆为 `structure_convention/graphics_dead_code/mod.rs` + `module_layout.rs` + `renderer_output_accessors.rs`，并继续细分为 `backend_owners.rs` / `gpu_resource_owners.rs` / `resource_streamer_cleanup.rs`；provider boilerplate 守卫已集中到 `structure_convention/provider_boilerplate.rs`；asset test-budget 守卫已继续细分为 `structure_convention/test_file_budget/asset_tests/{pack,facade,project,material}.rs` | — | `runtime_15_no_oversized_test_files`；`runtime_15_graphics_dead_code_guard_is_folder_backed`；`runtime_15_provider_boilerplate_guard_is_folder_backed` |
| | T2 三套测试 → 单一规则 + 消重复 | 内联 / `tests/` / `tests.rs` 收敛 | T1 | `runtime_15_test_layout_single_rule` |
| **M4 行数硬上限** | T1 与 `large_file_ownership_gate` 联动收残余 | 各 owner 拆分；`core/runtime/handle/registration/service_lists.rs` 已硬切为 folder-backed `core/runtime/handle/registration/service_lists/{mod,types,multi,specialized,shutdown}.rs` | M1-M3 | 复用 `large_file_ownership_gate`（`migration-debt → 0`）；`runtime_15_core_runtime_service_lists_are_folder_backed` |
| **M5 prelude 完整化 + 死代码清除** | T1 prelude 增补 asset/scene/ecs/ui 高频类型 + "prelude 必含"清单 | `prelude.rs`、各子系统 `prelude.rs` | M1 | `runtime_15_prelude_covers_required_types`（F9） |
| | T2 `#[allow(dead_code)]` sweep 删抑制暴露真僵尸（含 `runtime_ui`、shading registry） | `ui/mod.rs`、55 文件；`runtime_ui` 已拆为 `ui/public_runtime_frame.rs` + `ui/tests/runtime_ui_support`；asset worker/core module-entry runtime-owned 子切片已清理；script host `Vec3` / `ColorRgba` descriptor 子切片已清理；OffscreenTarget、RenderBackend、GpuTextureResource、GpuMaterialUniformResource、GpuMeshResource order signature、GpuModelResource identity、PostProcessLutTextureResource owner、OutputTargetTextureResource/PreparedOutputTargetTexture owner、MaterialRuntime capture seed/test texture sampling、ResourceStreamer diagnostics accessor、ResourceStreamer resolve texture id helper 与 particle GPU readback output accessor 子切片已清理；shading registry F11 子切片已由 render 收口 | — | `runtime_15_no_dead_code_suppression_in_production`（F10/F12）；`runtime_15_runtime_ui_dead_code_surface_is_test_support`；`runtime_15_runtime_owned_dead_code_suppression_cleanup`；`runtime_15_script_host_value_descriptors_do_not_suppress_dead_code`；`runtime_15_offscreen_target_texture_owner_cleanup`；`runtime_15_render_backend_state_owner_cleanup`；`runtime_15_gpu_texture_resource_owner_cleanup`；`runtime_15_gpu_material_uniform_owner_cleanup`；`runtime_15_gpu_mesh_order_signature_cleanup`；`runtime_15_gpu_model_identity_cleanup`；`runtime_15_post_process_lut_texture_owner_cleanup`；`runtime_15_output_target_texture_owner_cleanup`；`runtime_15_material_runtime_capture_seed_cleanup`；`runtime_15_resource_streamer_diagnostics_accessor_cleanup`；`runtime_15_resource_streamer_resolve_texture_id_cleanup`；`runtime_15_particle_gpu_readback_output_accessor_cleanup` |
| | T3 抽 `RuntimeProviderRegistration<P>` / `RuntimeProviderUpdate<S>` / `RuntimeProviderFeedback<G, V>` / `RuntimeProviderPrepareInput<'a, E>` 泛型 + 统一 `*Diagnostics` | `graphics/*_runtime_provider/*`、`graphics/runtime_provider/*`、`core/runtime/diagnostics/frame_diagnostics.rs`、`scene/world/performance_diagnostics.rs` | — | `runtime_15_no_duplicated_provider_boilerplate`（F13）；`runtime_15_provider_update_uses_shared_stats_owner`（F13）；`runtime_15_provider_feedback_uses_shared_payload_owner`（F13）；`runtime_15_provider_prepare_input_uses_shared_extract_generation_owner`（F13）；`runtime_15_diagnostics_use_frame_trait_without_world_wrapper`（F14） |

## 5. 审计与 guard 契约（本计划新增）

- **审计脚本**（进 `runtime_structure_audits/`，由 `audit_runtime_structure.py` 聚合）：`module_convention_gate.py` + `module_convention_gate_markdown.py`。字段：
  - `facade_reexport_line_budget`、`oversized_facade_files`
  - `mixed_visibility_mod_files`
  - `prefix_vocabulary_violations`、`plural_singular_violations`、`banned_name_modules`
  - `module_rs_without_descriptor`（R1.2 违例）
  - `oversized_test_files`、`duplicate_test_trees`
  - `module_convention_gate.m1_gate_status`、`classification_counts`、`migration_debt_count`、`exempt`
- **guard 测试**：`zircon_runtime/src/tests/runtime_absorption/structure_convention.rs`（沿用 runtime_absorption 范式），含 `runtime_15_module_convention_mirror_docs_match_structure_audit_counts`。
- **镜像文档**：`docs/zircon_runtime/structure/module-convention.md`，计数与审计一致。

## 6. 硬切 checklist（每个含删除义务的切片必过）

- [ ] 旧路径文件 / 模块已物理删除（不是注释 / feature 门）
- [ ] 全部调用方已迁移到新 owner / façade 路径（grep 旧符号零命中）
- [ ] 无兼容 re-export / facade 桥残留
- [ ] 同一职责无双路径并存
- [ ] 删除清单写进提交说明

## 7. 完成定义

`module_convention_gate.m1_gate_status = classified-and-clear`、各 violation 字段 = 0、`exempt` 仅含登记豁免项、`runtime_15_module_convention_mirror_docs_match_structure_audit_counts` 绿；`cargo test -p zircon_runtime --lib`（按过滤词收窄）+ `cargo fmt --all --check` + `python .../audit_runtime_structure.py --json` 无 risk。

## 2026-07-11 Runtime 15 Naming Boundary Current Debt Clear

新鲜完整结构审计确认 37 个顶层分组均无显式 `risks`，并把模块约定门禁的剩余输入
收敛到四处 UI 测试夹具文本与一处 graphics 内联测试名。等长 fixture 改名保持文本
offset 语义，测试名改为 `prior`，不修改生产行为或增加兼容豁免。当前直接命名审计为
`classified` 且四类债务列表全空，重组后的 module-convention gate 为
`classified-and-clear`、`migration_debt_count=0`、`risks=[]`；三文件 rustfmt 通过。
Cargo 因全部可见磁盘低于 50 GiB 阈值且外部编译通道活跃而延后。Runtime 15 仍保留
完整结构族 1297/1304 的 7 个 Render/UI 外部失败，不标记整体完成。详细证据见
[`15/2026-07-11-naming-boundary-current-debt-clear.md`](15/2026-07-11-naming-boundary-current-debt-clear.md)。

- fixed 已修复：[f18-asset-manager-review-guard-owner-drift](../../zircon_editor/editor/02/fixed-2026-07-14-f18-asset-manager-review-guard-owner-drift.md)
- fixed 已修复：[core-runtime-state-plugin-bridge-lifecycle-anchor-drift](../../zircon_editor/editor/02/fixed-2026-07-14-core-runtime-state-plugin-bridge-lifecycle-anchor-drift.md)
- 2026-07-14：ZrVM 具体 backend 已硬切到插件 owner；Runtime15 删除旧具体实现状态行、日期/状态映射和父级手写 row mirror，lock-poison 聚合读取改为 child source-set，插件默认测试 18/18、Runtime 计划状态 48/48 通过。F2 Scene lock guard 已同步当前局部 `MutexGuard` owner；完整 current-source structure harness 为 1297/1303，六个剩余失败均属于 Render/UI 或通用文件预算 owner。当前切片状态为 `runtime_15_zr_vm_concrete_backend_owner_hard_cut_plugin_tests_18_passed`；详细证据见 [`15/2026-07-14-zrvm-owner-status-hard-cut.md`](15/2026-07-14-zrvm-owner-status-hard-cut.md)。
- 2026-07-18 RenderProduct 测试上下文交接：`graphics/scene` 根级产品测试当前 35 个测试执行 35 次独立 offscreen backend 初始化。Runtime15 须把纯 material/shader/diagnostic projection 与真实 WGPU resident/upload 测试分层，并建立按 backend config + features/limits 键控的 test-only context pool；每测试 asset manager、streamer、error scope 保持独立，device-loss/error 测试走 serial key。兼容 key 内 backend 初始化应不超过 1 次，纯测试为 0 次，见 `PERF-MVP-406` 及 scene root product-test 静态证据。

## 2026-08-27 Runtime Operation Service Responsibility Owner Split

状态：`runtime_15_operation_service_responsibility_owner_split_static_passed_cargo_deferred`。

R1.3/M4 的当前源码切片把 `operation/service.rs` 中可独立审查的声明硬切到 folder-backed owner：容量策略归 `service/limits.rs`，任务表、phase indexes、maintenance state、handle 分配与 index compact 归 `service/task_state.rs`，JSON 精确字节预算与 UTF-8 截断归 `service/json_budget.rs`，prepare completion channel 协议声明归 `service/prepare_completion.rs`；raw-admission reservation 与释放不变量收口到既有 `service/admission.rs`。父 owner 只保留服务编排与窄的 crate-private re-export，从 862 行降到 716 行。当前 service/admission/completion/json-budget/limits/prepare-completion/task-state 行数为 716/248/332/39/28/29/59，均低于 800 行预算。

既有 phase-index 与 maintenance 源码合同改为读取 task-state owner，raw-admission 守卫改为读取 admission owner；新增 Python 静态结构回归锁定五类声明不回流父文件及父文件行数上限。operation admission、handle 分配、queued/ready index compact、deadline、completion、ABI、JSON 字节计数、UTF-8 截断和 retained-byte 算法均为原样搬移，未做性能算法修正或改变公共 API。

聚焦 Python 结构回归 exact 1/1、定向 Rust 文件 rustfmt、旧声明扫描与 scoped diff check 通过。完整聚合结构审计在当前巨大脏工作区的 124 秒窗口内未产出结果，受管 Cargo 亦未执行；该切片不能替代 Runtime 15 整体 acceptance，也未触发 milestone commit 或企微同步。

## 2026-08-27 Dynamic Runtime Project Test Owner Split

状态：`runtime_10_15_dynamic_project_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `dynamic_api/session/project.rs` 中 14 个既有 unit tests 原样硬切到 folder-backed `dynamic_api/session/project/tests.rs`。父 project owner 从 917 行降到 562 行，测试 child 为 353 行；另一会话新增的 `project/runtime61_characterization.rs` 保持独立 120 行 child 及显式 route，不被合并、改写或计入本切片完成范围。父 owner 现在只持有 project ABI path 解析、prepared project、scene/UI/navigation/script 加载和两个 test route。

新增 Python 静态结构回归锁定父文件不超过 800 行、旧内联 `mod tests {` 不回流、folder-backed test route 与 Runtime61 route 同时存在、14 个测试保持可见。HEAD 旧测试体与新 child 的 whitespace-normalized 源码逐字符等价，测试属性为 14/14；定向 rustfmt、scoped diff check 与 trailing-whitespace 扫描通过。项目启动、路径解析、manifest snapshot、scene load 和 script package 过滤算法均未改变；受管 Cargo 未执行，因此不声明 Runtime10、Runtime15 或 Runtime61 acceptance，也未触发 milestone commit/企微同步。

## 2026-08-27 Plugin System Registration Test Owner Split

状态：`runtime_06_15_plugin_system_registration_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `plugin/extension_registry/register/system_registration.rs` 末尾 3 个 per-World private-state/concurrency tests 原样硬切到 folder-backed `system_registration/tests.rs`。父 production owner 从 825 行降到 676 行，测试 child 为 149 行；另一会话新增的 `CallbackSceneSystem::retire` 生命周期实现保持在 production owner，签名与 `self.state.retire(world)` 调用均为 1/1。

新增 Python 静态结构回归锁定父文件不超过 800 行、旧内联测试不回流、3 个测试名与 `retire` 生命周期锚保持可见。HEAD 旧测试体与新 child whitespace-normalized 等价、测试属性 3/3；定向 rustfmt、scoped diff check 与空白扫描通过。该切片没有修正系统注册/调度算法：Runtime06 记录的 PERF-MVP-532/533 generation-owned per-World factory/state、同代 compiled world extension plan 和 reload/unload quiescence 仍开放；Cargo 未执行，因此不声明 Runtime06/15 acceptance 或性能瓶颈消失，也未触发 milestone commit/企微同步。

## 2026-08-27 Shader Prewarm Test Owner And Guard Routing Split

状态：`runtime_07_15_shader_prewarm_test_owner_and_source_guard_routing_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `graphics/shader/variant_cache/prewarm.rs` 中 11 个既有
unit tests 原样硬切到 folder-backed `prewarm/tests.rs`，并以显式 `#[path]` 继续挂载既有
`prewarm/tests/combined_validation_tests.rs`。父 route 从 811 行降到 142 行；生产 worker、
主体测试、组合测试分别为 251/667/159 行，全部低于 800 行 owner 预算。既有缓存键五维
source identity 修正保持不变，`prewarm/worker.rs` 与组合测试中的其它会话改动未被格式化、
改写或回退。

生产文件预算现把 route、worker、主体测试、组合测试登记为四个独立 source owner；缓存
artifact、source provenance、WGPU module、WGPU pipeline 与 validation summary 五个结构
守卫分别读取真实生产或测试文件，消除拆分后继续读取父 route 的假阳性。新增 Python
静态回归锁定 folder-backed route、11+2 测试数、四文件行数和缓存键身份参数。该回归
1/1 通过，定向 Rust `rustfmt --check` 通过，HEAD 旧测试体加当前身份参数后与新 child
whitespace-normalized 等价。Cargo 未执行，且本切片未改变预热算法或提供性能/功耗数据，
因此不声明 Runtime07/15 或 Render08 acceptance，也未触发 milestone commit/企微同步。

## 2026-08-27 Shader Module Registry Test Owner Split

状态：`runtime_08_15_shader_module_registry_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `graphics/shader/template/module_registry.rs` 中 11 个既有
unit tests 原样硬切到 folder-backed `module_registry/tests.rs`。父 production owner 从
870 行降到 656 行，测试 child 为 214 行；三个 `include_str!` 仅按新物理目录增加 `../`
路径层级。另一会话加入的 `PBR_COMMON_INCLUDE_TOKEN`、builtin closure 和三处依赖顺序断言
均完整保留，本切片未改 registry DFS、root-scoped construction、hash 或 module factory 算法。

生产文件预算现登记 registry production/test 两个真实 owner；新增 Python 静态回归锁定
folder-backed route、11 个测试、PBR common 断言和双 owner 预算路径。该回归与 prewarm
结构回归合计 4/4 通过，定向 Rust `rustfmt --check`、拆分前后 tests 源码 SHA-256 规范化
等价与 scoped diff check 通过。Cargo 未执行，因此不声明 Runtime08/15、Render08 或 shader
产品验收，也未触发 milestone commit/企微同步。

## 2026-08-27 Resolution Exact Dependency Test Owner Split

状态：`runtime_02_15_resolution_exact_dependency_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `core/runtime/tests/resolution/behavior.rs` 末尾 exact 4/5
dependency cached-key initialization 两测原样迁入 folder-backed
`behavior/exact_dependency_resolution.rs`。父 behavior owner 从 846 行降到 631 行；
dependency-cycle、exact-dependency、factory-panic 三个 child 分别为 115/217/258 行。父文件
10 个测试、三个 child 2/2/4 个测试均低于预算，另一会话新增的 `mod factory_panics;` 与
对应 4 个 panic/lifecycle tests 保持独立，不被合并或改写。

新增 Python 静态回归锁定三个 child route、父/child 测试数、exact 两个测试名不回流和
四文件行数；该回归 1/1、定向 Rust `rustfmt --check`、迁移测试体规范化 SHA-256 等价与
scoped diff check 通过。服务注册、解析、cached dependency keys、factory single-flight 和
lifecycle 算法均未修改；Cargo 未执行，因此不声明 Runtime02/15 或 service-registry 行为
验收，也未触发 milestone commit/企微同步。

## 2026-08-27 Widget Menu Control-Anchored Test Owner Split

状态：`runtime_09_15_widget_menu_control_anchored_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `ui/tests/widget_menu_behavior.rs` 中 5 个 control-anchored
overlay/frame-authority tests 原样迁入 folder-backed
`widget_menu_behavior/control_anchored_overlays.rs`。父 owner 从 861 行降到 625 行并保留
11 个通用 menu/popup behavior tests 与共享 fixtures，child 为 239 行。另一会话新增的
`component_event` 与 binding `mode` 字段保留在父级共享 helper，未被移动或回退。

新增 Python 静态回归锁定父/child 行数、11+5 测试、5 个测试名不回流及 typed binding
字段；该回归 1/1、定向 Rust `rustfmt --check`、迁移测试体规范化 SHA-256 等价与 scoped
diff check 通过。本切片未修改 UI 生产代码或交互算法；Cargo 未执行，因此不声明
Runtime09/15 acceptance，也未触发 milestone commit/企微同步。

## 2026-08-27 Activation Contention Test Owner Split

状态：`runtime_02_15_activation_contention_test_owner_split_static_passed_cargo_profile_deferred`。

R1.4/M3-M4 的当前源码切片把 `core/runtime/tests/activation/behavior/activation.rs` 的
2 个 contention/benchmark tests 与两个专用 helper 原样迁入 folder-backed
`activation/contention.rs`。父 owner 从 885 行降到 756 行并保留 11 个 activation tests，
child 为 132 行。750 ms、7 joiners、1 build 与 21 release samples 的原有性能合同未弱化。

新增 Python 静态回归锁定父/child 行数、11+2 tests、测试名不回流与四项性能锚；该回归
1/1、定向 Rust `rustfmt --check`、迁移块规范化 SHA-256 等价与 scoped diff check 通过。
本切片未修改 activation 算法，也未产生新性能样本；Cargo/profile 延后，因此不声明
Runtime02/15 或 optimize acceptance，也未触发 milestone commit/企微同步。

## 2026-08-27 RenderPipelineAsset Postprocess Plugin Input Test Owner Split

状态：`runtime_01_15_render_pipeline_postprocess_plugin_input_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把
`graphics/pipeline/render_pipeline_asset/compile_tests/postprocess_routes.rs` 中 6 个 plugin
scene-velocity / hybrid-GI input route tests 与 3 个专用 fixture 原样迁入 folder-backed
`postprocess_routes/plugin_inputs.rs`。父 postprocess route owner 从 881 行降到 622 行并保留
13 个 LUT、Bloom、blur、exposure、light-list、output-transfer 与 HZB route tests；child 为
262 行 / 6 tests。另一会话新增的 FXAA/SMAA terminal pass `has_side_effects` 断言仍留在父
owner，未被移动或回退。

现有 `runtime_15_render_pipeline_compile_tests_are_child_owners` 预算守卫已读取新 child 并将
父/child 分别纳入 800 行预算；新增 Python 静态回归锁定 13+6 测试、6 个测试名和 3 个
fixture 不回流、预算守卫使用真实路径及并发断言保持。该切片只移动测试物理 owner，未修改
RenderGraph 编译、插件 feature filter、资源依赖或 pass 排序算法。Python RED 先以 881 行
预算失败，切分后本 guard 2/2、与前五个绕阻塞切片联合 9/9 通过；9 个迁移函数
whitespace-normalized 等价，定向 `rustfmt --check` 与 scoped diff check 通过。Cargo/WGPU/
RenderDoc 均延后，因此不声明 Render01/Runtime15 acceptance 或性能瓶颈消失，也未触发
milestone commit/企微同步。

## 2026-08-27 Scene Post-Process Volumetric Fog Test Owner Split

状态：`runtime_07_15_scene_post_process_volumetric_fog_test_owner_split_static_passed_cargo_deferred`。

R1.4/M3-M4 的当前源码切片把 `scene/tests/render_post_process_extract.rs` 中 3 个 local
volumetric-fog extract tests 和唯一 `spawn_local_volumetric_box` fixture 原样迁入
folder-backed `render_post_process_extract/volumetric_fog.rs`。父 owner 从 823 行降到 677 行，
保留 11 个 camera/post-process volume tests；child 为 149 行 / 3 tests。另一会话正在推进的
fallible `spawn_node(...).expect(...)` 断言在父/child 中均完整保留。

生产边界复核确认 local volumetric fog 虽由 `PostProcessVolumeComponent` authoring，但按
camera render/culling layers 进入 `AdvancedLightingExtract.fog_volumes`，与按 camera volume
mask 进入 `PostProcessExtract.volumes` 的路径分离。新增 Python RED 先以 823 行预算失败，
迁移后 guard 1/1、4 个迁移函数 whitespace-normalized 等价、定向 rustfmt 与 scoped diff
check 通过。本切片未改 `collect_post_process_volumes`、fog bounds、layer routing 或 volume
evaluation 算法；Cargo/WGPU/RenderDoc 延后，不声明 Render07/Runtime15 acceptance 或性能
瓶颈消失，也未触发 milestone commit/企微同步。

## 2026-08-27 Native Registration Replay Typed-Error Owner Split

状态：`runtime_06_15_native_registration_replay_error_owner_split_static_passed_cargo_deferred`。

R1.3/M4 的当前源码切片把
`plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs` 中
`NativePluginRegistrationReplayError`、完整 Display 映射和 `Error::source` 链硬切到
folder-backed `registration_replay/error.rs`。父 production owner 从 955 行降到 785 行，
typed-error child 为 178 行；父模块以同级 `pub(super) use` 保持 live-host root 与 module-local
tests 的原内部路径，没有增加 public facade、alias 或兼容分支。

既有 F5 typed-error review guard 现在分别读取 orchestration 与 error owner，避免拆分后继续
扫描父文件形成假失败或假通过。新增 Python RED 先因 955 行预算和旧 guard 路径 2/2 失败，
迁移后通过；错误块除 folder-backed 层级所需的等效 `pub(in super::super)` visibility token
外 whitespace-normalized 等价，另一会话的 3 个
`sort_unstable` 和 optimization-test route 均保留，定向 rustfmt/scoped diff check 通过。本
切片随后补充了 generation 的 fail-closed 边界：replay 前显式校验 manifest 与 prepared
system 数量一致，且数量校验位于 known-component/access-authority 构造和空 manifest
早退之前；非空 manifest 缺少 replay bridge context 时返回 `BridgeCallScope` typed error，
不再依赖 release 模式会消失的
`debug_assert_eq!` 或 `expect`。新增 optimization-test source contracts 覆盖这两条边界，
定向 rustfmt/scoped diff check 与 native replay error-owner guard 通过。除上述错误路径
收束外，manifest parse、access authority、bridge call 和 system registration 算法保持不变；
当前 replay root 为 795 行（含 test-owner 挂载），optimization-test owner 为 198 行；Rust
1.94.1 直接 test harness 执行 4 passed、2 ignored（release performance lanes），未引入
外部依赖或 C 盘产物。
当前源码指纹：replay root SHA-256 为
`C9A80BE267518F1E903BDFCF1762F040A76161418D2EEC814FB026917AD169AC`，source-contract
test owner SHA-256 为
`EFDA9F946DEDEB5F1B0612D6285EA68585A8018990DC7FE3599329D5DD164205`。
Cargo/native 动态加载延后，不声明 Runtime06/15 acceptance 或性能瓶颈消失，也未触发
milestone commit/企微同步。Runtime06 主计划当前由另一会话持有写锁，本状态先由 Runtime15
与 Runtime06 编号产出记录作为权威镜像。

## 2026-08-27 FrameProfiler GPU Resolution Owner Split

状态：`runtime_17_15_frame_profiler_gpu_resolution_owner_split_static_passed_cargo_profile_deferred`。

R1.3/R1.4 与 Render17 PF-M1 的当前源码切片把
`graphics/runtime/render_framework/frame_profiler.rs` 中的延迟 GPU timer、pipeline statistics
结果归并以及 pass/subsystem budget 投影硬切到 folder-backed
`frame_profiler/gpu_resolution.rs`。父 owner 保留 current-frame profile 组装、有界 pending ring 和
stats 发布，从 936 行降到 796 行并保留 11 个 tests；GPU resolution child 为 153 行。父模块
只在原 crate-private 范围精选 re-export `FrameProfileWrite`，没有增加 public facade、兼容 alias
或第二份 profile 状态。

新增 Python RED 先以父 owner 936 行超预算失败；拆分后 source/status guard 2/2 通过。迁移的
`FrameProfileWrite`、2 个 merge 方法和 4 个匹配/预算 helper 共 7 项与 `HEAD` whitespace-
normalized 等价，定向 `rustfmt --check` 与 scoped diff check 通过。另一会话的
`GpuMemoryBudget`、memory warning 与 mesh-submission profile 变更完整保留，但不计入本切片。
本切片没有改变 pending ring 容量、pass 匹配、`Arc::make_mut`、延迟帧计算或 budget 算法，
也没有生成 CPU/GPU/allocator/RSS/power 样本；Cargo/WGPU/RenderDoc/PNG 延后，不声明
Render17/Runtime15 acceptance 或瓶颈消失，也未触发 milestone commit/企微同步。

## 2026-08-27 Dynamic Event Keyboard/IME And Gamepad Owner Split

状态：`runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_static_passed_cargo_deferred`。

R1.3/R1.4 的当前源码切片把 `dynamic_api/session/events.rs` 中 keyboard/IME payload 解析与
runtime/UI 双投影硬切到 223 行 `events/keyboard_ime.rs`，把 gamepad connection/button/axis
投影与 UI navigation/analog mapping 硬切到 115 行 `events/gamepad.rs`。父 owner 从 1038 行
降到 734 行并保留 6 个 tests，只拥有 ABI kind routing、viewport/pointer/window/lifecycle 编排
和共享 UI dispatch；child handler 只对父路由开放 `pub(super)`，没有新增 public facade、alias、
第二 input manager 或 event queue。

Python RED 先以 1038 行预算失败，迁移后 source/status guard 2/2 通过，定向 rustfmt 与 scoped
diff check 通过。3 个迁移项与 `HEAD` 规范化等价；4 个 handler 保留拆分前已经存在的
physical-input-before-UI、typed clock 和 metadata sequencing 改动，但不计入本切片。本切片
未改 event ordering、payload/UTF-8 limits、UI consumed 语义、gamepad threshold、coalescing、
recording 或 action mapping 算法，也没有新增性能样本。Cargo/app/UI/backend 验证延后，
不声明 Runtime10/12/15 acceptance，也未触发 milestone commit/企微同步。

## 2026-08-27 UiSurface Incremental Rebuild Owner Split

状态：`runtime_09_15_ui_surface_incremental_rebuild_owner_split_static_passed_cargo_profile_deferred`。

R1.3/R1.4 与 Runtime09 当前源码切片把
`zircon_runtime/src/ui/surface/surface/rebuild.rs` 中完整 `rebuild_dirty`、增量布局预算和
layout-engine selection report patch/merge helper 硬切到 711 行 folder-backed
`zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs`。500 行父 owner 继续拥有 full
rebuild、render extract、dirty mutation 与 `compute_layout`；child 只通过原有 inherent
`UiSurface` owner 执行，没有 facade、alias、第二 invalidation state 或 public API 扩张。结构
回归由 `tools/tests/test_runtime_ui_surface_incremental_rebuild_owner_structure.py` 固定父/子预算、
职责和并发行为锚点。

Python RED 先以 1194 行父文件超预算失败，迁移后 source guard 1/1 通过。`rebuild_dirty` 和
3 个 helper 共 4 项与拆分前 whitespace-normalized SHA-256 4/4 等价，定向 rustfmt 与 scoped
diff check 通过。本切片保留字体代次、布局 fallback、arranged/hit/render patch、导航索引和
frame publication 行为，未调整算法或阈值；Cargo/UI/profile/power 验证延后，不声明
Runtime09/15 acceptance 或性能收益，也未触发 milestone commit/企微同步。

## 2026-08-27 UiSurface Property Transaction Owner Split

状态：`runtime_09_15_ui_surface_property_transaction_owner_split_static_passed_cargo_profile_deferred`。

R1.3/R1.4 与 Runtime09 当前源码切片把 959 行
`zircon_runtime/src/ui/surface/surface.rs` 的完整 surface property transaction 硬切到 485 行
folder-backed `zircon_runtime/src/ui/surface/surface/property_transaction.rs`。483 行父 owner 继续
拥有 `UiSurface` state/construction、invalidation transaction、runtime style、hit/accessibility/
debug query 和 route projection；child 统一同步 tree property、component state、style、focus/
popup、editable text、clipboard revision 与 invalidation，未增加 facade、alias、第二 property
store 或 public API。结构回归由
`tools/tests/test_runtime_ui_surface_property_transaction_owner_structure.py` 固定父/子预算、职责和
并发源码锚点。

Python RED 先以 959 行父文件超预算失败，迁移后 source guard 1/1 通过。12 个移动方法/helper
与拆分前 whitespace-normalized SHA-256 12/12 等价，定向 rustfmt 与 scoped diff check 通过。
本切片完整保留 compiled binding/font generation/arranged visibility/virtual-list/hot-reload 和
editable-text/popup 并发改动，未调整 mutation、dirty、focus 或 popup 算法；Cargo/UI/profile/
power 验证延后，不声明 Runtime09/15 acceptance 或性能收益，也未触发 milestone commit/企微
同步。

## 2026-08-27 UI Pointer Component State Owner Split

状态：`runtime_09_15_ui_pointer_component_state_owner_split_static_passed_cargo_profile_deferred`。

R1.3/R1.4 与 Runtime09 当前源码切片把 887 行
`zircon_runtime/src/ui/surface/surface/pointer_component_events.rs` 中 hover/pressed/focus
component state、runtime pseudo-style propagation、render dirty 和 ancestor-root helper 硬切到
226 行 folder-backed
`zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs`。首次拆分后的
674 行父 owner 继续拥有 component event、damage、compiled binding 和 template-action payload
投影；child 仍修改同一 surface state/style/invalidation authority，没有 facade、第二事件路由或
cache。结构
回归由 `tools/tests/test_runtime_ui_pointer_component_state_owner_structure.py` 固定父/子预算、职责
与 event-owner 锚点。

Python RED 先以 887 行父文件超预算失败，迁移后 source guard 1/1 通过。7 个移动方法/helper
与拆分前 whitespace-normalized SHA-256 7/7 等价，定向 rustfmt 与 scoped diff check 通过。
本切片未调整 ancestor walk、style subtree、dirty、event ordering、binding 或 payload 算法；
Cargo/UI/profile/power 验证延后，不声明 Runtime09/15 acceptance 或性能收益，也未触发
milestone commit/企微同步。

同一 pointer event folder 随后进一步把 9 个 binding/action 方法原样迁入 262 行
`zircon_runtime/src/ui/surface/surface/pointer_component_events/template_action.rs`，父 owner 降到
426 行并只保留 event envelope、focus、damage 与 binding event emission。action child 统一拥有
compiled handle validation、action/route projection、missing-value policy 与 payload expression/
property resolution，仍消费同一 `UiSurface` tree、binding、control index 与 component state，
没有第二 dispatch 或 action registry。结构回归由
`tools/tests/test_runtime_ui_pointer_template_action_owner_structure.py` 固定父/子预算与职责。

状态：`runtime_09_15_ui_pointer_template_action_owner_split_static_passed_cargo_profile_deferred`。
结构 RED 先以 674 行父 owner 超出 550 行边界失败；迁移后 9/9 方法 whitespace-normalized
SHA-256 等价。该切片未改变 event ordering、compiled handle mapping、missing-value policy、
payload evaluation 或 allocation 算法；Cargo/UI/profile/power 验证继续延后，不声明 Runtime09/15
acceptance、性能收益或瓶颈消失，也未触发 milestone commit/企微同步。

## 2026-08-28 Winit Translation Domain Owner Split

状态：`runtime_09_15_winit_translation_domain_owner_split_static_passed_cargo_product_profile_deferred`。

R1.3/R1.4 与 Runtime09 当前源码切片把 785 行
`zircon_runtime/src/ui/platform_input/winit_translation.rs` 中平台事件路由与键盘、指针、IME、
窗口 metadata/metrics 翻译职责分开。530 行根 owner 保留唯一 `WindowEvent` dispatch、公开
modifier adapter 和约 430 行既有行为测试；40/161/52/51 行的
`winit_translation/{keyboard,pointer,ime,window}.rs` child 直接构造原有 pump event，不建立
第二 queue、window state、dispatch authority、DTO 或兼容 facade。结构回归由
`tools/tests/test_runtime_ui_winit_translation_owner_structure.py` 固定模块路由、预算、职责锚点
和四份计划状态镜像。

Python RED 先记录旧根 785 行超预算，拆分后 source/status guard 2/2 通过。17 个移动函数体
相对 `HEAD` 的去空白 SHA-256 17/17 等价，定向 rustfmt 与 scoped diff check 通过。事件顺序、
synthetic、touch ID、scroll scale、IME byte clamp、window metadata/metrics 与 normalization
算法未改变；Cargo/UI 产品/profile/power 验证延后，不声明 Runtime09/15 acceptance、性能收益
或瓶颈消失，也未触发 milestone commit/企微同步。

Scene property enumeration 应与既有 `property_access/write/{camera,mesh,lighting,animation,
physics}.rs` 使用同一组件域拓扑，而不是让一个 umbrella owner 持续吸收所有固定组件。当前
`zircon_runtime/src/scene/world/property_access/entries.rs` 为 210 行编排 owner；camera、mesh、
lighting、animation child 分别为 49/122/153/175 行，既有 physics child 保持 513 行。根 owner
只保留基础实体、domain order、dynamic metadata 与总 capacity；child 仍访问同一个 `World`，
不得建立第二 reflection registry、Editor cache、property DTO 或写入入口。结构回归由
`tools/tests/test_runtime_scene_property_entry_owner_structure.py` 固定路径、预算、顺序和职责。

状态：`runtime_08_15_scene_property_entry_component_owner_split_static_passed_cargo_profile_deferred`。
RED 先以旧根 567 行超过 280 行失败；迁移后 10/10 投影/capacity/helper 块与 `HEAD` 基线
whitespace-normalized SHA-256 等价。property order/path/value/animatable/capacity 算法均未改变；
Cargo、Editor Inspector 与 profile/power 门继续延后，不声明 Runtime08/15 acceptance、性能收益
或瓶颈消失，也未触发 milestone commit/企微同步。

UI asset hot-reload 索引与 retained node-resource 登记也必须按生命周期分责。758 行
`zircon_runtime/src/ui/template/asset/surface_index.rs` 根 owner 保留 surface/tree 正反向索引、
resource reverse edge、affected-surface selection 与 hot-reload targeting；175 行
`zircon_runtime/src/ui/template/asset/surface_index/node_resource_registration.rs` child 只负责从
已实例化 node metadata 容错投影 URI、kind 与 fallback。编译期严格 schema/diagnostic owner
仍是 `ui/template/asset/resource_ref/collect.rs`，不得为了代码复用合并两个不同错误合同。
`tools/tests/test_runtime_ui_asset_surface_node_resource_owner_structure.py` 固定父/子预算、模块
路由、严格/容错 owner 边界和文档镜像。

状态：`runtime_09_15_ui_asset_surface_node_resource_owner_split_static_passed_cargo_profile_deferred`。
结构 RED 先以 918 行根 owner 超过 800 行失败；迁移后 11/11 方法/helper 的
whitespace-normalized SHA-256 等价。该切片未改变 parser、schema、fallback、dedup、reverse
index 或 hot-reload 算法；Cargo/UI/profile/power 验证继续延后，不声明 Runtime09/15
acceptance、性能收益或瓶颈消失，也未触发 milestone commit/企微同步。

## 2026-08-28 Plugin Manifest Constructor Owner Split

状态：`runtime_06_15_plugin_manifest_constructor_owner_split_static_passed_cargo_deferred`。

R1.3/R1.4 与 Runtime06 当前源码切片把 497 行
`zircon_runtime/src/plugin/package_manifest/constructors.rs` 中 package 与 module descriptor
构造职责按描述层级硬切。4 行 root 只挂载 child；169 行 `constructors/module.rs` 拥有模块
kind、init level、module dependency、target mode 与 runtime `ModuleDescriptor` 投影；331 行
`constructors/package.rs` 拥有 package identity、capability/content、feature、shader、packaging 与
distribution 构造。二者继续为原公开类型提供相同固有方法，不新增 facade、builder、DTO 或旧
路径 shim。结构回归由
`tools/tests/test_runtime_plugin_manifest_constructor_owner_structure.py` 固定预算、职责和状态镜像。

RED 先以旧 root 497 行超预算失败；两个完整 `impl` 与两个 helper 相对 `HEAD` 的去空白
SHA-256 4/4 等价。该边界对照 Unreal Projects 中 `FPluginDescriptor` 组合独立
`FModuleDescriptor` 的职责分层。Cargo/plugin 产品验证延后，不声明 Runtime06/15 acceptance
或性能收益，也未触发 milestone commit/企微同步。

## 2026-08-28 Native System Access Owner Split

状态：`runtime_06_15_native_system_access_owner_split_static_passed_cargo_profile_deferred`。

R1.3/R1.4 与 Runtime06 当前源码切片把 510 行 native registration system-access owner 中的
plugin capability/ownership authorization 和 parse/authorize/resolve typed errors 分别迁入
89 行 `system_access/authority.rs` 与 123 行 `system_access/error.rs`。318 行
`system_access.rs` root 保留 declaration/plan、manifest parser、确定性排序、World access compile
和原有内联行为测试；parent re-export 继续保持 registration-manifest 内部类型路径，不增加
authority cache、access registry、World projection 或兼容 facade。结构回归由
`tools/tests/test_runtime_native_system_access_owner_structure.py` 固定职责、预算和状态镜像。

RED 先以旧 root 510 行超预算失败；12 个移动定义/实现块相对 `HEAD` 的去空白 SHA-256
12/12 等价。Unreal descriptor admission 与 module loading 分层为主边界参考，Bevy
`SystemParamAccess` 为冲突解析交叉检查。worker affinity/capability、owned/foreign stable ID、
`write:world` exclusivity、排序、resolve 和 error text 均未改变；Cargo/native product/profile
延后，不声明 Runtime06/15 acceptance 或性能收益，也未触发 milestone commit/企微同步。

## 2026-08-28 Asset Artifact Material/Shader Owner Split

状态：`runtime_04_15_asset_artifact_material_shader_owner_split_static_passed_cargo_deferred`。

Runtime04/15 将 `zircon_runtime/src/asset/artifact/cache_payload/material_shader.rs` 从 637 行
Material/Shader 混合 schema owner 硬切为 5 行 wiring root、161 行
`zircon_runtime/src/asset/artifact/cache_payload/material_shader/material.rs` 和 487 行
`zircon_runtime/src/asset/artifact/cache_payload/material_shader/shader.rs`。父级
`cache_payload.rs` 继续消费同名私有类型，bincode variant 顺序、字段顺序、Serde default 与
asset conversion 全部保持原合同；两个移动块相对 `HEAD` 的规范化 SHA-256 均完全相等。

Unreal 的 Materials 与 Shader/ShaderCore 所有权分离作为主参考，Bevy PBR material owner
作为轻量交叉检查。本切片不改序列化算法、缓存格式、导入行为或热路径，也不引入兼容
facade。`tools/tests/test_runtime_asset_artifact_material_shader_owner_structure.py` 锁定根文件
导航职责、子 owner 预算、字段顺序与四份状态镜像；Cargo/product validation 延后，不声明
Runtime04/15 acceptance、性能收益、milestone commit 或企微同步。

## 2026-08-28 ECS Component Registry Transfer Owner Split

状态：`runtime_08_15_component_registry_transfer_owner_split_static_passed_cargo_deferred`。

Runtime08/15 将 559 行
`zircon_runtime/src/scene/ecs/component/registry.rs` 收束为 154 行 identity/layout root、259 行
`zircon_runtime/src/scene/ecs/component/registry/transferred.rs` 事务 owner 和 174 行
`zircon_runtime/src/scene/ecs/component/registry/tests.rs` 测试 owner。结构回归由
`tools/tests/test_runtime_ecs_component_registry_transfer_owner_structure.py` 固定 owner 预算、
preflight/publish 不变量、六个行为测试与四份状态镜像；Runtime08 生产清单同步为 76 文件。

Unreal `CoreUObject` 稳定类型/布局 owner 与 package reload 阶段事务是主工程参考，Bevy
`Components`/queued registrator 是 ECS 交叉检查。16 个移动块相对 `HEAD` 的规范化 SHA-256
16/16 等价；不改 component ID、冲突判断、pending 顺序、reserve 或 publication 算法，
不引入第二 registry/facade。Cargo/product validation 延后，不声明 Runtime08/15 acceptance、
性能收益、milestone commit 或企微同步。

## 2026-08-28 Plugin Availability Evaluation/Selection Owner Split

状态：`runtime_06_15_plugin_availability_evaluation_selection_owner_split_static_passed_cargo_deferred`。

Runtime06/15 将 636 行
`zircon_runtime/src/plugin/runtime_profile/availability_projection.rs` 收束为 291 行 membership/
construction root、282 行
`zircon_runtime/src/plugin/runtime_profile/availability_projection/evaluation.rs` 和 91 行
`zircon_runtime/src/plugin/runtime_profile/availability_projection/selection.rs`。结构回归由
`tools/tests/test_runtime_plugin_availability_owner_structure.py` 固定 owner 预算、evaluation
判定顺序、manifest first-position/required merge 不变量与五份状态镜像。

Unreal provider status 与 project plugin reference/target rules 的分层为主参考。14 个移动块
相对 `HEAD` 的规范化 SHA-256 14/14 等价；不改 borrowed membership、线性 selection/index
merge、generation builder 或 availability 分类语义，也不引入第二 registry/cache/facade。
Cargo/product validation 延后，不声明 Runtime06/15 acceptance、性能收益、milestone commit
或企微同步。

## 2026-08-28 Asset Management Family/Record-Set Owner Split

状态：`runtime_04_15_asset_management_generation_static_implemented_cargo_deferred`。

Runtime04/15 将 572 行 `zircon_runtime/src/asset/management.rs` 收束为 164 行 DTO declaration
root、176 行 `zircon_runtime/src/asset/management/family.rs` 与 254 行
`zircon_runtime/src/asset/management/record_sets.rs`。结构回归由
`tools/tests/test_runtime_asset_management_owner_structure.py` 固定 owner 预算、11 个公开 DTO
顺序、family status/issue 分类不变量和 Model/Mesh/Scene/Entity/Material/Shader 聚合顺序。

Unreal `AssetRegistryState` 的 canonical entry/query index owner 与上层 aggregate/config owner
分层是主工程参考。21 个声明/实现块相对 `HEAD` 的规范化 SHA-256 21/21 等价；不改 API、
Serde 字段或聚合公式。`ProjectAssetManager` 现已补齐 immutable asset-only generation
projection、generation-fenced publication 与 graphics-side renderer-material composition；Cargo/
product/profile validation 延后，不声明 Runtime04/15 acceptance、性能收益、milestone commit
或企微同步。

## 2026-08-28 Viewport Output Planner Owner Split

状态：`runtime_render_09_15_viewport_output_planner_owner_split_static_passed_cargo_deferred`。

Render09/Runtime15 将 757 行
`zircon_runtime/src/graphics/types/viewport_render_output_target.rs` 收束为 83 行 target
declaration/resolution root、217 行
`zircon_runtime/src/graphics/types/viewport_render_output_target/writeback.rs`、217 行
`zircon_runtime/src/graphics/types/viewport_render_output_target/graph_import.rs` 与 268 行
`zircon_runtime/src/graphics/types/viewport_render_output_target/tests.rs`。结构回归由
`tools/tests/test_runtime_viewport_output_planner_owner_structure.py` 固定 owner 预算、规划顺序、
两个 format path 和 12 个测试。

Unreal `FRenderTarget` 的目标资源/尺寸/色彩契约与 `FSceneViewFamily` 目标绑定是主工程参考；
graph import 与 final writeback 保持消费阶段 owner。26 个生产/行为块相对 `HEAD` 的规范化
SHA-256 26/26 等价；不改 API、状态枚举、格式匹配或 conversion 决策，不引入第二 target
authority。PERF-MVP-417 retained camera-plan 仍开放；Cargo/product/profile validation 延后，
不声明 Render09/Runtime15 acceptance、性能收益、milestone commit 或企微同步。

## 2026-08-28 ECS Archetype Topology Equality Receipt

状态：`runtime_08_15_archetype_topology_equality_receipt_static_passed_cargo_deferred`。

Runtime08/15 删除了 `ArchetypeIndex` 的恒真 `PartialEq`，以 owner-local borrowed
`ArchetypeTopologySnapshot` 明确结构身份。receipt 比较两个 canonical lookup index、ordered
records 及 archetype ID/signature/entity rows，同时明确排除 performance counters 与 membership
revision history；既有 `World` derive 路径保持不变，没有引入 public DTO、第二快照存储、缓存或
兼容 facade。三项 Rust 回归由 `archetype/index/tests.rs` 子 owner 承担，focused Python guard
固定源码与状态镜像。Cargo/product/profile validation 延后，不声明 Runtime08/15 acceptance、
性能收益、milestone commit 或企微同步。

Runtime08 aggregate audit 的旧 source count 已从 75 同步到实际 76；继续暴露的 dual storage、
component-storage import、entity generation 与 observer bucket 四条风险属于正在变化的共享 owner，
不计入本 topology focused static pass，也未在本切片中关闭。

## 2026-08-29 Shader Prewarm Asset Inventory Owner Split

状态：`runtime_15_shader_prewarm_asset_inventory_owner_split_static_metadata_passed_cargo_product_deferred`。

Runtime15 将 781 行 shader-prewarm asset inventory 收束为 224 行采集编排 root、431 行 warm
snapshot/index owner 与 159 行安全目录遍历 owner。Unreal `FAssetDataGatherer` 与
`FAssetRegistryState` 的 discovery/state 分层是主参考；schema v4、排序、text budget、相对路径与
reparse 拒绝、payload-before-index 发布顺序均不变。结构守卫 `1/1` 与 actual-source isolated
metadata compile 通过，搬移源码为 3,784/3,784 normalized tokens；managed Cargo、产品执行与
profile 仍延后，不声明 Runtime15 acceptance、milestone commit 或企微同步。完整证据见
[`15/2026-08-29-shader-prewarm-asset-inventory-owner-split.md`](15/2026-08-29-shader-prewarm-asset-inventory-owner-split.md)。

## 2026-08-30 Runtime Resource Factory Failure Boundary

Runtime15 在现有 immutable `Fn() + Send + Sync` resource factory 上补齐了
world-extension 的失败边界。factory 现在只在生成资源值、尚未写入 `World` 的阶段被
`catch_unwind` 保护；panic payload 会沿既有 `WorldRuntimeExtensionError::registration_failed`
带着 `resource:<type>` key 返回，失败 World 不会留下半初始化资源，后续 World 仍可重试同一
factory。该边界没有把任意 scene callback 统一吞掉，也没有引入第二套错误、缓存或锁。

源码回归由 `resource_factory_panic_is_reported_without_partial_world_mutation` 覆盖；资源
concurrency/owner validation 相关测试共 7 项。当前只完成源码与静态检查，managed Cargo、
完整 Runtime06/15 gate、性能/功耗验证仍 pending，故不关闭里程碑、不提交 commit 或企微同步。
