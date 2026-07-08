pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 runtime plugin package manifest test folder split" {
        Some("runtime_15_runtime_plugin_package_manifest_tests_folder_split_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M3 runtime plugin package manifest capability-status test child-owner split"
    {
        Some("runtime_15_runtime_plugin_package_manifest_capability_status_tests_child_owner_split_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M3 runtime plugin catalog feature-dependency report test child-owner split"
    {
        Some("runtime_15_runtime_plugin_catalog_features_dependency_report_tests_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime plugin lifecycle fixture child-owner split" {
        Some("runtime_15_runtime_plugin_lifecycle_fixture_child_owner_split_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M3 runtime plugin lifecycle fixture row-data current-child route sync"
    {
        Some("runtime_15_runtime_plugin_lifecycle_fixture_row_data_current_child_route_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 export build plan test folder split" {
        Some("runtime_15_export_build_plan_tests_folder_split_static_passed_cargo_deferred")
    } else if slice
        == "Runtime 15 M3 export build plan profile feature matrix test child-owner split"
    {
        Some("runtime_15_export_build_plan_profile_feature_matrix_tests_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 export build plan platform test folder split" {
        Some(
            "runtime_15_export_build_plan_platform_tests_folder_split_static_passed_cargo_deferred",
        )
    } else if slice
        == "Runtime 15 M3 export build plan platform release-adapter test child-owner split"
    {
        Some("runtime_15_export_build_plan_platform_release_adapter_tests_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 gameplay host test folder split" {
        Some("runtime_15_gameplay_host_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 script VM gameplay host guard child-owner split" {
        Some("runtime_15_script_vm_gameplay_host_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 shader prewarm manifest test folder split" {
        Some("runtime_15_shader_prewarm_manifest_tests_folder_split_static_passed_cargo_deferred")
    } else {
        None
    }
}
