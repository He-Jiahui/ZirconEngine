pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 render shader definition bare-flag naming hard cutover" => Some(
            "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 GPU model embedded primitive naming hard cutover" => Some(
            "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 frame extract snapshot adapter naming hard cutover" => Some(
            "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 core framework render fixture naming hard cutover" => Some(
            "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
