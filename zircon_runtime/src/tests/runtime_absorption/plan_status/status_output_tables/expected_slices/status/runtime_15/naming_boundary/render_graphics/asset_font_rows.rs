pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover" => Some(
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 font/UI asset schema naming hard cutover" => {
            Some("runtime_15_font_ui_asset_schema_naming_hard_cutover_static_passed_cargo_deferred")
        }
        "Runtime 15 M2 font render-mode priority fixture naming hard cutover" => Some(
            "runtime_15_font_render_mode_priority_fixture_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
