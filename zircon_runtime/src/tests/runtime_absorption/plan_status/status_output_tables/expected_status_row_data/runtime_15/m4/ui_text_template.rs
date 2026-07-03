type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
        "Runtime 15 M4 font database descriptor helper owner split",
        &[
            "runtime_15_font_database_descriptor_helper_owner_split_static_passed_cargo_deferred",
            "graphics/text/font/database.rs",
            "graphics/text/font/descriptors.rs",
            "graphics/text/font/matching.rs",
            "runtime_15_font_database_descriptor_helpers_are_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 screen-space UI text font-id report owner split",
        &[
            "runtime_15_screen_space_ui_text_font_id_report_owner_split_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/ui/text.rs",
            "graphics/scene/scene_renderer/ui/text/font_id_report.rs",
            "runtime_15_screen_space_ui_text_font_id_report_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 screen-space UI text font-id report visibility sync",
        &[
            "runtime_15_screen_space_ui_text_font_id_report_visibility_sync_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/ui/text/font_id_report.rs",
            "pub(super) struct ScreenSpaceUiTextFontIdReport",
            "pub(super) text_batch_count",
            "runtime_15_screen_space_ui_text_font_id_report_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M4 screen-space UI text tests owner split",
        &[
            "runtime_15_screen_space_ui_text_tests_owner_split_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/ui/text.rs",
            "graphics/scene/scene_renderer/ui/text/tests.rs",
            "runtime_15_screen_space_ui_text_tests_are_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M4 screen-space UI text tests status-map anchor sync",
        &[
            "runtime_15_screen_space_ui_text_tests_status_map_anchor_sync_static_passed_cargo_deferred",
            "expected_slices/status/runtime_15/m4_surface_cleanup.rs",
            "expected_slices/date/runtime_15/m4_surface_cleanup.rs",
            "graphics/scene/scene_renderer/ui/text.rs",
            "graphics/scene/scene_renderer/ui/text/tests.rs",
            "runtime_15_screen_space_ui_text_tests_are_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M4 SDF atlas/render tests folder-backed guard sync",
        &[
            "runtime_15_sdf_atlas_render_tests_folder_backed_guard_sync_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs",
            "graphics/scene/scene_renderer/ui/sdf_atlas/tests/plan.rs",
            "graphics/scene/scene_renderer/ui/sdf_atlas/tests/allocation.rs",
            "graphics/scene/scene_renderer/ui/sdf_atlas/tests/owner.rs",
            "graphics/scene/scene_renderer/ui/sdf_render/vertices.rs",
            "graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs",
            "graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs",
            "graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs",
            "graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs",
            "runtime_15_screen_space_ui_sdf_atlas_tests_are_child_owner_split",
            "runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split",
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
        "Runtime 15 M4 UI dispatch input manager test owner split",
        &[
            "runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred",
            "ui/dispatch/input_manager/manager.rs",
            "ui/dispatch/input_manager/manager/tests.rs",
            "runtime_15_ui_dispatch_input_manager_tests_are_child_owner",
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
        "Runtime 15 M4 UI v2 style token-resolution owner split",
        &[
            "runtime_15_ui_v2_style_token_resolution_owner_split_static_passed_cargo_deferred",
            "ui/v2/style.rs",
            "ui/v2/style/tokens.rs",
            "runtime_15_ui_v2_style_token_resolution_is_child_owner",
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
];
