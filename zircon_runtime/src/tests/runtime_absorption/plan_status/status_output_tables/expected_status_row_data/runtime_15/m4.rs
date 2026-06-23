use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 M4 core runtime service-list owner split",
        &[
            "runtime_15_core_runtime_service_lists_folder_split_static_passed_cargo_lock_blocked",
            "core/runtime/handle/registration/service_lists/mod.rs",
            "core/runtime/handle/registration/service_lists/specialized.rs",
            "runtime_15_core_runtime_service_lists_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU command validation render-state owner split",
        &[
            "runtime_15_rhi_wgpu_command_validation_render_state_split_static_passed_cargo_lock_blocked",
            "rhi_wgpu/command_validation.rs",
            "rhi_wgpu/command_validation/render_state.rs",
            "runtime_15_rhi_wgpu_command_validation_state_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
        &[
            "runtime_15_rhi_wgpu_ui_surface_render_setup_owner_split_static_passed_cargo_timeout_no_result",
            "rhi_wgpu/ui_surface.rs",
            "rhi_wgpu/ui_surface/render_pass.rs",
            "runtime_15_rhi_wgpu_ui_surface_render_setup_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 RHI WGPU UI surface geometry test owner split",
        &[
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result",
            "rhi_wgpu/ui_surface/geometry.rs",
            "rhi_wgpu/ui_surface/geometry/tests.rs",
            "runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 material asset value/readiness helper owner split",
        &[
            "runtime_15_material_asset_value_readiness_owner_split_static_passed_cargo_timeout_no_result",
            "asset/assets/material/material_asset.rs",
            "asset/assets/material/material_asset/value_sync.rs",
            "runtime_15_material_asset_value_readiness_helpers_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 core runtime render-stats graph execution-resources owner split",
        &[
            "runtime_15_render_stats_graph_execution_resources_owner_split_static_passed_cargo_timeout_no_result",
            "core/runtime/diagnostics/render_stats_store/graph.rs",
            "core/runtime/diagnostics/render_stats_store/graph/execution_resources.rs",
            "runtime_15_render_stats_graph_execution_resources_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene fixed light reflection write-field owner split",
        &[
            "runtime_15_scene_fixed_light_reflection_write_fields_owner_split_static_passed_cargo_lock_blocked",
            "scene/reflect/fixed/lights.rs",
            "scene/reflect/fixed/lights/write_fields.rs",
            "runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene world property-access physics write owner split",
        &[
            "runtime_15_scene_world_property_access_physics_owner_split_static_passed_cargo_timeout_no_result",
            "scene/world/property_access/write.rs",
            "scene/world/property_access/write/physics.rs",
            "runtime_15_scene_world_property_access_physics_writes_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene world property-access physics entry owner split",
        &[
            "runtime_15_scene_world_property_access_physics_entries_owner_split_static_passed_cargo_lock_blocked",
            "scene/world/property_access/entries.rs",
            "scene/world/property_access/entries/physics.rs",
            "runtime_15_scene_world_property_access_physics_entries_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 scene world project I/O mesh owner split",
        &[
            "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
            "scene/world/project_io.rs",
            "scene/world/project_io/mesh.rs",
            "runtime_15_scene_world_project_io_mesh_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI text layout engine visual-order owner split",
        &[
            "runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred",
            "ui/text/layout_engine.rs",
            "ui/text/layout_engine/visual_order.rs",
            "runtime_15_ui_text_layout_engine_visual_order_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI layout arrange grid/masonry owner split",
        &[
            "runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred",
            "ui/layout/pass/arrange.rs",
            "ui/layout/pass/arrange/grid_masonry.rs",
            "runtime_15_ui_layout_arrange_grid_masonry_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI template MUI X DataGrid class owner split",
        &[
            "runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred",
            "ui/template/asset/compiler/style_apply/mui_x_classes.rs",
            "ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs",
            "runtime_15_ui_template_mui_x_data_grid_classes_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI template document validation owner split",
        &[
            "runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred",
            "ui/template/asset/document.rs",
            "ui/template/asset/document/validation.rs",
            "runtime_15_ui_template_document_validation_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI template style slot-contract owner split",
        &[
            "runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result",
            "ui/template/asset/compiler/style_apply.rs",
            "ui/template/asset/compiler/style_apply/slot_contract.rs",
            "runtime_15_ui_template_style_slot_contract_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI v2 style runtime-state owner split",
        &[
            "runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred",
            "ui/v2/style.rs",
            "ui/v2/style/runtime_state.rs",
            "runtime_15_ui_v2_style_runtime_state_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI accessibility extract state owner split",
        &[
            "runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred",
            "ui/accessibility/extract.rs",
            "ui/accessibility/extract/state.rs",
            "runtime_15_ui_accessibility_extract_state_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI component catalog editor-showcase helper owner split",
        &[
            "runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result",
            "ui/component/catalog/editor_showcase.rs",
            "ui/component/catalog/editor_showcase/helpers.rs",
            "runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split",
        &[
            "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred",
            "ui/component/state_reducer/keyboard/menu.rs",
            "ui/component/state_reducer/keyboard/menu/submenu.rs",
            "runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI component state-reducer tree view editing owner split",
        &[
            "runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred",
            "ui/component/state_reducer/tree_view.rs",
            "ui/component/state_reducer/tree_view/editing.rs",
            "runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI surface event-routing owner split",
        &[
            "runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred",
            "ui/surface/surface.rs",
            "ui/surface/surface/event_routing.rs",
            "ui/surface/surface/pointer_component_events.rs",
            "runtime_15_ui_surface_event_routing_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI surface property mutation metadata dirty owner split",
        &[
            "runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred",
            "ui/surface/property_mutation.rs",
            "ui/surface/property_mutation/metadata_dirty.rs",
            "runtime_15_ui_surface_property_mutation_metadata_dirty_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 UI surface render feedback command/color owner split",
        &[
            "runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred",
            "ui/surface/render/feedback.rs",
            "ui/surface/render/feedback/colors.rs",
            "ui/surface/render/feedback/commands.rs",
            "runtime_15_ui_surface_render_feedback_commands_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 UI surface default-interactions keyboard/timer owner split",
        &[
            "runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions.rs",
            "ui/surface/surface/default_interactions/keyboard.rs",
            "ui/surface/surface/default_interactions/timers.rs",
            "runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners",
        ],
    ),
    (
        "Runtime 15 M4 UI surface table column helper owner split",
        &[
            "runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions/table/mod.rs",
            "ui/surface/surface/default_interactions/table/columns.rs",
            "runtime_15_ui_surface_table_column_helpers_are_child_owner",
        ],
    ),
];
