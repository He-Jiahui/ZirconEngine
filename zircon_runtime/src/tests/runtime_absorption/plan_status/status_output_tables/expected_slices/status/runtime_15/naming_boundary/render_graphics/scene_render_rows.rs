pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover" => Some(
            "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover" => Some(
            "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover" => Some(
            "runtime_15_render_layer_schema_v1_mask_api_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
