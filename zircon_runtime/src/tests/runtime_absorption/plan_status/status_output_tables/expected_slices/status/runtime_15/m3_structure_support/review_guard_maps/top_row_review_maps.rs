pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review top-row status row-data child-owner split" => Some(
            "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 D-S7 static plugin manifest generation/parity review sync" => {
            Some("ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D7 core workspace dependency top-row closed status sync" => {
            Some("d7_core_workspace_dependency_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D7 core workspace dependency inheritance guard" => {
            Some("d7_core_workspace_dependency_inheritance_guard_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D8 runtime registration builder original evidence paths" => {
            Some("d8_runtime_registration_builder_original_paths_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync" => {
            Some("d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F5/F6/F7 typed-error top-row closed status sync" => {
            Some("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F8/F9/F10 runtime surface top-row closed status sync" => {
            Some("f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F13/F14 provider diagnostics top-row closed status sync" => {
            Some("f13_f14_provider_diagnostics_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F17/F18 lookup/manager top-row closed status sync" => {
            Some("f17_f18_lookup_manager_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 F19 scene renderer construction top-row closed status sync" => {
            Some("f19_scene_renderer_construction_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D9 editor/runtime mirror consumer guard" => {
            Some("d9_editor_runtime_mirror_consumers_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D5 editor authoring macro consumer guard" => {
            Some("d5_editor_authoring_macro_consumers_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D12 runtime helper export macro review sync" => {
            Some("d12_runtime_export_macro_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D1 capability single-source review sync" => {
            Some("d1_capability_single_source_review_synced_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D10 animation/physics bridge call migration" => {
            Some("d10_animation_physics_bridge_call_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D11 animation/physics TestRuntime fixture migration" => {
            Some("d11_animation_physics_test_runtime_fixture_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D13 importer manifest parity guard" => {
            Some("d13_importer_manifest_parity_guard_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 P0/DX priority D13 parity sync" => {
            Some("review_priority_recommendation_d13_parity_sync_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D13 importer top-row closed status sync" => {
            Some("d13_importer_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync" => {
            Some("ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 P0 F1/F2/F4 top-row closed status sync" => {
            Some("p0_f1_f2_f4_top_row_closed_status_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
