pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 graphics dead-code guard module split" {
        Some("runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 graphics dead-code guard child-owner split" {
        Some("runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup"
    {
        Some("runtime_15_graphics_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 provider boilerplate guard module split" {
        Some("runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 provider boilerplate guard child-owner split" {
        Some("runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 facade surface guard module split" {
        Some("runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard module split" {
        Some("runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup" {
        Some("runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code guard child-owner split" {
        Some("runtime_15_runtime_dead_code_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code documentation anchor cleanup" {
        Some("runtime_15_runtime_dead_code_documentation_anchor_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code module-gate status wording cleanup" {
        Some("runtime_15_runtime_dead_code_module_gate_status_wording_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime dead-code production-gate status wording cleanup" {
        Some("runtime_15_runtime_dead_code_production_gate_status_wording_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 diagnostics guard module split" {
        Some("runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 core framework test folder split" {
        Some("runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked")
    } else if slice == "Runtime 15 M3 picking test folder split" {
        Some("runtime_15_picking_tests_folder_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 core runtime deactivation blocked test folder split" {
        Some("runtime_15_core_runtime_deactivation_blocked_tests_folder_split_static_passed_cargo_deferred")
    } else {
        None
    }
}
