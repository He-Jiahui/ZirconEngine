pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 render feature fallback capability naming hard cutover" => Some(
            "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 render material stale texture fixture naming hard cutover" => Some(
            "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 render graph fallback fixture naming hard cutover" => Some(
            "runtime_15_render_graph_fallback_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
