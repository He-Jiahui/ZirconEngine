pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 graphics render-framework receiver naming hard cutover" => Some(
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 render framework trait/construction owner naming hard cutover" => Some(
            "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 graphics construction new owner naming hard cutover" => Some(
            "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
