pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 M3 lock-poison status row-data child-owner split" {
        Some(
            "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 lock poison policy route-owner split" {
        Some("runtime_15_lock_poison_policy_route_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 lock-poison split-layout guard folder-backed split" {
        Some("runtime_15_lock_poison_split_layout_guard_folder_backed_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 asset/render/input lock-poison guard child-owner split" {
        Some("runtime_15_asset_render_input_lock_poison_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 runtime services lock-poison guard child-owner split" {
        Some("runtime_15_runtime_services_lock_poison_guard_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module-convention status row-data child-owner split" {
        Some("runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module-convention status row-data owner child split" {
        Some("runtime_15_module_convention_status_row_data_owner_child_split_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard" {
        Some("runtime_15_module_convention_module_doc_frontmatter_uniqueness_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention gate guard folder-backed split" {
        Some("runtime_15_module_convention_gate_guard_folder_backed_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module-convention guard source reconciliation" {
        Some(
            "runtime_15_module_convention_guard_source_reconciliation_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 M3 module convention gate output contract" {
        Some("runtime_15_module_convention_gate_output_contract_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention non-render debt guard" {
        Some("runtime_15_module_convention_non_render_debt_guard_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 render-scoped migration debt handoff gate" {
        Some("runtime_15_render_scoped_migration_debt_handoff_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 hard-cutover allowed Hyper policy risk cleanup" {
        Some("runtime_15_hard_cutover_allowed_hyper_policy_risk_cleanup_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 M3 module convention gate audit-clear status mirror" {
        Some("runtime_15_module_convention_gate_audit_clear_status_mirror_core_min_cargo_passed_full_sweep_pending")
    } else if slice == "Runtime 15 M3 module convention zero-debt revalidation" {
        Some("runtime_15_module_convention_zero_debt_revalidation_static_passed_cargo_timeout_no_result")
    } else if slice == "Runtime 15 M3 module convention audit script family naming cleanup" {
        Some("runtime_15_module_convention_audit_script_family_naming_core_min_cargo_passed_full_sweep_pending")
    } else if slice == "Runtime 15 M3 dynamic scene absorption guard folder split" {
        Some("runtime_15_dynamic_scene_absorption_guard_folder_split_static_passed_cargo_deferred")
    } else {
        None
    }
}
