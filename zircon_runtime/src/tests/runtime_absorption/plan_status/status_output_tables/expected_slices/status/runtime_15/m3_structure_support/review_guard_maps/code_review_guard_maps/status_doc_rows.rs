pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 code review findings status-doc guard child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc guard folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status-mirror child-owner split" => {
            Some("runtime_15_code_review_findings_status_docs_status_mirror_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-doc map-source sync" => Some(
            "runtime_15_code_review_findings_status_docs_map_source_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_source_anchors_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc source anchors folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split" => Some(
            "runtime_15_code_review_findings_status_docs_status_anchors_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchors folder-backed split" => Some(
            "runtime_15_code_review_findings_status_docs_status_anchors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc child-anchor list child split" => Some(
            "runtime_15_code_review_findings_status_docs_child_anchor_list_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc child-anchor route folder-backed split" => {
            Some("runtime_15_code_review_findings_status_docs_child_anchor_route_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-doc root inventory child split" => Some(
            "runtime_15_code_review_findings_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 code review findings status-doc status anchor guard folder-backed split" => {
            Some("runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split" => {
            Some("runtime_15_code_review_findings_status_docs_status_anchor_child_ownership_child_split_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
