pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover" => Some(
            "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
        ),
        "Runtime 15 M2 DDS upload policy naming hard cutover" => {
            Some("runtime_15_dds_upload_policy_naming_hard_cutover_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
