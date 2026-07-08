pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 plan-status index-tables child-owner split" => Some("2026-07-01"),
        "Runtime 15 M3 plan-status index-tables parent guard folder-backed split" => {
            Some("2026-07-05")
        }
        "Runtime 15 M3 plan-status index-tables split-layout guard folder-backed split" => {
            Some("2026-07-06")
        }
        "Runtime 15 M3 plan-status index status-anchors folder-backed split"
        | "Runtime 15 M3 plan-status recent static guards folder-backed split"
        | "Runtime 15 M3 plan-status support helpers folder-backed split"
        | "Runtime 15 M3 plan-status closeout guards folder-backed split" => Some("2026-07-05"),
        "Runtime 15 M3 plan-status child-map source reconciliation" => Some("2026-07-07"),
        _ => None,
    }
}

// Status: runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_index_tables_parent_guard_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_index_tables_split_layout_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_index_status_anchors_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_recent_static_guards_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_support_helpers_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_closeout_guards_folder_backed_static_passed_cargo_deferred.
// Status: runtime_15_plan_status_child_map_source_reconciliation_static_passed_cargo_deferred.
// Files: plan_status/index_tables.rs; plan_status/index_tables/subplan_map.rs; plan_status/index_tables/status_anchors.rs; plan_status/index_tables/index_consistency.rs; plan_status/index_tables/split_layout.rs.
// Files: plan_status/index_tables/split_layout/child_owner.rs; plan_status/index_tables/split_layout/parent_guard.rs; plan_status/index_tables/split_layout/split_guard.rs.
// Files: plan_status/index_tables/status_anchors/runtime03_module_doc.rs; plan_status/index_tables/status_anchors/runtime07_scene_asset.rs; plan_status/index_tables/status_anchors/runtime07_owner_budget.rs; plan_status/index_tables/status_anchors/generated_status.rs; plan_status/index_tables/status_anchors/runtime10_behavior.rs; plan_status/index_tables/status_anchors/cargo_attempt.rs; plan_status/index_tables/status_anchors/split_layout.rs.
// Files: plan_status/recent_static_guards.rs; plan_status/recent_static_guards/document_sources.rs; plan_status/recent_static_guards/runtime_01_to_04.rs; plan_status/recent_static_guards/runtime_05_to_08.rs; plan_status/recent_static_guards/runtime_09_to_12.rs; plan_status/recent_static_guards/runtime_13_14_review_index.rs; plan_status/recent_static_guards/split_layout.rs.
// Files: plan_status/support.rs; plan_status/support/assertions.rs; plan_status/support/file_inventory.rs; plan_status/support/frontmatter.rs; plan_status/support/index_markdown.rs; plan_status/support/runtime_plan_sources.rs; plan_status/support/split_layout.rs.
// Files: plan_status/closeout.rs; plan_status/closeout/runtime_05_diagnostics.rs; plan_status/closeout/runtime_05_source_anchors.rs; plan_status/closeout/runtime_05_status.rs; plan_status/closeout/runtime_05_support_first.rs; plan_status/closeout/split_layout.rs.
// Guard: runtime_15_plan_status_index_tables_guard_child_owner_split.
// Guard: runtime_15_plan_status_index_tables_parent_guard_is_folder_backed.
// Guard: runtime_15_plan_status_index_tables_split_layout_is_folder_backed.
// Guard: runtime_15_plan_status_index_status_anchors_are_folder_backed.
// Guard: runtime_15_plan_status_recent_static_guards_are_folder_backed.
// Guard: runtime_15_plan_status_support_helpers_are_folder_backed.
// Guard: runtime_15_plan_status_closeout_guards_are_folder_backed.
