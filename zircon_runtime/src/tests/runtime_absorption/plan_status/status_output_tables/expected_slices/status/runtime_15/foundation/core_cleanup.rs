pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F9 runtime prelude required type coverage" {
        Some("runtime_15_prelude_required_types_coremin_check_passed")
    } else if slice == "Runtime 15 M1 graphics facade visibility review findings mirror" {
        Some(
            "runtime_15_graphics_facade_visibility_review_findings_mirror_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 runtime UI dead-code support split" {
        Some("runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed")
    } else if slice == "Runtime 15 M5 production dead-code suppression global gate" {
        Some("runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 dead-code review status sync" {
        Some("runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 dead-code runtime/editor boundary status guard" {
        Some("runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F12 production dead-code current-state wording cleanup" {
        Some(
            "runtime_15_f12_production_dead_code_current_state_wording_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F12 UI text edit-state dead-code suppression cleanup" {
        Some(
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup" {
        Some("runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F1 native host callback panic guard" {
        Some("runtime_15_native_host_callback_panic_guard_static_passed_cargo_deferred")
    } else {
        None
    }
}
