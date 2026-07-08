pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 runtime diagnostics test folder split" {
        Some("runtime_15_runtime_diagnostics_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI command list test folder split" {
        Some("runtime_15_rhi_command_list_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 RHI device contract test folder split" {
        Some("runtime_15_rhi_device_contract_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset pack test folder split" {
        Some("runtime_15_asset_pack_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset facade test folder split" {
        Some("runtime_15_asset_facade_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project zmeta test folder split" {
        Some("runtime_15_asset_project_zmeta_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project zmeta current 12-test guard sync" {
        Some("runtime_15_asset_project_zmeta_current_12_test_guard_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset project manager test folder split" {
        Some("runtime_15_asset_project_manager_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project manager current 11-test guard sync" {
        Some("runtime_15_asset_project_manager_current_11_test_guard_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset project flow sample test folder split" {
        Some("runtime_15_asset_project_flow_sample_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset project example vampire test folder split" {
        Some("runtime_15_asset_project_example_vampire_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset artifact store test folder split" {
        Some("runtime_15_asset_artifact_store_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset material test folder split" {
        Some("runtime_15_asset_material_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset mesh test root split" {
        Some("runtime_15_asset_mesh_tests_root_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset glTF importer test folder split" {
        Some("runtime_15_asset_gltf_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset glTF primitive fixture folder split" {
        Some("runtime_15_asset_gltf_primitive_fixtures_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset importer test folder split" {
        Some("runtime_15_asset_importer_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset scene test folder split" {
        Some("runtime_15_asset_scene_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 asset UI test folder split" {
        Some("runtime_15_asset_ui_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset pipeline manager test folder split" {
        Some("runtime_15_asset_pipeline_manager_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 test file budget guard folder split" {
        Some("runtime_15_test_file_budget_guard_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 test file budget guard root mod cutover" {
        Some("runtime_15_test_file_budget_guard_root_mod_cutover_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 no oversized test files global gate" {
        Some("runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 render product mesh-cache morph tests child-owner split" {
        Some("runtime_15_render_product_mesh_cache_morph_tests_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 UI text layout folder-backed owner split" {
        Some("runtime_15_ui_text_layout_folder_backed_owner_split_static_passed_cargo_deferred")
    } else {
        None
    }
}
