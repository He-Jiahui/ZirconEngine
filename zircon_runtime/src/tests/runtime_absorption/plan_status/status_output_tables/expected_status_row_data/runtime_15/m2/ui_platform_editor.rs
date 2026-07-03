type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover",
        &[
            "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/component/catalog/editor_showcase/descriptor_builders.rs",
            "ui/component/catalog/editor_showcase.rs",
            "runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 UI table sortingMode server literal allowed-context sync",
        &[
            "runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions/table/columns.rs",
            "non_network_server_naming.py",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
        ],
    ),
    (
        "Runtime 15 M2 platform input DOM keycode naming hard cutover",
        &[
            "runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "ui/platform_input/keyboard_map.rs",
            "dom_key_code",
            "runtime_15_platform_input_uses_dom_keycode_names",
        ],
    ),
    (
        "Runtime 15 M2 platform input runtime baseline test naming hard cutover",
        &[
            "runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/platform_input/winit_translation.rs",
            "runtime_input_baseline",
            "runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names",
        ],
    ),
    (
        "Runtime 15 M2 UI template schema source fixture naming hard cutover",
        &[
            "runtime_15_ui_template_schema_source_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/template/asset/schema/migrator.rs",
            "SourceTemplateFixture",
            "runtime_15_ui_template_schema_uses_source_fixture_names",
        ],
    ),
    (
        "Runtime 15 M2 input mouse-wheel line-delta naming hard cutover",
        &[
            "runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/input/mouse_wheel.rs",
            "vertical_line_delta",
            "runtime_15_input_mouse_wheel_line_delta_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 DDS upload policy naming hard cutover",
        &[
            "runtime_15_dds_upload_policy_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/assets/texture/upload_support/dds.rs",
            "asset/tests/assets/texture_upload_readiness/container_fixtures.rs",
            "dds_classic_fourcc_upload_layout",
            "dds_classic_cubemap_bytes",
            "runtime_15_dds_upload_policy_uses_classic_container_names",
        ],
    ),
    (
        "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover",
        &[
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/assets/material/material_asset.rs",
            "property_overrides_with_schema_v1_defaults",
            "texture_slots_with_schema_v1_defaults",
            "schema_v1_pbr_texture_slots",
            "naming_boundary/runtime_15_m2/asset_schema.rs",
            "runtime_15_material_asset_schema_v1_defaults_use_versioned_names",
        ],
    ),
    (
        "Runtime 15 M2 font/UI asset schema naming hard cutover",
        &[
            "runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/assets/font.rs",
            "asset/importer/ingest/ui_v2_document_import.rs",
            "asset/importer/ingest/import_ui_zui_asset.rs",
            "schema_v1_render_mode",
            "runtime_15_font_ui_asset_schema_names_use_current_policy_terms",
        ],
    ),
    (
        "Runtime 15 M2 font render-mode priority fixture naming hard cutover",
        &[
            "runtime_15_font_render_mode_priority_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/ui/font_asset.rs",
            "schema_v1_render_mode_takes_priority_over_strategy_default_mode",
            "runtime_15_font_render_mode_priority_fixture_uses_schema_v1_name",
            "module_convention_gate classified-and-clear",
            "migration_debt_count=0",
        ],
    ),
    (
        "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover",
        &[
            "runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred",
            "zircon_plugins/net/features/http/runtime/src/backend/client.rs",
            "zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs",
            "http1_client_policy::plain_http_client()",
            "external-hyper-http1-client-policy",
            "runtime_15_net_http_hyper_http1_client_policy_is_isolated",
        ],
    ),
    (
        "Runtime 15 M2 Hub message raw text policy hard cutover",
        &[
            "runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred",
            "zircon_hub/src/state/hub_message/message.rs",
            "zircon_hub/src/tauri_app/runtime_state/build_actions.rs",
            "HubMessage::raw_text",
            "runtime_15_hub_message_raw_text_policy_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 editor workbench authority-label naming hard cutover",
        &[
            "runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
            "Selected Condition_Night   editor authority",
            "non_network_server_naming.py",
            "runtime_15_editor_workbench_authority_label_uses_editor_name",
        ],
    ),
    (
        "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
        &[
            "runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs",
            "draw_host_workbench_window",
            "split_archived_table_text",
            "WorkbenchExtensionIconLibraryArchivedTableRow",
            "runtime_15_editor_workbench_archived_fixtures_use_current_names",
        ],
    ),
];
