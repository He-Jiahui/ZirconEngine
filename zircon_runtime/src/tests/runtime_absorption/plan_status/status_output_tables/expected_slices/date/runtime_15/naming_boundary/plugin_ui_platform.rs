pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M2 plugin static manifest contract owner naming hard cutover" {
        // Status anchor:
        // runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs,
        // plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs,
        // plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs,
        // plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs.
        // Guard anchor: runtime_15_plugin_static_manifest_contract_owners_use_domain_names.
        Some("2026-06-25")
    } else if slice
        == "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover"
    {
        // Status anchor:
        // runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: ui/component/catalog/editor_showcase/descriptor_builders.rs.
        // Guard anchor: runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 UI table sortingMode server literal allowed-context sync" {
        // Status anchor:
        // runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred.
        // Evidence anchor: ui/surface/surface/default_interactions/table/columns.rs.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchors: runtime_non_network_server_naming_is_classified_by_owner,
        // runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 platform input DOM keycode naming hard cutover" {
        // Status anchor:
        // runtime_15_platform_input_dom_keycode_naming_hard_cutover_static_passed_cargo_timeout_no_result.
        // Evidence anchor: ui/platform_input/keyboard_map.rs.
        // Function anchor: dom_key_code.
        // Guard anchor: runtime_15_platform_input_uses_dom_keycode_names.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 platform input runtime baseline test naming hard cutover" {
        // Status anchor:
        // runtime_15_platform_input_runtime_baseline_test_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor: ui/platform_input/winit_translation.rs.
        // Test-name anchor: runtime_input_baseline.
        // Guard anchor: runtime_15_platform_input_winit_tests_use_runtime_input_baseline_names.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 UI template schema source fixture naming hard cutover" {
        // Status anchor:
        // runtime_15_ui_template_schema_source_fixture_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // ui/template/asset/schema/migrator.rs,
        // zircon_runtime_interface/src/ui/template/asset/schema/report.rs.
        // Enum anchor: SourceTemplateFixture.
        // Guard anchor: runtime_15_ui_template_schema_uses_source_fixture_names.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 input mouse-wheel line-delta naming hard cutover" {
        // Status anchor:
        // runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // core/framework/input/mouse_wheel.rs,
        // input/runtime/default_input_manager.rs,
        // dynamic_api/session/events.rs.
        // Guard anchor: runtime_15_input_mouse_wheel_line_delta_uses_current_names.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 Net HTTP backend Hyper HTTP/1 client policy hard cutover" {
        // Status anchor:
        // runtime_15_net_http_hyper_http1_client_policy_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // zircon_plugins/net/features/http/runtime/src/backend/client.rs,
        // zircon_plugins/net/features/http/runtime/src/backend/http1_client_policy.rs.
        // Audit anchor: external-hyper-http1-client-policy.
        // Guard anchor: runtime_15_net_http_hyper_http1_client_policy_is_isolated.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 Hub message raw text policy hard cutover" {
        // Status anchor:
        // runtime_15_hub_message_raw_text_policy_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // zircon_hub/src/state/hub_message/message.rs,
        // zircon_hub/src/tauri_app/runtime_state/build_actions.rs.
        // Raw text anchor: HubMessage::raw_text.
        // Guard anchor: runtime_15_hub_message_raw_text_policy_uses_current_names.
        Some("2026-06-27")
    } else if slice == "Runtime 15 M2 editor workbench authority-label naming hard cutover" {
        // Status anchor:
        // runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchor:
        // zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs.
        // Output anchor: Selected Condition_Night   editor authority.
        // Audit anchor: non_network_server_naming.py.
        // Guard anchor: runtime_15_editor_workbench_authority_label_uses_editor_name.
        Some("2026-06-25")
    } else if slice == "Runtime 15 M2 editor Workbench archived fixture naming hard cutover" {
        // Status anchor:
        // runtime_15_editor_workbench_archived_fixture_naming_hard_cutover_static_passed_cargo_deferred.
        // Evidence anchors:
        // zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/host_window.rs,
        // zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/cells/text.rs.
        // Entry anchor: draw_host_workbench_window.
        // Guard anchor: runtime_15_editor_workbench_archived_fixtures_use_current_names.
        Some("2026-06-27")
    } else {
        None
    }
}
