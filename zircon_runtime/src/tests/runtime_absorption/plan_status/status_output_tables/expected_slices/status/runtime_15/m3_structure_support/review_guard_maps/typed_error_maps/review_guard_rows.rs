pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error convergence guard child-owner split" => Some(
            "runtime_15_typed_error_convergence_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native plugin loader typed-error review guard child-owner split" => Some(
            "runtime_15_native_plugin_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native ABI surfaces typed-error review guard child-owner split" => Some(
            "runtime_15_native_abi_surfaces_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native plugin descriptor ABI typed-error review guard child-owner split" => Some(
            "runtime_15_native_plugin_descriptor_abi_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 UI input typed-error review guard child-owner split" => Some(
            "runtime_15_ui_input_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native manifest sources typed-error review guard child-owner split" => Some(
            "runtime_15_native_manifest_sources_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 script host typed-error review guard child-owner split" => Some(
            "runtime_15_script_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 scene world typed-error review guard child-owner split" => Some(
            "runtime_15_scene_world_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset loader typed-error review guard child-owner split" => Some(
            "runtime_15_asset_loader_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 asset records typed-error review guard child-owner split" => Some(
            "runtime_15_asset_records_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 shader prewarm CLI typed-error review guard child-owner split" => Some(
            "runtime_15_shader_prewarm_cli_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host lifecycle-paths typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_lifecycle_paths_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host replay-runtime typed-error review guard child-owner split" => Some(
            "runtime_15_native_live_host_replay_runtime_typed_error_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
