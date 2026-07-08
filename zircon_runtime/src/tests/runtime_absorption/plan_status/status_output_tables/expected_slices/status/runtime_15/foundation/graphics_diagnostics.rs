pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 graphics facade visibility note" {
        Some(
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
        )
    } else if slice == "Runtime 15 M1 graphics facade visibility review findings mirror" {
        Some(
            "runtime_15_graphics_facade_visibility_review_findings_mirror_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F14 diagnostics normalization" {
        Some("runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed")
    } else {
        None
    }
}
