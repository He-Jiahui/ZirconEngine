pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings direct assertions child-ownership guard folder-backed split" => {
            Some("runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings F12 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_f12_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions child-owner split" => {
            Some("runtime_15_code_review_findings_root_parent_direct_assertions_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings root-parent direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_root_parent_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings render direct assertions child-owner split" => {
            Some("runtime_15_code_review_findings_render_direct_assertions_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings render direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_render_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings F8 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_f8_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings P0 direct assertions child-owner split" => Some(
            "runtime_15_code_review_findings_p0_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split" => {
            Some("runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings direct assertions child-source sync" => {
            Some("runtime_15_code_review_findings_direct_assertions_child_source_sync_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
